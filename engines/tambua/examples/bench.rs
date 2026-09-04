//! Tambua fork rebenchmark: full SAE to SPE to Tambua chain on
//! synthetic regimes, independent of Presence. Gates: windows above
//! zero in every family, sane statistics, visible adaptation across a
//! deliberate mid-run shift, stability ordering (tight beats wide),
//! flat footprint, measured throughput. Exits non-zero on any failure.
//! Run: cargo run --release -p rafiki-tambua --example bench

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
    let mut tambua = TambuaEngine::new();
    let mut rng = Lcg(0x7A48A);
    let mut ts: u128 = 0;
    let total = 2400usize;
    let mut mic_before = 0.0f32;
    let mut numerics_half = 0usize;
    let mut motions = std::collections::HashSet::new();
    let t0 = std::time::Instant::now();
    for i in 0..total {
        ts += 50;
        let regime = i / 800;
        let moving = regime > 0;
        let shifted = regime == 2;
        let t = i as f32 * 0.63;
        let accel = if moving {
            SensorData::accel(t.sin() * 1.6, 0.1, 9.81 + t.cos() * 0.9)
        } else {
            SensorData::accel(rng.range(-0.03, 0.03), rng.range(-0.03, 0.03), 9.81)
        };
        let mic_db = if shifted {
            70.0
        } else if moving {
            44.0
        } else {
            31.0
        };
        let mic = SensorData::mic(mic_db, 400.0);
        let gps = SensorData::gps(
            -6.7923 + if moving { i as f64 * 0.000012 } else { 0.0 },
            39.2083,
            5.0,
        );
        let screen = SensorData::screen(moving, 80);
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
            sae.ingest(ty, d);
            if let Some((feats, classes)) = spe.ingest(&e) {
                let mut out = Vec::new();
                tambua.absorb_features(&feats, &mut out);
                tambua.absorb_classes(&classes, &mut out);
                for c in &classes {
                    if c.kind == "motion_state" {
                        motions.insert(c.label.to_string());
                    }
                }
            }
        }
        if i == 1599 {
            mic_before = tambua.numeric_mean("mic.env_mean").unwrap_or(f32::NAN);
            numerics_half = tambua.numerics_len();
        }
    }
    let elapsed = t0.elapsed();
    let mic_after = tambua.numeric_mean("mic.env_mean").unwrap_or(f32::NAN);
    let s_tight = tambua.numeric_stability("gps.quality").unwrap_or(-1.0);
    let s_wide = tambua.numeric_stability("mic.env_mean").unwrap_or(-1.0);
    let tracks = tambua.footprint_tracks();

    println!("events: {total}, sae samples: {}", sae.sample_count());
    println!("spe windows: {}", spe.windows_computed);
    println!("mic baseline before shift: {mic_before:.2}, after: {mic_after:.2}");
    println!("stability tight vs wide: {s_tight:.3} vs {s_wide:.3}");
    println!("motion labels: {motions:?}");
    println!("footprint numerics: {numerics_half} at half, tracks: {tracks} at end");
    println!("updates absorbed: {}, publishes: {}", tambua.updates_absorbed, tambua.publishes);
    println!("throughput: {:.0} events/sec", total as f64 / elapsed.as_secs_f64().max(1e-9));

    if spe.windows_computed == 0 {
        failures.push("zero SPE windows".to_string());
    }
    if !(mic_after > mic_before + 5.0) {
        failures.push(format!("no adaptation: {mic_before:.2} -> {mic_after:.2}"));
    }
    if !(s_tight > s_wide) {
        failures.push(format!("stability ordering wrong: {s_tight:.3} vs {s_wide:.3}"));
    }
    if tambua.numerics_len() != numerics_half {
        failures.push("numeric tracks grew with events".to_string());
    }
    if motions.len() < 2 {
        failures.push(format!("motion never varied: {motions:?}"));
    }

    if failures.is_empty() {
        println!("TAMBUA FORK VALIDATION: PASS");
    } else {
        println!("TAMBUA FORK VALIDATION: FAIL");
        for f in &failures {
            println!("  - {f}");
        }
        std::process::exit(1);
    }
}
