use dioxus::prelude::*;
use server_fn::error::NoCustomError;
use tracing;

#[cfg(feature = "server")]
use crate::database::{get_database, models::Download as DbDownload, schema::save_download};

/// Save download info to database
#[cfg(feature = "server")]
pub async fn save_download_info(
    url: &str,
    title: &str,
    filename: &str,
    file_path: &str,
    format_type: &str,
    quality: &str,
    file_size: i64,
) -> Result<(), ServerFnError<NoCustomError>> {
    let video_id = DbDownload::extract_video_id(url);

    // Generate thumbnail URL if video ID is available
    let thumbnail_url = video_id
        .as_ref()
        .map(|id| DbDownload::generate_thumbnail_url(id));

    // Set initial values for the download record
    let download = DbDownload::new(
        url.to_string(),
        Some(title.to_string()),
        filename.to_string(),
        file_path.to_string(),
        format_type.to_string(),
        quality.to_string(),
        Some(file_size),
        thumbnail_url,
        video_id,
        None, // Duration not available
    );

    // Try to save to database
    if let Ok(pool) = get_database().await {
        if let Err(e) = save_download(&pool, &download).await {
            tracing::error!("Failed to save download history: {}", e);
            return Err(ServerFnError::<NoCustomError>::ServerError(format!(
                "Failed to save download history: {}",
                e
            )));
        } else {
            tracing::info!("Saved download history for: {}", title);
        }
    }
    Ok(())
}
