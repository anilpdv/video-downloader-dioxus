#[cfg(feature = "server")]
use dioxus::logger::tracing;
use dioxus::prelude::*;
#[cfg(feature = "server")]
use rusty_ytdl::search::YouTube;
#[cfg(feature = "server")]
use rusty_ytdl::{FFmpegArgs, Video, VideoFormat, VideoOptions, VideoQuality, VideoSearchOptions};
#[cfg(feature = "server")]
use serde_json;

#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

#[server(Download)]
pub async fn download_video(url: String) -> Result<Vec<u8>, ServerFnError> {
    // Try to download with default options first - this has the highest chance of success
    let video = match Video::new(&url) {
        Ok(v) => v,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Error creating video instance: {}",
                e
            )))
        }
    };

    // Get info about available formats
    let info = match video.get_basic_info().await {
        Ok(i) => i,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Error getting video info: {}",
                e
            )))
        }
    };

    // Check if we have any formats available
    if info.formats.is_empty() {
        return Err(ServerFnError::ServerError(
            "No video formats available for this video. It may be restricted or private."
                .to_string(),
        ));
    }

    // Print some debug info about the video
    tracing::info!("Total formats available: {:?}", info.formats);

    // Find formats with both video and audio
    let video_with_audio_formats: Vec<_> = info
        .formats
        .iter()
        .filter(|f| f.has_video && f.has_audio)
        .collect();

    tracing::info!(
        "Formats with both video and audio: {}",
        video_with_audio_formats.len()
    );

    // Find video-only formats (for debugging)
    let video_only_formats: Vec<_> = info
        .formats
        .iter()
        .filter(|f| f.has_video && !f.has_audio)
        .collect();

    tracing::info!("Video-only formats: {:?}", video_only_formats);

    // Find audio-only formats (for debugging)
    let audio_only_formats: Vec<_> = info
        .formats
        .iter()
        .filter(|f| !f.has_video && f.has_audio)
        .collect();

    tracing::info!("Audio-only formats: {:?}", audio_only_formats);

    // Create video options with explicit settings to ensure we get video+audio
    let video_options = VideoOptions {
        quality: VideoQuality::Highest,
        filter: VideoSearchOptions::VideoAudio, // Explicitly request both video and audio
        ..Default::default()
    };

    // Create a video instance with custom options
    let custom_video = match Video::new_with_options(&url, video_options) {
        Ok(v) => v,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Error creating video with options: {}",
                e
            )));
        }
    };

    // Try to use FFmpeg for better video/audio handling
    tracing::info!("Attempting to use FFmpeg for video processing");

    // Set up FFmpeg arguments to ensure we get an MP4 with both video and audio
    let ffmpeg_args = FFmpegArgs {
        format: Some("mp4".to_string()),
        audio_filter: None,
        video_filter: None,
    };

    // Try to stream with FFmpeg first
    let stream_result = custom_video.stream_with_ffmpeg(Some(ffmpeg_args)).await;

    match stream_result {
        Ok(stream) => {
            println!("Successfully created FFmpeg stream");

            // Collect all chunks into a single buffer
            let mut buffer = Vec::new();
            let mut chunk_count = 0;

            // Stream the video data
            loop {
                let chunk_result = match stream.chunk().await {
                    Ok(maybe_chunk) => maybe_chunk,
                    Err(e) => {
                        return Err(ServerFnError::ServerError(format!(
                            "Error downloading chunk: {}",
                            e
                        )));
                    }
                };

                match chunk_result {
                    Some(chunk) => {
                        buffer.extend_from_slice(&chunk);
                        chunk_count += 1;
                        if chunk_count % 10 == 0 {
                            println!(
                                "Downloaded {} chunks, current size: {} bytes",
                                chunk_count,
                                buffer.len()
                            );
                        }
                    }
                    None => break, // End of stream
                }
            }

            tracing::info!(
                "Download complete: {} chunks, {} bytes",
                chunk_count,
                buffer.len()
            );

            // Return the complete buffer
            return Ok(buffer);
        }
        Err(e) => {
            // FFmpeg not available or failed, try fallback method
            tracing::info!(
                "FFmpeg stream failed: {}. Falling back to standard stream.",
                e
            );

            // Use standard stream as fallback
            let stream = match custom_video.stream().await {
                Ok(s) => s,
                Err(e) => {
                    return Err(ServerFnError::ServerError(format!(
                        "Error streaming video: {}",
                        e
                    )));
                }
            };

            // Collect all chunks into a single buffer
            let mut buffer = Vec::new();
            let mut chunk_count = 0;

            // Stream the video data
            loop {
                let chunk_result = match stream.chunk().await {
                    Ok(maybe_chunk) => maybe_chunk,
                    Err(e) => {
                        return Err(ServerFnError::ServerError(format!(
                            "Error downloading chunk: {}",
                            e
                        )));
                    }
                };

                match chunk_result {
                    Some(chunk) => {
                        buffer.extend_from_slice(&chunk);
                        chunk_count += 1;
                        if chunk_count % 10 == 0 {
                            println!(
                                "Downloaded {} chunks, current size: {} bytes",
                                chunk_count,
                                buffer.len()
                            );
                        }
                    }
                    None => break, // End of stream
                }
            }

            println!(
                "Fallback download complete: {} chunks, {} bytes",
                chunk_count,
                buffer.len()
            );

            // Return the complete buffer
            return Ok(buffer);
        }
    }
}

