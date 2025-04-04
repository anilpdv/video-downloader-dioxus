use dioxus::prelude::*;

use serde_json;
use server_fn::error::NoCustomError;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(feature = "server")]
use tokio::fs;
use tracing;
#[cfg(feature = "server")]
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

/// Structure to track download progress
#[cfg(feature = "server")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DownloadProgress {
    downloaded_bytes: u64,
    total_bytes: u64,
    eta_seconds: u64,
    status: String,
}

#[cfg(feature = "server")]
impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            downloaded_bytes: 0,
            total_bytes: 0,
            eta_seconds: 0,
            status: "Initializing...".to_string(),
        }
    }
}

/// Check if yt-dlp is installed and download it if not found
#[cfg(feature = "server")]
async fn ensure_yt_dlp_available() -> Result<PathBuf, ServerFnError<NoCustomError>> {
    tracing::info!("Checking for yt-dlp installation");

    // First check if it's in PATH
    let yt_dlp_check = Command::new("yt-dlp").arg("--version").output();

    match yt_dlp_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            tracing::info!("Found yt-dlp in PATH: {}", version.trim());

            // On Unix-like systems, try to find the actual path using "which"
            #[cfg(not(target_os = "windows"))]
            {
                if let Ok(output) = Command::new("which").arg("yt-dlp").output() {
                    if output.status.success() {
                        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !path_str.is_empty() {
                            let actual_path = PathBuf::from(path_str);
                            tracing::info!("Resolved yt-dlp path: {:?}", actual_path);
                            return Ok(actual_path);
                        }
                    }
                }
            }

            // Either on Windows or if "which" didn't work, just return the command name
            return Ok(PathBuf::from("yt-dlp"));
        }
        Ok(_) | Err(_) => {
            tracing::warn!("yt-dlp not found in PATH, attempting to download it");

            // Try to download yt-dlp to a known location
            let download_dir = std::env::temp_dir().join("youtube_dl_cache");
            std::fs::create_dir_all(&download_dir).map_err(|e| {
                ServerFnError::<NoCustomError>::ServerError(format!(
                    "Failed to create download dir: {}",
                    e
                ))
            })?;

            match youtube_dl::download_yt_dlp(&download_dir).await {
                Ok(path) => {
                    tracing::info!("Downloaded yt-dlp to {:?}", path);
                    // Verify it works
                    let downloaded_check = Command::new(&path).arg("--version").output();

                    match downloaded_check {
                        Ok(output) if output.status.success() => {
                            tracing::info!("Downloaded yt-dlp is working");
                            return Ok(path);
                        }
                        _ => {
                            tracing::error!("Downloaded yt-dlp failed verification");
                            return Err(ServerFnError::<NoCustomError>::ServerError(
                                "Downloaded yt-dlp failed verification. Make sure it has executable permissions.".to_string(),
                            ));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to download yt-dlp: {}", e);
                    return Err(ServerFnError::<NoCustomError>::ServerError(format!(
                        "Failed to download yt-dlp: {}",
                        e
                    )));
                }
            }
        }
    }
}

#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError<NoCustomError>> {
    Ok(input)
}

/// Get progress information for an ongoing download
#[server(GetDownloadProgress)]
pub async fn get_download_progress(
    url: String,
) -> Result<(u64, u64, u64, String), ServerFnError<NoCustomError>> {
    tracing::info!("Checking progress for URL: {}", url);

    #[cfg(feature = "server")]
    {
        // Create a unique ID based on the URL to track this specific download
        let progress_id = format!("download_{}", url.len());
        let progress_file = std::env::temp_dir().join(format!("{}.progress", progress_id));

        if progress_file.exists() {
            match fs::read_to_string(&progress_file).await {
                Ok(content) => match serde_json::from_str::<DownloadProgress>(&content) {
                    Ok(progress) => {
                        return Ok((
                            progress.downloaded_bytes,
                            progress.total_bytes,
                            progress.eta_seconds,
                            progress.status,
                        ));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse progress data: {}", e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read progress file: {}", e);
                }
            }
        }

        // Return default values if no progress is found
        Ok((0, 0, 0, "Initializing download...".to_string()))
    }

    #[cfg(not(feature = "server"))]
    Err(ServerFnError::<NoCustomError>::ServerError(
        "Server feature not enabled".to_string(),
    ))
}

