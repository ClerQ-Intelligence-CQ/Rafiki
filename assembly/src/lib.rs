//! Rafiki assembly: the five forked engines wired into one pipeline.
//!
//! Teka (sae) to Chuja (spe) to Fikiri to Tambua to Pacha, plain
//! function calls, no bus, no runtime, no new machinery. Each engine's
//! output type feeds the next engine's input type directly. All
//! intelligence stays inside the engine crates, unchanged from their
//! validated fork states; this layer only orchestrates. Engine fields
//! are public so benches and tests can configure horizons without
//! forked helper constructors.

use rafiki_fikiri::FikiriEngine;
use rafiki_pacha::{PachaEngine, TwinSnapshot};
use rafiki_sae::{AcquisitionEngine, SensorData, SensorEvent, SensorType};
use rafiki_spe::Processor;
use rafiki_tambua::{AnomalyDetected, TambuaEngine};
use std::sync::{Arc, Mutex};

/// One assembled pipeline: all five engines plus shared state.
/// Bench-scale horizons differ from production defaults; see constructors.
pub struct Pipeline {
    pub sae: AcquisitionEngine,
    pub spe: Processor,
    pub fikiri: FikiriEngine,
    pub tambua: TambuaEngine,
    pub pacha: PachaEngine,
    bus: Arc<Mutex<Vec<SensorEvent>>>,
    pub fired: Vec<AnomalyDetected>,
}

impl Pipeline {
    /// Production horizons (staleness 60 s, clear 30 s, Tambua 300 s).
    pub fn new() -> Self {
        let bus = Arc::new(Mutex::new(Vec::new()));
        Self {
            sae: AcquisitionEngine::new(Arc::clone(&bus)),
            spe: Processor::new(),
            fikiri: FikiriEngine::new(),
            tambua: TambuaEngine::new(),
            pacha: PachaEngine::new(),
            bus,
            fired: Vec::new(),
        }
    }

    /// Bench-scale horizons (staleness/clear 5 s, Tambua 10 s) for
    /// compressed-time validation runs. Same code path otherwise.
    pub fn bench() -> Self {
        let bus = Arc::new(Mutex::new(Vec::new()));
        Self {
            sae: AcquisitionEngine::new(Arc::clone(&bus)),
            spe: Processor::new(),
            fikiri: FikiriEngine::new(),
            tambua: TambuaEngine::with_window_ms(10_000),
            pacha: PachaEngine::with_horizons(5_000, 5_000),
            bus,
            fired: Vec::new(),
        }
    }

    /// Absorb one sensor reading through all five engines. Returns the
    /// current twin snapshot after the absorb.
    pub fn absorb(
        &mut self,
        sensor_type: SensorType,
        data: SensorData,
        timestamp_ms: u128,
    ) -> TwinSnapshot {
        let stream = match sensor_type {
            SensorType::Accelerometer => "accel",
            SensorType::Gps => "gps",
            SensorType::Barometer => "baro",
            SensorType::MicAmplitude => "mic",
            SensorType::ScreenState => "screen",
        };
        self.pacha.note_liveness(stream, timestamp_ms);
        self.sae.ingest(sensor_type, data.clone());
        // Drain this step's SAE output events into SPE.
        let events: Vec<SensorEvent> = {
            let mut bus = self.bus.lock().unwrap();
            bus.drain(..).collect()
        };
        for e in &events {
            if let Some((feats, classes)) = self.spe.ingest(e) {
                let mut trash = Vec::new();
                self.fikiri.absorb_features(&feats, &mut trash);
                self.fikiri.absorb_classes(&classes, &mut trash);
                for f in &feats {
                    if let Some((mean, var, stab)) = self.fikiri.numeric_full(&f.name) {
                        self.tambua.absorb_baseline(&f.name, mean, var, stab);
                        self.pacha.absorb_baseline(&f.name, mean, stab);
                    }
                    self.pacha.absorb_feature(&f.name, f.value, timestamp_ms);
                    let dev = self.tambua.score_feature(
                        timestamp_ms,
                        &f.name,
                        f.value,
                        &mut self.fired,
                    );
                    self.pacha.absorb_deviation(&f.name, dev.deviation);
                }
                for c in &classes {
                    self.tambua.score_class(timestamp_ms, &c.kind, &c.label, &mut self.fired);
                }
            }
        }
        for a in self.fired.drain(..) {
            self.pacha.absorb_anomaly(&a);
        }
        self.pacha.tick(timestamp_ms);
        self.pacha.poll_publish(timestamp_ms);
        self.pacha.snapshot(timestamp_ms)
    }

    pub fn anomaly_count(&self) -> u64 {
        self.tambua.anomalies_fired
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_absorbs_end_to_end() {
        let mut p = Pipeline::bench();
        let mut ts = 0u128;
        for _ in 0..200 {
            ts += 50;
            p.absorb(
                SensorType::Accelerometer,
                SensorData::accel(0.0, 0.0, 9.81),
                ts,
            );
            p.absorb(SensorType::MicAmplitude, SensorData::mic(35.0, 300.0), ts);
        }
        assert!(p.sae.sample_count() > 0);
        assert!(p.spe.windows_computed > 0);
        let s = p.pacha.snapshot(ts);
        assert!(!s.features.is_empty());
    }
}