// Alternative streaming approach with options
#[server(DownloadWithOptions)]
pub async fn download_with_options(
    url: String,
    audio_only: bool,
) -> Result<Vec<u8>, ServerFnError> {
    // Set up options based on user preference
    let video_options = if audio_only {
        VideoOptions {
            quality: VideoQuality::Lowest,
            filter: VideoSearchOptions::Audio,
            ..Default::default()
        }
    } else {
        VideoOptions::default()
    };

    // Create video instance with options
    let video = match Video::new_with_options(&url, video_options) {
        Ok(v) => v,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    // Stream the video data
    let stream = match video.stream().await {
        Ok(s) => s,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    // Collect all chunks into a single buffer
    let mut buffer = Vec::new();

    // The chunk() method returns a future that resolves to a Result<Option<Bytes>, VideoError>
    loop {
        // Get the next chunk as a Result<Option<Bytes>, VideoError>
        let chunk_result = match stream.chunk().await {
            Ok(maybe_chunk) => maybe_chunk,
            Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
        };

        // Check if we have a chunk or reached the end
        match chunk_result {
            Some(chunk) => buffer.extend_from_slice(&chunk),
            None => break, // End of stream
        }
    }

    // Return the complete buffer
    Ok(buffer)
}

// Additional function to get video details without downloading
#[server(GetVideoInfo)]
pub async fn get_video_info(url: String) -> Result<String, ServerFnError> {
    let video = match Video::new(&url) {
        Ok(v) => v,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    let info = match video.get_basic_info().await {
        Ok(i) => i,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    // Return video title and other basic details as JSON
    let details_json = match serde_json::to_string(&info.video_details) {
        Ok(json) => json,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    Ok(details_json)
}

#[server(SearchYoutube)]
pub async fn search_youtube(query: String) -> Result<(), ServerFnError> {
    let youtube = YouTube::new().unwrap();

    let res = youtube.search(&query, None).await;

    println!("{res:#?}");

    Ok(())
}

#[server(DownloadWithQuality)]
pub async fn download_with_quality(
    url: String,
    format_type: String,
    quality: String,
) -> Result<Vec<u8>, ServerFnError> {
    tracing::info!(
        "Download with quality - Format: {}, Quality: {}",
        format_type,
        quality
    );

    // Validate the URL first
    if !url.contains("youtube.com/watch?v=") && !url.contains("youtu.be/") {
        return Err(ServerFnError::ServerError(
            "Invalid YouTube URL. Please provide a valid YouTube video URL.".to_string(),
        ));
    }

    // For audio downloads, use the specialized audio-only approach
    if format_type.to_lowercase() == "audio" {
        return download_audio_only(url).await;
    }

    // For video downloads, use a specialized video approach
    return download_video_with_quality(url, quality).await;
}

// Specialized function for audio-only downloads
#[cfg(feature = "server")]
async fn download_audio_only(url: String) -> Result<Vec<u8>, ServerFnError> {
    println!("Using specialized audio-only download approach");

    // Create a video instance with audio-only options
    let options = VideoOptions {
        quality: VideoQuality::Highest, // For audio, highest quality is typically best
        filter: VideoSearchOptions::Audio, // Audio only
        ..Default::default()
    };

    let video = match Video::new_with_options(&url, options) {
        Ok(v) => v,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Error creating audio download: {}",
                e
            )))
        }
    };

    // Add FFmpeg args specifically for MP3 output
    let ffmpeg_args = FFmpegArgs {
        format: Some("mp3".to_string()), // Force MP3 output format
        audio_filter: None,
        video_filter: None,
    };

    // First try using FFmpeg to get a proper MP3
    println!("Attempting audio download with FFmpeg");
    let stream_result = video.stream_with_ffmpeg(Some(ffmpeg_args)).await;

    if let Ok(stream) = stream_result {
        println!("Successfully created FFmpeg audio stream");

        // Collect all chunks into a buffer
        let mut buffer = Vec::new();
        let mut chunk_count = 0;

        // Stream the audio data
        loop {
            match stream.chunk().await {
                Ok(Some(chunk)) => {
                    buffer.extend_from_slice(&chunk);
                    chunk_count += 1;
                    if chunk_count % 10 == 0 {
                        println!(
                            "Downloaded {} audio chunks, current size: {} bytes",
                            chunk_count,
                            buffer.len()
                        );
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    println!("Error downloading audio chunk: {}", e);
                    break;
                }
            }
        }

        if !buffer.is_empty() {
            println!(
                "Audio download complete: {} chunks, {} bytes",
                chunk_count,
                buffer.len()
            );
            return Ok(buffer);
        }
    } else {
        println!("FFmpeg approach failed, trying fallback");
    }

    // If FFmpeg fails, try the regular download
    println!("Falling back to standard audio download");
    let stream = match video.stream().await {
        Ok(s) => s,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Error streaming audio: {}",
                e
            )))
        }
    };

    // Collect all chunks into a buffer
    let mut buffer = Vec::new();
    let mut chunk_count = 0;

    // Stream the audio data
    loop {
        match stream.chunk().await {
            Ok(Some(chunk)) => {
                buffer.extend_from_slice(&chunk);
                chunk_count += 1;
                if chunk_count % 10 == 0 {
                    println!(
                        "Downloaded {} audio chunks, current size: {} bytes",
                        chunk_count,
                        buffer.len()
                    );
                }
            }
            Ok(None) => break,
            Err(e) => {
                return Err(ServerFnError::ServerError(format!(
                    "Error downloading audio chunk: {}",
                    e
                )))
            }
        }
    }

    if buffer.is_empty() {
        return Err(ServerFnError::ServerError(
            "Download resulted in empty audio data. Try a different video.".to_string(),
        ));
    }

    println!(
        "Audio download complete: {} chunks, {} bytes",
        chunk_count,
        buffer.len()
    );

    Ok(buffer)
}

