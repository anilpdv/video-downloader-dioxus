use crate::server::download::download_with_quality;
use dioxus::prelude::*;

// Web platform-specific imports
#[cfg(feature = "web")]
use js_sys::{Array, Uint8Array};
#[cfg(feature = "web")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(feature = "web")]
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, HtmlMediaElement, Url};

// For non-web platforms
#[cfg(not(feature = "web"))]
use base64::{engine::general_purpose::STANDARD, Engine};

// Enum for format type selection
#[derive(Clone, PartialEq)]
pub enum FormatType {
    Video,
    Audio,
}

// Enum for quality selection
#[derive(Clone, PartialEq)]
pub enum Quality {
    Highest,
    Medium,
    Lowest,
}

// Web-specific blob URL implementation
#[cfg(feature = "web")]
fn use_blob_url(data: Option<Vec<u8>>, mime_type: &str) -> Signal<Option<String>> {
    let mut url = use_signal(|| None::<String>);

    // Create blob URL
    if let Some(bytes) = data {
        let uint8_array = Uint8Array::new_with_length(bytes.len() as u32);
        uint8_array.copy_from(&bytes);

        let array = Array::new();
        array.push(&uint8_array.buffer().into());

        let mut blob_options = BlobPropertyBag::new();
        blob_options.type_(mime_type);

        if let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&array, &blob_options) {
            if let Ok(blob_url) = Url::create_object_url_with_blob(&blob) {
                url.set(Some(blob_url));
            }
        }
    }

    url
}

// Non-web fallback implementation using base64
#[cfg(not(feature = "web"))]
fn use_blob_url(data: Option<Vec<u8>>, mime_type: &str) -> Signal<Option<String>> {
    let mut url = use_signal(|| None::<String>);

    if let Some(bytes) = data {
        let base64_data = STANDARD.encode(&bytes);
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);
        url.set(Some(data_url));
    }

    url
}

// Web-specific download trigger
#[cfg(feature = "web")]
fn trigger_download(url: &str, filename: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(anchor) = document.create_element("a") {
                if let Ok(anchor_element) = anchor.dyn_into::<HtmlAnchorElement>() {
                    anchor_element.set_href(url);
                    anchor_element.set_download(filename);

                    // Set display:none using setAttribute instead of style()
                    let _ = anchor_element.set_attribute("style", "display: none");

                    if let Some(body) = document.body() {
                        let _ = body.append_child(&anchor_element);
                        anchor_element.click();
                        let _ = body.remove_child(&anchor_element);
                    }
                }
            }
        }
    }
}

// No-op for non-web platforms
#[cfg(not(feature = "web"))]
fn trigger_download(_url: &str, _filename: &str) {
    // This is a no-op for non-web platforms
    // Downloads happen via data URLs in the anchor element
}

