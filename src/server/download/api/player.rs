//! API for media playback

use dioxus::prelude::*;
use std::path::Path;

use crate::database::models::Download;
use crate::database::schema;
use crate::server::download::player::MediaPlayer;

/// Play a download using the best available player
#[server]
pub async fn play_download(download_id: i64) -> Result<bool, ServerFnError> {
    tracing::info!("Playing download with ID: {}", download_id);

    // Get database connection
    let pool = crate::database::get_database().await?;

    // Fetch the download details
    let download = match schema::get_download_by_id(&pool, download_id).await? {
        Some(download) => download,
        None => {
            tracing::error!("Download not found: {}", download_id);
            return Err(ServerFnError::ServerError("Download not found".to_string()));
        }
    };

    // Check if the file exists
    let file_path = Path::new(&download.file_path);
    if !file_path.exists() {
        tracing::error!("File not found: {:?}", file_path);
        return Err(ServerFnError::ServerError(format!(
            "File not found: {}",
            file_path.display()
        )));
    }

    // Initialize the player with the best available option
    let player = MediaPlayer::new();

    // Play the media using the appropriate player
    match player.play_media(file_path) {
        Ok(_) => {
            tracing::info!("Started playback successfully");
            Ok(true)
        }
        Err(e) => {
            tracing::error!("Failed to play media: {}", e);
            Err(ServerFnError::ServerError(format!(
                "Failed to play media: {}",
                e
            )))
        }
    }
}

/// Get the current player type that will be used
#[server]
pub async fn get_player_info() -> Result<String, ServerFnError> {
    let player = MediaPlayer::new();
    let player_type = player.get_player_info();
    Ok(player_type)
}
