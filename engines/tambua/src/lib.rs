//! Rafiki Tambua: baseline engine.
//!
//! Public re-anchor of the Penemue baseline pipeline. Per-feature,
//! per-individual running baselines over SPE output: decay-weighted
//! Welford statistics for numerics, decay-weighted frequency
//! distributions for categoricals, and a stability score derived from
//! the standard error of the mean. No batch retraining, no population
//! averages, no "ready after N days" cutoff.
//!
//! Stability derivation: S = n / (n + K * (1 + CV)), with CV the
//! coefficient of variation (std / |mean|). It grows with evidence,
//! rises faster for low-natural-variance features at equal sample
//! counts, and lives in [0, 1). Downstream weights by S continuously.

use rafiki_spe::{Classification, Feature};
use std::collections::HashMap;
/// Softness constant for the stability curve. A curve shape shared by
/// every feature, not a readiness threshold.
pub const STABILITY_K: f32 = 50.0;
/// Decay time constant in samples. Old observations weight down so the
/// baseline tracks drift instead of freezing into history.
pub const DECAY_TAU: f32 = 500.0;

/// Stability from effective count and coefficient of variation. Pure
/// function of running statistics; no thresholds, no clocks.
pub fn stability(effective_count: f32, mean: f32, variance: f32) -> f32 {
    let std = variance.max(0.0).sqrt();
    let cv = std / (mean.abs() + 1e-6);
    (effective_count / (effective_count + STABILITY_K * (1.0 + cv))).clamp(0.0, 1.0)
}

/// Stability band for change reporting: 0 low, 1 mid, 2 high.
pub fn stability_band(s: f32) -> u8 {
    if s < 0.33 {
        0
    } else if s < 0.66 {
        1
    } else {
        2
    }
}

/// Decay-weighted Welford state. Constant memory: three scalars plus
/// config, regardless of history length.
#[derive(Debug, Clone)]
pub struct WelfordDecayed {
    n_eff: f32,
    mean: f32,
    m2: f32,
    tau: f32,
}

impl WelfordDecayed {
    pub fn new() -> Self {
        Self {
            n_eff: 0.0,
            mean: 0.0,
            m2: 0.0,
            tau: DECAY_TAU,
        }
    }
    pub fn observe(&mut self, x: f32) {
        let w = 1.0 - 1.0 / self.tau.max(1.0);
        self.n_eff = w * self.n_eff + 1.0;
        let delta = x - self.mean;
        self.mean += delta / self.n_eff.max(1e-9);
        self.m2 = w * self.m2 + delta * (x - self.mean);
        if !self.mean.is_finite() {
            self.mean = 0.0;
        }
        if !self.m2.is_finite() || self.m2 < 0.0 {
            self.m2 = 0.0;
        }
    }
    pub fn variance(&self) -> f32 {
        if self.n_eff <= 0.0 {
            0.0
        } else {
            (self.m2 / self.n_eff).max(0.0)
        }
    }
    pub fn stability(&self) -> f32 {
        stability(self.n_eff, self.mean, self.variance())
    }
    pub fn mean(&self) -> f32 {
        self.mean
    }
    pub fn effective_count(&self) -> f32 {
        self.n_eff
    }
}

impl Default for WelfordDecayed {
    fn default() -> Self {
        Self::new()
    }
}

/// Decay-weighted categorical distribution. Constant memory in the
/// number of observed categories, never in event count.
#[derive(Debug, Clone, Default)]
pub struct CategoryDist {
    counts: HashMap<String, f32>,
    tau: f32,
}

impl CategoryDist {
    pub fn with_tau(tau: f32) -> Self {
        Self {
            counts: HashMap::new(),
            tau,
        }
    }
    pub fn observe(&mut self, category: &str) {
        let w = 1.0 - 1.0 / self.tau.max(1.0);
        for v in self.counts.values_mut() {
            *v *= w;
        }
        *self.counts.entry(category.to_string()).or_insert(0.0) += 1.0;
    }
    pub fn effective_count(&self) -> f32 {
        self.counts.values().sum()
    }
    pub fn stability(&self) -> f32 {
        stability(self.effective_count(), 1.0, 0.0)
    }
    pub fn leader(&self) -> Option<String> {
        self.counts
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k.clone())
    }
}

