//! Rafiki Tambua: anomaly detection engine.
//!
//! Public re-anchor of the Baraqiel convergence pipeline. Compares live
//! SPE values against baseline state and surfaces anomalies ONLY on
//! multi-signal convergence: deviations co-occurring across at least
//! two independent sensor families within a rolling window. A single
//! deviating stream, however loud, is noise.
//!
//! Scoring: z = |value - baseline_mean| / baseline_std, confidence
//! scaled by baseline stability (low stability reported transparently,
//! never suppressed). Categorical labels score by rarity under a local
//! decay-weighted distribution. The convergence bar scales with
//! trustworthy signal: required features = max(2, ceil(trustworthy /
//! 8)), families spanned at least 2. Derived kinds (classifications,
//! meta) are scored and reported but never vote. A scale-relative
//! epsilon floor kills zero-variance z explosions. Window capped and
//! time-evicted. Memory bounded by window cap and distinct-feature
//! count, never event count. No ML of any kind.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Convergence window span in milliseconds of event time.
pub const WINDOW_MS: u128 = 300_000;
/// Hard cap on window entries: the memory bound, independent of load.
pub const WINDOW_CAP: usize = 4096;
/// Per-feature deviation bar (z units). Structural, not population.
pub const DEVIATION_BAR: f32 = 2.0;

fn family_of(feature: &str) -> &str {
    match feature.split('.').next().unwrap_or("") {
        "accel" => "accel",
        "mic" => "mic",
        "gps" => "gps",
        "screen" => "screen",
        "meta" => "meta",
        other => other,
    }
}

/// Independent evidence for convergence: the four sensor families
/// only. Classifications and meta derive from those streams, so
/// counting them as separate signals would let one loud sensor plus
/// its own echoes pass as convergence.
fn counts_as_evidence(feature: &str) -> bool {
    matches!(family_of(feature), "accel" | "mic" | "gps" | "screen")
}

/// Numerical hygiene floor: with near-zero baseline variance any
/// microscopic floating move explodes into a huge z. Same epsilon for
/// every feature; far below any real signal.
fn clears_floor(mean: f32, value: f32) -> bool {
    (value - mean).abs() > 1e-4 * mean.abs().max(1.0)
}

/// Per-feature deviation state, continuously inspectable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviationScore {
    pub feature: String,
    pub deviation: f32,
    pub confidence: f32,
    pub stable: bool,
}

/// Internal convergence record: what co-deviated, how far, aggregate
/// confidence, and the window it was judged over.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceEvent {
    pub features: Vec<String>,
    pub deviations: Vec<DeviationScore>,
    pub confidence: f32,
    pub timestamp_ms: u128,
    pub window_ms: u128,
}

/// Dispatched anomaly: convergence only, never single-signal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnomalyDetected {
    pub features: Vec<DeviationScore>,
    pub confidence: f32,
    pub timestamp_ms: u128,
    pub window_ms: u128,
}

/// Baseline view per feature, fed from the baseline engine.
#[derive(Debug, Clone, Default)]
pub struct BaselineView {
    pub mean: f32,
    pub variance: f32,
    pub stability: f32,
}

#[derive(Debug, Clone, Default)]
struct CategoryView {
    counts: HashMap<String, f32>,
}

impl CategoryView {
    fn observe(&mut self, label: &str, tau: f32) {
        let w = 1.0 - 1.0 / tau.max(1.0);
        for v in self.counts.values_mut() {
            *v *= w;
        }
        *self.counts.entry(label.to_string()).or_insert(0.0) += 1.0;
    }
    fn rarity(&self, label: &str) -> f32 {
        let total: f32 = self.counts.values().sum();
        if total <= 0.0 {
            return 0.5;
        }
        1.0 - self.counts.get(label).copied().unwrap_or(0.0) / total
    }
}

#[derive(Debug, Clone)]
struct WindowEntry {
    timestamp_ms: u128,
    feature: String,
    deviation: f32,
    confidence: f32,
    mean: f32,
    value: f32,
}

/// The engine: baseline views, live deviation state, rolling
/// convergence window, fired anomalies.
#[derive(Debug, Default)]
pub struct TambuaEngine {
    baselines: HashMap<String, BaselineView>,
    categories: HashMap<String, CategoryView>,
    window: VecDeque<WindowEntry>,
    pub anomalies_fired: u64,
    window_ms: u128,
}

impl TambuaEngine {
    pub fn new() -> Self {
        Self { window_ms: WINDOW_MS, ..Self::default() }
    }

