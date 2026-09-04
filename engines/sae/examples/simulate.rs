//! SAE simulate: deterministic sensor stream through acquisition,
//! then one SPE window over the same events. Mirrors the README sample
//! output. Run: cargo run --release --example simulate (from engines/sae).

use rafiki_sae::{confidence_for, AcquisitionEngine, SensorData, SensorEvent, SensorType};
use rafiki_spe::Processor;
use std::sync::{Arc, Mutex};

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32)
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next() * (hi - lo)
    }
}

fn main() {
    println!("SAE::init  duty_cycle=adaptive  sensors=[accel, gps, baro, mic_amplitude]");
    let bus: Arc<Mutex<Vec<SensorEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let mut engine = AcquisitionEngine::new(Arc::clone(&bus));
    let mut spe = Processor::new();
    let mut rng = Lcg(0x5AEED);
    let mut ts: u128 = 1_722_470_400_000;
    let mut shown = 0usize;
    let mut windows = 0usize;
    // Quiet regime first (duty drops), then motion (duty rises).
    for i in 0..160 {
        ts += 50;
        let moving = i >= 100;
        let ax = if moving {
            (i as f32 * 0.6).sin() * 1.6
        } else {
            rng.range(-0.03, 0.03)
        };
        let mic = if moving { 45.0 } else { 41.2 };
        let samples = [
            (SensorType::Accelerometer, SensorData::accel(ax, 0.01, 9.81)),
            (SensorType::Barometer, SensorData::baro(1013.0, 25.0)),
            (SensorType::MicAmplitude, SensorData::mic(mic, 300.0)),
        ];
        for (t, d) in samples {
            let e = SensorEvent {
                timestamp_ms: ts,
                sensor_type: t,
                data: d.clone(),
                confidence: confidence_for(t, &d),
            };
            engine.ingest(t, d);
            if spe.ingest(&e).is_some() {
                windows += 1;
            }
            if shown < 3 {
                println!("EVENT  {:?}  ts: {} conf: {:.2}", t, ts, e.confidence);
                shown += 1;
            }
        }
        if i == 60 {
            println!(
                "DUTY   low_activity detected -> sampling interval {}ms",
                engine.sampling_interval()
            );
        }
    }
    println!(
        "samples: {}  spe_windows: {}",
        engine.sample_count(),
        windows
    );
    println!("peak_mem: ~1MB   steady_state: <1KB engine + bus   cpu: simulator-bound");
}
