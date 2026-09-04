//! Rafiki SPE: signal processing engine.
//!
//! Public re-anchor of the Chuja feature pipeline. Three stages over
//! the SAE event stream: preprocessing (windows, gravity separation,
//! GPS smoothing, adaptive mic floor), bounded statistical feature
//! extraction, and composite classification with signal-weighted
//! confidence. No ML, no fixed population thresholds, deterministic.

use rafiki_sae::{SensorData, SensorEvent};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const SHORT_WIN: usize = 32;
pub const LONG_WIN: usize = 256;
pub const MAX_LAG: usize = 16;
const BANDS: [f32; 3] = [0.05, 0.15, 0.35];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    pub name: &'static str,
    pub value: f32,
    pub confidence: f32,
    pub completeness: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Classification {
    pub kind: &'static str,
    pub label: &'static str,
    pub confidence: f32,
}

fn mean(xs: &[f32]) -> f32 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f32>() / xs.len() as f32
    }
}

fn variance(xs: &[f32], m: f32) -> f32 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().map(|x| (x - m) * (x - m)).sum::<f32>() / xs.len() as f32
    }
}

fn finite(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

fn goertzel(xs: &[f32], freq: f32) -> f32 {
    if xs.is_empty() {
        return 0.0;
    }
    let m = mean(xs);
    let coeff = 2.0 * (2.0 * std::f32::consts::PI * freq).cos();
    let (mut s1, mut s2) = (0.0f32, 0.0f32);
    for &x in xs {
        let s0 = (x - m) + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - coeff * s1 * s2).max(0.0).sqrt()
}

/// Bounded-lag autocorrelation peak over at most the last 64 samples.
/// O(MAX_LAG * 64) per call regardless of sustained load.
fn periodicity(xs: &[f32]) -> f32 {
    let n = xs.len().min(64);
    if n < MAX_LAG + 2 {
        return 0.0;
    }
    let xs = &xs[xs.len() - n..];
    let m = mean(xs);
    let var = variance(xs, m);
    if var <= 1e-9 {
        return 0.0;
    }
    let mut best = 0.0f32;
    for lag in 1..=MAX_LAG {
        let mut acc = 0.0f32;
        for i in 0..(n - lag) {
            acc += (xs[i] - m) * (xs[i + lag] - m);
        }
        let r = acc / ((n - lag) as f32 * var);
        if r > best {
            best = r;
        }
    }
    best.clamp(0.0, 1.0)
}

#[derive(Debug, Default)]
struct Rolling {
    buf: VecDeque<f32>,
    cap: usize,
    missing: usize,
}

impl Rolling {
    fn new(cap: usize) -> Self {
        Self { buf: VecDeque::with_capacity(cap), cap, missing: 0 }
    }
    fn push(&mut self, v: f32) {
        if self.buf.len() == self.cap {
            self.buf.pop_front();
        }
        self.buf.push_back(v);
    }
    fn clean(&self) -> Vec<f32> {
        self.buf.iter().copied().filter(|v| v.is_finite()).collect()
    }
    fn filled(&self, need: usize) -> bool {
        self.clean().len() >= need
    }
    fn completeness(&self) -> f32 {
        let total = (self.buf.len() + self.missing) as f32;
        if total <= 0.0 {
            0.0
        } else {
            self.buf.len() as f32 / total
        }
    }
}

#[derive(Debug)]
pub struct Processor {
    accel_mag: Rolling,
    gravity: [f32; 3],
    mic_env: Rolling,
    noise_floor: f32,
    gps_trail: VecDeque<(f64, f64, f32)>,
    alt: Rolling,
    screen_on_ms: u128,
    screen_off_ms: u128,
    unlocks: u64,
    longest_off_ms: u128,
    last_screen_on: bool,
    last_change_ms: u128,
    initialized: bool,
    events_seen: u64,
    pub windows_computed: u64,
    pub family_counts: [u64; 4],
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            accel_mag: Rolling::new(LONG_WIN),
            gravity: [0.0, 0.0, 9.81],
            mic_env: Rolling::new(LONG_WIN),
            noise_floor: 30.0,
            gps_trail: VecDeque::with_capacity(LONG_WIN),
            alt: Rolling::new(LONG_WIN),
            screen_on_ms: 0,
            screen_off_ms: 0,
            unlocks: 0,
            longest_off_ms: 0,
            last_screen_on: true,
            last_change_ms: 0,
            initialized: false,
            events_seen: 0,
            windows_computed: 0,
            family_counts: [0; 4],
        }
    }
}

