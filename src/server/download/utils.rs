use std::io;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing;

use super::types::DownloadProgress;

#[cfg(feature = "server")]
pub async fn find_downloaded_file(dir: impl AsRef<Path>) -> io::Result<PathBuf> {
    let dir_path = dir.as_ref();
    let mut entries = fs::read_dir(dir_path).await?;

    tracing::info!(
        "Scanning directory {} for downloaded files",
        dir_path.display()
    );

    // First try to find any media file
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        tracing::info!("Found file: {:?}", path);

        if path.is_file() {
            // Check the file extension for media types
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                // Add more audio formats to the list
                if [
                    "mp4", "mp3", "m4a", "webm", "mkv", "opus", "ogg", "wav", "aac", "flac",
                ]
                .contains(&ext_str.as_str())
                {
                    // Get file size for logging
                    if let Ok(metadata) = fs::metadata(&path).await {
                        tracing::info!(
                            "Found media file: {} ({} bytes)",
                            path.file_name().unwrap_or_default().to_string_lossy(),
                            metadata.len()
                        );
                    }
                    return Ok(path);
                }
            }
        }
    }

    // Try again to find ANY file if we didn't find a media file
    let mut entries = fs::read_dir(dir_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            tracing::info!(
                "Falling back to non-media file: {}",
                path.file_name().unwrap_or_default().to_string_lossy(),
            );
            return Ok(path);
        }
    }

    tracing::error!("No files found in directory: {}", dir_path.display());
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No downloaded file found",
    ))
}

#[cfg(feature = "server")]
pub fn parse_progress_line(line: &str) -> Option<DownloadProgress> {
    let mut progress = DownloadProgress::default();

    // Check if the line contains progress information
    if line.starts_with("[download]") && line.contains("%") {
        // Parse percentage
        if let Some(percent_idx) = line.find('%') {
            if let Some(percent_start) = line[..percent_idx].rfind(' ') {
                if let Ok(percent) = line[percent_start + 1..percent_idx].trim().parse::<f64>() {
                    progress.status = format!("Downloading: {:.1}%", percent);

                    // Try to parse the file size
                    if let Some(of_idx) = line.find(" of ") {
                        if let Some(size_end) = line[of_idx + 4..].find(' ') {
                            let size_str = line[of_idx + 4..of_idx + 4 + size_end].trim();
                            progress.total_bytes = parse_size(size_str).unwrap_or(0);
                            progress.downloaded_bytes =
                                ((percent / 100.0) * progress.total_bytes as f64) as u64;
                        }
                    }

                    // Try to parse ETA
                    if let Some(eta_idx) = line.find(" ETA ") {
                        let eta_str = line[eta_idx + 5..].trim();
                        progress.eta_seconds = parse_eta(eta_str).unwrap_or(0);
                    }

                    return Some(progress);
                }
            }
        }
    } else if line.contains("Merger") || line.contains("ffmpeg") {
        progress.status = "Processing video...".to_string();
        progress.downloaded_bytes = progress.total_bytes; // Assume download is complete
        return Some(progress);
    }

    None
}

#[cfg(feature = "server")]
pub fn parse_size(size_str: &str) -> Option<u64> {
    let mut num_str = String::new();
    let mut unit_str = String::new();

    for c in size_str.chars() {
        if c.is_digit(10) || c == '.' {
            num_str.push(c);
        } else if c.is_alphabetic() {
            unit_str.push(c);
        }
    }

    match num_str.parse::<f64>() {
        Ok(num) => match unit_str.to_uppercase().as_str() {
            "B" => Some(num as u64),
            "KB" | "KIB" => Some((num * 1024.0) as u64),
            "MB" | "MIB" => Some((num * 1024.0 * 1024.0) as u64),
            "GB" | "GIB" => Some((num * 1024.0 * 1024.0 * 1024.0) as u64),
            _ => None,
        },
        Err(_) => None,
    }
}

#[cfg(feature = "server")]
pub fn parse_eta(eta_str: &str) -> Option<u64> {
    let parts: Vec<&str> = eta_str.split(':').collect();

    match parts.len() {
        2 => {
            // MM:SS format
            match (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                (Ok(minutes), Ok(seconds)) => Some(minutes * 60 + seconds),
                _ => None,
            }
        }
        3 => {
            // HH:MM:SS format
            match (
                parts[0].parse::<u64>(),
                parts[1].parse::<u64>(),
                parts[2].parse::<u64>(),
            ) {
                (Ok(hours), Ok(minutes), Ok(seconds)) => {
                    Some(hours * 3600 + minutes * 60 + seconds)
                }
                _ => None,
            }
        }
        _ => None,
    }
}
