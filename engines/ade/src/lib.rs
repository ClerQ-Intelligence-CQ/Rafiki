//! Rafiki ADE: anomaly detection contracts.
//!
//! Anomalies surface on multi-signal convergence: a single deviating
//! stream is noise, several deviating together is signal. Research
//! status: contracts only. Thresholds stay adaptive via the baseline;
//! nothing here hardcodes a population cutoff.

use rafiki_be::{BaselineEngine, BaselineError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnomalyError {
    #[error("baseline problem: {0}")]
    Baseline(#[from] BaselineError),
    #[error("need at least {0} streams to judge convergence, got {1}")]
    TooFewStreams(usize, usize),
}

/// One surfaced anomaly: which streams converged, how far, when.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub streams: Vec<String>,
    pub max_deviation: f64,
    pub mean_deviation: f64,
    pub timestamp_ms: u128,
    pub acknowledged: bool,
}

/// Anomaly detector interface over any baseline implementation.
pub trait AnomalyDetector {
    /// Minimum converging streams required (adaptive floor lives in
    /// the implementation, never hardcoded here beyond the default).
    fn convergence_floor(&self) -> usize {
        2
    }
    fn judge(
        &mut self,
        baseline: &mut dyn BaselineEngine,
        features: &[rafiki_spe::Feature],
        timestamp_ms: u128,
    ) -> Result<Option<Anomaly>, AnomalyError>;
    fn open_anomalies(&self) -> Vec<Anomaly>;
    fn acknowledge(&mut self, index: usize) -> bool;
}
