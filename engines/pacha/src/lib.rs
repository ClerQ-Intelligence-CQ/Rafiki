//! Rafiki Pacha: twin state engine.
//!
//! Public re-anchor of the Penemue living aggregate. Consumes the four
//! upstream engines and assembles one compact, continuously-updated,
//! serializable twin: current values, baseline references, deviation
//! status, freshness, liveness, and any active anomaly. Fixed-size
//! state per feature, no history buffers, update in place. Snapshots
//! publish on transitions only (anomaly flips, staleness flips, first
//! sightings), never per event. No ML of any kind.

use rafiki_tambua::AnomalyDetected;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Default staleness horizon in milliseconds of event time.
pub const STALE_AFTER_MS: u128 = 60_000;
/// Default anomaly clear horizon: no fresh anomaly within this span
/// closes the open convergence.
pub const CLEAR_AFTER_MS: u128 = 30_000;

/// One feature's slot in the twin. Fixed size by construction: no
/// vectors, no history buffers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinFeatureState {
    pub feature: String,
    pub value: f32,
    pub baseline_mean: f32,
    pub stability: f32,
    pub deviation: f32,
    pub last_updated_ms: u128,
    pub live: bool,
    pub stale: bool,
}

/// Active anomaly summary. Absent (None) means no open convergence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinAnomalyState {
    pub features: Vec<String>,
    pub confidence: f32,
    pub since_ms: u128,
}

/// The whole twin. Must serialize to kilobytes; benches assert it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinSnapshot {
    pub features: Vec<TwinFeatureState>,
    pub anomaly: Option<TwinAnomalyState>,
    pub completeness: f32,
    pub freshness_ms: u128,
}

impl TwinSnapshot {
    pub fn serialized_size(&self) -> usize {
        serde_json::to_vec(self).map(|b| b.len()).unwrap_or(usize::MAX)
    }
}

#[derive(Debug, Clone, Default)]
struct Slot {
    value: f32,
    mean: f32,
    stability: f32,
    deviation: f32,
    updated_ms: u128,
    ever_seen: bool,
}

/// The engine: fixed per-feature slots plus optional open anomaly.
#[derive(Debug, Default)]
pub struct PachaEngine {
    slots: HashMap<String, Slot>,
    stream_seen: HashMap<String, u128>,
    anomaly: Option<(Vec<String>, f32, u128)>,
    stale_after_ms: u128,
    clear_after_ms: u128,
    last_anomaly_ts: u128,
    last_published_anomaly: bool,
    last_stale: HashSet<String>,
    last_count: usize,
    pub snapshots_published: u64,
}

impl PachaEngine {
    pub fn new() -> Self {
        Self {
            stale_after_ms: STALE_AFTER_MS,
            clear_after_ms: CLEAR_AFTER_MS,
            ..Self::default()
        }
    }

    pub fn with_horizons(stale_after_ms: u128, clear_after_ms: u128) -> Self {
        Self { stale_after_ms, clear_after_ms, ..Self::default() }
    }

    /// Liveness per sensor stream.
    pub fn note_liveness(&mut self, stream: &str, timestamp_ms: u128) {
        self.stream_seen.insert(stream.to_string(), timestamp_ms);
    }

    /// Current value from the feature stream.
    pub fn absorb_feature(&mut self, feature: &str, value: f32, timestamp_ms: u128) {
        let slot = self.slots.entry(feature.to_string()).or_default();
        slot.value = value;
        slot.updated_ms = timestamp_ms;
        slot.ever_seen = true;
    }

    /// Baseline reference from the baseline engine.
    pub fn absorb_baseline(&mut self, feature: &str, mean: f32, stability: f32) {
        let slot = self.slots.entry(feature.to_string()).or_default();
        slot.mean = mean;
        slot.stability = stability;
        slot.ever_seen = true;
    }

    /// Deviation status from the anomaly engine.
    pub fn absorb_deviation(&mut self, feature: &str, deviation: f32) {
        if let Some(slot) = self.slots.get_mut(feature) {
            slot.deviation = deviation;
        }
    }

    /// Open or refresh the active anomaly. Returns true when the
    /// snapshot-worthy state changed (open or new timestamp).
    pub fn absorb_anomaly(&mut self, anomaly: &AnomalyDetected) -> bool {
        let feats: Vec<String> = anomaly.features.iter().map(|d| d.feature.clone()).collect();
        let changed = self.anomaly.as_ref().map(|(f, _, _)| f) != Some(&feats);
        self.anomaly = Some((feats, anomaly.confidence, anomaly.timestamp_ms));
        self.last_anomaly_ts = anomaly.timestamp_ms;
        if changed {
            self.maybe_publish_count();
        }
        changed
    }

    /// Age out an anomaly nobody has refreshed.
    pub fn tick(&mut self, now_ms: u128) -> bool {
        if self.anomaly.is_some() && now_ms.saturating_sub(self.last_anomaly_ts) > self.clear_after_ms
        {
            self.anomaly = None;
            self.maybe_publish_count();
            return true;
        }
        false
    }

    fn maybe_publish_count(&mut self) {
        self.snapshots_published += 1;
    }

