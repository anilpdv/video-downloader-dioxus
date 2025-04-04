use dioxus::prelude::*;
use serde_json;
use server_fn::error::NoCustomError;
use tokio::fs;
use tracing;

use crate::server::download::types::DownloadProgress;

/// Get progress information for an ongoing download
#[server(GetDownloadProgress)]
pub async fn get_download_progress(
    url: String,
) -> Result<(u64, u64, u64, String), ServerFnError<NoCustomError>> {
    tracing::info!("Checking progress for URL: {}", url);

    #[cfg(feature = "server")]
    {
        // Create a unique ID based on the URL to track this specific download
        let progress_id = format!("download_{}", url.len());
        let progress_file = std::env::temp_dir().join(format!("{}.progress", progress_id));

        if progress_file.exists() {
            match fs::read_to_string(&progress_file).await {
                Ok(content) => match serde_json::from_str::<DownloadProgress>(&content) {
                    Ok(progress) => {
                        return Ok((
                            progress.downloaded_bytes,
                            progress.total_bytes,
                            progress.eta_seconds,
                            progress.status,
                        ));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse progress data: {}", e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read progress file: {}", e);
                }
            }
        }

        // Return default values if no progress is found
        Ok((0, 0, 0, "Initializing download...".to_string()))
    }

    #[cfg(not(feature = "server"))]
    Err(ServerFnError::<NoCustomError>::ServerError(
        "Server feature not enabled".to_string(),
    ))
}
