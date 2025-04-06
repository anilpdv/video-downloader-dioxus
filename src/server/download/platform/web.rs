use crate::components::download_progress::{DownloadInfo, DownloadStatus};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::{SinkExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::{Array, Date, Uint8Array};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, Url};
use web_sys::{Request, RequestInit, RequestMode, Response};

// Web-specific download functionality
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub progress: f64,
    pub downloaded_size: String,
    pub total_size: String,
    pub speed: String,
    pub eta: String,
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            progress: 0.0,
            downloaded_size: "0 KB".to_string(),
            total_size: "Unknown".to_string(),
            speed: "0 KB/s".to_string(),
            eta: "Unknown".to_string(),
        }
    }
}

// Create a blob URL from bytes for web downloads
pub fn create_blob_url(data: &[u8], mime_type: &str) -> Option<String> {
    let array = Array::new();
    let uint8_array = unsafe { Uint8Array::view(data) };
    array.push(&uint8_array);

    let options = BlobPropertyBag::new();
    options.set_type(mime_type);

    if let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&array, &options) {
        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
            return Some(url);
        }
    }

    None
}

// Convert file size in bytes to human-readable format
pub fn format_file_size(size_bytes: f64) -> String {
    if size_bytes < 1024.0 {
        format!("{:.0} B", size_bytes)
    } else if size_bytes < 1024.0 * 1024.0 {
        format!("{:.1} KB", size_bytes / 1024.0)
    } else if size_bytes < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB", size_bytes / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size_bytes / (1024.0 * 1024.0 * 1024.0))
    }
}

// Format duration in seconds to human-readable time
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        return format!("{}s", seconds);
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        return format!("{}m {}s", mins, secs);
    } else {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        return format!("{}h {}m", hours, mins);
    }
}

// Simulate download progress for web (in a real app, you'd use fetch with progress events)
pub async fn simulate_download_progress(
    total_size_bytes: f64,
    callback: js_sys::Function,
) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();

    // Simulate multiple progress updates
    for i in 1..=10 {
        let progress = i as f64 / 10.0;
        let downloaded = total_size_bytes * progress;

        let progress_data = DownloadProgress {
            progress,
            downloaded_size: format_file_size(downloaded),
            total_size: format_file_size(total_size_bytes),
            speed: format!("{} KB/s", (100.0 + (i as f64 * 50.0)) as u32),
            eta: format!("{}s", (10 - i)),
        };

        // Convert to JsValue to pass to callback
        let js_progress = match serde_json::to_string(&progress_data) {
            Ok(json_str) => JsValue::from_str(&json_str),
            Err(_) => JsValue::NULL,
        };
        callback.call1(&JsValue::NULL, &js_progress)?;

        // Wait for a bit to simulate download time
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    &resolve, 500, // 500ms delay between updates
                )
                .unwrap();
        });

        wasm_bindgen_futures::JsFuture::from(promise).await?;
    }

    Ok(())
}

// Function to download data with progress in web
pub async fn download_with_progress(
    url: &str,
    file_name: &str,
    progress_callback: js_sys::Function,
) -> Result<String, JsValue> {
    // In a real app, you would:
    // 1. Fetch the URL with progress reporting
    // 2. Convert response to blob
    // 3. Create object URL
    // 4. Trigger download

    // For this example, we'll simulate progress and create a fake blob URL
    let total_size = 15.0 * 1024.0 * 1024.0; // Simulate a 15MB file

    // Simulate progress updates
    simulate_download_progress(total_size, progress_callback.clone()).await?;

    // Create a dummy blob with "Hello World" (in a real app, this would be the file content)
    let dummy_data = "Hello World! This is a simulated download.".as_bytes();
    let mime_type = if file_name.ends_with(".mp3") {
        "audio/mpeg"
    } else if file_name.ends_with(".mp4") {
        "video/mp4"
    } else {
        "application/octet-stream"
    };

    if let Some(blob_url) = create_blob_url(dummy_data, mime_type) {
        // In a real app, you would download the blob URL automatically
        // For this example, we'll just return the URL for the UI to handle
        Ok(blob_url)
    } else {
        Err(JsValue::from_str("Failed to create blob URL"))
    }
}