    pub fn with_window_ms(window_ms: u128) -> Self {
        Self { window_ms, ..Self::default() }
    }

    pub fn absorb_baseline(&mut self, feature: &str, mean: f32, variance: f32, stability: f32) {
        self.baselines.insert(
            feature.to_string(),
            BaselineView { mean, variance: variance.max(0.0), stability: stability.clamp(0.0, 1.0) },
        );
    }

    /// Score one live feature value. Returns its deviation state and
    /// appends any fired anomaly to out. Low stability is reported
    /// transparently via confidence, never suppressed.
    pub fn score_feature(
        &mut self,
        timestamp_ms: u128,
        feature: &str,
        value: f32,
        out: &mut Vec<AnomalyDetected>,
    ) -> DeviationScore {
        let (deviation, confidence, stable, mean) = match self.baselines.get(feature) {
            Some(b) => {
                let std = b.variance.sqrt().max(1e-6);
                let z = ((value - b.mean).abs() / std).min(1e3);
                (z, (z / (z + 2.0)) * b.stability, b.stability >= 0.66, b.mean)
            }
            None => (0.0, 0.0, false, 0.0),
        };
        let score =
            DeviationScore { feature: feature.to_string(), deviation, confidence, stable };
        if deviation > DEVIATION_BAR {
            self.push_window(timestamp_ms, feature, deviation, confidence, mean, value);
            self.judge(timestamp_ms, out);
        }
        self.evict_old(timestamp_ms);
        score
    }

    /// Score one live categorical label by rarity under the locally
    /// tracked decay-weighted distribution.
    pub fn score_class(
        &mut self,
        timestamp_ms: u128,
        kind: &str,
        label: &str,
        out: &mut Vec<AnomalyDetected>,
    ) -> DeviationScore {
        let dist = self.categories.entry(kind.to_string()).or_default();
        let rarity = dist.rarity(label);
        dist.observe(label, 500.0);
        let deviation = rarity * 4.0;
        let confidence = rarity * 0.7;
        let score = DeviationScore {
            feature: kind.to_string(),
            deviation,
            confidence,
            stable: false,
        };
        if deviation > DEVIATION_BAR {
            self.push_window(timestamp_ms, kind, deviation, confidence, f32::NAN, f32::NAN);
            self.judge(timestamp_ms, out);
        }
        self.evict_old(timestamp_ms);
        score
    }

    fn push_window(
        &mut self,
        timestamp_ms: u128,
        feature: &str,
        deviation: f32,
        confidence: f32,
        mean: f32,
        value: f32,
    ) {
        if self.window.len() >= WINDOW_CAP {
            self.window.pop_front();
        }
        self.window.push_back(WindowEntry {
            timestamp_ms,
            feature: feature.to_string(),
            deviation,
            confidence,
            mean,
            value,
        });
    }

    fn evict_old(&mut self, now_ms: u128) {
        let cutoff = now_ms.saturating_sub(self.window_ms);
        while self.window.front().map(|e| e.timestamp_ms < cutoff).unwrap_or(false) {
            self.window.pop_front();
        }
    }

    /// Trustworthy feature count: baselines at high stability band.
    pub fn trustworthy_count(&self) -> usize {
        self.baselines.values().filter(|b| b.stability >= 0.66).count()
    }

    /// Required deviating features: scales with trustworthy signal.
    pub fn required_count(&self) -> usize {
        (self.trustworthy_count().max(1) as f32 / 8.0).ceil().max(2.0) as usize
    }

    fn judge(&mut self, now_ms: u128, out: &mut Vec<AnomalyDetected>) {
        self.evict_old(now_ms);
        let mut by_feature: HashMap<&str, (f32, f32)> = HashMap::new();
        for e in &self.window {
            if !counts_as_evidence(&e.feature) || !clears_floor(e.mean, e.value) {
                continue;
            }
            let slot = by_feature.entry(e.feature.as_str()).or_insert((0.0, 0.0));
            if e.deviation > slot.0 {
                *slot = (e.deviation, e.confidence);
            }
        }
        let need = self.required_count();
        if by_feature.len() < need {
            return;
        }
        let families: std::collections::HashSet<&str> =
            by_feature.keys().map(|f| family_of(f)).collect();
        if families.len() < 2 {
            return;
        }
        let deviations: Vec<DeviationScore> = by_feature
            .iter()
            .map(|(f, (d, c))| DeviationScore {
                feature: (*f).to_string(),
                deviation: *d,
                confidence: *c,
                stable: true,
            })
            .collect();
        let confidence =
            (deviations.iter().map(|d| d.confidence).sum::<f32>() / deviations.len().max(1) as f32)
                .min(1.0);
        let features: Vec<String> = deviations.iter().map(|d| d.feature.clone()).collect();
        let _ = ConvergenceEvent {
            features: features.clone(),
            deviations: deviations.clone(),
            confidence,
            timestamp_ms: now_ms,
            window_ms: self.window_ms,
        };
        self.anomalies_fired += 1;
        out.push(AnomalyDetected { features: deviations, confidence, timestamp_ms: now_ms, window_ms: self.window_ms });
        self.window.clear();
    }