#[component]
pub fn Download() -> Element {
    let mut url = use_signal(String::new);
    let mut filename = use_signal(String::new);
    let mut format_type = use_signal(|| FormatType::Video);
    let mut quality = use_signal(|| Quality::Highest);
    let mut status = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut download_ready = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let mut download_data = use_signal(|| None::<Vec<u8>>);

    let handle_download = move |_| {
        if url().trim().is_empty() {
            error.set(Some("Please enter a valid YouTube URL".into()));
            return;
        }

        if filename().trim().is_empty() {
            error.set(Some("Please enter a filename".into()));
            return;
        }

        loading.set(true);
        error.set(None);
        status.set(Some("Processing download...".into()));
        download_ready.set(false);

        // Convert format type to string
        let format_str = match format_type() {
            FormatType::Video => "video",
            FormatType::Audio => "audio",
        };

        // Convert quality to string
        let quality_str = match quality() {
            Quality::Highest => "highest",
            Quality::Medium => "medium",
            Quality::Lowest => "lowest",
        };

        // Execute the download
        let url_clone = url().clone();
        let format_str_clone = format_str.to_string();
        let quality_str_clone = quality_str.to_string();

        spawn(async move {
            let result =
                download_with_quality(url_clone, format_str_clone, quality_str_clone).await;
            match result {
                Ok(data) => {
                    loading.set(false);
                    if data.is_empty() {
                        error.set(Some("Download resulted in empty data. Try again.".into()));
                    } else {
                        download_data.set(Some(data));
                        status.set(Some(format!(
                            "Download complete! File size: {:.2} MB",
                            download_data().as_ref().unwrap().len() as f64 / (1024.0 * 1024.0)
                        )));
                        download_ready.set(true);
                    }
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(format!("Download Failed: {}", e)));
                    status.set(None);
                }
            }
        });
    };

    let mut handle_format_change = move |new_format: FormatType| {
        // Clone new_format before moving it
        let new_format_clone = new_format.clone();
        format_type.set(new_format);

        // Update filename extension based on format type
        if !filename().is_empty() {
            let extension = match new_format_clone {
                FormatType::Video => "mp4",
                FormatType::Audio => "mp3",
            };

            // Remove any existing extension
            let filename_str = filename(); // Store in a local variable first
            let base_name = if filename_str.contains('.') {
                let parts: Vec<&str> = filename_str.split('.').collect();
                parts[0].to_string()
            } else {
                filename_str
            };

            filename.set(format!("{}.{}", base_name, extension));
        }
    };

    let format_video_class = match format_type() {
        FormatType::Video => {
            "flex-1 bg-red-600 hover:bg-red-700 text-white font-bold py-3 px-6 rounded"
        }
        _ => "flex-1 bg-gray-700 hover:bg-red-600 text-white font-bold py-3 px-6 rounded",
    };

    let format_audio_class = match format_type() {
        FormatType::Audio => {
            "flex-1 bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-6 rounded"
        }
        _ => "flex-1 bg-gray-700 hover:bg-blue-600 text-white font-bold py-3 px-6 rounded",
    };

    let quality_highest_class = match quality() {
        Quality::Highest => "flex-1 bg-red-600 text-white py-2 px-4 rounded",
        _ => "flex-1 bg-gray-700 hover:bg-gray-600 text-white py-2 px-4 rounded",
    };

    let quality_medium_class = match quality() {
        Quality::Medium => "flex-1 bg-red-600 text-white py-2 px-4 rounded",
        _ => "flex-1 bg-gray-700 hover:bg-gray-600 text-white py-2 px-4 rounded",
    };

    let quality_lowest_class = match quality() {
        Quality::Lowest => "flex-1 bg-red-600 text-white py-2 px-4 rounded",
        _ => "flex-1 bg-gray-700 hover:bg-gray-600 text-white py-2 px-4 rounded",
    };

    let download_button_class = "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded w-full disabled:opacity-50 disabled:cursor-not-allowed";
    let button_text = if loading() {
        "Processing..."
    } else {
        "Download"
    };

    // Quality selection component
    let quality_selection = if matches!(format_type(), FormatType::Video) {
        rsx! {
            div { class: "mb-6",
                label { class: "block text-gray-300 mb-2", "Video Quality" }
                div { class: "flex gap-3",
                    button {
                        class: "{quality_highest_class}",
                        onclick: move |_| quality.set(Quality::Highest),
                        "Highest"
                    }
                    button {
                        class: "{quality_medium_class}",
                        onclick: move |_| quality.set(Quality::Medium),
                        "Medium"
                    }
                    button {
                        class: "{quality_lowest_class}",
                        onclick: move |_| quality.set(Quality::Lowest),
                        "Lowest"
                    }
                }
            }
        }
    } else {
        rsx! {}
    };

    // Error message component
    let error_message = if let Some(err) = error() {
        rsx! {
            div { class: "mt-4 bg-red-800 text-white p-3 rounded",
                p { "{err}" }
            }
        }
    } else {
        rsx! {}
    };

    // Status message component
    let status_message = if let Some(stat) = status() {
        rsx! {
            div { class: "mt-4 bg-blue-900 text-white p-3 rounded",
                p { "{stat}" }
            }
        }
    } else {
        rsx! {}
    };

    // Download ready component
    let download_ready_component = if download_ready() {
        if let Some(data) = download_data() {
            let mime_type = match format_type() {
                FormatType::Video => "video/mp4",
                FormatType::Audio => "audio/mpeg",
            };

            // Get the extension based on the chosen format
            let extension = match format_type() {
                FormatType::Video => "mp4",
                FormatType::Audio => "mp3",
            };

            // Ensure the filename has the correct extension
            let download_filename = if filename().ends_with(extension) {
                filename().clone()
            } else {
                // Remove any existing extension and add the correct one
                let filename_str = filename().clone(); // Store in a local variable first
                let base_name = if filename_str.contains('.') {
                    let parts: Vec<&str> = filename_str.split('.').collect();
                    parts[0].to_string()
                } else {
                    filename_str
                };
                format!("{}.{}", base_name, extension)
            };

            // Create a blob URL for the binary data
            let blob_url = use_blob_url(Some(data.clone()), mime_type);

            // Handler for download button
            let download_handler = move |_| {
                if let Some(url) = blob_url() {
                    trigger_download(&url, &download_filename);
                }
            };

            rsx! {
                div { class: "mt-6 p-6 bg-green-900 bg-opacity-20 rounded-lg border border-green-700",
                    p { class: "text-green-400 font-medium mb-4",
                        "✓ Your file is ready to download!"
                    }

                    // Separate components for the two format types
                    match format_type() {
                        FormatType::Video => rsx! {
                            p { class: "text-gray-300 mb-4",
                                "File format: "
                                span { class: "font-bold", "Video (MP4)" }
                            }
                        },
                        FormatType::Audio => rsx! {
                            p { class: "text-gray-300 mb-4",
                                "File format: "
                                span { class: "font-bold", "Audio (MP3)" }
                            }
                        },
                    }

                    div { class: "text-center",
                        button {
                            class: "inline-block w-full sm:w-auto px-6 py-3 bg-green-600 hover:bg-green-700 rounded-lg font-medium text-white transition-colors duration-200",
                            onclick: download_handler,
                            "Save to Device"
                        }
                    }

                    // Also separate components for the preview
                    match format_type() {
                        FormatType::Video => {
                            if let Some(url) = blob_url() {
                                rsx! {
                                    div { class: "mt-4 pt-4 border-t border-green-700",
                                        p { class: "text-gray-300 mb-2", "Preview:" }
                                        video { class: "w-full max-h-96 rounded", controls: true, src: "{url}" }
                                    }
                                }
                            } else {
                                rsx! {
                                    div { "Loading preview..." }
                                }
                            }
                        }
                        FormatType::Audio => {
                            if let Some(url) = blob_url() {
                                rsx! {
                                    div { class: "mt-4 pt-4 border-t border-green-700",
                                        p { class: "text-gray-300 mb-2", "Preview:" }
                                        audio { class: "w-full", controls: true, src: "{url}" }
                                    }
                                }
                            } else {
                                rsx! {
                                    div { "Loading preview..." }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            rsx! {}
        }
    } else {
        rsx! {}
    };

    rsx! {
        div { class: "min-h-screen bg-gray-900 text-white",
            div { class: "container mx-auto px-4 py-8",
                div { class: "text-center mb-10",
                    h1 { class: "text-4xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent",
                        "YouTube Downloader"
                    }
                    p { class: "text-gray-400 mt-2", "Download videos and audio from YouTube" }
                }

                div { class: "max-w-2xl mx-auto bg-gray-800 p-6 rounded-lg shadow-lg",
                    // Format Selection Buttons
                    div { class: "mb-6",
                        label { class: "block text-gray-300 mb-2", "Download Format" }
                        div { class: "flex gap-4",
                            button {
                                class: "{format_video_class}",
                                onclick: move |_| handle_format_change(FormatType::Video),
                                "🎬 Video (MP4)"
                            }
                            button {
                                class: "{format_audio_class}",
                                onclick: move |_| handle_format_change(FormatType::Audio),
                                "🎵 Audio (MP3)"
                            }
                        }
                    }

                    // Quality Selection (only shown for video)
                    {quality_selection}

                    div { class: "mb-6",
                        label { class: "block text-gray-300 mb-2", "YouTube URL" }
                        input {
                            class: "w-full bg-gray-700 text-white border border-gray-600 rounded py-2 px-3 focus:outline-none focus:border-blue-500",
                            placeholder: "https://www.youtube.com/watch?v=...",
                            value: "{url}",
                            oninput: move |evt| url.set(evt.value().clone()),
                        }
                    }

                    div { class: "mb-6",
                        label { class: "block text-gray-300 mb-2", "Filename" }
                        input {
                            class: "w-full bg-gray-700 text-white border border-gray-600 rounded py-2 px-3 focus:outline-none focus:border-blue-500",
                            placeholder: "Enter filename without extension",
                            value: "{filename}",
                            oninput: move |evt| filename.set(evt.value().clone()),
                        }
                    }

                    div { class: "mb-4",
                        button {
                            class: "{download_button_class}",
                            onclick: handle_download,
                            disabled: loading(),
                            "{button_text}"
                        }
                    }

                    // Error messages
                    {error_message}

                    // Status messages
                    {status_message}

                    // Show download button when ready
                    {download_ready_component}
                }

                div { class: "max-w-2xl mx-auto mt-8 p-4 bg-gray-800 rounded-lg text-gray-300",
                    h3 { class: "text-xl font-bold mb-2", "Information" }
                    ul { class: "list-disc pl-5 space-y-1",
                        li { "Choose Video (MP4) to download both video and audio." }
                        li { "Choose Audio (MP3) if you only need the audio track." }
                        li { "Quality affects the resolution and file size (for video downloads)." }
                        li { "Some videos may not be available in all quality levels." }
                        li { "If a download fails, try a different quality or format." }
                    }
                }
            }
        }
    }
}