// Create blob URL from response data
pub async fn create_blob_url_from_response(url: &str, _filename: &str) -> Result<String, String> {
    let mut opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|err| format!("Failed to create request: {:?}", err))?;

    let window = web_sys::window().ok_or_else(|| "No window".to_string())?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|err| format!("Failed to fetch: {:?}", err))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not a Response".to_string())?;

    let blob = JsFuture::from(
        resp.blob()
            .map_err(|err| format!("Failed to get blob: {:?}", err))?,
    )
    .await
    .map_err(|err| format!("Failed to await blob: {:?}", err))?;

    let blob: Blob = blob.dyn_into().map_err(|_| "Not a blob".to_string())?;

    let blob_url = Url::create_object_url_with_blob(&blob)
        .map_err(|err| format!("Failed to create object URL: {:?}", err))?;

    Ok(blob_url)
}

// Real download function with progress tracking for web
pub async fn download_with_progress_real<F>(
    url: &str,
    filename: &str,
    on_progress: F,
) -> Result<String, String>
where
    F: Fn(DownloadInfo) + 'static,
{
    // Create a channel for progress updates - we don't use this for now
    let (_tx, _rx) = channel::<DownloadInfo>(10);

    // Get file size with HEAD request
    let opts = RequestInit::new();
    let opts = {
        let mut o = opts;
        o.set_method("HEAD");
        o.set_mode(RequestMode::Cors);
        o
    };

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|err| format!("Failed to create request: {:?}", err))?;

    let window = web_sys::window().ok_or_else(|| "No window".to_string())?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|err| format!("Failed to fetch: {:?}", err))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not a Response".to_string())?;

    // Get content-length header if available
    let total_size = if let Ok(Some(content_length)) = resp.headers().get("content-length") {
        content_length.parse::<f64>().unwrap_or(1024.0 * 1024.0)
    } else {
        // Default to 1MB if no content-length
        1024.0 * 1024.0
    };

    // Clone URL and filename for the progress loop
    let url_clone = url.to_string();
    let filename_clone = filename.to_string();

    // Start download process in a background task
    wasm_bindgen_futures::spawn_local(async move {
        // Log the start of download
        log(&format!("Starting download for {}", url_clone));

        let mut download_info = DownloadInfo {
            url: url_clone.clone(),
            file_name: filename_clone.clone(),
            status: DownloadStatus::Downloading,
            progress: 0.0,
            downloaded_size: "0 B".to_string(),
            total_size: format_file_size(total_size),
            speed: "0 B/s".to_string(),
            eta: "calculating...".to_string(),
            blob_url: None,
        };

        // Call the progress callback with initial state
        on_progress(download_info.clone());

        // Start the actual download directly - don't use a HEAD request which might not work
        let fetch_opts = RequestInit::new();
        // No need to set method as GET is default

        let fetch_request = match Request::new_with_str_and_init(&url_clone, &fetch_opts) {
            Ok(req) => req,
            Err(err) => {
                download_info.status =
                    DownloadStatus::Failed(format!("Failed to create request: {:?}", err));
                on_progress(download_info);
                return;
            }
        };

        let fetch_resp_value = match JsFuture::from(window.fetch_with_request(&fetch_request)).await
        {
            Ok(resp) => resp,
            Err(err) => {
                download_info.status =
                    DownloadStatus::Failed(format!("Failed to fetch: {:?}", err));
                on_progress(download_info);
                return;
            }
        };

        let fetch_resp: Response = match fetch_resp_value.dyn_into() {
            Ok(resp) => resp,
            Err(_) => {
                download_info.status =
                    DownloadStatus::Failed("Response is not a Response".to_string());
                on_progress(download_info);
                return;
            }
        };

        if !fetch_resp.ok() {
            download_info.status =
                DownloadStatus::Failed(format!("HTTP error: {}", fetch_resp.status()));
            on_progress(download_info);
            return;
        }

        // Get content type
        let mime_type = match fetch_resp.headers().get("content-type") {
            Ok(Some(ct)) => ct,
            _ => {
                if filename_clone.ends_with(".mp4") {
                    "video/mp4".to_string()
                } else if filename_clone.ends_with(".mp3") {
                    "audio/mpeg".to_string()
                } else {
                    "application/octet-stream".to_string()
                }
            }
        };

        log(&format!("Content type: {}", mime_type));

        // For simplicity, we'll use the Blob API directly
        let blob_promise = match fetch_resp.blob() {
            Ok(promise) => promise,
            Err(err) => {
                download_info.status =
                    DownloadStatus::Failed(format!("Failed to get blob: {:?}", err));
                on_progress(download_info);
                return;
            }
        };

        // Log we're about to start simulating progress
        log("Starting progress simulation");

        // Record start time using JavaScript Date API for timing
        let download_start_time = Date::now();

        // Since WebAssembly doesn't support tracking actual download progress yet,
        // we'll simulate it while we wait for the blob to load
        let chunks_count = 10;
        for i in 1..chunks_count {
            let progress = i as f64 / chunks_count as f64;
            let downloaded = total_size * progress;

            // Calculate elapsed time using JavaScript Date API
            let now = Date::now();
            let elapsed_ms = now - download_start_time;
            let elapsed_sec = elapsed_ms / 1000.0;

            download_info.progress = progress;
            download_info.downloaded_size = format_file_size(downloaded);

            // Calculate simulated speed
            let speed = if elapsed_sec > 0.0 {
                downloaded / elapsed_sec
            } else {
                1000.0
            };
            download_info.speed = format!("{}/s", format_file_size(speed));

            // Estimate time remaining
            let remaining_bytes = total_size - downloaded;
            let eta_seconds = if speed > 0.0 {
                (remaining_bytes / speed) as i64
            } else {
                0
            };
            download_info.eta = format_duration(eta_seconds);

            on_progress(download_info.clone());

            // Wait a bit between progress updates
            TimeoutFuture::new(300).await;
        }

        // Get the actual blob
        log("Waiting for blob to be available");
        let blob_result = JsFuture::from(blob_promise).await;
        let blob = match blob_result {
            Ok(blob_value) => match blob_value.dyn_into::<Blob>() {
                Ok(b) => b,
                Err(_) => {
                    download_info.status =
                        DownloadStatus::Failed("Failed to convert to Blob".to_string());
                    on_progress(download_info);
                    return;
                }
            },
            Err(err) => {
                download_info.status =
                    DownloadStatus::Failed(format!("Failed to await blob: {:?}", err));
                on_progress(download_info);
                return;
            }
        };

        // Create object URL
        log("Creating blob URL");
        let blob_url = match Url::create_object_url_with_blob(&blob) {
            Ok(url) => url,
            Err(err) => {
                download_info.status =
                    DownloadStatus::Failed(format!("Failed to create object URL: {:?}", err));
                on_progress(download_info);
                return;
            }
        };

        // Log to console for debugging
        log(&format!(
            "Download complete. Blob URL created: {}",
            blob_url
        ));

        // Mark download as complete
        download_info.status = DownloadStatus::Completed;
        download_info.blob_url = Some(blob_url.clone());
        download_info.progress = 1.0;
        download_info.downloaded_size = download_info.total_size.clone();
        on_progress(download_info);

        // Trigger download automatically
        if let Err(err) = download_file(&blob_url, &filename_clone) {
            log(&format!("Error triggering download: {}", err));
        }
    });

    // Return immediately, progress will be reported via callback
    Ok("Download started".to_string())
}

