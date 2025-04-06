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

/// Search YouTube videos with structured results
#[server(SearchYoutubeVideos)]
pub async fn search_youtube_videos(
    query: String,
) -> Result<
    Vec<crate::server::download::core::types::VideoSearchResult>,
    ServerFnError<NoCustomError>,
> {
    tracing::info!("Searching YouTube for query: {}", query);

    #[cfg(feature = "server")]
    {
        use rusty_ytdl::search::{SearchOptions, SearchType, YouTube};

        let youtube = YouTube::new().map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!("YouTube init error: {}", e))
        })?;

        let search_options = SearchOptions {
            limit: 20,
            search_type: SearchType::Video,
            safe_search: false,
        };
        tracing::info!("Search options: {:?}", search_options);

        let results = youtube
            .search(&query, Some(&search_options))
            .await
            .map_err(|e| {
                ServerFnError::<NoCustomError>::ServerError(format!("Search error: {}", e))
            })?;

        let mut videos = Vec::new();

        for result in results {
            match result {
                rusty_ytdl::search::SearchResult::Video(video) => {
                    let thumbnail_url = video
                        .thumbnails
                        .iter()
                        .find(|t| t.width >= 320)
                        .map(|t| t.url.clone())
                        .unwrap_or_else(|| {
                            video
                                .thumbnails
                                .first()
                                .map(|t| t.url.clone())
                                .unwrap_or_default()
                        });

                    videos.push(crate::server::download::core::types::VideoSearchResult {
                        id: video.id.clone(),
                        url: format!("https://www.youtube.com/watch?v={}", video.id),
                        title: video.title,
                        thumbnail_url,
                        duration: video.duration_raw,
                        channel_name: video.channel.name,
                        uploaded_at: video.uploaded_at,
                        views: format!("{} views", video.views),
                    });
                }
                _ => continue,
            }
        }

        tracing::info!("Videos: {:?}", videos.first());

        Ok(videos)
    }

    #[cfg(not(feature = "server"))]
    Err(ServerFnError::<NoCustomError>::ServerError(
        "Server feature not enabled".to_string(),
    ))
}
