//! Rafiki BE: baseline engine contracts.
//!
//! The baseline engine keeps an online adaptive personal baseline: what
//! this person's signals normally look like, updated continuously, never
//! batch-retrained. Research status: contracts only. The cold-start
//! question (minimum collection period before output is trustworthy,
//! derived from the data, never hardcoded) is answered here first.

use rafiki_spe::{Classification, Feature};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BaselineError {
    #[error("baseline is still cold: {collected}/{required} samples")]
    Cold { collected: u64, required: u64 },
    #[error("no baseline for stream {0}")]
    UnknownStream(String),
}

/// A personal baseline for one feature stream: running mean and
/// variance plus the sample count behind them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamBaseline {
    pub feature: String,
    pub mean: f64,
    pub variance: f64,
    pub samples: u64,
    pub trusted: bool,
}

impl StreamBaseline {
    pub fn cold(feature: &str) -> Self {
        Self {
            feature: feature.to_string(),
            mean: 0.0,
            variance: 0.0,
            samples: 0,
            trusted: false,
        }
    }
    /// Welford online update. Returns the z-distance of the new sample
    /// from the prior baseline (before absorbing it).
    pub fn observe(&mut self, value: f32) -> f64 {
        let prior_mean = self.mean;
        let prior_var = self.variance;
        self.samples += 1;
        let n = self.samples as f64;
        let delta = value as f64 - self.mean;
        self.mean += delta / n;
        self.variance += delta * (value as f64 - self.mean);
        let std = (prior_var / n.max(1.0)).sqrt();
        if std > 1e-9 {
            ((value as f64 - prior_mean) / std).abs()
        } else {
            0.0
        }
    }
}

/// Baseline engine interface. Implementations absorb SPE features and
/// answer deviation queries against the personal baseline.
pub trait BaselineEngine {
    fn absorb(&mut self, features: &[Feature]) -> Result<(), BaselineError>;
    fn deviation(&self, feature: &str, value: f32) -> Result<f64, BaselineError>;
    fn baseline(&self, feature: &str) -> Result<&StreamBaseline, BaselineError>;
    fn trusted_features(&self) -> Vec<String>;
    fn classes_snapshot(&self) -> Vec<Classification>;
}