    /// Publish check over transitions: anomaly flips, staleness flips,
    /// first sightings. Returns true when a snapshot dispatch is due.
    pub fn poll_publish(&mut self, now_ms: u128) -> bool {
        let stale: HashSet<String> = self
            .slots
            .iter()
            .filter(|(_, s)| s.ever_seen && now_ms.saturating_sub(s.updated_ms) > self.stale_after_ms)
            .map(|(k, _)| k.clone())
            .collect();
        let anomaly_open = self.anomaly.is_some();
        let count = self.slots.len();
        if anomaly_open != self.last_published_anomaly || stale != self.last_stale || count != self.last_count
        {
            self.last_published_anomaly = anomaly_open;
            self.last_stale = stale;
            self.last_count = count;
            self.maybe_publish_count();
            return true;
        }
        false
    }

    /// Full snapshot at a timestamp. Stale flags and completeness are
    /// computed here, never stored, so they cannot go out of date.
    pub fn snapshot(&self, now_ms: u128) -> TwinSnapshot {
        let mut features: Vec<TwinFeatureState> = self
            .slots
            .iter()
            .map(|(name, s)| {
                let stale = s.ever_seen && now_ms.saturating_sub(s.updated_ms) > self.stale_after_ms;
                TwinFeatureState {
                    feature: name.clone(),
                    value: s.value,
                    baseline_mean: s.mean,
                    stability: s.stability,
                    deviation: s.deviation,
                    last_updated_ms: s.updated_ms,
                    live: !stale,
                    stale,
                }
            })
            .collect();
        features.sort_by(|a, b| a.feature.cmp(&b.feature));
        let live_count = features.iter().filter(|f| f.live).count();
        let completeness =
            if features.is_empty() { 0.0 } else { live_count as f32 / features.len() as f32 };
        let anomaly = self.anomaly.as_ref().map(|(feats, conf, since)| TwinAnomalyState {
            features: feats.clone(),
            confidence: *conf,
            since_ms: *since,
        });
        TwinSnapshot { features, anomaly, completeness, freshness_ms: now_ms }
    }

    pub fn track_count(&self) -> usize {
        self.slots.len()
    }
    pub fn anomaly_open(&self) -> bool {
        self.anomaly.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rafiki_tambua::DeviationScore;

    fn engine() -> PachaEngine {
        PachaEngine::with_horizons(5_000, 5_000)
    }

    fn feed(e: &mut PachaEngine, name: &str, value: f32, ts: u128) {
        e.note_liveness("test", ts);
        e.absorb_feature(name, value, ts);
        e.absorb_baseline(name, value, 0.9);
    }

    #[test]
    fn composition_holds_all_parts() {
        let mut e = engine();
        feed(&mut e, "mic.env_mean", 40.0, 1000);
        e.tick(1000);
        let s = e.snapshot(1000);
        assert_eq!(s.features.len(), 1);
        let f = &s.features[0];
        assert_eq!(f.value, 40.0);
        assert!(f.live && !f.stale);
        assert!(s.anomaly.is_none());
        assert!((s.completeness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn held_back_feature_goes_stale() {
        let mut e = engine();
        feed(&mut e, "mic.env_mean", 40.0, 0);
        feed(&mut e, "accel.mag_var", 0.2, 0);
        for t in (1000..8000).step_by(500) {
            feed(&mut e, "mic.env_mean", 40.0, t);
        }
        e.tick(8000);
        let s = e.snapshot(8000);
        let stale: Vec<&str> =
            s.features.iter().filter(|f| f.stale).map(|f| f.feature.as_str()).collect();
        assert_eq!(stale, vec!["accel.mag_var"]);
        assert!(s.completeness < 1.0);
    }

    #[test]
    fn anomaly_reflects_then_clears() {
        let mut e = engine();
        feed(&mut e, "mic.env_mean", 40.0, 1000);
        assert!(!e.anomaly_open());
        e.absorb_anomaly(&AnomalyDetected {
            features: vec![DeviationScore {
                feature: "mic.env_mean".to_string(),
                deviation: 5.0,
                confidence: 0.8,
                stable: true,
            }],
            confidence: 0.8,
            timestamp_ms: 2000,
            window_ms: 10_000,
        });
        assert!(e.anomaly_open());
        assert!(e.snapshot(2000).anomaly.is_some());
        e.tick(9000);
        assert!(!e.anomaly_open());
        assert!(e.snapshot(9000).anomaly.is_none());
    }

    #[test]
    fn serialized_size_is_kilobytes() {
        let mut e = engine();
        for i in 0..46 {
            let name = format!("feat.{i:02}");
            feed(&mut e, &name, i as f32, 1000);
        }
        let s = e.snapshot(1000);
        let bytes = s.serialized_size();
        assert!(bytes < 16384, "{bytes} bytes");
        assert!(bytes > 1000);
        let back: TwinSnapshot = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn footprint_flat_across_thousands_of_absorbs() {
        let mut e = engine();
        for i in 0..5000 {
            feed(&mut e, "mic.env_mean", 40.0 + (i % 7) as f32, i as u128 * 50);
        }
        assert_eq!(e.track_count(), 1);
        let s = e.snapshot(250_000);
        assert_eq!(s.serialized_size(), e.snapshot(250_000).serialized_size());
    }
}
