//! Rafiki TSE: twin state engine contracts.
//!
//! The living twin: a compact serializable snapshot of baseline plus
//! open anomalies, kilobytes not megabytes. Research status: contracts
//! only. The twin must round-trip through JSON byte-identically (save,
//! load, compare) before any engine is considered wired.

use rafiki_ade::Anomaly;
use rafiki_be::StreamBaseline;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TwinError {
    #[error("snapshot failed: {0}")]
    Snapshot(String),
    #[error("restore failed: {0}")]
    Restore(String),
}

/// The twin: baselines plus open anomalies plus a schema version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Twin {
    pub schema: u32,
    pub baselines: Vec<StreamBaseline>,
    pub anomalies: Vec<Anomaly>,
    pub updated_ms: u128,
}

impl Twin {
    pub fn empty(updated_ms: u128) -> Self {
        Self { schema: 1, baselines: Vec::new(), anomalies: Vec::new(), updated_ms }
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, TwinError> {
        serde_json::to_vec(self).map_err(|e| TwinError::Snapshot(e.to_string()))
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TwinError> {
        serde_json::from_slice(bytes).map_err(|e| TwinError::Restore(e.to_string()))
    }
    pub fn byte_size(&self) -> usize {
        self.to_bytes().map(|b| b.len()).unwrap_or(0)
    }
}

/// Twin store interface: persist and restore whole twins.
pub trait TwinStore {
    fn save(&mut self, twin: &Twin) -> Result<(), TwinError>;
    fn load(&self) -> Result<Twin, TwinError>;
    fn size_bytes(&self) -> usize;
}
