use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Structure to track download progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub eta_seconds: u64,
    pub status: String,
}

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

/// Video search result model for communication with client
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VideoSearchResult {
    pub id: String,
    pub url: String,
    pub title: String,
    pub thumbnail_url: String,
    pub duration: String,
    pub channel_name: String,
    pub uploaded_at: Option<String>,
    pub views: String,
}
