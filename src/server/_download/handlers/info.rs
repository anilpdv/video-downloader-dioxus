use dioxus::prelude::*;
use serde_json;
use server_fn::error::NoCustomError;
use std::time::Duration;

#[cfg(feature = "server")]
use tokio::time::timeout;
use tracing;

#[cfg(feature = "server")]
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

/// Get video info without downloading
#[server(GetVideoInfo)]
pub async fn get_video_info(url: String) -> Result<String, ServerFnError<NoCustomError>> {
    tracing::info!("Getting video info for: {}", url);

    #[cfg(feature = "server")]
    {
        // Set a timeout to prevent hanging indefinitely
        let info_future = async {
            let mut youtube_dl = YoutubeDl::new(&url);

            // Add timeout options to make it faster
            youtube_dl.socket_timeout("30");
            youtube_dl.extra_arg("--no-playlist"); // Skip playlist processing
            youtube_dl.extra_arg("--flat-playlist"); // Don't extract video info for each item

            // Limit to essential fields to speed things up
            youtube_dl.extra_arg("--write-info-json");
            youtube_dl.extra_arg("--skip-download");

            let output = youtube_dl.run_async().await.map_err(|e| {
                ServerFnError::<NoCustomError>::ServerError(format!(
                    "Error fetching video info: {}",
                    e
                ))
            })?;

            match output {
                YoutubeDlOutput::SingleVideo(video) => Ok(video),
                YoutubeDlOutput::Playlist(_) => Err(ServerFnError::<NoCustomError>::ServerError(
                    "URL points to a playlist, not a single video".to_string(),
                )),
            }
        };

        // Apply a timeout of 30 seconds to avoid hanging
        let result = match timeout(Duration::from_secs(30), info_future).await {
            Ok(result) => result,
            Err(_) => {
                return Err(ServerFnError::<NoCustomError>::ServerError(
                    "Timed out while fetching video info".to_string(),
                ));
            }
        }?;

        // Convert the video info to JSON
        let json_str = serde_json::to_string(&result).map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Error serializing video info: {}",
                e
            ))
        })?;

        Ok(json_str)
    }

    #[cfg(not(feature = "server"))]
    Err(ServerFnError::<NoCustomError>::ServerError(
        "Server feature not enabled".to_string(),
    ))
}
