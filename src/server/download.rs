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

/// Download video with highest quality
#[server(Download)]
pub async fn download_video(url: String) -> Result<Vec<u8>, ServerFnError<NoCustomError>> {
    tracing::info!("Download request for URL: {}", url);
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

    // Ensure yt-dlp is available
    let yt_dlp_path = match ensure_yt_dlp_available().await {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("Could not find or download yt-dlp: {}", e);
            return Err(e);
        }
    };

    let mut youtube_dl = YoutubeDl::new(url);

    // Handle both full paths and PATH-based commands
    if yt_dlp_path.to_string_lossy().contains("/") || yt_dlp_path.to_string_lossy().contains("\\") {
        // For full paths, use the path directly
        tracing::info!("Using full path for yt-dlp: {:?}", yt_dlp_path);
        youtube_dl.youtube_dl_path(&yt_dlp_path);
    } else {
        // For commands in PATH, just use the command name
        tracing::info!("Using yt-dlp from PATH");
        youtube_dl.youtube_dl_path("yt-dlp");
    }

    let output = match youtube_dl.run_async().await.map_err(|e| {
        tracing::error!("Error fetching video info: {}", e);
        ServerFnError::<NoCustomError>::ServerError(format!("Error fetching video info: {}", e))
    })? {
        YoutubeDlOutput::SingleVideo(video) => {
            tracing::info!(
                "Successfully fetched info for video: {}",
                video.title.as_deref().unwrap_or("Unknown")
            );
            video
        }
        YoutubeDlOutput::Playlist(_) => {
            tracing::info!("URL points to a playlist, not a single video");
            return Err(ServerFnError::<NoCustomError>::ServerError(
                "URL points to a playlist, not a single video".to_string(),
            ));
        }
    };

    // Convert the video info to JSON
    let json_str = serde_json::to_string(&output).map_err(|e| {
        tracing::error!("Error serializing video info: {}", e);
        ServerFnError::<NoCustomError>::ServerError(format!("Error serializing video info: {}", e))
    })?;

    tracing::info!("Successfully processed video info");
    Ok(json_str)
}

