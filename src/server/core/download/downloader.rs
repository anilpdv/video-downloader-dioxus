use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tracing;

#[cfg(feature = "server")]
use super::ytdlp::{ensure_yt_dlp_available, run_ytdlp_command};
#[cfg(feature = "server")]
use crate::server::core::download::progress::{mark_download_complete, save_progress};
#[cfg(feature = "server")]
use crate::server::core::download::DownloadProgress;
#[cfg(feature = "server")]
use crate::server::core::storage::database::save_download_info;
#[cfg(feature = "server")]
use crate::server::core::storage::file::{create_clean_filename, ensure_media_directory};
#[cfg(feature = "server")]
use crate::server::utils::common::parse_progress_line;

#[cfg(feature = "server")]
use tokio::fs;
#[cfg(feature = "server")]
use tokio::sync::mpsc::{channel, Sender};
#[cfg(feature = "server")]
use tokio::task::spawn;
#[cfg(feature = "server")]
use tokio::time::timeout;

/// Download a video with the specified format type and quality
#[cfg(feature = "server")]
pub async fn download_with_quality(
    url: String,
    format_type: String,
    quality: String,
) -> Result<String, String> {
    tracing::info!(
        "Download with format: {}, quality: {}, URL: {}",
        format_type,
        quality,
        url
    );

    // Validate URL format
    if !url.contains("youtube.com/watch?v=") && !url.contains("youtu.be/") {
        return Err("Invalid YouTube URL. Please provide a valid YouTube video URL.".to_string());
    }

    // Create a progress tracker
    let mut progress = DownloadProgress::default();
    progress.status = "Initializing download...".to_string();
    save_progress(&url, &progress).await?;

    // Create temporary directory for the download
    let temp_dir = std::env::temp_dir().join(format!("youtube_dl_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Periodically update initialization progress (0-5%)
    let url_clone = url.clone();
    spawn(async move {
        for i in 0..30 {
            // timeout after 15 seconds
            let mut progress = DownloadProgress::default();
            let percent = ((i as f64) / 30.0 * 5.0).min(5.0); // Max 5% during init
            progress.downloaded_bytes = (percent * 100.0) as u64;
            progress.total_bytes = 10000;
            progress.status = format!("Preparing download ({}s)...", i / 2);
            let _ = save_progress(&url_clone, &progress).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    });

    // Ensure yt-dlp is available
    let yt_dlp_path = match ensure_yt_dlp_available().await {
        Ok(path) => path,
        Err(e) => return Err(format!("Failed to ensure yt-dlp is available: {}", e)),
    };

    // Get video info first to determine title
    progress.status = "Fetching video information...".to_string();
    progress.downloaded_bytes = 500; // 5%
    save_progress(&url, &progress).await?;

    // Build info command
    let mut info_cmd = Command::new(&yt_dlp_path);
    info_cmd
        .arg("-j")
        .arg("--no-playlist")
        .arg(&url)
        .stdout(Stdio::piped());

    tracing::info!("Running yt-dlp info command: {:?}", info_cmd);

    // Run info command and capture output
    let info_output = match info_cmd.output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                tracing::error!("yt-dlp info command failed: {}", stderr);
                return Err(format!("Failed to get video info: {}", stderr));
            }
        }
        Err(e) => {
            tracing::error!("Failed to execute yt-dlp info command: {}", e);
            return Err(format!("Failed to execute yt-dlp info command: {}", e));
        }
    };

    // Parse the JSON output to get the title
    let video_info: serde_json::Value = match serde_json::from_str(&info_output) {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("Failed to parse video info: {}", e);
            return Err(format!("Failed to parse video info: {}", e));
        }
    };

    // Extract title and duration
    let title = video_info["title"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();
    let duration_secs = video_info["duration"].as_f64().unwrap_or(0.0) as u64;

    tracing::info!(
        "Will download: {} (duration: {} seconds)",
        title,
        duration_secs
    );

    // Update progress
    progress.status = format!("Starting download of: {}", title);
    progress.downloaded_bytes = 1000; // 10%
    save_progress(&url, &progress).await?;

    // Create the media directory
    let media_dir = match ensure_media_directory() {
        Some(dir) => dir,
        None => return Err("Failed to create media directory".to_string()),
    };

    // Prepare file extension based on format
    let extension = if format_type == "audio" { "mp3" } else { "mp4" };

    // Create a clean filename
    let filename = create_clean_filename(&title, extension);
    let output_path = media_dir.join(&filename);
    let output_path_str = output_path.to_string_lossy().to_string();

    tracing::info!("Output path: {}", output_path_str);

    // Prepare download options based on format_type and quality
    let mut options = Vec::new();

    if format_type == "audio" {
        options.push("-x".to_string());
        options.push("--audio-format".to_string());
        options.push("mp3".to_string());

        if quality == "highest" {
            options.push("--audio-quality".to_string());
            options.push("0".to_string());
        } else if quality == "medium" {
            options.push("--audio-quality".to_string());
            options.push("5".to_string());
        } else {
            options.push("--audio-quality".to_string());
            options.push("7".to_string());
        }
    } else {
        // Video format
        if quality == "highest" {
            options.push("-f".to_string());
            options.push("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string());
        } else if quality == "medium" {
            options.push("-f".to_string());
            options.push("bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720][ext=mp4]/best".to_string());
        } else {
            options.push("-f".to_string());
            options.push("bestvideo[height<=480][ext=mp4]+bestaudio[ext=m4a]/best[height<=480][ext=mp4]/best".to_string());
        }
    }

    // Add output template
    options.push("-o".to_string());
    options.push(format!("{}", filename));

    // Execute the download command with progress tracking
    let url_clone = url.clone();

    // Start the download in a separate process that we can read from
    let mut cmd = Command::new(&yt_dlp_path);
    cmd.arg(&url)
        .current_dir(&media_dir)
        .args(options)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    tracing::info!("Running yt-dlp download command: {:?}", cmd);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to spawn yt-dlp: {}", e);
            return Err(format!("Failed to spawn yt-dlp: {}", e));
        }
    };

    // Read output to track progress
    let stdout = match child.stdout.take() {
        Some(out) => BufReader::new(out),
        None => return Err("Failed to capture stdout".to_string()),
    };

    let stderr = match child.stderr.take() {
        Some(err) => BufReader::new(err),
        None => return Err("Failed to capture stderr".to_string()),
    };

    // Track progress from both stdout and stderr
    let url_for_stdout = url.clone();
    let stdout_handle = spawn(async move {
        for line in stdout.lines() {
            if let Ok(line_str) = line {
                tracing::debug!("yt-dlp stdout: {}", line_str);
                if let Some(p) = parse_progress_line(&line_str) {
                    let _ = save_progress(&url_for_stdout, &p).await;
                }
            }
        }
    });

    let url_for_stderr = url.clone();
    let stderr_handle = spawn(async move {
        for line in stderr.lines() {
            if let Ok(line_str) = line {
                tracing::debug!("yt-dlp stderr: {}", line_str);
                // Also try to parse progress from stderr
                if let Some(p) = parse_progress_line(&line_str) {
                    let _ = save_progress(&url_for_stderr, &p).await;
                }
            }
        }
    });

    // Wait for the download to complete
    match child.wait() {
        Ok(status) => {
            if !status.success() {
                return Err(format!("yt-dlp exited with status: {}", status));
            }
        }
        Err(e) => {
            return Err(format!("Failed to wait for yt-dlp: {}", e));
        }
    }

    // Wait for progress readers to finish
    let _ = timeout(std::time::Duration::from_secs(2), stdout_handle).await;
    let _ = timeout(std::time::Duration::from_secs(2), stderr_handle).await;

    // Mark download as complete
    mark_download_complete(&url).await?;

    // Get file size
    let file_size = match fs::metadata(&output_path).await {
        Ok(metadata) => metadata.len() as i64,
        Err(e) => {
            tracing::error!("Failed to get file size: {}", e);
            0
        }
    };

    // Save download info to database
    match save_download_info(
        &url,
        &title,
        &filename,
        &output_path_str,
        &format_type,
        &quality,
        file_size,
    )
    .await
    {
        Ok(_) => {
            tracing::info!("Saved download info to database");
        }
        Err(e) => {
            tracing::error!("Failed to save download info: {}", e);
        }
    }

    // Return success with the file path
    Ok(format!("Successfully downloaded: {}", title))
}
