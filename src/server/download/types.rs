use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Structure to track download progress
#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub eta_seconds: u64,
    pub status: String,
}

#[cfg(feature = "server")]
impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            downloaded_bytes: 0,
            total_bytes: 0,
            eta_seconds: 0,
            status: "Initializing...".to_string(),
        }
    }
}