/// Search YouTube videos
#[server(SearchYoutube)]
pub async fn search_youtube(query: String) -> Result<String, ServerFnError<NoCustomError>> {
    tracing::info!("Searching YouTube for: {}", query);

    // Ensure yt-dlp is available
    let yt_dlp_path = match ensure_yt_dlp_available().await {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("Could not find or download yt-dlp: {}", e);
            return Err(e);
        }
    };

    // Create search options for YouTube
    let search_options = youtube_dl::SearchOptions::youtube(query).with_count(10); // Get 10 results

    // Run the search with proper path handling
    let mut youtube_dl = YoutubeDl::search_for(&search_options);

    // Handle both full paths and PATH-based commands
    if yt_dlp_path.to_string_lossy().contains("/") || yt_dlp_path.to_string_lossy().contains("\\") {
        // For full paths, use the path directly
        tracing::info!("Using full path for yt-dlp: {:?}", yt_dlp_path);
        youtube_dl.youtube_dl_path(&yt_dlp_path);
    } else {
        // For commands in PATH, just use the command name
        tracing::info!("Using yt-dlp from PATH");
        youtube_dl.youtube_dl_path("yt-dlp");
    }

    let output = youtube_dl.run_async().await.map_err(|e| {
        tracing::error!("Error searching: {}", e);
        ServerFnError::<NoCustomError>::ServerError(format!("Error searching: {}", e))
    })?;

    // Convert the output to JSON
    let json_str = serde_json::to_string(&output).map_err(|e| {
        tracing::error!("Error serializing search results: {}", e);
        ServerFnError::<NoCustomError>::ServerError(format!(
            "Error serializing search results: {}",
            e
        ))
    })?;

    tracing::info!("Search complete, found results");
    Ok(json_str)
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
        tracing::error!("Invalid YouTube URL provided: {}", url);
        return Err(ServerFnError::<NoCustomError>::ServerError(
            "Invalid YouTube URL. Please provide a valid YouTube video URL.".to_string(),
        ));
    }

    // Ensure yt-dlp is available
    let yt_dlp_path = match ensure_yt_dlp_available().await {
        Ok(path) => {
            tracing::info!("Using yt-dlp at path: {:?}", path);
            path
        }
        Err(e) => {
            tracing::error!("Could not find or download yt-dlp: {}", e);
            let message = format!("yt-dlp executable not found. Please ensure that yt-dlp is installed \
                on your system and is in your PATH. You can install it with: \
                macOS: 'brew install yt-dlp' or Linux: 'sudo curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && sudo chmod a+rx /usr/local/bin/yt-dlp'");
            return Err(ServerFnError::<NoCustomError>::ServerError(message));
        }
    };

    // Check if the file exists before proceeding, but only if it's a full path
    // If it's just "yt-dlp", it means it was found in PATH and we don't need to check
    if !yt_dlp_path.to_string_lossy().contains("/") && !yt_dlp_path.to_string_lossy().contains("\\")
    {
        tracing::info!("Using yt-dlp from PATH, skipping file existence check");
    } else if !yt_dlp_path.exists() {
        let path_str = yt_dlp_path.display().to_string();
        tracing::error!("yt-dlp path points to non-existent file: {}", path_str);
        return Err(ServerFnError::<NoCustomError>::ServerError(format!(
            "yt-dlp executable not found at {}. Please install yt-dlp",
            path_str
        )));
    }

    // Create temporary directory for the download
    let temp_dir = std::env::temp_dir().join(format!("youtube_dl_{}", std::process::id()));

    tracing::info!("Creating temp directory at {:?}", temp_dir);
    std::fs::create_dir_all(&temp_dir).map_err(|e| {
        tracing::error!("Failed to create temp directory: {}", e);
        ServerFnError::<NoCustomError>::ServerError(format!(
            "Failed to create temp directory: {}",
            e
        ))
    })?;

    let temp_dir_path = temp_dir.to_string_lossy().to_string();

    // Try to resolve the video URL first to get some basic info
    let video_info = {
        let mut youtube_dl = YoutubeDl::new(&url);

        // Handle both full paths and PATH-based commands
        if yt_dlp_path.to_string_lossy().contains("/")
            || yt_dlp_path.to_string_lossy().contains("\\")
        {
            // For full paths, use the path directly
            youtube_dl.youtube_dl_path(&yt_dlp_path);
        } else {
            // For commands in PATH, just use the command name
            youtube_dl.youtube_dl_path("yt-dlp");
        }

        match youtube_dl.run_async().await {
            Ok(output) => match output {
                YoutubeDlOutput::SingleVideo(video) => Some(video),
                _ => None,
            },
            Err(e) => {
                tracing::warn!("Could not get video info before download: {}", e);
                None
            }
        }
    };

    if let Some(info) = &video_info {
        tracing::info!(
            "Will download: {} ({})",
            info.title.as_deref().unwrap_or("Unknown title"),
            info.duration
                .as_ref()
                .map(|d| format!("{:?}", d))
                .unwrap_or_else(|| "Unknown duration".to_string())
        );
    }

    // Configure youtube-dl options based on format type and quality
    let mut youtube_dl = YoutubeDl::new(&url);

    // Set yt-dlp path and output directory
    // Handle both full paths and PATH-based commands
    if yt_dlp_path.to_string_lossy().contains("/") || yt_dlp_path.to_string_lossy().contains("\\") {
        // For full paths, use the path directly
        tracing::info!("Using full path for yt-dlp: {:?}", yt_dlp_path);
        youtube_dl.youtube_dl_path(&yt_dlp_path);
    } else {
        // For commands in PATH, just use the command name
        tracing::info!("Using yt-dlp from PATH");
        youtube_dl.youtube_dl_path("yt-dlp");
    }
    youtube_dl.output_directory(&temp_dir_path);

    // Add timeout to prevent hangs
    youtube_dl.socket_timeout("30");
    youtube_dl.process_timeout(std::time::Duration::from_secs(300)); // 5 minute timeout

    // Configure format selection based on format_type and quality
    tracing::info!("Configuring download format");
    match format_type.to_lowercase().as_str() {
        "audio" => {
            youtube_dl.extract_audio(true);
            youtube_dl.format("bestaudio");
            youtube_dl.output_template("audio");
            tracing::info!("Set up audio-only download with best quality");
        }
        "video" => {
            // Configure video quality
            match quality.to_lowercase().as_str() {
                "lowest" => {
                    youtube_dl.format("worstvideo[ext=mp4]+worstaudio[ext=m4a]/worst[ext=mp4]");
                    tracing::info!("Set up video download with lowest quality");
                }
                "highest" | _ => {
                    youtube_dl.format("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]");
                    tracing::info!("Set up video download with highest quality");
                }
            }
            youtube_dl.output_template("video");
        }
        _ => {
            tracing::error!("Invalid format type: {}", format_type);
            return Err(ServerFnError::<NoCustomError>::ServerError(
                "Invalid format type. Please specify 'audio' or 'video'.".to_string(),
            ));
        }
    }

    // Execute the download
    tracing::info!("Starting download with yt-dlp...");

    match youtube_dl.download_to_async(&temp_dir).await {
        Ok(()) => tracing::info!("Download completed successfully"),
        Err(e) => {
            tracing::error!("Download error: {}", e);

            // If we have a youtube-dl error with exit code, log the detailed stderr
            if let youtube_dl::Error::ExitCode { code, stderr } = &e {
                tracing::error!("yt-dlp exit code {}, stderr: {}", code, stderr);
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
        tracing::error!("Failed to find downloaded file: {}", e);
        ServerFnError::<NoCustomError>::ServerError(format!(
            "Failed to find downloaded file: {}",
            e
        ))
    })?;

    tracing::info!("Found downloaded file: {}", downloaded_file.display());

    // Read the file content
    tracing::info!("Reading file content");
    let content = fs::read(&downloaded_file).await.map_err(|e| {
        tracing::error!("Failed to read downloaded file: {}", e);
        ServerFnError::<NoCustomError>::ServerError(format!(
            "Failed to read downloaded file: {}",
            e
        ))
    })?;

    // Clean up the temp directory
    tracing::info!("Cleaning up temporary directory");
    if let Err(e) = fs::remove_dir_all(&temp_dir).await {
        tracing::warn!("Failed to clean up temp directory: {}", e);
    }

    tracing::info!("Downloaded {} bytes successfully", content.len());
    Ok(content)
}