/// One published baseline update: which feature moved, its stability.
/// The engine emits these only on meaningful shifts (mean moved beyond
/// a quarter std, or stability band changed), never per sample.
#[derive(Debug, Clone)]
pub struct BaselineUpdate {
    pub feature: String,
    pub mean: f32,
    pub stability: f32,
}

/// The engine: per-feature baselines over SPE output.
/// The engine: per-feature baselines over SPE output. Implements the
/// `rafiki_be::BaselineEngine` contract (see the impl block below), so
/// `be/` stays the interface definition and this crate the implementation.
#[derive(Debug, Default)]
pub struct TambuaEngine {
    numerics: HashMap<String, (WelfordDecayed, f32, u8)>,
    categories: HashMap<String, CategoryDist>,
    snapshots: HashMap<String, rafiki_be::StreamBaseline>,
    last_classes: Vec<Classification>,
    pub updates_absorbed: u64,
    pub publishes: u64,
}

impl TambuaEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn absorb_features(&mut self, feats: &[Feature], out: &mut Vec<BaselineUpdate>) {
        for f in feats {
            let entry = self
                .numerics
                .entry(f.name.to_string())
                .or_insert_with(|| (WelfordDecayed::new(), 0.0, 0));
            entry.0.observe(f.value);
            let mean = entry.0.mean();
            let var = entry.0.variance();
            let stab = entry.0.stability();
            let std = var.max(0.0).sqrt();
            if (mean - entry.1).abs() > 0.25 * std.max(1e-6) || stability_band(stab) != entry.2 {
                entry.1 = mean;
                entry.2 = stability_band(stab);
                self.publishes += 1;
                out.push(BaselineUpdate {
                    feature: f.name.to_string(),
                    mean,
                    stability: stab,
                });
            }
            self.updates_absorbed += 1;
            self.snapshots.insert(
                f.name.to_string(),
                rafiki_be::StreamBaseline {
                    feature: f.name.to_string(),
                    mean: mean as f64,
                    variance: var as f64,
                    samples: entry.0.effective_count() as u64,
                    trusted: stability_band(stab) == 2,
                },
            );
        }
    }

    pub fn absorb_classes(&mut self, classes: &[Classification], out: &mut Vec<BaselineUpdate>) {
        self.last_classes = classes.to_vec();
        for c in classes {
            let dist = self
                .categories
                .entry(c.kind.to_string())
                .or_insert_with(|| CategoryDist {
                    counts: HashMap::new(),
                    tau: DECAY_TAU,
                });
            let before = dist.leader();
            let band_before = stability_band(dist.stability());
            dist.observe(c.label);
            if before != dist.leader() || band_before != stability_band(dist.stability()) {
                self.publishes += 1;
                out.push(BaselineUpdate {
                    feature: c.kind.to_string(),
                    mean: f32::NAN,
                    stability: dist.stability(),
                });
            }
            self.updates_absorbed += 1;
        }
    }

    pub fn numeric_mean(&self, feature: &str) -> Option<f32> {
        self.numerics.get(feature).map(|(w, _, _)| w.mean())
    }

    pub fn numeric_stability(&self, feature: &str) -> Option<f32> {
        self.numerics.get(feature).map(|(w, _, _)| w.stability())
    }

    pub fn numerics_len(&self) -> usize {
        self.numerics.len()
    }

    /// State scales with distinct features, never with event count.
    pub fn footprint_tracks(&self) -> usize {
        self.numerics.len()
            + self
                .categories
                .values()
                .map(|d| d.counts.len())
                .sum::<usize>()
    }
}

impl rafiki_be::BaselineEngine for TambuaEngine {
    fn absorb(&mut self, features: &[Feature]) -> Result<(), rafiki_be::BaselineError> {
        let mut out = Vec::new();
        self.absorb_features(features, &mut out);
        Ok(())
    }

