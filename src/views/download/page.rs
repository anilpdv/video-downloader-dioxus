use dioxus::prelude::*;

use crate::views::download::components::download_button::DownloadButton;
use crate::views::download::components::download_form::DownloadForm;
use crate::views::download::components::download_ready::DownloadReady;
use crate::views::download::components::progress_indicator::ProgressIndicator;
use crate::views::download::services::{execute_download, handle_download_file, update_filename};
use crate::views::download::types::{FormatType, Quality};

#[component]
pub fn DownloadPage(url: String, format: String) -> Element {
    // Form state - initialize with props if provided
    let mut url = use_signal(|| url);
    let mut filename = use_signal(String::new);
    let mut format_type = use_signal(|| {
        // Set format type based on the format parameter
        match format.as_str() {
            "audio" => FormatType::Audio,
            _ => FormatType::Video, // Default to video
        }
    });
    let mut quality = use_signal(|| Quality::Highest);

    // UI state
    let mut status = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut download_ready = use_signal(|| false);
    let mut loading = use_signal(|| false);

    // Download state
    let download_data = use_signal(|| None::<Vec<u8>>);
    let mut blob_url = use_signal(|| None::<String>);

    // Progress tracking
    let mut progress_percent = use_signal(|| 0);
    let mut progress_speed = use_signal(|| String::new());
    let mut progress_eta = use_signal(|| String::new());

    // Flag to track if download is in progress and we should poll for updates
    let mut download_in_progress = use_signal(|| false);

    // Handle format type change
    let handle_format_change = move |new_format: FormatType| {
        format_type.set(new_format.clone());

        // Always update the filename extension when format changes
        if !filename().is_empty() {
            let new_filename = update_filename(&filename(), &new_format);
            filename.set(new_filename);
        }
    };

    // Handle URL validation
    let is_url_valid = !url().is_empty();

    // Handle the download button click
    let handle_download = move |_| {
        // Validate inputs
        if url().trim().is_empty() {
            error.set(Some("Please enter a valid YouTube URL".into()));
            return;
        }

        if filename().trim().is_empty() {
            error.set(Some("Please enter a filename".into()));
            return;
        }

        // Reset state for new download
        loading.set(true);
        error.set(None);
        status.set(Some("Initializing download...".into()));
        download_ready.set(false);
        blob_url.set(None);
        progress_percent.set(0); // Start at 0% for real progress updates
        progress_eta.set("Calculating...".into());
        progress_speed.set(String::new());

        // Set download_in_progress flag to true to enable progress polling
        download_in_progress.set(true);

        // Execute the actual download with real progress updates
        execute_download(
            url().clone(),
            format_type(),
            quality(),
            &download_in_progress,
            &progress_percent,
            &status,
            &progress_eta,
            &loading,
            &error,
            &download_data,
            &blob_url,
            &download_ready,
        );
    };

    // Handle save file click
    let download_handler = move |_| {
        // Get the extension and update filename if needed
        let extension = format_type().get_extension();
        let download_filename = if filename().ends_with(extension) {
            filename().clone()
        } else {
            update_filename(&filename(), &format_type())
        };

        // Create the appropriate handler for this platform
        let handler = handle_download_file(
            blob_url(),
            download_data(),
            &download_filename,
            &format_type(),
        );

        // Execute the handler
        handler();
    };

    // Error message component
    let error_message = if let Some(err) = error() {
        rsx! {
            div { class: "mt-4 bg-accent-rose bg-opacity-10 text-accent-rose p-3 rounded",
                p { "{err}" }
            }
        }
    } else {
        rsx! {}
    };

    // Status message component
    let status_message = if let Some(stat) = status() {
        rsx! {
            div { class: "mt-4 bg-background-card text-text-primary p-3 rounded border border-border",
                p { "{stat}" }
            }
        }
    } else {
        rsx! {}
    };

    // Render main component
    rsx! {
        div { class: "container mx-auto px-4 py-8",
            div { class: "max-w-3xl mx-auto",
                h1 { class: "text-3xl font-bold mb-4 text-text-primary", "Download Video & Audio" }
                p { class: "mb-8 text-text-secondary",
                    "Enter a URL to download videos or audio from various platforms."
                }

                // Download form
                div { class: "bg-background-card rounded-xl shadow-md p-6 border border-border",
                    // Form components
                    DownloadForm {
                        url: url.clone(),
                        filename: filename.clone(),
                        format_type: format_type.clone(),
                        quality: quality.clone(),
                        loading: loading.clone(),
                        on_format_change: handle_format_change,
                    }

                    // Progress indicator
                    ProgressIndicator {
                        loading: loading.clone(),
                        progress_percent: progress_percent.clone(),
                        progress_eta: progress_eta.clone(),
                        status: status.clone(),
                    }

                    // Error messages
                    {error_message}

                    // Status messages
                    {status_message}

                    // Show download content when ready
                    DownloadReady {
                        download_ready: download_ready.clone(),
                        format_type: format_type.clone(),
                        filename: filename().clone(),
                        on_save_click: download_handler,
                    }

                    // Download button
                    if !download_ready() {
                        DownloadButton {
                            loading: loading.clone(),
                            is_url_valid,
                            download_ready: download_ready.clone(),
                            onclick: handle_download,
                        }
                    }
                }
            }
        }
    }
}
