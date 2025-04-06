use dioxus::prelude::*;
use serde_json;
use server_fn::error::NoCustomError;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "server")]
use tokio::fs;
use tracing;

#[cfg(feature = "server")]
use super::database::save_download_info;
#[cfg(feature = "server")]
use crate::server::download::{storage, types::DownloadProgress, utils};

#[cfg(feature = "server")]
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

/// Download video with highest quality
#[server(DownloadVideo)]
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
        // Create a unique ID and progress file immediately
        let progress_id = format!("download_{}", url.len());
        let progress_file = std::env::temp_dir().join(format!("{}.progress", progress_id));

        // Initialize progress file with 0% progress right away - THIS IS CRITICAL
        let mut initial_progress = DownloadProgress::default();
        initial_progress.status = "Initializing download...".to_string();
        initial_progress.downloaded_bytes = 0; // Start at 0%
        initial_progress.total_bytes = 100; // Set to 100 for percentage calculation
        initial_progress.eta_seconds = 0;
        if let Ok(json) = serde_json::to_string(&initial_progress) {
            let _ = std::fs::write(&progress_file, json);
        }

        // Create temporary directory for the download - this should be fast
        let temp_dir = std::env::temp_dir().join(format!("youtube_dl_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Failed to create temp directory: {}",
                e
            ))
        })?;

        let temp_dir_path = temp_dir.to_string_lossy().to_string();
        tracing::info!("Created temp directory at {:?}", temp_dir_path);

        // Store URL for background tasks
        let url_str = url.clone();

        // Start a task to update the progress file periodically during initialization
        tokio::spawn({
            let progress_file = progress_file.clone();
            async move {
                // Update progress every 500ms while we initialize (show 0-5% progress)
                for i in 0..30 {
                    // timeout after 15 seconds
                    let mut progress = DownloadProgress::default();
                    let percent = ((i as f64) / 30.0 * 5.0).min(5.0) as u64; // Max 5% during init
                    progress.downloaded_bytes = percent;
                    progress.total_bytes = 100;
                    progress.status = format!("Preparing download ({}s)...", i / 2);
                    if let Ok(json) = serde_json::to_string(&progress) {
                        let _ = std::fs::write(&progress_file, json);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
        });

        // Pre-configure youtube-dl with basic options common to all formats
        let mut youtube_dl = YoutubeDl::new(&url);
        youtube_dl.output_directory(&temp_dir_path);
        youtube_dl.extra_arg("--verbose");
        youtube_dl.socket_timeout("60");

        // Get video info first to determine estimated size and title
        // But do it in a way that doesn't block the UI
        let video_info_task = tokio::spawn({
            let url = url_str.clone();
            async move {
                // Create a quick youtube-dl instance just for getting info
                let mut info_dl = YoutubeDl::new(&url);
                info_dl.socket_timeout("30");

                // We only need certain fields, so let's extract those
                match info_dl.run_async().await {
                    Ok(YoutubeDlOutput::SingleVideo(video)) => {
                        // Initialize result with defaults
                        let mut title = String::from("Unknown");
                        let mut size: u64 = 0;
                        let mut duration_secs: u64 = 0;

                        // Extract title
                        if let Some(video_title) = &video.title {
                            title = video_title.clone();
                        }

                        // Extract duration if available (for better progress estimation)
                        if let Some(duration_float) = video.duration {
                            // Safely convert the duration to u64 based on the type
                            match duration_float {
                                serde_json::Value::Number(num) => {
                                    if let Some(n) = num.as_f64() {
                                        duration_secs = n as u64;
                                    }
                                }
                                // Handle other cases gracefully
                                _ => {}
                            }
                        }

                        // Extract estimated size
                        if let Some(formats) = video.formats {
                            for format in formats {
                                if let Some(format_size) = format.filesize {
                                    // Make sure both values are the same type (u64)
                                    let size_u64 = format_size as u64;
                                    if size_u64 > size {
                                        size = size_u64;
                                    }
                                }
                            }
                        }

                        Some((title, size, duration_secs))
                    }
                    _ => None,
                }
            }
        });

        // Update progress to 5% - Video info fetch started
        let mut progress = DownloadProgress::default();
        progress.downloaded_bytes = 5;
        progress.total_bytes = 100;
        progress.status = "Fetching video information...".to_string();
        if let Ok(json) = serde_json::to_string(&progress) {
            let _ = std::fs::write(&progress_file, json);
        }

        // Wait for video info (but don't block too long - max 10 seconds)
        let (video_title, estimated_size, duration_secs) =
            tokio::time::timeout(tokio::time::Duration::from_secs(10), video_info_task)
                .await
                .unwrap_or(Ok(None))
                .unwrap_or(None)
                .unwrap_or((String::from("Unknown"), 0, 0));

        tracing::info!(
            "Will download: {:?} (est. size: {}, duration: {} seconds)",
            video_title,
            estimated_size,
            duration_secs
        );

        // Update progress file with title and move to 10% progress once we have video info
        let mut progress = DownloadProgress::default();
        progress.downloaded_bytes = 10; // 10% progress after getting video info
        progress.total_bytes = 100;
        progress.status = format!("Starting download: {}", video_title);
        if let Ok(json) = serde_json::to_string(&progress) {
            let _ = std::fs::write(&progress_file, json);
        }

        // Calculate approximate download size based on quality
        let adjusted_estimated_size = if estimated_size == 0 {
            // If we couldn't get the size, estimate based on duration and quality
            match quality.to_lowercase().as_str() {
                "lowest" => duration_secs * 50 * 1024, // ~50KB per second for lowest quality
                "medium" => duration_secs * 250 * 1024, // ~250KB per second for medium quality
                "highest" | _ => duration_secs * 500 * 1024, // ~500KB per second for highest quality
            }
        } else {
            estimated_size
        };

        // Launch a separate task to monitor the download progress by checking file size
        let progress_file_clone = progress_file.clone();
        let temp_dir_clone = temp_dir.clone();
        let video_title_clone = video_title.clone();
        let estimated_size_clone = adjusted_estimated_size.max(1024 * 1024); // Ensure at least 1MB to avoid division by zero

        tokio::spawn(async move {
            let start_time = std::time::Instant::now();
            let mut last_size: u64 = 0;
            let mut last_update = std::time::Instant::now();
            let mut stalled_counter = 0;
            let mut download_started = false;
            let mut max_progress = 10; // Start at 10% (after getting video info)

            loop {
                // Check every 250ms for more responsive updates
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;

                // Check if the temp directory still exists (download might have finished)
                if !temp_dir_clone.exists() {
                    break;
                }

                // Calculate current size of all files in the directory
                let mut current_size: u64 = 0;
                let mut is_processing = false;
                let mut has_part_file = false;

                // Read directory to find all files
                if let Ok(mut entries) = fs::read_dir(&temp_dir_clone).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.is_file() {
                            // Check if file is a temporary/part file
                            let filename = path.file_name().unwrap_or_default().to_string_lossy();

                            // Detect if we're downloading (part files) or processing (media files)
                            if filename.ends_with(".part") || filename.contains(".f") {
                                has_part_file = true;
                            }

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
                if last_update.elapsed().as_millis() > 250 {
                    // Update progress status
                    let mut progress = DownloadProgress::default();

                    // Mark that download has started when we see a part file
                    if has_part_file && !download_started {
                        download_started = true;
                        // Keep progress at 10% initially when download actually starts
                        max_progress = 10;
                    }

                    // Calculate metrics based on download state
                    if is_processing {
                        // If we're processing (merging video and audio), set to 80-90%
                        progress.status = "Processing video...".to_string();
                        let process_percent = 80
                            + (std::time::Instant::now()
                                .duration_since(start_time)
                                .as_secs()
                                % 10);
                        progress.downloaded_bytes = process_percent as u64;
                        progress.total_bytes = 100;
                        progress.eta_seconds = 0;
                    } else if current_size > 0 && download_started {
                        // Calculate download speed
                        let elapsed_secs = start_time.elapsed().as_secs().max(1); // Avoid division by zero
                        let download_speed = current_size as f64 / elapsed_secs as f64;

                        // Calculate progress percentage between 10% and 80%
                        let raw_percent = if estimated_size_clone > 0 {
                            // Use min to cap at 80% for download phase
                            ((current_size as f64 / estimated_size_clone as f64) * 70.0 + 10.0)
                                .min(80.0)
                        } else {
                            // If we don't have a size estimate, use time-based estimation
                            // Cap at 80% and ensure it doesn't decrease
                            let time_based =
                                ((elapsed_secs as f64).min(600.0) / 600.0 * 70.0 + 10.0).min(80.0);
                            time_based
                        };

                        // Ensure progress never decreases and increases smoothly
                        let percent = (raw_percent as u64).max(max_progress);
                        max_progress = percent;

                        // Set progress values
                        progress.downloaded_bytes = percent;
                        progress.total_bytes = 100;

                        // Calculate ETA based on download speed and estimated remaining size
                        if download_speed > 100.0 {
                            // Only show ETA if speed is reasonable
                            let remaining_bytes = estimated_size_clone.saturating_sub(current_size);
                            let eta_secs = (remaining_bytes as f64 / download_speed) as u64;
                            progress.eta_seconds = eta_secs;

                            // Format ETA for display
                            let eta_display = if eta_secs > 60 {
                                format!("{:.1} min", eta_secs as f64 / 60.0)
                            } else {
                                format!("{} sec", eta_secs)
                            };

                            // Calculate and display download speed
                            let speed_display = if download_speed > 1024.0 * 1024.0 {
                                format!("{:.1} MB/s", download_speed / (1024.0 * 1024.0))
                            } else {
                                format!("{:.1} KB/s", download_speed / 1024.0)
                            };

                            progress.status = format!(
                                "Downloading: {}% of {} ({}), ETA: {}",
                                percent, video_title_clone, speed_display, eta_display
                            );
                        } else {
                            progress.status =
                                format!("Downloading: {}% of {}", percent, video_title_clone);
                        }

                        // Check if download is stalled
                        if current_size == last_size && has_part_file {
                            stalled_counter += 1;
                            // After 10 seconds with no progress, indicate stalled download
                            if stalled_counter >= 40 {
                                // 10 seconds (40 * 250ms)
                                progress.status = format!(
                                    "Download stalled at {}% ({:.1} MB)...",
                                    percent,
                                    current_size as f64 / (1024.0 * 1024.0)
                                );
                            }
                        } else {
                            stalled_counter = 0;
                            last_size = current_size;
                        }
                    } else {
                        // Waiting for download to start
                        if download_started {
                            progress.downloaded_bytes = 10; // 10% while waiting for first bytes
                        } else {
                            // Gradually increase from 5% to 10% during initialization
                            let elapsed_secs = start_time.elapsed().as_secs();
                            progress.downloaded_bytes = 5 + (elapsed_secs.min(10) as u64).max(0);
                            // 5-10% during waiting
                        }
                        progress.total_bytes = 100;
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

        // Configure format selection right away based on format_type and quality
        // This is fast and can be done synchronously
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

        // Update progress to 15% - Download about to start
        let mut progress = DownloadProgress::default();
        progress.downloaded_bytes = 15;
        progress.total_bytes = 100;
        progress.status = format!("Starting download: {}", video_title);
        if let Ok(json) = serde_json::to_string(&progress) {
            let _ = std::fs::write(&progress_file, json);
        }

        // Execute the download
        tracing::info!("Starting download with yt-dlp...");
        match youtube_dl.download_to_async(&temp_dir).await {
            Ok(()) => tracing::info!("Download completed successfully"),
            Err(e) => {
                tracing::error!("Download error: {}", e);

                // Update progress file with error
                let mut progress = DownloadProgress::default();
                progress.status = format!("Error: {}", e);
                progress.downloaded_bytes = 100; // Set to 100% to indicate we're done (with error)
                progress.total_bytes = 100;
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
        let downloaded_file = utils::find_downloaded_file(&temp_dir).await.map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Failed to find downloaded file: {}",
                e
            ))
        })?;

        tracing::info!("Found downloaded file: {}", downloaded_file.display());

        // Update progress file with completion status - 90%
        let mut progress = DownloadProgress::default();
        progress.status = "Download complete, preparing file...".to_string();
        progress.downloaded_bytes = 90;
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

        // Update progress file - 95%
        progress.status = "Saving file to permanent location...".to_string();
        progress.downloaded_bytes = 95;
        if let Ok(json) = serde_json::to_string(&progress) {
            let _ = std::fs::write(&progress_file, json);
        }

        // Create a permanent path for database record
        let mut file_path_for_db = downloaded_file.to_string_lossy().to_string();

        // Save to a permanent location with proper permissions
        let extension = downloaded_file
            .extension()
            .unwrap_or_default()
            .to_string_lossy();
        let file_name = storage::create_clean_filename(&video_title, &extension);
        let mut saved_to_permanent = false;

        // For desktop apps, save in Documents folder next to database
        #[cfg(feature = "desktop")]
        {
            if let Some(media_dir) = storage::ensure_media_directory() {
                let permanent_path = media_dir.join(&file_name);

                // Try to save the file with proper permissions
                if storage::save_file_with_permissions(&permanent_path, &content) {
                    tracing::info!("Media file saved to: {}", permanent_path.display());
                    file_path_for_db = permanent_path.to_string_lossy().to_string();
                    saved_to_permanent = true;
                }
            }
        }

        // Also save to Downloads folder for convenience
        if let Some(download_dir) = dirs::download_dir() {
            let download_path = download_dir.join(&file_name);
            if storage::save_file_with_permissions(&download_path, &content) {
                tracing::info!(
                    "Copy saved to Downloads folder: {}",
                    download_path.display()
                );
                if !saved_to_permanent {
                    file_path_for_db = download_path.to_string_lossy().to_string();
                    saved_to_permanent = true;
                }
            }
        }

        // Update progress file - 99%
        progress.status = "Finalizing...".to_string();
        progress.downloaded_bytes = 99;
        if let Ok(json) = serde_json::to_string(&progress) {
            let _ = std::fs::write(&progress_file, json);
        }

        // Always clean up temporary files
        tracing::info!("Cleaning up temporary files");
        let _ = fs::remove_dir_all(&temp_dir).await;

        // Set progress to 100% before removing the progress file
        progress.status = "Download complete!".to_string();
        progress.downloaded_bytes = 100;
        progress.total_bytes = 100;
        if let Ok(json) = serde_json::to_string(&progress) {
            let _ = std::fs::write(&progress_file, json);
        }

        // Keep the progress file around briefly to ensure UI can read the 100% completion
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let _ = fs::remove_file(&progress_file).await;

        // Get file size
        let file_size = content.len() as i64;

        // Save download info to database in the background
        #[cfg(feature = "server")]
        {
            let url = url.clone();
            let video_title = video_title.clone();
            let file_name = file_name.clone();
            let file_path_for_db = file_path_for_db.clone();
            let format_type = format_type.clone();
            let quality = quality.clone();

            tokio::spawn(async move {
                if let Err(e) = save_download_info(
                    &url,
                    &video_title,
                    &file_name,
                    &file_path_for_db,
                    &if format_type.is_empty() {
                        "video".to_string()
                    } else {
                        format_type
                    },
                    &if quality.is_empty() {
                        "best".to_string()
                    } else {
                        quality
                    },
                    file_size,
                )
                .await
                {
                    tracing::error!("Database error: {}", e);
                }
            });
        }

        tracing::info!("Downloaded {} bytes successfully", content.len());
        Ok(content)
    }

    #[cfg(not(feature = "server"))]
    Err(ServerFnError::<NoCustomError>::ServerError(
        "Server feature not enabled".to_string(),
    ))
}

#[cfg(feature = "server")]
pub async fn download_video_with_progress(
    url: String,
    format: Option<String>,
    quality: Option<String>,
    path: Option<String>,
    set_progress: impl Fn(f32, Option<String>, Option<String>) + Clone + Send + 'static,
) -> Result<String, String> {
    // Immediately update progress to show user something is happening - 0%
    set_progress(0.0, Some("Fetching video information...".to_string()), None);

    // Start a background task to get video info
    let video_info_future = super::info::get_video_info(url.clone());

    // First fetch video information
    let video_info = video_info_future
        .await
        .map_err(|e| format!("Error fetching video info: {}", e))?;

    // Parse the JSON to get title and other metadata
    let video_data: serde_json::Value = serde_json::from_str(&video_info)
        .map_err(|e| format!("Error parsing video info: {}", e))?;

    let title = video_data
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown_title")
        .to_string();

    // Update progress to show we're starting the download - 5%
    set_progress(0.05, Some(format!("Starting download of: {}", title)), None);

    // Start the video download - 10%
    set_progress(0.1, Some("Preparing download...".to_string()), None);

    // Use a custom progress callback that updates both our local progress tracking
    // and the provided set_progress callback
    struct ProgressTracker {
        progress: f32,
        file_exists: bool,
    }

    let progress_tracker = std::sync::Arc::new(std::sync::Mutex::new(ProgressTracker {
        progress: 0.1, // Start at 10%
        file_exists: false,
    }));

    // Create a clone of the progress tracker for the monitoring task
    let progress_tracker_clone = progress_tracker.clone();

    // Set up a task to update progress based on the progress file
    let url_len = url.len();
    let set_progress_clone = set_progress.clone();
    let progress_updater = tokio::spawn(async move {
        let progress_file = std::env::temp_dir().join(format!("download_{}.progress", url_len));

        // Check progress every 250ms
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;

            if progress_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&progress_file) {
                    if let Ok(progress) = serde_json::from_str::<DownloadProgress>(&content) {
                        let mut tracker = progress_tracker_clone.lock().unwrap();

                        // Convert to 0.0-1.0 range and ensure it never decreases
                        let new_progress = (progress.downloaded_bytes as f32
                            / progress.total_bytes as f32)
                            .max(tracker.progress);

                        // Update tracker
                        tracker.progress = new_progress;

                        // Pass progress and status to the callback
                        set_progress_clone(new_progress, Some(progress.status), None);

                        // Mark file as exists if we're past 50%
                        if new_progress > 0.5 {
                            tracker.file_exists = true;
                        }
                    }
                }
            } else if progress_tracker_clone.lock().unwrap().file_exists {
                // Progress file gone but we already saw good progress
                set_progress_clone(0.99, Some("Finalizing download...".to_string()), None);
                break;
            } else {
                // Progress file doesn't exist yet, continue checking
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    });

    // Now actually perform the download
    let download_future = download_with_quality(
        url.clone(),
        format.clone().unwrap_or_else(|| "video".to_string()),
        quality.clone().unwrap_or_else(|| "highest".to_string()),
    );
    let result = download_future.await;

    // Try to wait for progress updater to finish or timeout after 2 seconds
    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), progress_updater).await;

    match result {
        Ok(content) => {
            // For database, we need to save some metadata
            set_progress(0.95, Some("Saving video file...".to_string()), None);

            // Since we have the content as Vec<u8>, we need to save it to a file first
            let temp_file = std::env::temp_dir().join(format!(
                "{}_{}.mp4",
                title.replace(" ", "_"),
                chrono::Utc::now().timestamp()
            ));

            let file_path = temp_file.to_string_lossy().to_string();
            let filename = temp_file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Create a permanent path for database record
            let mut file_path_for_db = file_path.clone();

            // Save to a permanent location with proper permissions
            let file_name = storage::create_clean_filename(&title, "mp4");
            let mut saved_to_permanent = false;

            // For desktop apps, save in Documents folder next to database
            #[cfg(feature = "desktop")]
            {
                if let Some(media_dir) = storage::ensure_media_directory() {
                    let permanent_path = media_dir.join(&file_name);

                    // Try to save the file with proper permissions
                    if storage::save_file_with_permissions(&permanent_path, &content) {
                        tracing::info!("Media file saved to: {}", permanent_path.display());
                        file_path_for_db = permanent_path.to_string_lossy().to_string();
                        saved_to_permanent = true;
                    }
                }
            }

            // Also save to Downloads folder for convenience
            if let Some(download_dir) = dirs::download_dir() {
                let download_path = download_dir.join(&file_name);
                if storage::save_file_with_permissions(&download_path, &content) {
                    tracing::info!(
                        "Copy saved to Downloads folder: {}",
                        download_path.display()
                    );
                    if !saved_to_permanent {
                        file_path_for_db = download_path.to_string_lossy().to_string();
                        saved_to_permanent = true;
                    }
                }
            }

            // Get file size
            let file_size = content.len() as i64;

            // Update progress to 99%
            set_progress(0.99, Some("Updating database...".to_string()), None);

            // Save download info to database (in background)
            #[cfg(feature = "server")]
            {
                let url = url.clone();
                let title = title.clone();
                let filename = filename.clone();
                let file_path_for_db = file_path_for_db.clone();
                let format = format.clone();
                let quality = quality.clone();

                tokio::spawn(async move {
                    if let Err(e) = save_download_info(
                        &url,
                        &title,
                        &filename,
                        &file_path_for_db,
                        &format.clone().unwrap_or_else(|| "video".to_string()),
                        &quality.clone().unwrap_or_else(|| "best".to_string()),
                        file_size,
                    )
                    .await
                    {
                        tracing::error!("Database error: {}", e);
                    }
                });
            }

            set_progress(1.0, Some("Download complete!".to_string()), None);
            Ok(file_path)
        }
        Err(e) => {
            set_progress(1.0, Some(format!("Error: {}", e)), None);
            Err(format!("Download error: {}", e))
        }
    }
}
