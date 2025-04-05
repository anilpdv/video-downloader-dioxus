use dioxus::prelude::*;
use serde_json;
use server_fn::error::NoCustomError;
use std::time::Duration;

#[cfg(feature = "server")]
use tokio::time::timeout;
use tracing;

#[cfg(feature = "server")]
use youtube_dl::{SearchOptions, YoutubeDl};

/// Search YouTube videos
#[server(SearchYoutube)]
pub async fn search_youtube(query: String) -> Result<String, ServerFnError<NoCustomError>> {
    tracing::info!("Searching YouTube for: {}", query);

    #[cfg(feature = "server")]
    {
        // Create search options for YouTube
        let search_options = SearchOptions::youtube(query).with_count(10); // Get 10 results

        // Wrap the search in an async block for timeout
        let search_future = async {
            // Run the search with timeout settings
            let mut dl = YoutubeDl::search_for(&search_options);
            dl.socket_timeout("20"); // Set 20-second socket timeout
            dl.extra_arg("--flat-playlist"); // Skip extracting detailed video info

            dl.run_async().await.map_err(|e| {
                ServerFnError::<NoCustomError>::ServerError(format!("Error searching: {}", e))
            })
        };

        // Apply a timeout of 30 seconds to avoid hanging
        let output = match timeout(Duration::from_secs(30), search_future).await {
            Ok(result) => result?,
            Err(_) => {
                return Err(ServerFnError::<NoCustomError>::ServerError(
                    "Search operation timed out".to_string(),
                ));
            }
        };

        // Convert the output to JSON
        let json_str = serde_json::to_string(&output).map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Error serializing search results: {}",
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
