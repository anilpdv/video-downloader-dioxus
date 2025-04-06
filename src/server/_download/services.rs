#[cfg(feature = "server")]
use crate::database::{get_database, models::Download, schema::get_all_downloads};

// Convert a Download database model to a DownloadItem DTO
#[cfg(feature = "server")]
pub fn convert_download_to_item(download: Download) -> crate::views::downloads::DownloadItem {
    let file_exists = std::path::Path::new(&download.file_path).exists();

    crate::views::downloads::DownloadItem {
        id: download.id,
        title: download
            .title
            .clone()
            .unwrap_or_else(|| "Untitled download".to_string()),
        filename: download.filename.clone(),
        file_path: download.file_path.clone(),
        format_type: download.format_type.clone(),
        quality: download.quality.clone(),
        file_size: download.file_size,
        duration: download.duration,
        date_downloaded: download.format_date(),
        thumbnail_url: download.thumbnail_url,
        file_exists,
    }
}

// Fetch all downloads from the database
#[cfg(feature = "server")]
pub async fn fetch_downloads() -> Vec<crate::views::downloads::DownloadItem> {
    if let Ok(pool) = get_database().await {
        match get_all_downloads(&pool).await {
            Ok(results) => results.into_iter().map(convert_download_to_item).collect(),
            Err(e) => {
                tracing::error!("Failed to get downloads from database: {}", e);
                Vec::new()
            }
        }
    } else {
        tracing::error!("Failed to get database connection");
        Vec::new()
    }
}

// Empty implementation for non-server builds
#[cfg(not(feature = "server"))]
pub async fn fetch_downloads() -> Vec<crate::views::downloads::DownloadItem> {
    Vec::new()
}

// File operations
#[cfg(feature = "server")]
pub fn open_file(path: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("cmd").args(["/c", "start", "", path]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let _ = Command::new("xdg-open").arg(path).spawn();
    }
}

#[cfg(feature = "server")]
pub fn open_containing_folder(path: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("explorer").args(["/select,", path]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let parent = std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        let _ = Command::new("open").arg(parent).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let parent = std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        let _ = Command::new("xdg-open").arg(parent).spawn();
    }
}

// Empty implementations for non-server builds
#[cfg(not(feature = "server"))]
pub fn open_file(_: &str) {}

#[cfg(not(feature = "server"))]
pub fn open_containing_folder(_: &str) {}
