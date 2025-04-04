use dioxus::prelude::*;
use serde_json;
use server_fn::error::NoCustomError;
use std::path::PathBuf;
use tokio::fs;
use tracing;

use super::database::save_download_info;
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
        let downloaded_file = utils::find_downloaded_file(&temp_dir).await.map_err(|e| {
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

        // Always clean up temporary files
        tracing::info!("Cleaning up temporary files");
        let _ = fs::remove_dir_all(&temp_dir).await;
        let _ = fs::remove_file(&progress_file).await;

        // Get file size
        let file_size = content.len() as i64;

        // Save download info to database
        #[cfg(feature = "server")]
        {
            if let Err(e) = save_download_info(
                &url,
                &video_title,
                &file_name,
                &file_path_for_db,
                &if format_type.is_empty() {
                    "video".to_string()
                } else {
                    format_type.clone()
                },
                &if quality.is_empty() {
                    "best".to_string()
                } else {
                    quality.clone()
                },
                file_size,
            )
            .await
            {
                tracing::error!("Database error: {}", e);
            }
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
    // First fetch video information
    let video_info = super::info::get_video_info(url.clone())
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

    // Download the video - this returns Vec<u8>
    let result = download_with_quality(
        url.clone(),
        format.clone().unwrap_or_else(|| "video".to_string()),
        quality.clone().unwrap_or_else(|| "highest".to_string()),
    )
    .await;

    match result {
        Ok(content) => {
            // For database, we need to save some metadata
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

            // Save download info to database
            #[cfg(feature = "server")]
            {
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
            }

            Ok(file_path)
        }
        Err(e) => Err(format!("Download error: {}", e)),
    }
}
