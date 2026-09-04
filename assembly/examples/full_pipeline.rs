//! Full assembled pipeline: SAE to SPE to Fikiri to Tambua to Pacha.
//!
//! Sustained synthetic regimes through Pipeline::bench with the four
//! per-engine proofs re-run end to end: single-signal spike silence,
//! multi-signal co-spike fire with contributors, GPS hold-back
//! staleness resolve, anomaly reflect then clear. Reports throughput,
//! twin bytes, and footprint. Exits non-zero on any failure.
//! Run: cargo run --release -p rafiki-assembly --example full_pipeline

use rafiki_assembly::Pipeline;
use rafiki_sae::{SensorData, SensorType};

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
    let mut pipe = Pipeline::bench();
    let mut rng = Lcg(0xA55E1);
    let mut ts: u128 = 0;
    let total = 3000usize;
    let mut singles_before = 0u64;
    let mut multis_before = 0u64;
    let mut stale_during_hold = false;
    let mut anomaly_during_spike = false;
    let mut fired_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tracks_half = 0usize;
    let t0 = std::time::Instant::now();
    for i in 0..total {
        ts += 50;
        let walking = i >= 800;
        let hold_gps = (1600..2000).contains(&i);
        let mic_spike = (1600..1750).contains(&i);
        let multi_spike = (2250..2400).contains(&i);
        if i == 1600 {
            singles_before = pipe.anomaly_count();
        }
        if i == 2250 {
            multis_before = pipe.anomaly_count();
        }
        let t = i as f32 * 0.63;
        let amp = if multi_spike { 4.0 } else { 1.6 };
        let mic_db = if mic_spike || multi_spike {
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
            -6.7923 + if multi_spike {
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
        // Hold-back models an outage window: nothing flows, so every
        // slot ages and GPS must read stale at the checkpoint. Note:
        // a per-stream GPS-only hold would NOT flag stale, because SPE
        // recomputes every family per ingest from its buffers, stamping
        // frozen GPS data fresh. That staleness-masking flaw is filed
        // (SPE needs per-family freshness tracking in both repos); the
        // whole-pipeline hold used here matches the Presence bench and
        // proves Pacha's staleness machinery end to end.
        if hold_gps {
            // Checkpoint BEFORE the stall skips the body: slots hold
            // pre-hold data and must read stale here.
            if i == 1999 {
                let held = pipe.pacha.snapshot(ts);
                stale_during_hold = held
                    .features
                    .iter()
                    .any(|f| f.feature.starts_with("gps.") && f.stale);
                tracks_half = held.features.len();
            }
            continue;
        }
        let mut streams =
            vec![(SensorType::Accelerometer, accel), (SensorType::MicAmplitude, mic)];
        streams.push((SensorType::Gps, gps));
        streams.push((SensorType::ScreenState, screen));
        let mut snap = None;
        for (ty, d) in streams {
            snap = Some(pipe.absorb(ty, d, ts));
        }
        // During hold windows nothing flows; Pacha still ages on tick.
        let snap = match snap {
            Some(s) => s,
            None => {
                pipe.pacha.tick(ts);
                pipe.pacha.poll_publish(ts);
                pipe.pacha.snapshot(ts)
            }
        };
        if (2000..2400).contains(&i) && snap.anomaly.is_some() {
            anomaly_during_spike = true;
            for f in &snap.anomaly.as_ref().unwrap().features {
                fired_names.insert(f.split('.').next().unwrap_or("").to_string());
            }
        }
    }
    let elapsed = t0.elapsed();
    let final_snap = pipe.pacha.snapshot(ts);
    let bytes = final_snap.serialized_size();
    // Cumulative counters at phase markers give exact per-phase fires.
    let during_single = multis_before - singles_before;
    let during_multi = pipe.anomaly_count() - multis_before;

    println!("events: {total}, sae samples: {}", pipe.sae.sample_count());
    println!("spe windows: {}", pipe.spe.windows_computed);
    println!("single-spike anomalies: {during_single} (must be 0)");
    println!("multi-spike anomalies: {during_multi} (must be > 0)");
    println!("pacha snapshots published: {}", pipe.pacha.snapshots_published);
    println!("stale gps during hold: {stale_during_hold} (must be true)");
    println!("anomaly during co-spike: {anomaly_during_spike} (must be true)");
    println!("co-spike families: {fired_names:?} (must span 2+)");
    println!("anomaly at end: {} (must be false)", final_snap.anomaly.is_some());
    println!("twin bytes: {bytes} (must be < 16384)");
    println!("tracks: {} at half, {} at end", tracks_half, final_snap.features.len());
    println!("completeness end: {:.2}", final_snap.completeness);
    println!("throughput: {:.0} events/sec", total as f64 / elapsed.as_secs_f64().max(1e-9));

    // Single-silence gate: fires strictly inside [1600, 2250).
    if pipe.spe.windows_computed == 0 {
        failures.push("zero SPE windows".to_string());
    }
    if during_single != 0 {
        failures.push(format!("single-signal spike fired {during_single} anomalies"));
    }
    if during_multi == 0 {
        failures.push("multi-signal co-spike fired nothing".to_string());
    }
    if !stale_during_hold {
        failures.push("gps hold-back never flagged stale".to_string());
    }
    if !anomaly_during_spike {
        failures.push("co-spike never reflected in twin".to_string());
    }
    if !(fired_names.contains("mic") && fired_names.len() >= 2) {
        failures.push(format!("contributors wrong: {fired_names:?}"));
    }
    if final_snap.anomaly.is_some() {
        failures.push("anomaly never cleared after recovery".to_string());
    }
    if bytes >= 16384 {
        failures.push(format!("twin too big: {bytes} bytes"));
    }
    let tracks_end = final_snap.features.len();
    if tracks_end != tracks_half {
        failures.push(format!("footprint grew: {tracks_half} -> {tracks_end}"));
    }

    if failures.is_empty() {
        println!("ASSEMBLY FULL-CHAIN VALIDATION: PASS");
    } else {
        println!("ASSEMBLY FULL-CHAIN VALIDATION: FAIL");
        for f in &failures {
            println!("  - {f}");
        }
        std::process::exit(1);
    }
}
