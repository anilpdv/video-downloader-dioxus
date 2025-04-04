use crate::server::download::download_with_quality;
use crate::views::download::platforms::{create_blob_url, format_eta};
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

// Handle the download progress simulation
pub fn simulate_download_progress(
    simulating: &Signal<bool>,
    sim_progress: &Signal<i32>,
    status_sig: &Signal<Option<String>>,
    sim_eta: &Signal<String>,
    format_type: &FormatType,
    quality: &Quality,
    error_signal: &Signal<Option<String>>,
) {
    // Spawn a simplified progress simulation that runs every 2 seconds
    spawn({
        let mut simulating = simulating.clone();
        let mut sim_progress = sim_progress.clone();
        let mut status_sig = status_sig.clone();
        let mut sim_eta = sim_eta.clone();
        let mut error_signal = error_signal.clone();

        async move {
            // Start at 5% immediately
            sim_progress.set(5);
            status_sig.set(Some("Download started (5%)".to_string()));

            let mut current = 5;
            let step_size = 5;
            let mut time_waited = 0;
            let timeout_duration = 180; // 3 minutes max

            while simulating() && current < 85 {
                // Sleep for 2 seconds to simulate progress
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

                            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                closure.as_ref().unchecked_ref(),
                                2000,
                            );
                        });

                        let _ = JsFuture::from(promise).await;
                    }
                }

                #[cfg(not(feature = "web"))]
                {
                    use std::thread;
                    use std::time::Duration;
                    thread::sleep(Duration::from_secs(2));
                }

                if !simulating() {
                    break;
                }

                // Increment progress
                current += step_size;
                sim_progress.set(current);

                // Update status message
                status_sig.set(Some(format!("Downloading... ({}%)", current)));

                // Calculate and update ETA
                let remaining = (85 - current) / step_size;
                let eta_seconds = (remaining * 2) as u64; // 2 seconds per step, convert to u64
                sim_eta.set(format_eta(eta_seconds));

                // Check for timeout
                time_waited += 2;
                if time_waited > timeout_duration {
                    status_sig.set(Some("Download taking longer than expected...".to_string()));
                    if time_waited > timeout_duration + 60 {
                        // After 4 minutes total, assume something went wrong
                        error_signal.set(Some("Download timed out. Please try again.".into()));
                        simulating.set(false);
                        break;
                    }
                }
            }
        }
    });
}

// Execute download and handle results
pub fn execute_download(
    url: String,
    format_type: FormatType,
    quality: Quality,
    simulating: &Signal<bool>,
    sim_progress: &Signal<i32>,
    status_sig: &Signal<Option<String>>,
    sim_eta: &Signal<String>,
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
        let mut simulating = simulating.clone();
        let mut status_sig = status_sig.clone();
        let mut sim_progress = sim_progress.clone();
        let mut sim_eta = sim_eta.clone();
        let mut error_signal = error_signal.clone();
        let mut loading = loading.clone();
        let mut download_data = download_data.clone();
        let mut blob_url = blob_url.clone();
        let mut download_ready = download_ready.clone();
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
            let mut simulating_for_progress = simulating.clone();
            let mut sim_progress_for_progress = sim_progress.clone();
            let mut status_sig_for_progress = status_sig.clone();
            let mut sim_eta_for_progress = sim_eta.clone();

            // Start a background task to poll for progress updates
            let progress_task = spawn({
                let url = url_for_progress.clone();
                async move {
                    // Import GetDownloadProgress function
                    use crate::server::download::handlers::get_download_progress;

                    // Check progress every 250ms while download is in progress
                    while simulating_for_progress() {
                        match get_download_progress(url.clone()).await {
                            Ok((downloaded, total, eta_seconds, status)) => {
                                // Calculate progress percentage (0-100)
                                let progress_pct = if total > 0 {
                                    ((downloaded as f64 / total as f64) * 100.0) as i32
                                } else {
                                    // If total is 0, use the raw downloaded value (assuming 0-100 range)
                                    downloaded as i32
                                };

                                // Update UI with real progress
                                sim_progress_for_progress.set(progress_pct);
                                status_sig_for_progress.set(Some(status.clone()));

                                // Format ETA
                                if eta_seconds > 0 {
                                    use crate::views::download::platforms::format_eta;
                                    sim_eta_for_progress.set(format_eta(eta_seconds));
                                }
                            }
                            Err(e) => {
                                // If we can't get progress, just continue - don't update UI
                                tracing::warn!("Failed to get download progress: {}", e);
                            }
                        }

                        // Short delay between progress checks
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
                                            250, // 250ms delay
                                        );
                                });

                                let _ = JsFuture::from(promise).await;
                            }
                        }

                        #[cfg(not(feature = "web"))]
                        {
                            use std::time::Duration;
                            tokio::time::sleep(Duration::from_millis(250)).await;
                        }

                        // Stop checking if we're no longer simulating
                        if !simulating_for_progress() {
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

            // Stop simulation and progress polling
            simulating.set(false);

            match result {
                Ok(data) => {
                    // Set progress to 100% for completion
                    sim_progress.set(100);
                    sim_eta.set("0s".into());

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