#[cfg(feature = "server")]
async fn find_downloaded_file(dir: impl AsRef<Path>) -> io::Result<PathBuf> {
    let dir_path = dir.as_ref();
    let dir_str = dir_path.to_string_lossy();

    tracing::info!("Scanning directory {} for downloaded files", dir_str);

    // Check if the directory exists
    if !dir_path.exists() {
        tracing::error!("Download directory does not exist: {}", dir_str);
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Download directory does not exist: {}", dir_str),
        ));
    }

    // Check if it's actually a directory
    if !dir_path.is_dir() {
        tracing::error!("Path is not a directory: {}", dir_str);
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Path is not a directory: {}", dir_str),
        ));
    }

    // Collect all entries first to avoid the 'continue' in the while loop condition issue
    let entries = match fs::read_dir(dir_path).await {
        Ok(mut entries) => {
            let mut result: Vec<fs::DirEntry> = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                result.push(entry);
            }
            result
        }
        Err(e) => {
            tracing::error!("Failed to read directory {}: {}", dir_str, e);
            return Err(e);
        }
    };

    // First look for files
    for entry in &entries {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            let size = match fs::metadata(&path).await {
                Ok(meta) => meta.len(),
                Err(_) => 0,
            };
            tracing::info!("Found file: {} ({} bytes)", filename, size);
            return Ok(path);
        }
    }

    // If no files found, try subdirectories
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            tracing::info!("Checking subdirectory: {}", path.display());
            match fs::read_dir(&path).await {
                Ok(mut subentries) => {
                    // Check for any files in this subdirectory
                    while let Ok(Some(subentry)) = subentries.next_entry().await {
                        let subpath = subentry.path();
                        if subpath.is_file() {
                            let filename =
                                subpath.file_name().unwrap_or_default().to_string_lossy();
                            let size = match fs::metadata(&subpath).await {
                                Ok(meta) => meta.len(),
                                Err(_) => 0,
                            };
                            tracing::info!(
                                "Found file in subdirectory: {} ({} bytes)",
                                filename,
                                size
                            );
                            return Ok(subpath);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read subdirectory {}: {}", path.display(), e);
                }
            }
        }
    }

    tracing::error!(
        "No files found in {} or its immediate subdirectories",
        dir_str
    );
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No downloaded file found",
    ))
}
