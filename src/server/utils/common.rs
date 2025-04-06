use std::io;
use std::path::{Path, PathBuf};

#[cfg(feature = "server")]
use crate::server::core::download::DownloadProgress;
#[cfg(feature = "server")]
use tokio::fs;
use tracing;

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

    // Check if the line contains download progress information
    if line.contains("[download]") && line.contains("%") {
        // Parse percentage
        if let Some(percent_idx) = line.find('%') {
            if let Some(percent_start) = line[..percent_idx].rfind(' ') {
                if let Ok(percent) = line[percent_start + 1..percent_idx].trim().parse::<f64>() {
                    // Save basic progress information
                    progress.downloaded_bytes = (percent * 100.0) as u64;
                    progress.total_bytes = 10000; // Using 10000 as a base so we can have precise percentages
                    progress.status = format!("Downloading: {:.1}%", percent);

                    // Parse size information if available (e.g., "23.5MiB of 50.3MiB")
                    if let Some(of_idx) = line.find(" of ") {
                        if let Some(at_idx) = line[of_idx..].find(" at ") {
                            let size_str = &line[of_idx + 4..of_idx + at_idx];
                            if let Some(size_bytes) = parse_size(size_str) {
                                progress.total_bytes = size_bytes;

                                // Calculate downloaded bytes based on percentage
                                progress.downloaded_bytes =
                                    ((percent / 100.0) * progress.total_bytes as f64) as u64;

                                // Try to extract download speed
                                if let Some(at_idx) = line.find(" at ") {
                                    if let Some(eta_idx) = line[at_idx..].find(" ETA ") {
                                        let speed_str = &line[at_idx + 4..at_idx + eta_idx];

                                        // Update status with more information
                                        progress.status = format!(
                                            "Downloading: {:.1}% at {}",
                                            percent, speed_str
                                        );
                                    }
                                }
                            }
                        }
                    }

                    // Try to parse ETA
                    if let Some(eta_idx) = line.find(" ETA ") {
                        let eta_str = line[eta_idx + 5..].trim();
                        progress.eta_seconds = parse_eta(eta_str).unwrap_or(0);

                        // If we couldn't parse the ETA but have a string, set a default
                        if progress.eta_seconds == 0 && !eta_str.is_empty() {
                            // Set a default based on the percentage
                            // Higher percentage means less time remaining
                            progress.eta_seconds = ((100.0 - percent) / 10.0) as u64 * 60;
                        }
                    }

                    return Some(progress);
                }
            }
        }
    }
    // Check if the line indicates a post-processing or merging stage
    else if line.contains("Merger")
        || line.contains("ffmpeg")
        || line.contains("Merging formats")
        || line.contains("Post-process")
    {
        progress.status = "Processing video...".to_string();
        progress.downloaded_bytes = 85; // Post-processing is approximately 85-95% of the total process
        progress.total_bytes = 100;
        progress.eta_seconds = 30; // Default ETA for processing
        return Some(progress);
    }
    // Check if the line indicates writing to disk (final stage)
    else if line.contains("Writing video thumbnail")
        || line.contains("has already been downloaded")
        || line.contains("Deleting original file")
    {
        progress.status = "Finalizing download...".to_string();
        progress.downloaded_bytes = 95; // Final stages are 95-99% complete
        progress.total_bytes = 100;
        progress.eta_seconds = 5;
        return Some(progress);
    }
    // Check for destination output (download complete)
    else if line.contains("Destination:") || line.contains("[ExtractAudio]") {
        progress.status = "Download complete, processing file...".to_string();
        progress.downloaded_bytes = 90;
        progress.total_bytes = 100;
        progress.eta_seconds = 10;
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
