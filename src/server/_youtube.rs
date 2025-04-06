use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::error::NoCustomError;

// Video search result model for communication with client
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

// Server function for YouTube search
#[server(SearchYoutubeVideos)]
pub async fn search_youtube_videos(
    query: String,
) -> Result<Vec<VideoSearchResult>, ServerFnError<NoCustomError>> {
    tracing::info!("Searching YouTube for query: {}", query);

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

                    videos.push(VideoSearchResult {
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

        tracing::info!("Videos: {:?}", videos[0]);

        Ok(videos)
    }
}

#[server(DownloadYoutubeVideo)]
pub async fn download_youtube_video(
    video_id: String,
    title: String,
    is_audio: bool,
) -> Result<String, ServerFnError<NoCustomError>> {
    tracing::info!(
        "Download request: video_id={}, title={}, is_audio={}",
        video_id,
        title,
        is_audio
    );

    #[cfg(feature = "server")]
    {
        // Build the URL
        let url = format!("https://www.youtube.com/watch?v={}", video_id);

        // Determine format type
        let format_type = if is_audio { "audio" } else { "video" };
        let quality = "highest";

        // Use your existing download infrastructure
        match crate::server::download::video::download_with_quality(
            url.clone(),
            format_type.to_string(),
            quality.to_string(),
        )
        .await
        {
            Ok(_) => {
                // Re-use your existing success handling logic
                // File paths and database updates are handled in the download_with_quality function
                Ok(format!("Successfully downloaded: {}", title))
            }
            Err(e) => Err(ServerFnError::<NoCustomError>::ServerError(format!(
                "Download failed: {}",
                e
            ))),
        }
    }

    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::<NoCustomError>::ServerError(
            "Server feature not enabled".to_string(),
        ))
    }
}