/// Download video with highest quality
#[server(Download)]
pub async fn download_video(url: String) -> Result<Vec<u8>, ServerFnError<NoCustomError>> {
    download_with_quality(url, "video".to_string(), "highest".to_string()).await
}

/// Download with specific options
#[server(DownloadWithOptions)]
pub async fn download_with_options(
    url: String,
    audio_only: bool,
) -> Result<Vec<u8>, ServerFnError<NoCustomError>> {
    tracing::info!(
        "Download options request: URL={}, audio_only={}",
        url,
        audio_only
    );
    if audio_only {
        download_with_quality(url, "audio".to_string(), "highest".to_string()).await
    } else {
        download_with_quality(url, "video".to_string(), "highest".to_string()).await
    }
}

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

/// Search YouTube videos
#[server(SearchYoutube)]
pub async fn search_youtube(query: String) -> Result<String, ServerFnError<NoCustomError>> {
    tracing::info!("Searching YouTube for: {}", query);

    #[cfg(feature = "server")]
    {
        // Create search options for YouTube
        let search_options = youtube_dl::SearchOptions::youtube(query).with_count(10); // Get 10 results

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

#[cfg(feature = "server")]
fn parse_progress_line(line: &str) -> Option<DownloadProgress> {
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
fn parse_size(size_str: &str) -> Option<u64> {
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
fn parse_eta(eta_str: &str) -> Option<u64> {
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

/// Download with specific format_type and quality
#[server(DownloadWithQuality)]
pub async fn download_with_quality(
    url: String,
    format_type: String,
    quality: String,
) -> Result<Vec<u8>, ServerFnError<NoCustomError>> {
    tracing::info!(
        "Download with format: {}, quality: {}, URL: {}",
        format_type,
        quality,
        url
    );

    // Validate URL format
    if !url.contains("youtube.com/watch?v=") && !url.contains("youtu.be/") {
        return Err(ServerFnError::<NoCustomError>::ServerError(
            "Invalid YouTube URL. Please provide a valid YouTube video URL.".to_string(),
        ));
    }

    #[cfg(feature = "server")]
    {
        // Create temporary directory for the download
        let temp_dir = std::env::temp_dir().join(format!("youtube_dl_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Failed to create temp directory: {}",
                e
            ))
        })?;

        let temp_dir_path = temp_dir.to_string_lossy().to_string();
        tracing::info!("Creating temp directory at {:?}", temp_dir_path);

        // Generate a unique ID for this download
        let progress_id = format!("download_{}", url.len());
        let progress_file = std::env::temp_dir().join(format!("{}.progress", progress_id));

        // Initialize progress file
        let initial_progress = DownloadProgress::default();
        if let Ok(json) = serde_json::to_string(&initial_progress) {
            let _ = std::fs::write(&progress_file, json);
        }

        // Configure youtube-dl options based on format type and quality
        let mut youtube_dl = YoutubeDl::new(&url);

        // Set output directory
        youtube_dl.output_directory(&temp_dir_path);

        // Add verbose output for better progress information
        youtube_dl.extra_arg("--verbose");
        youtube_dl.socket_timeout("60");

        // Get video info first to determine estimated size and title
        let mut video_title = String::from("Unknown");
        let mut estimated_size: u64 = 0;

        match youtube_dl.clone().socket_timeout("30").run_async().await {
            Ok(YoutubeDlOutput::SingleVideo(video)) => {
                if let Some(title) = &video.title {
                    video_title = title.clone();
                    tracing::info!("Will download: {:?}", title);

                    // Update progress file with title
                    let mut progress = initial_progress.clone();
                    progress.status = format!("Preparing to download: {}", title);
                    if let Ok(json) = serde_json::to_string(&progress) {
                        let _ = std::fs::write(&progress_file, json);
                    }
                }

                // Try to get filesize from formats
                if let Some(formats) = video.formats {
                    for format in formats {
                        if let Some(size) = format.filesize {
                            // Make sure both values are the same type (u64)
                            let size_u64 = size as u64;
                            if size_u64 > estimated_size {
                                estimated_size = size_u64;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        tracing::info!("Configuring download format");
        // Configure format selection based on format_type and quality
        match format_type.to_lowercase().as_str() {
            "audio" => {
                youtube_dl.extract_audio(true);
                youtube_dl.format("bestaudio");
                youtube_dl.extra_arg("-x"); // Extract audio
                youtube_dl.extra_arg("--audio-format"); // Specify format
                youtube_dl.extra_arg("mp3"); // Force MP3 format
                youtube_dl.extra_arg("--audio-quality"); // Specify quality
                youtube_dl.extra_arg("0"); // Best quality (0=best, 9=worst)
                youtube_dl.output_template("audio");
                tracing::info!("Set up audio download with highest quality (mp3 format)");
            }
            "video" => {
                // Configure video quality
                match quality.to_lowercase().as_str() {
                    "lowest" => {
                        youtube_dl.format("worstvideo[ext=mp4]+worstaudio[ext=m4a]/worst[ext=mp4]");
                        tracing::info!("Set up video download with lowest quality");
                    }
                    "medium" => {
                        youtube_dl.format("bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720][ext=mp4]");
                        tracing::info!("Set up video download with medium quality (720p)");
                    }
                    "highest" | _ => {
                        youtube_dl.format("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]");
                        tracing::info!("Set up video download with highest quality");
                    }
                }
                youtube_dl.output_template("video");
            }
            _ => {
                return Err(ServerFnError::<NoCustomError>::ServerError(
                    "Invalid format type. Please specify 'audio' or 'video'.".to_string(),
                ));
            }
        }

        // Launch a separate task to monitor the download progress by checking file size
        let progress_file_clone = progress_file.clone();
        let temp_dir_clone = temp_dir.clone();
        let video_title_clone = video_title.clone();
        let estimated_size_clone = estimated_size;

        tokio::spawn(async move {
            let start_time = std::time::Instant::now();
            let mut last_size: u64 = 0;
            let mut last_update = std::time::Instant::now();
            let mut stalled_counter = 0;

            loop {
                // Check every 1 second
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // Check if the temp directory still exists (download might have finished)
                if !temp_dir_clone.exists() {
                    break;
                }

                // Calculate current size of all files in the directory
                let mut current_size: u64 = 0;
                let mut is_processing = false;

                // Read directory to find all files
                if let Ok(mut entries) = fs::read_dir(&temp_dir_clone).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.is_file() {
                            // Check if file is a temporary/part file
                            let filename = path.file_name().unwrap_or_default().to_string_lossy();

                            // Check if we're in the processing phase (ffmpeg merging, etc)
                            if filename.contains(".mkv")
                                || filename.contains(".mp4")
                                || filename.contains(".webm")
                                || filename.contains(".mp3")
                            {
                                is_processing = true;
                            }

                            if let Ok(metadata) = fs::metadata(&path).await {
                                current_size += metadata.len();
                            }
                        }
                    }
                } else {
                    // Directory might be gone or inaccessible
                    break;
                }

                // Update progress
                if last_update.elapsed().as_millis() > 500 {
                    // Update at most twice per second
                    // Calculate progress metrics
                    let elapsed_secs = start_time.elapsed().as_secs();
                    let download_speed = if elapsed_secs > 0 {
                        current_size as f64 / elapsed_secs as f64
                    } else {
                        0.0
                    };

                    let mut progress = DownloadProgress::default();

                    if is_processing {
                        // If we're processing (merging video and audio), set a specific status
                        progress.status = "Processing video...".to_string();
                        // Set to 99% to indicate almost done
                        progress.downloaded_bytes = 99;
                        progress.total_bytes = 100;
                        progress.eta_seconds = 0;
                    } else if current_size > 0 {
                        // Normal download progress
                        if estimated_size_clone > 0 {
                            // We have an estimated total size
                            progress.downloaded_bytes = current_size;
                            progress.total_bytes = estimated_size_clone;

                            // Calculate ETA if we have a non-zero download speed
                            if download_speed > 0.0 {
                                let remaining_bytes = if estimated_size_clone > current_size {
                                    estimated_size_clone - current_size
                                } else {
                                    0
                                };
                                let eta_secs = (remaining_bytes as f64 / download_speed) as u64;
                                progress.eta_seconds = eta_secs;
                            }

                            // Calculate percentage
                            let percent =
                                (current_size as f64 / estimated_size_clone as f64 * 100.0) as u64;
                            progress.status =
                                format!("Downloading: {}% of {}", percent, video_title_clone);
                        } else {
                            // No estimated size, just show downloaded size
                            progress.downloaded_bytes = current_size;
                            // Set an arbitrary total that's higher than current to show progress
                            progress.total_bytes = current_size.saturating_add(10 * 1024 * 1024); // Add 10MB
                            progress.status = format!(
                                "Downloading: {:.2} MB of {}",
                                current_size as f64 / (1024.0 * 1024.0),
                                video_title_clone
                            );
                        }

                        // Check if download is stalled
                        if current_size == last_size {
                            stalled_counter += 1;
                            // After 10 seconds with no progress, indicate stalled download
                            if stalled_counter >= 10 {
                                progress.status = format!(
                                    "Download stalled at {:.2} MB...",
                                    current_size as f64 / (1024.0 * 1024.0)
                                );
                            }
                        } else {
                            stalled_counter = 0;
                            last_size = current_size;
                        }
                    } else {
                        // No progress yet
                        progress.status = format!("Starting download of {}...", video_title_clone);
                    }

                    // Write progress to file
                    if let Ok(json) = serde_json::to_string(&progress) {
                        let _ = fs::write(&progress_file_clone, json).await;
                    }

                    last_update = std::time::Instant::now();
                }
            }
        });

        // Execute the download
        tracing::info!("Starting download with yt-dlp...");
        match youtube_dl.download_to_async(&temp_dir).await {
            Ok(()) => tracing::info!("Download completed successfully"),
            Err(e) => {
                tracing::error!("Download error: {}", e);

                // Update progress file with error
                let mut progress = DownloadProgress::default();
                progress.status = format!("Error: {}", e);
                if let Ok(json) = serde_json::to_string(&progress) {
                    let _ = fs::write(&progress_file, json).await;
                }

                // Try to clean up the temp directory
                let _ = fs::remove_dir_all(&temp_dir).await;
                return Err(ServerFnError::<NoCustomError>::ServerError(format!(
                    "Download failed: {}",
                    e
                )));
            }
        }

        // Find the downloaded file
        tracing::info!("Looking for downloaded file in {:?}", temp_dir);
        let downloaded_file = find_downloaded_file(&temp_dir).await.map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Failed to find downloaded file: {}",
                e
            ))
        })?;

        tracing::info!("Found downloaded file: {}", downloaded_file.display());

        // Update progress file with completion status
        let mut progress = DownloadProgress::default();
        progress.status = "Download complete, preparing file...".to_string();
        progress.downloaded_bytes = 100;
        progress.total_bytes = 100;
        if let Ok(json) = serde_json::to_string(&progress) {
            let _ = fs::write(&progress_file, json).await;
        }

        // Read the file content
        tracing::info!("Reading file content");
        let content = fs::read(&downloaded_file).await.map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Failed to read downloaded file: {}",
                e
            ))
        })?;

        // Clean up the temp directory
        tracing::info!("Cleaning up temporary directory");
        let _ = fs::remove_dir_all(&temp_dir).await;

        // Clean up progress file
        let _ = fs::remove_file(&progress_file).await;

        tracing::info!("Downloaded {} bytes successfully", content.len());
        Ok(content)
    }

    #[cfg(not(feature = "server"))]
    Err(ServerFnError::<NoCustomError>::ServerError(
        "Server feature not enabled".to_string(),
    ))
}

#[cfg(feature = "server")]
async fn find_downloaded_file(dir: impl AsRef<Path>) -> io::Result<std::path::PathBuf> {
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
