//! Pacha fork rebenchmark: full five-fork-engine chain on synthetic
//! regimes, independent of Presence. Gates: SPE windows above zero,
//! GPS hold-back flagged stale then resolved, co-spike anomaly
//! reflected then cleared, serialized twin bytes under the hard bound,
//! flat footprint, measured throughput. Exits non-zero on failure.
//! Run: cargo run --release -p rafiki-pacha --example bench

use rafiki_fikiri::FikiriEngine;
use rafiki_pacha::PachaEngine;
use rafiki_sae::{AcquisitionEngine, SensorData, SensorEvent, SensorType};
use rafiki_spe::Processor;
use rafiki_tambua::TambuaEngine;
use std::sync::{Arc, Mutex};

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32)
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next() * (hi - lo)
    }
}

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let bus: Arc<Mutex<Vec<SensorEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let mut sae = AcquisitionEngine::new(Arc::clone(&bus));
    let mut spe = Processor::new();
    let mut fikiri = FikiriEngine::new();
    let mut tambua = TambuaEngine::new();
    let mut pacha = PachaEngine::with_horizons(5_000, 5_000);
    let mut rng = Lcg(0x9AC4A);
    let mut ts: u128 = 0;
    let total = 3000usize;
    let mut stale_during_hold = false;
    let mut anomaly_during_spike = false;
    let mut fired: Vec<rafiki_tambua::AnomalyDetected> = Vec::new();
    let t0 = std::time::Instant::now();
    for i in 0..total {
        ts += 50;
        let walking = i >= 800;
        let hold_gps = (1600..2000).contains(&i);
        let co_spike = (2000..2150).contains(&i);
        let t = i as f32 * 0.63;
        let amp = if co_spike { 4.0 } else { 1.6 };
        let mic_db = if co_spike {
            78.0
        } else if walking {
            44.0
        } else {
            31.0
        };
        let accel = if walking {
            SensorData::accel(t.sin() * amp, 0.1, 9.81 + t.cos() * 0.9)
        } else {
            SensorData::accel(rng.range(-0.03, 0.03), rng.range(-0.03, 0.03), 9.81)
        };
        let mic = SensorData::mic(mic_db, 400.0);
        let gps = SensorData::gps(
            -6.7923 + if co_spike {
                0.002 + i as f64 * 0.000012
            } else if walking {
                i as f64 * 0.000012
            } else {
                0.0
            },
            39.2083,
            5.0,
        );
        let screen = SensorData::screen(walking, 80);
        for (ty, d) in [
            (SensorType::Accelerometer, accel),
            (SensorType::MicAmplitude, mic),
            (SensorType::Gps, gps),
            (SensorType::ScreenState, screen),
        ] {
            let e = SensorEvent {
                timestamp_ms: ts,
                sensor_type: ty,
                data: d.clone(),
                confidence: rafiki_sae::confidence_for(ty, &d),
            };
            pacha.note_liveness("sae", ts);
            sae.ingest(ty, d);
            if hold_gps {
                continue;
            }
            if let Some((feats, classes)) = spe.ingest(&e) {
                let mut trash = Vec::new();
                fikiri.absorb_features(&feats, &mut trash);
                fikiri.absorb_classes(&classes, &mut trash);
                for f in &feats {
                    if let Some((mean, var, stab)) = fikiri.numeric_full(&f.name) {
                        tambua.absorb_baseline(&f.name, mean, var, stab);
                        pacha.absorb_baseline(&f.name, mean, stab);
                    }
                    pacha.absorb_feature(&f.name, f.value, ts);
                    let dev = tambua.score_feature(ts, &f.name, f.value, &mut fired);
                    pacha.absorb_deviation(&f.name, dev.deviation);
                }
                for c in &classes {
                    tambua.score_class(ts, &c.kind, &c.label, &mut fired);
                }
            }
        }
        for a in fired.drain(..) {
            pacha.absorb_anomaly(&a);
        }
        pacha.tick(ts);
        pacha.poll_publish(ts);
        if i == 1999 {
            stale_during_hold = pacha
                .snapshot(ts)
                .features
                .iter()
                .any(|f| f.feature.starts_with("gps.") && f.stale);
        }
        if (2000..2400).contains(&i) && pacha.snapshot(ts).anomaly.is_some() {
            anomaly_during_spike = true;
        }
    }
    let elapsed = t0.elapsed();
    let final_snap = pacha.snapshot(ts);
    let bytes = final_snap.serialized_size();

    println!("events: {total}, sae samples: {}", sae.sample_count());
    println!("spe windows: {}", spe.windows_computed);
    println!("tambua anomalies fired: {}", tambua.anomalies_fired);
    println!("pacha snapshots published: {}", pacha.snapshots_published);
    println!("stale gps during hold: {stale_during_hold} (must be true)");
    println!("anomaly during co-spike: {anomaly_during_spike} (must be true)");
    println!("anomaly at end: {} (must be false)", final_snap.anomaly.is_some());
    println!("twin bytes: {bytes} (must be < 16384)");
    println!("tracks: {}, completeness end: {:.2}", pacha.track_count(), final_snap.completeness);
    println!("throughput: {:.0} events/sec", total as f64 / elapsed.as_secs_f64().max(1e-9));

    if spe.windows_computed == 0 {
        failures.push("zero SPE windows".to_string());
    }
    if tambua.anomalies_fired == 0 {
        failures.push("tambua never fired".to_string());
    }
    if !stale_during_hold {
        failures.push("gps hold-back never flagged stale".to_string());
    }
    if !anomaly_during_spike {
        failures.push("co-spike never reflected in twin".to_string());
    }
    if final_snap.anomaly.is_some() {
        failures.push("anomaly never cleared after recovery".to_string());
    }
    if bytes >= 16384 {
        failures.push(format!("twin too big: {bytes} bytes"));
    }

    if failures.is_empty() {
        println!("PACHA FORK VALIDATION: PASS");
    } else {
        println!("PACHA FORK VALIDATION: FAIL");
        for f in &failures {
            println!("  - {f}");
        }
        std::process::exit(1);
    }
}
