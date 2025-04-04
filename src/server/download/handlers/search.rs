use dioxus::prelude::*;
use serde_json;
use server_fn::error::NoCustomError;
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

        // Run the search
        let output = YoutubeDl::search_for(&search_options)
            .run_async()
            .await
            .map_err(|e| {
                ServerFnError::<NoCustomError>::ServerError(format!("Error searching: {}", e))
            })?;

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