// Specialized function for video downloads
#[cfg(feature = "server")]
async fn download_video_with_quality(
    url: String,
    quality: String,
) -> Result<Vec<u8>, ServerFnError> {
    tracing::info!("Using specialized video download approach");

    // Create a basic video instance first to get available formats
    let video = match Video::new(&url) {
        Ok(v) => v,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Error creating video instance: {}",
                e
            )))
        }
    };

    // Get video info to see available formats
    let info = match video.get_basic_info().await {
        Ok(i) => i,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Error getting video info: {}",
                e
            )))
        }
    };

    tracing::info!("Video found: {}", info.video_details.title);
    tracing::info!("Total formats available: {:?}", info.formats);

    // Find formats that have both video and audio
    let combined_formats: Vec<_> = info
        .formats
        .iter()
        .filter(|f| f.has_video && f.has_audio)
        .collect();

    // Find video-only formats
    let video_formats: Vec<_> = info
        .formats
        .iter()
        .filter(|f| f.has_video && !f.has_audio)
        .collect();

    // Find audio-only formats
    let audio_formats: Vec<_> = info
        .formats
        .iter()
        .filter(|f| !f.has_video && f.has_audio)
        .collect();

    tracing::info!(
        "Found {:?} formats with both video and audio",
        combined_formats
    );
    tracing::info!("Found {:?} video-only formats", video_formats);
    tracing::info!("Found {:?} audio-only formats", audio_formats);

    // Log info about some of the formats
    for (i, fmt) in combined_formats.iter().enumerate().take(3) {
        tracing::info!(
            "Combined format {}: itag={}, quality={:?}, has_video={}, has_audio={}, mime_type={:?}",
            i,
            fmt.itag,
            fmt.quality,
            fmt.has_video,
            fmt.has_audio,
            fmt.mime_type
        );
    }

    for (i, fmt) in video_formats.iter().enumerate().take(3) {
        tracing::info!(
            "Video format {}: itag={}, quality={:?}, mime_type={:?}",
            i,
            fmt.itag,
            fmt.quality,
            fmt.mime_type
        );
    }

    for (i, fmt) in audio_formats.iter().enumerate().take(3) {
        tracing::info!(
            "Audio format {}: itag={}, quality={:?}, mime_type={:?}",
            i,
            fmt.itag,
            fmt.quality,
            fmt.mime_type
        );
    }

    // Set quality option based on user preference
    let quality_option = match quality.to_lowercase().as_str() {
        "lowest" => VideoQuality::Lowest,
        _ => VideoQuality::Highest, // Default to highest for best video quality
    };

    // Create options with both video and audio specifically requested
    // We'll use FFmpeg to combine them properly
    let options = VideoOptions {
        quality: quality_option.clone(),
        filter: VideoSearchOptions::VideoAudio,
        ..Default::default()
    };

    // Create video instance with our options
    let video_instance = match Video::new_with_options(&url, options) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Error with video and audio options: {}", e);
            // Try with default options as fallback
            match Video::new(&url) {
                Ok(v) => v,
                Err(e) => {
                    return Err(ServerFnError::ServerError(format!(
                        "Error creating video: {}",
                        e
                    )))
                }
            }
        }
    };

    // Set up improved FFmpeg arguments for proper MP4 output with both video and audio
    let ffmpeg_args = FFmpegArgs {
        format: Some("mp4".to_string()),
        // Force AAC audio codec with decent bitrate for audio
        audio_filter: Some("-c:a aac -b:a 192k -ar 44100".to_string()),
        // Force H.264 video codec with good quality settings and compatibility
        video_filter: Some(
            "-c:v libx264 -preset medium -crf 22 -pix_fmt yuv420p -movflags +faststart".to_string(),
        ),
    };

    // Try to stream with FFmpeg - this should handle separate video/audio streams
    tracing::info!("Attempting video download with FFmpeg (handles separate streams)");

    // Always try FFmpeg first - it's our best shot at getting video+audio combined
    if let Ok(stream) = video_instance
        .stream_with_ffmpeg(Some(ffmpeg_args.clone()))
        .await
    {
        tracing::info!("Successfully created FFmpeg stream with optimized settings");

        // Collect the data
        let mut buffer = Vec::new();
        let mut chunk_count = 0;

        loop {
            match stream.chunk().await {
                Ok(Some(chunk)) => {
                    buffer.extend_from_slice(&chunk);
                    chunk_count += 1;
                    if chunk_count % 10 == 0 {
                        tracing::info!(
                            "Downloaded {} video chunks, current size: {} bytes",
                            chunk_count,
                            buffer.len()
                        );
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    tracing::error!("Error downloading video chunk: {}", e);
                    break;
                }
            }
        }

        if !buffer.is_empty() {
            tracing::info!(
                "Video download complete: {} chunks, {} bytes",
                chunk_count,
                buffer.len()
            );
            return Ok(buffer);
        }
    } else {
        tracing::error!("FFmpeg approach with optimized settings failed, trying alternative");
    }

    // If the first attempt failed, try another approach with more explicit format selection

    // Find best video format based on quality
    let best_video_format = if !video_formats.is_empty() {
        // Directly choose based on quality without using sorted_formats variable
        if quality.to_lowercase() == "lowest" {
            // Find the lowest resolution format
            video_formats
                .iter()
                .min_by_key(|f| f.height.unwrap_or(0))
                .cloned() // Clone the format to avoid reference issues
        } else {
            // Find the highest resolution format (default)
            video_formats
                .iter()
                .max_by_key(|f| f.height.unwrap_or(0))
                .cloned() // Clone the format to avoid reference issues
        }
    } else {
        None
    };

    // Find best audio format (prefer higher quality)
    let best_audio_format = if !audio_formats.is_empty() {
        // Return a clone rather than a reference
        audio_formats.first().cloned()
    } else {
        None
    };

    if let (Some(video_fmt), Some(audio_fmt)) = (best_video_format, best_audio_format) {
        tracing::info!(
            "Trying explicit format combo: Video itag={} ({}p), Audio itag={}",
            video_fmt.itag,
            video_fmt.height.unwrap_or(0),
            audio_fmt.itag
        );

        // Create custom options targeting these specific formats
        let options = VideoOptions {
            quality: quality_option.clone(),
            filter: VideoSearchOptions::VideoAudio,
            ..Default::default()
        };

        let explicit_video = match Video::new_with_options(&url, options) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error with explicit format options: {}", e);
                return Err(ServerFnError::ServerError(format!(
                    "Error with format selection: {}",
                    e
                )));
            }
        };

        // Try with FFmpeg again with our explicit format selection
        if let Ok(stream) = explicit_video.stream_with_ffmpeg(Some(ffmpeg_args)).await {
            tracing::info!("Successfully created explicit format FFmpeg stream");

            let mut buffer = Vec::new();
            let mut chunk_count = 0;

            loop {
                match stream.chunk().await {
                    Ok(Some(chunk)) => {
                        buffer.extend_from_slice(&chunk);
                        chunk_count += 1;
                        if chunk_count % 10 == 0 {
                            tracing::info!(
                                "Downloaded {} explicit format chunks, size: {} bytes",
                                chunk_count,
                                buffer.len()
                            );
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        tracing::error!("Error downloading explicit format chunk: {}", e);
                        break;
                    }
                }
            }

            if !buffer.is_empty() {
                tracing::info!(
                    "Explicit format download complete: {} chunks, {} bytes",
                    chunk_count,
                    buffer.len()
                );
                return Ok(buffer);
            }
        }
    }

    // Try using rusty_ytdl's default approach which might handle separate streams
    tracing::info!("Trying default download approach");
    let default_video = match Video::new(&url) {
        Ok(v) => v,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Error creating video: {}",
                e
            )))
        }
    };

    // Last resort: Try direct streaming with rusty_ytdl's default handling
    match default_video.stream().await {
        Ok(stream) => {
            tracing::info!("Successfully created default stream");

            // Collect the data
            let mut buffer = Vec::new();
            let mut chunk_count = 0;

            loop {
                match stream.chunk().await {
                    Ok(Some(chunk)) => {
                        buffer.extend_from_slice(&chunk);
                        chunk_count += 1;
                        if chunk_count % 10 == 0 {
                            tracing::info!(
                                "Downloaded {} default chunks, current size: {} bytes",
                                chunk_count,
                                buffer.len()
                            );
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        tracing::error!("Error downloading default chunk: {}", e);
                        break;
                    }
                }
            }

            if !buffer.is_empty() {
                tracing::info!(
                    "Default download complete: {} chunks, {} bytes",
                    chunk_count,
                    buffer.len()
                );
                return Ok(buffer);
            }

            Err(ServerFnError::ServerError(
                "Failed to download video with any method. The video might be restricted."
                    .to_string(),
            ))
        }
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Error streaming video: {}. The video might be restricted or unavailable.",
            e
        ))),
    }
}
