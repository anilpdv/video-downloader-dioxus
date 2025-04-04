use crate::views::download::handlers::{execute_download, update_filename};
use crate::views::download::platforms::trigger_download;
use crate::views::download::types::{FormatType, Quality};
use dioxus::prelude::*;

#[cfg(feature = "desktop")]
use crate::views::download::platforms::save_to_disk;

#[component]
pub fn Download() -> Element {
    // Form state
    let mut url = use_signal(String::new);
    let mut filename = use_signal(String::new);
    let mut format_type = use_signal(|| FormatType::Video);
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

    // Flag to track if progress simulation is active
    let mut simulating = use_signal(|| false);

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

        // Set simulating flag to true to enable progress polling
        simulating.set(true);

        // Execute the actual download with real progress updates
        execute_download(
            url().clone(),
            format_type(),
            quality(),
            &simulating,
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

    // Define CSS classes based on selected options
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

    // Progress bar component
    let progress_component = if loading() && progress_percent() > 0 {
        let eta_section = if !progress_eta().is_empty() {
            rsx! {
                div { class: "mt-1 text-sm text-gray-400 flex justify-between",
                    span { "Estimated time: {progress_eta()}" }
                }
            }
        } else {
            rsx! {}
        };

        // Get status message for display
        let status_text = match status() {
            Some(stat) => stat,
            None => "Downloading...".to_string(),
        };

        rsx! {
            div { class: "mt-4",
                div { class: "mb-2 flex justify-between",
                    span { class: "text-gray-300", "{status_text}" }
                    span { class: "text-gray-300", "{progress_percent()}%" }
                }
                div { class: "w-full bg-gray-700 rounded-full h-2.5",
                    div {
                        class: "bg-blue-600 h-2.5 rounded-full transition-all duration-500 ease-out",
                        style: "width: {progress_percent()}%",
                    }
                }
                {eta_section}
            }
        }
    } else {
        rsx! {}
    };

    // Quality selection component - only shown for video format
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

    // Download ready component - shown when download is complete
    let download_ready_component = if download_ready() {
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

        // Generate the preview section based on platform
        let preview_section = if cfg!(feature = "web") {
            match format_type() {
                FormatType::Video => {
                    if let Some(url) = blob_url() {
                        rsx! {
                            div { class: "mt-4 pt-4 border-t border-green-700",
                                p { class: "text-gray-300 mb-2", "Preview:" }
                                video {
                                    class: "w-full max-h-96 rounded",
                                    controls: true,
                                    src: "{url}",
                                }
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
                                audio {
                                    class: "w-full",
                                    controls: true,
                                    src: "{url}",
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div { "Loading preview..." }
                        }
                    }
                }
            }
        } else if cfg!(feature = "desktop") {
            // Desktop specific message
            rsx! {
                div { class: "mt-4 pt-4 border-t border-green-700 text-gray-300",
                    p { "Click the button above to choose where to save your file." }
                }
            }
        } else {
            rsx! {}
        };

        rsx! {
            div { class: "mt-6 p-6 bg-green-900 bg-opacity-20 rounded-lg border border-green-700",
                p { class: "text-green-400 font-medium mb-4", "✓ Your file is ready to download!" }

                // Separate components for the two format types
                match format_type() {
                    FormatType::Video => rsx! {
                        p { class: "text-gray-300 mb-4",
                            "File format: "
                            span { class: "font-bold text-red-400", "Video (MP4)" }
                        }
                    },
                    FormatType::Audio => rsx! {
                        p { class: "text-gray-300 mb-4",
                            "File format: "
                            span { class: "font-bold text-blue-400", "Audio (MP3)" }
                        }
                    },
                }

                div { class: "text-center",
                    button {
                        class: "inline-block w-full sm:w-auto px-6 py-3 bg-green-600 hover:bg-green-700 rounded-lg font-medium text-white transition-colors duration-200",
                        onclick: download_handler,
                        "{save_button_text}"
                    }
                }

                // Include the preview section
                {preview_section}
            }
        }
    } else {
        rsx! {}
    };

    // Main component UI
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

                    // Progress bar
                    {progress_component}

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