    fn deviation(&self, feature: &str, value: f32) -> Result<f64, rafiki_be::BaselineError> {
        match self.snapshots.get(feature) {
            Some(b) => {
                let std = b.variance.sqrt().max(1e-9);
                Ok(((value as f64 - b.mean).abs() / std).min(1e6))
            }
            None => Err(rafiki_be::BaselineError::UnknownStream(feature.to_string())),
        }
    }

    fn baseline(
        &self,
        feature: &str,
    ) -> Result<&rafiki_be::StreamBaseline, rafiki_be::BaselineError> {
        self.snapshots
            .get(feature)
            .ok_or_else(|| rafiki_be::BaselineError::UnknownStream(feature.to_string()))
    }

    fn trusted_features(&self) -> Vec<String> {
        self.snapshots
            .iter()
            .filter(|(_, b)| b.trusted)
            .map(|(k, _)| k.clone())
            .collect()
    }

    fn classes_snapshot(&self) -> Vec<Classification> {
        self.last_classes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welford_matches_batch_statistics() {
        let xs: Vec<f32> = (1..=200).map(|i| i as f32 * 0.5).collect();
        let mut w = WelfordDecayed {
            n_eff: 0.0,
            mean: 0.0,
            m2: 0.0,
            tau: 1e9,
        };
        for &x in &xs {
            w.observe(x);
        }
        let n = xs.len() as f32;
        let mean = xs.iter().sum::<f32>() / n;
        let var = xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>() / n;
        assert!((w.mean() - mean).abs() < 0.5);
        assert!((w.variance() - var).abs() < 1.0);
    }

    #[test]
    fn decayed_mean_tracks_a_shifted_distribution() {
        let mut w = WelfordDecayed::new();
        for _ in 0..2000 {
            w.observe(10.0);
        }
        assert!((w.mean() - 10.0).abs() < 0.5);
        for _ in 0..2000 {
            w.observe(20.0);
        }
        assert!(w.mean() > 17.0, "mean stuck at {}", w.mean());
    }

    #[test]
    fn tight_variance_stabilizes_faster_than_wide() {
        let mut tight = WelfordDecayed::new();
        let mut wide = WelfordDecayed::new();
        for i in 0..300 {
            let t = (i as f32 * 0.7).sin();
            tight.observe(50.0 + t * 0.2);
            wide.observe(50.0 + t * 12.0);
        }
        assert!(tight.stability() > wide.stability());
    }

    #[test]
    fn engine_absorbs_and_reports() {
        let mut e = TambuaEngine::new();
        let mut out = Vec::new();
        let feats = vec![
            Feature {
                name: "a",
                value: 1.0,
                confidence: 1.0,
                completeness: 1.0,
            },
            Feature {
                name: "b",
                value: 2.0,
                confidence: 1.0,
                completeness: 1.0,
            },
        ];
        for _ in 0..5000 {
            e.absorb_features(&feats, &mut out);
        }
        assert_eq!(e.numerics_len(), 2);
        assert!(e.footprint_tracks() <= 4);
        assert!(!out.is_empty());
        assert_eq!(e.numeric_mean("a"), Some(e.numeric_mean("a").unwrap()));
    }

    #[test]
    fn implements_be_contract() {
        use rafiki_be::BaselineEngine;
        let mut e = TambuaEngine::new();
        let feats = vec![Feature {
            name: "a",
            value: 1.0,
            confidence: 1.0,
            completeness: 1.0,
        }];
        for _ in 0..500 {
            e.absorb(&feats).unwrap();
        }
        let b = e.baseline("a").unwrap();
        assert!((b.mean - 1.0).abs() < 0.1);
        assert!(e.deviation("a", 1.0).unwrap() < e.deviation("a", 10.0).unwrap());
        assert!(e.baseline("nope").is_err());
        assert!(e.deviation("nope", 0.0).is_err());
    }
}
