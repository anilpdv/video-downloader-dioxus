use crate::views::download::types::{FormatType, Quality};

/// Data transfer object for download requests
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub format_type: FormatType,
    pub quality: Quality,
}

impl DownloadRequest {
    pub fn new(url: String, format_type: FormatType, quality: Quality) -> Self {
        Self {
            url,
            format_type,
            quality,
        }
    }
}

/// Data transfer object for download responses
#[derive(Debug, Clone)]
pub struct DownloadResponse {
    pub data: Vec<u8>,
    pub filename: String,
    pub format_type: FormatType,
}

impl DownloadResponse {
    pub fn new(data: Vec<u8>, filename: String, format_type: FormatType) -> Self {
        Self {
            data,
            filename,
            format_type,
        }
    }
}

/// Progress information for downloads
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub percent: u8,
    pub eta: String,
    pub speed: String,
    pub status: String,
}

impl ProgressInfo {
    pub fn new(percent: u8, eta: String, speed: String, status: String) -> Self {
        Self {
            percent,
            eta,
            speed,
            status,
        }
    }

    pub fn updating() -> Self {
        Self {
            percent: 0,
            eta: "Calculating...".into(),
            speed: "".into(),
            status: "Initializing download...".into(),
        }
    }

    pub fn completed() -> Self {
        Self {
            percent: 100,
            eta: "".into(),
            speed: "".into(),
            status: "Download complete!".into(),
        }
    }

    pub fn failed(error: &str) -> Self {
        Self {
            percent: 0,
            eta: "".into(),
            speed: "".into(),
            status: format!("Download failed: {}", error),
        }
    }
}
