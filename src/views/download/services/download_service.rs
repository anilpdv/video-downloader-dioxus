use crate::server::download::download_with_quality;
use crate::views::download::platforms::create_blob_url;
use crate::views::download::types::{FormatType, Quality};
use dioxus::prelude::*;

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
        let format_type_clone = format_type.clone();

        async move {
            // Start progress tracking in a separate task
            super::progress_service::track_download_progress(
                url_clone.clone(),
                &download_in_progress,
                &progress_percent,
                &status_sig,
                &progress_eta,
            );

            // Execute the server function to start the actual download
            let result =
                download_with_quality(url_clone, format_str.clone(), quality_str.clone()).await;

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
                                create_blob_url(&data, format_type_clone.get_mime_type())
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