    pub fn deviation_state(&self, live: &HashMap<String, f32>) -> Vec<DeviationScore> {
        let mut out = Vec::new();
        for (name, value) in live {
            if let Some(b) = self.baselines.get(name) {
                let std = b.variance.sqrt().max(1e-6);
                let z = ((value - b.mean).abs() / std).min(1e3);
                out.push(DeviationScore {
                    feature: name.clone(),
                    deviation: z,
                    confidence: (z / (z + 2.0)) * b.stability,
                    stable: b.stability >= 0.66,
                });
            }
        }
        out
    }

    pub fn window_len(&self) -> usize {
        self.window.len()
    }
    pub fn baseline_count(&self) -> usize {
        self.baselines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rafiki_spe::Feature;

    fn engine_with_baseline() -> TambuaEngine {
        let mut e = TambuaEngine::new();
        e.absorb_baseline("mic.env_mean", 40.0, 25.0, 0.9);
        e.absorb_baseline("accel.mag_var", 0.2, 0.01, 0.9);
        e.absorb_baseline("gps.displacement_rate", 0.1, 0.01, 0.9);
        e
    }

    #[test]
    fn z_scoring_uses_baseline_variance() {
        let mut e = engine_with_baseline();
        let mut out = Vec::new();
        let near = e.score_feature(0, "mic.env_mean", 42.0, &mut out);
        let far = e.score_feature(0, "mic.env_mean", 70.0, &mut out);
        assert!(near.deviation < 1.0);
        assert!(far.deviation > DEVIATION_BAR);
        assert!(far.confidence > near.confidence);
    }

    #[test]
    fn single_spike_never_fires() {
        let mut e = engine_with_baseline();
        let mut out = Vec::new();
        for i in 0..50 {
            e.score_feature(i * 1000, "mic.env_mean", 90.0, &mut out);
        }
        assert_eq!(e.anomalies_fired, 0);
        assert!(out.is_empty());
    }

    #[test]
    fn multi_signal_spike_fires_with_contributors() {
        let mut e = engine_with_baseline();
        let mut out = Vec::new();
        for i in 0..10 {
            let t = i * 1000;
            e.score_feature(t, "mic.env_mean", 90.0, &mut out);
            e.score_feature(t, "accel.mag_var", 5.0, &mut out);
            e.score_feature(t, "gps.displacement_rate", 8.0, &mut out);
        }
        assert!(e.anomalies_fired > 0);
        assert!(!out.is_empty());
        let names: Vec<String> =
            out[0].features.iter().map(|d| d.feature.clone()).collect();
        assert!(names.iter().any(|n| n.starts_with("mic.")));
        assert!(names.iter().any(|n| n.starts_with("accel.")));
    }

    #[test]
    fn required_count_scales_with_trust() {
        let mut e = TambuaEngine::new();
        assert_eq!(e.required_count(), 2);
        for i in 0..40 {
            e.absorb_baseline(&format!("f{i}"), 1.0, 0.01, 0.95);
        }
        assert!(e.required_count() > 2);
    }

    #[test]
    fn categorical_rarity_path() {
        let mut e = TambuaEngine::new();
        let mut out = Vec::new();
        for i in 0..60 {
            e.score_class(i * 1000, "motion_state", "stationary", &mut out);
        }
        let novel = e.score_class(61_000, "motion_state", "vehicular", &mut out);
        let known = e.score_class(62_000, "motion_state", "stationary", &mut out);
        assert!(novel.deviation > known.deviation);
    }

    #[test]
    fn window_is_bounded() {
        let mut e = TambuaEngine::with_window_ms(10_000);
        e.absorb_baseline("mic.env_mean", 40.0, 25.0, 0.9);
        let mut out = Vec::new();
        for i in 0..(WINDOW_CAP + 500) {
            e.score_feature(i as u128 * 1000, "mic.env_mean", 90.0, &mut out);
        }
        assert!(e.window_len() <= WINDOW_CAP);
        assert_eq!(e.anomalies_fired, 0);
    }
}