impl Processor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest(&mut self, e: &SensorEvent) -> Option<(Vec<Feature>, Vec<Classification>)> {
        self.events_seen += 1;
        if !self.initialized {
            self.last_screen_on = matches!(&e.data, SensorData::ScreenState(s) if s.on);
            self.last_change_ms = e.timestamp_ms;
            self.initialized = true;
        }
        match &e.data {
            SensorData::Accelerometer(a) => {
                let mag = (a.x * a.x + a.y * a.y + a.z * a.z).sqrt();
                self.accel_mag.push(finite(mag));
                for i in 0..3 {
                    let v = [a.x, a.y, a.z][i];
                    self.gravity[i] += 0.1 * (v - self.gravity[i]);
                }
            }
            SensorData::MicAmplitude(m) => {
                self.mic_env.push(finite(m.envelope_db.max(0.0)));
                self.noise_floor += 0.01 * (m.envelope_db.max(0.0) - self.noise_floor);
            }
            SensorData::Gps(g) => {
                if self.gps_trail.len() == LONG_WIN {
                    self.gps_trail.pop_front();
                }
                self.gps_trail.push_back((g.latitude, g.longitude, g.accuracy_m.max(0.0)));
            }
            SensorData::Barometer(b) => {
                let alt = 44330.0 * (1.0 - (b.pressure_hpa / 1013.25).powf(1.0 / 5.255));
                self.alt.push(finite(alt));
            }
            SensorData::ScreenState(s) => {
                let dt = e.timestamp_ms.saturating_sub(self.last_change_ms);
                if s.on == self.last_screen_on {
                    if s.on {
                        self.screen_on_ms += dt.min(10_000);
                    } else {
                        self.screen_off_ms += dt.min(10_000);
                        if self.screen_off_ms > self.longest_off_ms {
                            self.longest_off_ms = self.screen_off_ms;
                        }
                    }
                } else {
                    if s.on {
                        self.unlocks += 1;
                    }
                    self.last_screen_on = s.on;
                    self.last_change_ms = e.timestamp_ms;
                }
            }
        }
        if !self.accel_mag.filled(SHORT_WIN) || !self.mic_env.filled(SHORT_WIN) {
            return None;
        }
        let feats = self.extract();
        let classes = self.classify(&feats);
        self.windows_computed += 1;
        Some((feats, classes))
    }

    fn feat(&self, name: &'static str, value: f32, confidence: f32, completeness: f32) -> Feature {
        Feature {
            name,
            value: finite(value),
            confidence: confidence.clamp(0.0, 1.0),
            completeness: completeness.clamp(0.0, 1.0),
        }
    }

    fn extract(&mut self) -> Vec<Feature> {
        let mut out = Vec::with_capacity(30);
        let mag = self.accel_mag.clean();
        let tail: Vec<f32> =
            mag.iter().rev().take(SHORT_WIN).copied().collect::<Vec<_>>().into_iter().rev().collect();
        let comp = self.accel_mag.completeness();
        let m = mean(&tail);
        let v = variance(&tail, m);
        let mut zc = 0usize;
        for w in tail.windows(2) {
            if (w[0] - m) * (w[1] - m) < 0.0 {
                zc += 1;
            }
        }
        let zcr = zc as f32 / tail.len().max(1) as f32;
        let buf64: Vec<f32> =
            mag.iter().rev().take(64).copied().collect::<Vec<_>>().into_iter().rev().collect();
        let mut band = 0usize;
        let mut band_mag = -1.0f32;
        for (i, f) in BANDS.iter().enumerate() {
            let g = goertzel(&buf64, *f);
            if g > band_mag {
                band_mag = g;
                band = i;
            }
        }
        let mut jerk = 0.0f32;
        for w in tail.windows(2) {
            jerk += (w[1] - w[0]).abs();
        }
        jerk /= tail.len().max(1) as f32;
        let g = self.gravity;
        let pitch = (g[0] / (g[1] * g[1] + g[2] * g[2]).sqrt().max(1e-6)).atan();
        let roll = (g[1] / (g[0] * g[0] + g[2] * g[2]).sqrt().max(1e-6)).atan();
        let energy: f32 = tail.iter().sum::<f32>() * 0.05;
        let p2p = tail.iter().copied().fold(f32::NEG_INFINITY, f32::max)
            - tail.iter().copied().fold(f32::INFINITY, f32::min);
        let period = periodicity(&buf64);
        let conf_motion = (v / (v + 0.5)).clamp(0.1, 1.0) * comp;
        for (name, value) in [
            ("accel.mag_mean", m),
            ("accel.mag_var", v),
            ("accel.zero_crossing_rate", zcr),
            ("accel.dominant_band", band as f32),
            ("accel.jerk_mean", jerk),
            ("accel.pitch", pitch),
            ("accel.roll", roll),
            ("accel.motion_energy", energy),
            ("accel.peak_to_peak", p2p),
            ("accel.periodicity", period),
        ] {
            out.push(self.feat(name, value, conf_motion, comp));
        }
        self.family_counts[0] += 1;

        let env = self.pre_mic_clean();
        let etail: Vec<f32> =
            env.iter().rev().take(SHORT_WIN).copied().collect::<Vec<_>>().into_iter().rev().collect();
        let ecomp = self.mic_env.completeness();
        let em = mean(&etail);
        let ev = variance(&etail, em);
        let estd = ev.sqrt();
        let peak = etail.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let gate = (self.noise_floor + 3.0 * estd.max(1.0)).max(self.noise_floor + 2.0);
        let mut transients: Vec<(usize, f32)> = Vec::new();
        let mut in_t = false;
        let mut t_start = 0usize;
        let mut t_peak = 0.0f32;
        for (i, &x) in etail.iter().enumerate() {
            if !in_t && x > gate {
                in_t = true;
                t_start = i;
                t_peak = x;
            } else if in_t {
                if x > t_peak {
                    t_peak = x;
                }
                if x < gate {
                    transients.push((t_start, t_peak));
                    in_t = false;
                }
            }
        }
        let tcount = transients.len() as f32;
        let mut rise = 0.0f32;
        let mut decay = 0.0f32;
        let mut gaps: Vec<f32> = Vec::new();
        let mut prev_end = 0usize;
        for (idx, &(s0, p0)) in transients.iter().enumerate() {
            let width = etail.len().saturating_sub(s0).min(8).max(1);
            rise += (p0 - etail[s0]) / width as f32;
            decay += (p0 - gate).max(0.0) / width as f32;
            if idx > 0 {
                gaps.push((s0 - prev_end) as f32);
            }
            prev_end = s0;
        }
        if !transients.is_empty() {
            rise /= transients.len() as f32;
            decay /= transients.len() as f32;
        }
        let gap_mean = if gaps.is_empty() { 0.0 } else { mean(&gaps) };
        let crest = if em > 1e-6 { peak / em } else { 0.0 };
        let impulsiveness = (crest / 8.0).min(2.0);
        let conf_mic = ecomp * (0.4 + 0.6 * (estd / (estd + 10.0)));
        for (name, value) in [
            ("mic.env_mean", em),
            ("mic.env_std", estd),
            ("mic.peak", peak),
            ("mic.rise_time", rise),
            ("mic.decay_time", decay),
            ("mic.transient_count", tcount),
            ("mic.crest_factor", crest),
            ("mic.inter_transient_mean", gap_mean),
            ("mic.impulsiveness", impulsiveness),
        ] {
            out.push(self.feat(name, value, conf_mic, ecomp));
        }
        self.family_counts[1] += 1;

        let trail: Vec<(f64, f64, f32)> = self
            .gps_trail
            .iter()
            .rev()
            .take(SHORT_WIN)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let gq = self.gps_quality();
        let mut disp = 0.0f64;
        let mut heading_changes = 0.0f32;
        let mut last_bearing = 0.0f64;
        let mut first_bearing = true;
        for w in trail.windows(2) {
            let dlat = w[1].0 - w[0].0;
            let dlng = w[1].1 - w[0].1;
            disp += (dlat * dlat + dlng * dlng).sqrt() * 111_320.0;
            let b = dlng.atan2(dlat);
            if !first_bearing {
                heading_changes += (b - last_bearing).abs() as f32;
            }
            first_bearing = false;
            last_bearing = b;
        }
        let win_s = (trail.len().max(1) as f64) * 0.05;
        let disp_rate = (disp / win_s.max(1e-6)) as f32;
        let alts = self.alt.clean();
        let atail: Vec<f32> =
            alts.iter().rev().take(SHORT_WIN).copied().collect::<Vec<_>>().into_iter().rev().collect();
        let am = mean(&atail);
        let avar = variance(&atail, am);
        let arate = if atail.len() > 1 {
            (atail[atail.len() - 1] - atail[0]) / win_s.max(1e-6) as f32
        } else {
            0.0
        };
        let dwell = if disp_rate < 0.5 { 1.0 } else { (1.0 - disp_rate / 5.0).max(0.0) };
        let gcomp = (trail.len() as f32 / SHORT_WIN as f32).min(1.0);
        for (name, value) in [
            ("gps.displacement_rate", disp_rate),
            ("gps.altitude_rate", arate),
            ("gps.dwell_fraction", dwell),
            ("gps.heading_change_rate", heading_changes),
            ("gps.altitude_var", avar),
            ("gps.quality", gq),
        ] {
            out.push(self.feat(name, value, gq.max(0.1) * gcomp.max(0.1), gcomp));
        }
        self.family_counts[2] += 1;

        let total_ms = (self.screen_on_ms + self.screen_off_ms).max(1) as f32;
        let off_frac = self.screen_off_ms as f32 / total_ms;
        let hours = (self.events_seen.max(1) as f32 * 0.05 / 3600.0).max(1.0 / 3600.0);
        let unlocks_ph = self.unlocks as f32 / hours;
        let longest_off_s = self.longest_off_ms as f32 / 1000.0;
        for (name, value) in [
            ("screen.off_fraction", off_frac),
            ("screen.unlocks_per_hour", unlocks_ph),
            ("screen.longest_off_s", longest_off_s),
        ] {
            out.push(self.feat(name, value, 0.9, 1.0));
        }
        self.family_counts[3] += 1;

        let stab = 1.0 / (1.0 + v + estd + avar * 0.01);
        out.push(self.feat("meta.stability_score", stab, 0.8, comp.min(ecomp)));
        let sq = (conf_motion + conf_mic + gq) / 3.0;
        out.push(self.feat("meta.signal_quality", sq, 0.9, 1.0));
        out
    }

    fn pre_mic_clean(&self) -> Vec<f32> {
        self.mic_env.clean()
    }

    fn gps_quality(&self) -> f32 {
        let recent: Vec<f32> = self.gps_trail.iter().rev().take(8).map(|t| t.2).collect();
        if recent.is_empty() {
            return 0.0;
        }
        (1.0 - mean(&recent) / 50.0).clamp(0.0, 1.0)
    }

    fn class(&self, kind: &'static str, label: &'static str, confidence: f32) -> Classification {
        Classification { kind, label, confidence: confidence.clamp(0.0, 1.0) }
    }

    fn get(feats: &[Feature], name: &str) -> f32 {
        feats.iter().find(|f| f.name == name).map(|f| f.value).unwrap_or(0.0)
    }

    fn classify(&self, feats: &[Feature]) -> Vec<Classification> {
        let g = |n: &str| Self::get(feats, n);
        let mag_var = g("accel.mag_var");
        let period = g("accel.periodicity");
        let energy = g("accel.motion_energy");
        let disp = g("gps.displacement_rate");
        let dwell = g("gps.dwell_fraction");
        let gq = g("gps.quality");
        let env = g("mic.env_mean");
        let tcount = g("mic.transient_count");
        let off_frac = g("screen.off_fraction");
        let sq = g("meta.signal_quality");
        let conf = (sq * 0.6 + 0.4).min(1.0);

        let motion = if disp > 3.0 && mag_var > 0.5 {
            "vehicular"
        } else if period > 0.45 && mag_var > 0.15 {
            "walking"
        } else if mag_var < 0.05 && disp < 0.5 {
            "stationary"
        } else {
            "irregular"
        };
        let rest = if energy < 0.6 && off_frac > 0.6 {
            "resting-still"
        } else if energy > 2.0 || off_frac < 0.2 {
            "active"
        } else {
            "uncertain"
        };
        let acoustic = if tcount >= 3.0 {
            "impulsive-dominant"
        } else if env > 55.0 {
            "loud"
        } else if env > 38.0 {
            "moderate"
        } else {
            "quiet"
        };
        let transient = if tcount >= 1.0 { "detected" } else { "none" };
        let alt = g("gps.altitude_rate");
        let terrain = if gq < 0.3 {
            "unknown"
        } else if alt.abs() > 0.8 {
            "highland"
        } else if disp > 3.0 {
            "urban-dense"
        } else if dwell > 0.8 {
            "indoor-likely"
        } else {
            "coastal"
        };
        let pitch = g("accel.pitch").abs();
        let device = if mag_var < 0.03 && pitch > 1.0 {
            "on-table"
        } else if mag_var > 0.3 {
            "in-hand"
        } else {
            "in-pocket"
        };
        let stability = g("meta.stability_score");
        vec![
            self.class("motion_state", motion, conf),
            self.class("rest_state", rest, conf),
            self.class("acoustic_env", acoustic, conf),
            self.class("transient_event", transient, conf),
            self.class("terrain_class", terrain, conf * gq.max(0.2)),
            self.class("device_context", device, conf),
            self.class(
                "stability_band",
                if stability > 0.7 {
                    "stable"
                } else if stability > 0.4 {
                    "mixed"
                } else {
                    "volatile"
                },
                conf,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rafiki_sae::SensorData;

    fn ev(ts: u128, data: SensorData) -> SensorEvent {
        let t = data.sensor_type();
        SensorEvent {
            timestamp_ms: ts,
            sensor_type: t,
            data: data.clone(),
            confidence: rafiki_sae::confidence_for(t, &data),
        }
    }

    #[test]
    fn warms_up_then_computes() {
        let mut p = Processor::new();
        for i in 0..10 {
            let e = ev(i as u128 * 50, SensorData::accel(0.0, 0.0, 9.81));
            let m = ev(i as u128 * 50, SensorData::mic(30.0, 200.0));
            assert!(p.ingest(&e).is_none());
            assert!(p.ingest(&m).is_none());
        }
        let mut got = false;
        for i in 10..40 {
            let e = ev(i as u128 * 50, SensorData::accel(0.0, 0.0, 9.81));
            let m = ev(i as u128 * 50, SensorData::mic(30.0, 200.0));
            p.ingest(&e);
            if p.ingest(&m).is_some() {
                got = true;
            }
        }
        assert!(got);
        assert!(p.windows_computed > 0);
    }

    #[test]
    fn thirty_features_minimum_all_finite() {
        let mut p = Processor::new();
        let mut last = (Vec::new(), Vec::new());
        for i in 0..64 {
            let t = i as f32 * 0.6;
            let e = ev(
                i as u128 * 50,
                SensorData::accel(t.sin() * 1.5, 0.1, 9.81 + t.cos() * 0.8),
            );
            let m = ev(i as u128 * 50, SensorData::mic(45.0, 500.0));
            p.ingest(&e);
            if let Some(out) = p.ingest(&m) {
                last = out;
            }
        }
        assert!(last.0.len() >= 30, "got {} features", last.0.len());
        for f in &last.0 {
            assert!(f.value.is_finite(), "{} not finite", f.name);
            assert!((0.0..=1.0).contains(&f.confidence), "{} conf", f.name);
        }
        assert_eq!(last.1.len(), 7);
    }
}
