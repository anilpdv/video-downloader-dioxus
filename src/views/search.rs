use crate::common::Toaster;
use crate::server::youtube::{download_youtube_video, search_youtube_videos, VideoSearchResult};
use crate::Route;
// Import from the public re-exports instead of private modules
use crate::views::download::{FormatType, Quality};
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{
        bs_icons::BsSearch,
        fa_solid_icons::{FaDownload, FaMusic, FaVideo},
    },
    Icon,
};
use futures_timer::Delay;
use std::time::Duration;

#[component]
pub fn Search() -> Element {
    let mut search_query = use_signal(|| String::new());
    let mut searching = use_signal(|| false);
    let mut search_results = use_signal(|| Vec::<VideoSearchResult>::new());
    let mut toaster = use_signal(|| None::<Toaster>);
    let mut selected_format = use_signal(|| FormatType::Video);
    let navigator = use_navigator();

    // States for download tracking
    let mut loading = use_signal(|| false);
    let mut status = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut download_ready = use_signal(|| false);
    let download_data = use_signal(|| None::<Vec<u8>>);
    let mut blob_url = use_signal(|| None::<String>);
    let mut progress_percent = use_signal(|| 0);
    let mut progress_eta = use_signal(|| String::new());
    let mut download_in_progress = use_signal(|| false);

    let mut search_result =
        use_resource(
            move || async move { search_youtube_videos(search_query().to_string()).await },
        );

    use_effect(move || {
        tracing::info!("search_result: {:?}", search_result());
        match &*search_result.read_unchecked() {
            Some(Ok(results)) => {
                searching.set(false);
                let results_clone = results.clone();
                tracing::info!("results_clone: {:?}", results_clone);
                search_results.set(results_clone);
            }
            Some(Err(e)) => {
                searching.set(false);
                toaster.set(Some(Toaster::Error(format!("Search error: {}", e))));
            }
            None => {}
        }
    });

    // Function to handle download requests - navigates to download route
    let mut handle_download = move |video: VideoSearchResult| {
        // Get the video URL
        let video_url = format!("https://www.youtube.com/watch?v={}", video.id);

        // Get the format type
        let format_string = if selected_format() == FormatType::Audio {
            "audio"
        } else {
            "video"
        };

        // Navigate to download route with parameters
        navigator.push(Route::Download {
            url: video_url,
            format: format_string.to_string(),
        });
    };

    let on_key_press = move |e: Event<KeyboardData>| {
        if e.key().to_string() == "Enter" {
            let query = search_query().trim().to_string();
            if query.is_empty() {
                toaster.set(Some(Toaster::Warning(
                    "Please enter a search query".to_string(),
                )));
                return;
            }

            searching.set(true);
            search_results.set(Vec::new());

            search_result.restart();
        }
    };

    rsx! {
        div { class: "container mx-auto py-6 px-4",
            h1 { class: "text-3xl font-bold mb-4 text-text-primary", "Search Videos" }
            p { class: "mb-6 text-text-secondary",
                "Search for videos and download them in various formats."
            }

            // Search form
            div { class: "mb-8 bg-background-card rounded-xl p-6 border border-border shadow-md",
                // Format selection tabs
                div { class: "mb-6",
                    div { class: "flex space-x-4",
                        button {
                            class: "px-6 py-2 rounded-lg flex items-center",
                            class: if selected_format() == FormatType::Video { "bg-accent-teal text-text-primary" } else { "bg-background-medium text-text-secondary hover:bg-background-hover" },
                            onclick: move |_| selected_format.set(FormatType::Video),
                            Icon {
                                icon: FaVideo,
                                width: 16,
                                height: 16,
                                class: "mr-2",
                            }
                            "Video (MP4)"
                        }
                        button {
                            class: "px-6 py-2 rounded-lg flex items-center",
                            class: if selected_format() == FormatType::Audio { "bg-accent-amber text-text-primary" } else { "bg-background-medium text-text-secondary hover:bg-background-hover" },
                            onclick: move |_| selected_format.set(FormatType::Audio),
                            Icon {
                                icon: FaMusic,
                                width: 16,
                                height: 16,
                                class: "mr-2",
                            }
                            "Audio (MP3)"
                        }
                    }
                }

                // Search input
                div { class: "flex",
                    input {
                        class: "flex-1 bg-background-medium border border-border text-text-primary text-sm rounded-l-lg focus:ring-accent-teal focus:border-accent-teal block w-full p-3",
                        r#type: "text",
                        placeholder: "Search for videos...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value().clone()),
                        onkeydown: on_key_press,
                        disabled: searching(),
                    }
                    button {
                        class: "bg-accent-teal hover:bg-opacity-80 text-text-primary font-medium rounded-r-lg text-sm px-5 py-3 focus:outline-none transition-colors",
                        onclick: move |_| {
                            searching.set(true);
                            search_result.restart();
                        },
                        disabled: searching(),
                        if searching() {
                            // Loading spinner
                            div { class: "animate-spin w-5 h-5",
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    class: "w-5 h-5",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15",
                                    }
                                }
                            }
                        } else {
                            Icon { icon: BsSearch, width: 20, height: 20 }
                        }
                    }
                }
            }

            // Progress bar component
            if loading() && progress_percent() > 0 {
                div { class: "mt-4 mb-6 bg-background-card rounded-xl p-6 border border-border shadow-md",
                    // Title of what's downloading
                    h3 { class: "font-medium text-lg mb-4 text-text-primary", "Download Progress" }

                    div { class: "mb-2 flex justify-between",
                        span { class: "text-text-secondary",
                            if let Some(stat) = status() {
                                "{stat}"
                            } else {
                                "Downloading..."
                            }
                        }
                        span { class: "text-text-secondary", "{progress_percent()}%" }
                    }
                    div { class: "w-full bg-background-medium rounded-full h-2.5",
                        div {
                            class: "bg-accent-teal h-2.5 rounded-full transition-all duration-1000 ease-in-out",
                            style: "width: {progress_percent()}%",
                        }
                    }
                    if !progress_eta().is_empty() {
                        div { class: "mt-1 text-sm text-text-muted flex justify-between",
                            span { "Estimated time: {progress_eta()}" }
                        }
                    }

                    // Error message if any
                    if let Some(err) = error() {
                        div { class: "mt-4 bg-accent-rose bg-opacity-10 text-accent-rose p-3 rounded",
                            p { "{err}" }
                        }
                    }

                    // Download ready content
                    if download_ready() {
                        div { class: "mt-4 p-3 bg-background-card rounded-lg border border-accent-green",
                            p { class: "text-accent-green font-medium mb-2",
                                "✓ Your file is ready to download!"
                            }
                            div { class: "text-center mt-4",
                                button { class: "inline-block w-full sm:w-auto px-6 py-3 bg-accent-green bg-opacity-80 hover:bg-opacity-100 rounded-lg font-medium text-text-primary transition-colors duration-200 shadow-sm",
                                    "Save File"
                                }
                            }
                        }
                    }
                }
            }

            // Results section with loading state
            if searching() {
                div { class: "py-12 text-center",
                    div { class: "animate-spin w-10 h-10 mb-4 text-accent-teal mx-auto",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",
                            stroke_width: "2",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15",
                            }
                        }
                    }
                    p { class: "text-text-secondary text-lg", "Searching..." }
                }
            } else if !search_query().is_empty() && search_results().is_empty() {
                div { class: "py-12 text-center",
                    p { class: "text-text-secondary text-lg", "No results found for your search." }
                }
            } else if !search_results().is_empty() {
                div {
                    h2 { class: "text-xl font-semibold mb-4", "Search Results" }
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                        // Map through search results
                        {
                            search_results()
                                .into_iter()
                                .map(|video| {
                                    let video_clone = video.clone();
                                    rsx! {
                                        div { class: "bg-background-card rounded-xl shadow-md overflow-hidden hover:shadow-lg transition-all duration-300 border border-border",
                                            // Thumbnail area
                                            div { class: "relative aspect-video bg-background-dark",
                                                img {
                                                    class: "w-full h-full object-cover",
                                                    src: "{video.thumbnail_url}",
                                                    alt: "{video.title}",
                                                }
                                                // Duration badge
                                                div { class: "absolute bottom-2 right-2 bg-background-darker bg-opacity-75 text-text-primary text-xs px-2 py-1 rounded-full",
                                                    "{video.duration}"
                                                }
                                            }
                                            // Video info
                                            div { class: "p-4",
                                                h3 { class: "font-medium text-lg mb-2 line-clamp-2 text-text-primary", "{video.title}" }
                                                div { class: "flex justify-between text-sm text-text-muted mb-2",
                                                    div { "{video.channel_name}" }
                                                }
                                                div { class: "flex justify-between text-xs text-text-muted mb-4",
                                                    if let Some(uploaded) = &video.uploaded_at {
                                                        div { "{uploaded}" }
                                                    }
                                                    div { "{video.views}" }
                                                }
                                                // Download button - changes icon based on format
                                                div { class: "flex space-x-2",
                                                    button {
                                                        class: "flex-1 bg-accent-teal hover:bg-opacity-80 text-text-primary py-2 px-3 rounded-lg text-sm transition-colors flex items-center justify-center",
                                                        disabled: loading(),
                                                        onclick: move |_| handle_download(video_clone.clone()),
                                                        if selected_format() == FormatType::Audio {
                                                            Icon {
                                                                icon: FaMusic,
                                                                width: 16,
                                                                height: 16,
                                                                class: "mr-2",
                                                            }
                                                            "Download MP3"
                                                        } else {
                                                            Icon {
                                                                icon: FaDownload,
                                                                width: 16,
                                                                height: 16,
                                                                class: "mr-2",
                                                            }
                                                            "Download MP4"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                })
                        }
                    }
                }
            } else {
                // Initial state
                div { class: "text-center py-16 bg-background-card rounded-xl border border-border shadow-md",
                    div { class: "flex justify-center mb-6",
                        Icon {
                            icon: BsSearch,
                            width: 52,
                            height: 52,
                            class: "text-text-muted",
                        }
                    }
                    p { class: "text-xl font-medium text-text-primary", "Search for YouTube Videos" }
                    p { class: "text-text-secondary mt-2 max-w-md mx-auto",
                        "Enter a search term to find videos. You can download them as MP4 videos or MP3 audio files."
                    }
                }
            }

            // Toast notifications
            {
                if let Some(toast) = &toaster() {
                    let (bg_color, icon) = match toast {
                        Toaster::Success(_) => ("bg-success-500", "✓"),
                        Toaster::Error(_) => ("bg-danger-500", "✗"),
                        Toaster::Warning(_) => ("bg-warning-500", "⚠"),
                        Toaster::Info(_) => ("bg-accent-teal", "ℹ"),
                    };
                    let message = match toast {
                        Toaster::Success(msg) => msg,
                        Toaster::Error(msg) => msg,
                        Toaster::Warning(msg) => msg,
                        Toaster::Info(msg) => msg,
                    };
                    rsx! {
                        div { class: "fixed bottom-5 right-5 {bg_color} text-text-primary px-6 py-4 rounded-lg shadow-lg max-w-md",
                            div { class: "flex items-center",
                                span { class: "text-xl mr-2", "{icon}" }
                                span { "{message}" }
                            }
                        }
                    }
                } else {
                    rsx! { "" }
                }
            }
        }
    }
}
