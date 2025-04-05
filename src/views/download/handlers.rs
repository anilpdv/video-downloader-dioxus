use crate::server::download::download_with_quality;
// Only import what we need
#[cfg(feature = "web")]
use crate::views::download::platforms::create_blob_url;
use crate::views::download::types::{FormatType, Quality};
use dioxus::prelude::*;

// At the top of the file, create a platform-specific timing module
#[cfg(feature = "web")]
mod timing {
    use js_sys::Date;

    pub struct WebTime {
        start_time: f64,
    }

    impl WebTime {
        pub fn new() -> Self {
            Self {
                start_time: Date::now(),
            }
        }

        pub fn elapsed_secs_f32(&self) -> f32 {
            let now = Date::now();
            ((now - self.start_time) / 1000.0) as f32
        }
    }
}

#[cfg(not(feature = "web"))]
mod timing {
    use std::time::Instant;

    pub struct NativeTime {
        start_time: Instant,
    }

    impl NativeTime {
        pub fn new() -> Self {
            Self {
                start_time: Instant::now(),
            }
        }

        pub fn elapsed_secs_f32(&self) -> f32 {
            self.start_time.elapsed().as_secs_f32()
        }
    }
}

// Create type aliases for the platform-specific time
#[cfg(not(feature = "web"))]
use timing::NativeTime as TimeTracker;
#[cfg(feature = "web")]
use timing::WebTime as TimeTracker;

// Update filename based on selected format
pub fn update_filename(filename: &str, format_type: &FormatType) -> String {
    let extension = format_type.get_extension();

    // Don't modify if empty
    if filename.is_empty() {
        return String::new();
    }

    // Remove any existing extension and add the correct one
    let base_name = if filename.contains('.') {
        let parts: Vec<&str> = filename.split('.').collect();
        parts[0].to_string()
    } else {
        filename.to_string()
    };

    format!("{}.{}", base_name, extension)
}

