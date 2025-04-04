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

                    // Use a longer delay between progress checks to reduce request frequency
                    let poll_interval = std::time::Duration::from_millis(1000);

                    // Track the last progress value to implement smoothing
                    let mut last_progress = 0;
                    let mut last_status = String::new();
                    let mut status_update_timer = std::time::Instant::now();

                    // Track when download began for time-based progress
                    let download_start_time = std::time::Instant::now();
                    let estimated_completion_seconds = 180.0; // Estimate 3 minutes for full download

                    // Count how many times we've seen a high percentage to determine if it's stuck
                    let mut high_percent_count = 0;

                    // Are we in the "downloading" phase (past initialization)
                    let mut in_download_phase = false;

                    // Check progress while download is in progress
                    while download_in_progress_for_polling() {
                        match get_download_progress(url.clone()).await {
                            Ok((downloaded, total, eta_seconds, status)) => {
                                // Mark that we're in download phase if status contains "Downloading"
                                if status.contains("Downloading") {
                                    in_download_phase = true;
                                }

                                // Calculate actual progress from backend (0-100)
                                let backend_progress = if total > 0 {
                                    ((downloaded as f64 / total as f64) * 100.0) as i32
                                } else {
                                    downloaded as i32
                                };

                                // Calculate time-based progress component (0-100)
                                let elapsed_seconds = download_start_time.elapsed().as_secs_f64();
                                let time_ratio =
                                    (elapsed_seconds / estimated_completion_seconds).min(1.0);
                                let time_progress = (time_ratio * 90.0) as i32; // Max 90% from time

                                // Detect if backend is reporting a large jump (like 7% → 85%)
                                let is_large_jump = backend_progress > last_progress + 20;

                                // Handle case where backend jumps to high percentage and stalls
                                if backend_progress > 80 {
                                    high_percent_count += 1;

                                    // If we've seen high percentages multiple times, it might be stalled
                                    if high_percent_count > 3 {
                                        // Calculate a smoother progress based on time passed
                                        // This will show progress from 80% to 95% based on time
                                        let stall_progress =
                                            80 + ((time_ratio * 15.0) as i32).min(15);

                                        // Only update if the smooth progress is increasing
                                        if stall_progress > last_progress {
                                            progress_percent_for_polling.set(stall_progress);
                                            last_progress = stall_progress;

                                            // Update status to indicate "Processing..."
                                            if status_update_timer.elapsed().as_secs() >= 3 {
                                                let progress_msg =
                                                    format!("Processing... {}%", stall_progress);
                                                status_sig_for_polling.set(Some(progress_msg));
                                                status_update_timer = std::time::Instant::now();
                                            }
                                        }
                                    }
                                } else {
                                    high_percent_count = 0;
                                }

                                // Calculate smooth progress value based on current state
                                let smooth_progress = if in_download_phase {
                                    if is_large_jump {
                                        // If backend made a big jump, create intermediate values
                                        // Progress based on time but capped below backend value
                                        let max_allowed = backend_progress - 5;
                                        last_progress + 1.min(max_allowed - last_progress)
                                    } else {
                                        // Otherwise use the backend value with slight smoothing
                                        // Never go backward, and never jump more than 2% at once
                                        let next_progress = backend_progress.max(last_progress);
                                        last_progress + (next_progress - last_progress).min(2)
                                    }
                                } else {
                                    // During initialization phase, use time-based progress
                                    // Cap at 20% during initialization
                                    time_progress.min(20)
                                };

                                // Only update UI if progress has changed
                                if smooth_progress > last_progress {
                                    progress_percent_for_polling.set(smooth_progress);
                                    last_progress = smooth_progress;
                                }

                                // Update status if it's significantly different
                                if status != last_status
                                    && status_update_timer.elapsed().as_secs() >= 3
                                {
                                    // Show percentage in status message
                                    let status_with_percent = if !status.contains("%") {
                                        format!("{} ({}%)", status, smooth_progress)
                                    } else {
                                        status.clone()
                                    };

                                    status_sig_for_polling.set(Some(status_with_percent));
                                    last_status = status;
                                    status_update_timer = std::time::Instant::now();
                                }

                                // Format ETA
                                if eta_seconds > 0 {
                                    progress_eta_for_polling.set(format_eta(eta_seconds));
                                }
                            }
                            Err(e) => {
                                // If we can't get progress, just continue - don't update UI
                                tracing::warn!("Failed to get download progress: {}", e);
                            }
                        }

                        // Delay between progress checks
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
                                            poll_interval.as_millis() as i32,
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