// Helper function to log to console
pub fn log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

// Helper function to trigger a file download in the browser
pub fn download_file(blob_url: &str, filename: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "No window".to_string())?;
    let document = window.document().ok_or_else(|| "No document".to_string())?;

    // Create a temporary anchor element
    let a = document
        .create_element("a")
        .map_err(|err| format!("Failed to create element: {:?}", err))?;

    // Set href and download attributes
    a.set_attribute("href", blob_url)
        .map_err(|err| format!("Failed to set href: {:?}", err))?;
    a.set_attribute("download", filename)
        .map_err(|err| format!("Failed to set download: {:?}", err))?;

    // Hide the element
    a.set_attribute("style", "display: none")
        .map_err(|err| format!("Failed to set style: {:?}", err))?;

    // Add to document, click it, and remove it
    document
        .body()
        .ok_or_else(|| "No body".to_string())?
        .append_child(&a)
        .map_err(|err| format!("Failed to append child: {:?}", err))?;

    let a_element: web_sys::HtmlElement =
        a.dyn_into().map_err(|_| "Not an HTMLElement".to_string())?;

    a_element.click();

    // Clean up
    a_element
        .parent_node()
        .ok_or_else(|| "No parent node".to_string())?
        .remove_child(&a_element)
        .map_err(|err| format!("Failed to remove child: {:?}", err))?;

    Ok(())
}

// Helper function to generate random numbers in WASM
fn rand() -> f64 {
    js_sys::Math::random()
}