// Execute download and handle results
pub fn execute_download(
    url: String,
    format_type: FormatType,
    quality: Quality,
    download_in_progress: &Signal<bool>,
    progress_percent: &Signal<i32>,
    status_sig: &Signal<Option<String>>,
    progress_eta: &Signal<String>,
    loading: &Signal<bool>,
    error_signal: &Signal<Option<String>>,
    download_data: &Signal<Option<Vec<u8>>>,
    blob_url: &Signal<Option<String>>,
    download_ready: &Signal<bool>,
) {
    spawn({
        let url_clone = url.clone();
        let format_str = format_type.to_string();
        let quality_str = quality.to_string();
        let mut download_in_progress = download_in_progress.clone();
        let mut status_sig = status_sig.clone();
        let mut progress_percent = progress_percent.clone();
        let mut progress_eta = progress_eta.clone();
        let mut error_signal = error_signal.clone();
        let mut loading = loading.clone();
        let mut download_data = download_data.clone();
        let mut blob_url = blob_url.clone();
        let mut download_ready = download_ready.clone();
        #[cfg(feature = "web")]
        let format_type = format_type.clone();

        async move {
            // Add debug status message
            status_sig.set(Some(format!(
                "Starting download for {} as {} format, quality: {}",
                url_clone, format_str, quality_str
            )));

            // Start timer for tracking elapsed time
            let start_time = TimeTracker::new();

            // Create a progress checker task that runs in parallel
            let url_for_progress = url_clone.clone();
            let mut download_in_progress_for_polling = download_in_progress.clone();
            let mut progress_percent_for_polling = progress_percent.clone();
            let mut status_sig_for_polling = status_sig.clone();
            let mut progress_eta_for_polling = progress_eta.clone();

            // Start a background task to poll for progress updates
            let _progress_task = spawn({
                let url = url_for_progress.clone();
                async move {
                    // Import GetDownloadProgress function
                    use crate::server::download::handlers::get_download_progress;
                    use crate::views::download::platforms::format_eta;

                    // Poll interval - check every second (in milliseconds for web)
                    #[cfg(feature = "web")]
                    let poll_interval_ms = 1000;

                    #[cfg(not(feature = "web"))]
                    let poll_interval = std::time::Duration::from_millis(1000);

                    // Store time-based progress information using platform-specific time tracking
                    #[cfg(feature = "web")]
                    let start_time_ms = js_sys::Date::now();

                    #[cfg(not(feature = "web"))]
                    let start_time = std::time::Instant::now();

                    // Define time-based progress stages with fixed percentages
                    let stages = [
                        // (time in seconds, max percentage)
                        (0.0, 0),   // Start at 0%
                        (1.0, 5),   // After 1s: 5%
                        (2.0, 10),  // After 2s: 10%
                        (3.0, 15),  // After 3s: 15%
                        (4.0, 20),  // After 4s: 20%
                        (6.0, 25),  // After 6s: 25%
                        (8.0, 30),  // After 8s: 30%
                        (10.0, 35), // After 10s: 35%
                        (12.0, 40), // After 12s: 40%
                        (14.0, 45), // After 14s: 45%
                        (16.0, 50), // After 16s: 50%
                        (18.0, 55), // After 18s: 55%
                        (20.0, 60), // After 20s: 60%
                        (23.0, 65), // After 23s: 65%
                        (26.0, 70), // After 26s: 70%
                        (30.0, 75), // After 30s: 75%
                        (35.0, 80), // After 35s: 80%
                        (40.0, 85), // After 40s: 85%
                        (45.0, 90), // After 45s: 90%
                        (50.0, 95), // After 50s: 95%
                    ];

                    // Track download state
                    let mut current_progress = 0;
                    let mut last_status = String::new();
                    let mut download_complete = false;

                    // Track when we can assume the download is complete
                    let mut seen_backend_complete = false;

                    // Platform-specific time tracking for last status update
                    #[cfg(feature = "web")]
                    let mut last_status_update_ms = js_sys::Date::now();

                    #[cfg(not(feature = "web"))]
                    let mut last_status_update = std::time::Instant::now();

                    // Keep polling while download is in progress
                    while download_in_progress_for_polling() {
                        // Get actual progress from server
                        let backend_info = match get_download_progress(url.clone()).await {
                            Ok(info) => Some(info),
                            Err(e) => {
                                tracing::warn!("Failed to get download progress: {}", e);
                                None
                            }
                        };

                        // Calculate time-based progress first
                        #[cfg(feature = "web")]
                        let elapsed_secs = (js_sys::Date::now() - start_time_ms) / 1000.0;

                        #[cfg(not(feature = "web"))]
                        let elapsed_secs = start_time.elapsed().as_secs_f64();

                        // Find appropriate stage based on elapsed time
                        let time_progress = {
                            let mut progress = stages[0].1;
                            for (stage_time, stage_progress) in stages.iter() {
                                if elapsed_secs >= *stage_time {
                                    progress = *stage_progress;
                                } else {
                                    break;
                                }
                            }
                            progress
                        };

                        // Check if backend indicates completion
                        if let Some((downloaded, total, eta_seconds, status)) = backend_info {
                            // Backend progress calculation (0-100)
                            let backend_pct = if total > 0 {
                                ((downloaded as f64 / total as f64) * 100.0) as i32
                            } else {
                                downloaded as i32
                            };

                            // Detect download completion from status or 100% progress
                            if status.contains("complete")
                                || status.contains("Complete")
                                || status.contains("finished")
                                || status.contains("Finished")
                                || backend_pct >= 100
                            {
                                seen_backend_complete = true;
                            }

                            // Calculate elapsed time for status update checks
                            #[cfg(feature = "web")]
                            let status_elapsed_secs =
                                (js_sys::Date::now() - last_status_update_ms) / 1000.0;

                            #[cfg(not(feature = "web"))]
                            let status_elapsed_secs = last_status_update.elapsed().as_secs() as f64;

                            // Use the status for messages but ignore backend percentage jumps
                            if status != last_status && status_elapsed_secs >= 2.0 {
                                // Clean up status message and remove any existing percentages
                                let clean_status = if status.contains("%") {
                                    // Remove any existing percentage in the status
                                    let percent_pos = status.find('%').unwrap_or(status.len());
                                    if percent_pos > 3 {
                                        // Find where the percentage starts (usually a digit)
                                        let mut start_pos = percent_pos - 1;
                                        while start_pos > 0
                                            && status
                                                .chars()
                                                .nth(start_pos - 1)
                                                .map_or(false, |c| {
                                                    c.is_digit(10) || c == ' ' || c == '('
                                                })
                                        {
                                            start_pos -= 1;
                                        }

                                        // Combine the parts without the percentage
                                        format!(
                                            "{}{}",
                                            status[0..start_pos].trim_end(),
                                            status[(percent_pos + 1)..].trim_start()
                                        )
                                    } else {
                                        status.clone()
                                    }
                                } else {
                                    status.clone()
                                };

                                // Add our calculated percentage
                                let status_msg = if clean_status.is_empty() {
                                    format!("Downloading... ({}%)", current_progress)
                                } else {
                                    format!("{} ({}%)", clean_status, current_progress)
                                };

                                status_sig_for_polling.set(Some(status_msg));
                                last_status = status;

                                #[cfg(feature = "web")]
                                {
                                    last_status_update_ms = js_sys::Date::now();
                                }

                                #[cfg(not(feature = "web"))]
                                {
                                    last_status_update = std::time::Instant::now();
                                }
                            }

                            // Format ETA if available
                            if eta_seconds > 0 {
                                progress_eta_for_polling.set(format_eta(eta_seconds));
                            }
                        }

                        // If download is complete according to backend, jump to 100%
                        if seen_backend_complete && !download_complete {
                            current_progress = 100;
                            progress_percent_for_polling.set(current_progress);
                            status_sig_for_polling
                                .set(Some("Download complete! (100%)".to_string()));
                            download_complete = true;
                        }
                        // Otherwise use time-based progress and update status when needed
                        else {
                            // Calculate new progress based on time
                            let new_time_progress = {
                                let mut progress = stages[0].1;
                                for (stage_time, stage_progress) in stages.iter() {
                                    if elapsed_secs >= *stage_time {
                                        progress = *stage_progress;
                                    } else {
                                        break;
                                    }
                                }
                                progress
                            };

                            // Only update if progress has increased
                            if new_time_progress > current_progress {
                                // Update progress
                                current_progress = new_time_progress;
                                progress_percent_for_polling.set(current_progress);

                                // Calculate elapsed time for milestone checks
                                #[cfg(feature = "web")]
                                let milestone_elapsed_secs =
                                    (js_sys::Date::now() - last_status_update_ms) / 1000.0;

                                #[cfg(not(feature = "web"))]
                                let milestone_elapsed_secs =
                                    last_status_update.elapsed().as_secs() as f64;

                                // For certain milestone percentages, update status message
                                if (current_progress == 25
                                    || current_progress == 50
                                    || current_progress == 75
                                    || current_progress == 85)
                                    && milestone_elapsed_secs >= 2.0
                                {
                                    let milestone_message = match current_progress {
                                        25 => "Downloading video... (25%)",
                                        50 => "Download half complete (50%)",
                                        75 => "Download almost complete (75%)",
                                        85 => "Processing download... (85%)",
                                        _ => "Downloading... ({}%)",
                                    };

                                    status_sig_for_polling.set(Some(
                                        milestone_message
                                            .replace("{}", &current_progress.to_string()),
                                    ));

                                    #[cfg(feature = "web")]
                                    {
                                        last_status_update_ms = js_sys::Date::now();
                                    }

                                    #[cfg(not(feature = "web"))]
                                    {
                                        last_status_update = std::time::Instant::now();
                                    }
                                }
                            }

                            // Calculate elapsed time for stuck download check
                            #[cfg(feature = "web")]
                            let stuck_elapsed_secs =
                                (js_sys::Date::now() - last_status_update_ms) / 1000.0;

                            #[cfg(not(feature = "web"))]
                            let stuck_elapsed_secs = last_status_update.elapsed().as_secs() as f64;

                            // Additional check for stuck downloads - if we've been at a high percentage for too long
                            if current_progress >= 85
                                && !seen_backend_complete
                                && stuck_elapsed_secs >= 5.0
                            {
                                // Update status to indicate we're still working
                                status_sig_for_polling.set(Some(format!(
                                    "Processing video... ({}%)",
                                    current_progress
                                )));

                                #[cfg(feature = "web")]
                                {
                                    last_status_update_ms = js_sys::Date::now();
                                }

                                #[cfg(not(feature = "web"))]
                                {
                                    last_status_update = std::time::Instant::now();
                                }
                            }
                        }

                        // Delay before next poll
                        #[cfg(feature = "web")]
                        {
                            use js_sys::Promise;
                            use wasm_bindgen::prelude::*;
                            use wasm_bindgen_futures::JsFuture;

                            if let Some(window) = web_sys::window() {
                                let promise = Promise::new(&mut |resolve, _| {
                                    let closure = Closure::once_into_js(move || {
                                        resolve.call0(&JsValue::NULL).unwrap();
                                    });

                                    let _ = window
                                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                                            closure.as_ref().unchecked_ref(),
                                            poll_interval_ms,
                                        );
                                });

                                let _ = JsFuture::from(promise).await;
                            }
                        }

                        #[cfg(not(feature = "web"))]
                        {
                            tokio::time::sleep(poll_interval).await;
                        }

                        // Stop checking if we're no longer downloading
                        if !download_in_progress_for_polling() {
                            break;
                        }
                    }
                }
            });

            // Execute the server function to start the actual download
            let result =
                download_with_quality(url_clone, format_str.clone(), quality_str.clone()).await;

            // Show elapsed time in status
            let elapsed = start_time.elapsed_secs_f32();
            status_sig.set(Some(format!(
                "Download request completed in {:.1}s",
                elapsed
            )));

            // Stop progress polling
            download_in_progress.set(false);

            match result {
                Ok(data) => {
                    // Set progress to 100% for completion
                    progress_percent.set(100);
                    progress_eta.set("0s".into());

                    if data.is_empty() {
                        error_signal
                            .set(Some("Download resulted in empty data. Try again.".into()));
                        status_sig.set(Some("Download failed - server returned empty data".into()));
                    } else {
                        // Process the data based on platform
                        #[cfg(feature = "web")]
                        {
                            // Create blob URL for web
                            if let Some(url_string) =
                                create_blob_url(&data, format_type.get_mime_type())
                            {
                                blob_url.set(Some(url_string));
                            }
                        }

                        status_sig.set(Some(format!(
                            "Download complete! File size: {:.2} MB. Click button to save.",
                            data.len() as f64 / (1024.0 * 1024.0)
                        )));

                        download_data.set(Some(data));
                        download_ready.set(true);
                    }
                }
                Err(e) => {
                    // Handle error
                    error_signal.set(Some(format!("Download failed: {}", e)));
                    status_sig.set(Some("Download error occurred".into()));
                }
            }

            // Set loading to false
            loading.set(false);
        }
    });
}
