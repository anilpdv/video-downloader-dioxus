use dioxus::prelude::*;
use serde_json;
use server_fn::error::NoCustomError;
use tracing;

#[cfg(feature = "server")]
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

/// Get video info without downloading
#[server(GetVideoInfo)]
pub async fn get_video_info(url: String) -> Result<String, ServerFnError<NoCustomError>> {
    tracing::info!("Getting video info for: {}", url);

    #[cfg(feature = "server")]
    {
        let output = match YoutubeDl::new(url).run_async().await.map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!("Error fetching video info: {}", e))
        })? {
            YoutubeDlOutput::SingleVideo(video) => video,
            YoutubeDlOutput::Playlist(_) => {
                return Err(ServerFnError::<NoCustomError>::ServerError(
                    "URL points to a playlist, not a single video".to_string(),
                ));
            }
        };

        // Convert the video info to JSON
        let json_str = serde_json::to_string(&output).map_err(|e| {
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
