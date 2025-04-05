use crate::views::download::handlers::{execute_download, update_filename};
use crate::views::download::platforms::trigger_download;
use crate::views::download::types::{FormatType, Quality};
use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_solid_icons::FaDownload, Icon};

#[cfg(feature = "desktop")]
use crate::views::download::platforms::save_to_disk;

// Download Component
#[component]
pub fn Download(url: String, format: String) -> Element {
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

    // Define button text based on platform
    let save_button_text = if cfg!(feature = "desktop") {
        "Choose Where to Save"
    } else {
        "Save to Device"
    };

    // Handle format type change
    let mut handle_format_change = move |new_format: FormatType| {
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

    let get_button_class = move || {
        if loading() {
            "w-full text-text-invert bg-accent-teal cursor-not-allowed font-medium rounded-lg text-sm px-5 py-3 text-center shadow-sm"
        } else if !is_url_valid {
            "w-full text-text-muted bg-background-medium cursor-not-allowed rounded-lg text-sm px-5 py-3 text-center border border-border"
        } else if download_ready() {
            "w-full text-text-invert bg-accent-green hover:bg-opacity-80 font-medium rounded-lg text-sm px-5 py-3 text-center transition-colors shadow-sm"
        } else {
            "w-full text-text-invert bg-accent-teal hover:bg-opacity-80 font-medium rounded-lg text-sm px-5 py-3 text-center transition-colors shadow-sm"
        }
    };

    let render_button_content = move || {
        if loading() {
            rsx! {
                span { class: "flex items-center justify-center",
                    // Spinner
                    svg {
                        class: "animate-spin -ml-1 mr-3 h-5 w-5 text-text-invert",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        circle {
                            class: "opacity-25",
                            cx: "12",
                            cy: "12",
                            r: "10",
                            stroke: "currentColor",
                            stroke_width: "4",
                        }
                        path {
                            class: "opacity-75",
                            fill: "currentColor",
                            d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                        }
                    }
                    "Processing..."
                }
            }
        } else if download_ready() {
            rsx! {
                span { "Download Ready" }
            }
        } else {
            rsx! {
                span { class: "flex items-center justify-center",
                    Icon {
                        icon: FaDownload,
                        width: 16,
                        height: 16,
                        class: "mr-2",
                    }
                    "Download Now"
                }
            }
        }
    };

    // Define download content
    let download_content = if download_ready() {
        // Get the extension and update filename if needed
        let extension = format_type().get_extension();
        let download_filename = if filename().ends_with(extension) {
            filename().clone()
        } else {
            update_filename(&filename(), &format_type())
        };

        // Define platform-specific download handlers
        #[cfg(feature = "web")]
        let download_handler = move |_| {
            if let Some(url) = blob_url() {
                trigger_download(&url, &download_filename);
            }
        };

        #[cfg(feature = "desktop")]
        let download_handler = {
            let data_clone = download_data.clone();
            let download_filename_clone = download_filename.clone();
            let mut status_clone = status.clone();
            let mut error_clone = error.clone();

            move |_| {
                if let Some(data) = data_clone() {
                    let _ =
                        save_to_disk(&data, &download_filename_clone, &status_clone, &error_clone);
                }
            }
        };

        #[cfg(not(any(feature = "web", feature = "desktop")))]
        let download_handler = move |_| {
            // Fallback for other platforms
            if let Some(url) = blob_url() {
                trigger_download(&url, &download_filename);
            }
        };

        rsx! {
            div { class: "mt-6 p-6 bg-background-card rounded-lg border border-accent-green",
                p { class: "text-accent-green font-medium mb-4",
                    "✓ Your file is ready to download!"
                }

                // Separate components for the two format types
                match format_type() {
                    FormatType::Video => rsx! {
                        p { class: "text-text-secondary mb-4",
                            "File format: "
                            span { class: "font-bold text-accent-teal", "Video (MP4)" }
                        }
                    },
                    FormatType::Audio => rsx! {
                        p { class: "text-text-secondary mb-4",
                            "File format: "
                            span { class: "font-bold text-accent-amber", "Audio (MP3)" }
                        }
                    },
                }

                div { class: "text-center",
                    button {
                        class: "inline-block w-full sm:w-auto px-6 py-3 bg-accent-green bg-opacity-80 hover:bg-opacity-100 rounded-lg font-medium text-text-primary transition-colors duration-200 shadow-sm",
                        onclick: download_handler,
                        "{save_button_text}"
                    }
                }
            }
        }
    } else {
        rsx! {}
    };

    // Progress bar component
    let progress_component = if loading() && progress_percent() > 0 {
        let eta_section = if !progress_eta().is_empty() {
            rsx! {
                div { class: "mt-1 text-sm text-text-muted flex justify-between",
                    span { "Estimated time: {progress_eta()}" }
                }
            }
        } else {
            rsx! {}
        };

        // Get status message for display
        let status_text = match status() {
            Some(stat) => {
                // If status contains "Downloading", make sure we show the percentage
                if stat.contains("Downloading") {
                    format!("{}", stat)
                } else {
                    // Otherwise just show the status message
                    stat
                }
            }
            None => "Downloading...".to_string(),
        };

        rsx! {
            div { class: "mt-4",
                div { class: "mb-2 flex justify-between",
                    span { class: "text-text-secondary", "{status_text}" }
                    span { class: "text-text-secondary", "{progress_percent()}%" }
                }
                div { class: "w-full bg-background-medium rounded-full h-2.5",
                    div {
                        class: "bg-accent-teal h-2.5 rounded-full transition-all duration-1000 ease-in-out",
                        style: "width: {progress_percent()}%",
                    }
                }
                {eta_section}
            }
        }
    } else {
        rsx! {}
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
                    // Format Selection Buttons
                    div { class: "mb-6",
                        label { class: "block mb-2 text-sm font-medium text-text-primary",
                            "Download Format"
                        }
                        div { class: "grid grid-cols-2 gap-2",
                            // Audio option
                            button {
                                key: "audio",
                                class: if format_type() == FormatType::Audio { "bg-accent-amber bg-opacity-20 text-accent-amber border border-accent-amber text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                                onclick: move |_| handle_format_change(FormatType::Audio),
                                disabled: loading(),
                                "🎵 Audio (MP3)"
                            }
                            // Video option
                            button {
                                key: "video",
                                class: if format_type() == FormatType::Video { "bg-accent-teal bg-opacity-20 text-accent-teal border border-accent-teal text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                                onclick: move |_| handle_format_change(FormatType::Video),
                                disabled: loading(),
                                "🎬 Video (MP4)"
                            }
                        }
                    }

                    // Quality selection
                    div { class: "mb-6",
                        label { class: "block mb-2 text-sm font-medium text-text-primary",
                            if format_type() == FormatType::Audio {
                                "Audio Quality"
                            } else {
                                "Video Quality"
                            }
                        }
                        div { class: "grid grid-cols-3 gap-2",
                            button {
                                class: if quality() == Quality::Highest { "bg-accent-teal bg-opacity-20 text-accent-teal border border-accent-teal text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                                onclick: move |_| quality.set(Quality::Highest),
                                disabled: loading(),
                                "High"
                            }
                            button {
                                class: if quality() == Quality::Medium { "bg-accent-teal bg-opacity-20 text-accent-teal border border-accent-teal text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                                onclick: move |_| quality.set(Quality::Medium),
                                disabled: loading(),
                                "Medium"
                            }
                            button {
                                class: if quality() == Quality::Lowest { "bg-accent-teal bg-opacity-20 text-accent-teal border border-accent-teal text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                                onclick: move |_| quality.set(Quality::Lowest),
                                disabled: loading(),
                                "Low"
                            }
                        }
                    }

                    // URL input group
                    div { class: "mb-6",
                        label { class: "block mb-2 text-sm font-medium text-text-primary",
                            "Video URL"
                        }
                        div { class: "flex",
                            input {
                                class: "flex-1 bg-background-medium border border-border text-text-primary text-sm rounded-l-lg focus:ring-accent-teal focus:border-accent-teal block w-full p-2.5",
                                r#type: "text",
                                placeholder: "Enter video URL (YouTube, Vimeo, etc.)",
                                value: "{url}",
                                oninput: move |e| url.set(e.value().clone()),
                                disabled: loading(),
                            }
                            button {
                                class: "bg-background-medium hover:bg-background-hover text-text-primary border border-l-0 border-border font-medium rounded-r-lg text-sm px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-accent-teal",
                                r#type: "button",
                                onclick: move |_| {
                                    url.set("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string());
                                },
                                disabled: loading(),
                                "Paste"
                            }
                        }
                    }

                    // Filename input
                    div { class: "mb-6",
                        label { class: "block mb-2 text-sm font-medium text-text-primary",
                            "Custom filename (optional)"
                        }
                        input {
                            class: "bg-background-medium border border-border text-text-primary text-sm rounded-lg focus:ring-accent-teal focus:border-accent-teal block w-full p-2.5",
                            r#type: "text",
                            placeholder: "Enter custom filename (without extension)",
                            value: "{filename}",
                            oninput: move |e| filename.set(e.value().clone()),
                            disabled: loading(),
                        }
                    }

                    // Progress bar
                    {progress_component}

                    // Error messages
                    {error_message}

                    // Status messages
                    {status_message}

                    // Show download content when ready
                    {download_content}

                    // Download button
                    if !download_ready() {
                        button {
                            class: get_button_class(),
                            disabled: loading() || url().is_empty() || filename().is_empty(),
                            onclick: handle_download,
                            {render_button_content()}
                        }
                    }
                }
            }
        }
    }
}
