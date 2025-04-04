#[cfg(feature = "server")]
use crate::database::models::Download;
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{
        bs_icons::{BsExclamationTriangleFill, BsSearch, BsXLg},
        fa_solid_icons::{FaCalendar, FaDatabase, FaDownload, FaFolder, FaMusic, FaPlay, FaVideo},
        hi_outline_icons::{HiFilm, HiMusicNote, HiViewGrid},
    },
    Icon,
};

#[component]
pub fn Downloads() -> Element {
    rsx! {
        div { class: "container mx-auto py-6 px-4",
            h1 { class: "text-3xl font-bold mb-4", "My Downloads" }
            p { class: "mb-6 text-gray-600",
                "Access and play your downloaded videos and audio files."
            }
            ServerContent {}
        }
    }
}

#[component]
fn ServerContent() -> Element {
    // Add explicit logging to debug feature flags
    tracing::info!("ServerContent rendering with cfg(feature = \"desktop\"): {}, cfg(feature = \"server\"): {}", 
        cfg!(feature = "desktop"), 
        cfg!(feature = "server")
    );

    #[cfg(feature = "server")]
    {
        let downloads = use_signal(|| Vec::<Download>::new());
        let loading = use_signal(|| true);
        let mut active_tab = use_signal(|| "all".to_string());

        // Search functionality
        let mut search_query = use_signal(|| String::new());

        // Only fetch downloads when the component is first mounted
        use_effect(move || {
            if loading() {
                use crate::database::{get_database, schema::get_all_downloads};

                let mut downloads_clone = downloads.clone();
                let mut loading_clone = loading.clone();

                use_future(move || async move {
                    tracing::info!("Loading downloads from database...");
                    if let Ok(pool) = get_database().await {
                        match get_all_downloads(&pool).await {
                            Ok(results) => {
                                tracing::info!("Found {} downloads", results.len());
                                downloads_clone.set(results);
                            }
                            Err(e) => {
                                tracing::error!("Failed to get downloads from database: {}", e);
                            }
                        }
                    } else {
                        tracing::error!("Failed to get database connection");
                    }
                    loading_clone.set(false);
                });
            }
        });

        if loading() {
            return rsx! {
                div { class: "flex flex-col items-center justify-center py-16",
                    div { class: "animate-spin w-12 h-12 mb-4 text-blue-500",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            class: "h-12 w-12",
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
                    p { class: "text-gray-600", "Loading your downloads..." }
                }
            };
        }

        // Filter downloads based on active tab and search query
        let filtered_downloads = {
            let query = search_query().to_lowercase();

            let tab_filtered = if active_tab() == "all" {
                downloads().clone()
            } else {
                downloads()
                    .iter()
                    .filter(|d| d.format_type == active_tab())
                    .cloned()
                    .collect::<Vec<Download>>()
            };

            // Apply search filter if query is not empty
            if query.is_empty() {
                tab_filtered
            } else {
                tab_filtered
                    .into_iter()
                    .filter(|d| {
                        d.title
                            .as_ref()
                            .map_or(false, |t| t.to_lowercase().contains(&query))
                            || d.filename.to_lowercase().contains(&query)
                    })
                    .collect()
            }
        };

        let audio_count = downloads()
            .iter()
            .filter(|d| d.format_type == "audio")
            .count();
        let video_count = downloads()
            .iter()
            .filter(|d| d.format_type == "video")
            .count();

        if downloads().is_empty() {
            return rsx! {
                div { class: "text-center py-16 bg-gray-50 rounded-xl border border-gray-200 shadow-sm",
                    div { class: "flex justify-center mb-6",
                        Icon {
                            icon: FaDownload,
                            width: 52,
                            height: 52,
                            class: "text-gray-400",
                        }
                    }
                    p { class: "text-xl font-medium text-gray-700", "No downloads yet" }
                    p { class: "text-gray-500 mt-2 max-w-md mx-auto",
                        "Your downloaded files will appear here. Try downloading a video or audio file from the home page."
                    }
                }
            };
        }

        // Show downloads with tabs
        return rsx! {
            // Search bar
            div { class: "mb-6 relative",
                div { class: "relative",
                    span { class: "absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none",
                        Icon {
                            icon: BsSearch,
                            width: 16,
                            height: 16,
                            class: "text-gray-400",
                        }
                    }
                    input {
                        class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full pl-10 p-2.5",
                        r#type: "text",
                        placeholder: "Search downloads...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value().clone()),
                    }
                }
            }
            // Tab navigation
            div { class: "mb-6 border-b border-gray-200",
                div { class: "flex flex-wrap -mb-px",
                    // All tab
                    button {
                        class: if active_tab() == "all" { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-blue-600 border-b-2 border-blue-600 rounded-t-lg" } else { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-gray-500 border-b-2 border-transparent hover:text-gray-600 hover:border-gray-300 rounded-t-lg" },
                        onclick: move |_| active_tab.set("all".to_string()),
                        Icon {
                            icon: HiViewGrid,
                            width: 18,
                            height: 18,
                            class: "mr-2",
                        }
                        "All ({downloads().len()})"
                    }

                    // Audio tab
                    button {
                        class: if active_tab() == "audio" { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-blue-600 border-b-2 border-blue-600 rounded-t-lg" } else { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-gray-500 border-b-2 border-transparent hover:text-gray-600 hover:border-gray-300 rounded-t-lg" },
                        onclick: move |_| active_tab.set("audio".to_string()),
                        Icon {
                            icon: HiMusicNote,
                            width: 18,
                            height: 18,
                            class: "mr-2",
                        }
                        "Audio ({audio_count})"
                    }

                    // Video tab
                    button {
                        class: if active_tab() == "video" { "inline-flex items-center py-3 px-4 text-sm font-medium text-blue-600 border-b-2 border-blue-600 rounded-t-lg" } else { "inline-flex items-center py-3 px-4 text-sm font-medium text-gray-500 border-b-2 border-transparent hover:text-gray-600 hover:border-gray-300 rounded-t-lg" },
                        onclick: move |_| active_tab.set("video".to_string()),
                        Icon {
                            icon: HiFilm,
                            width: 18,
                            height: 18,
                            class: "mr-2",
                        }
                        "Video ({video_count})"
                    }
                }
            }

            // No files found message when filter is applied
            if filtered_downloads.is_empty() {
                div { class: "text-center py-12 bg-gray-50 rounded-xl border border-gray-200 shadow-sm",
                    if !search_query().is_empty() {
                        div { class: "flex flex-col items-center",
                            Icon {
                                icon: BsSearch,
                                width: 40,
                                height: 40,
                                class: "text-gray-400 mb-4",
                            }
                            p { class: "text-lg font-medium text-gray-700",
                                "No results found for \"{search_query()}\""
                            }
                            p { class: "text-gray-500 mt-2",
                                "Try different keywords or clear your search"
                            }
                            button {
                                class: "mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg text-sm hover:bg-blue-700 transition-colors",
                                onclick: move |_| search_query.set(String::new()),
                                "Clear Search"
                            }
                        }
                    } else {
                        div { class: "flex flex-col items-center",
                            if active_tab() == "audio" {
                                Icon {
                                    icon: FaMusic,
                                    width: 40,
                                    height: 40,
                                    class: "text-gray-400 mb-4",
                                }
                            } else {
                                Icon {
                                    icon: FaVideo,
                                    width: 40,
                                    height: 40,
                                    class: "text-gray-400 mb-4",
                                }
                            }
                            p { class: "text-lg font-medium text-gray-700",
                                "No {active_tab()} files found"
                            }
                            p { class: "text-gray-500 mt-2",
                                "Try switching to a different category or download some {active_tab()} files."
                            }
                        }
                    }
                }
            } else {
                // Grid display of downloads
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                    for download in filtered_downloads {
                        DownloadCard { download: download.clone() }
                    }
                }
            }
        };
    }

    // Only show "unavailable" message if server feature is not available
    #[cfg(not(feature = "server"))]
    return rsx! {
        div { class: "text-center py-12 bg-gray-50 rounded-lg",
            p { class: "text-xl font-medium text-gray-500",
                "Download history is not available in web mode"
            }
            p { class: "text-gray-400 mt-2",
                "For full functionality including download history, please use the desktop app."
            }
        }
    };
}

#[cfg(feature = "server")]
#[component]
fn DownloadCard(download: Download) -> Element {
    let file_exists = std::path::Path::new(&download.file_path).exists();
    let is_video = &download.format_type == "video";
    let is_audio = &download.format_type == "audio";
    let mut show_player = use_signal(|| false);

    rsx! {
        div { class: "bg-white rounded-xl shadow-md overflow-hidden hover:shadow-lg transition-all duration-300 border border-gray-200 transform hover:-translate-y-1",
            // Thumbnail area
            div { class: "relative aspect-video bg-gray-200",
                if let Some(ref thumbnail) = download.thumbnail_url {
                    img {
                        class: "w-full h-full object-cover",
                        src: "{thumbnail}",
                        alt: "Thumbnail",
                    }
                } else {
                    div { class: "w-full h-full flex items-center justify-center bg-gradient-to-r from-gray-700 to-gray-900 text-white",
                        if is_audio {
                            Icon {
                                icon: FaMusic,
                                width: 48,
                                height: 48,
                                class: "text-purple-300",
                            }
                        } else {
                            Icon {
                                icon: FaVideo,
                                width: 48,
                                height: 48,
                                class: "text-blue-300",
                            }
                        }
                    }
                }

                // Format badge
                div { class: if is_audio { "absolute top-2 right-2 bg-purple-500 text-white text-xs px-2 py-1 rounded-full flex items-center" } else { "absolute top-2 right-2 bg-blue-500 text-white text-xs px-2 py-1 rounded-full flex items-center" },
                    if is_audio {
                        Icon {
                            icon: FaMusic,
                            width: 10,
                            height: 10,
                            class: "mr-1",
                        }
                    } else {
                        Icon {
                            icon: FaVideo,
                            width: 10,
                            height: 10,
                            class: "mr-1",
                        }
                    }
                    if is_audio {
                        "MP3"
                    } else {
                        "Video"
                    }
                }

                // Duration badge
                if let Some(_) = download.duration {
                    div { class: "absolute bottom-2 right-2 bg-black bg-opacity-75 text-white text-xs px-2 py-1 rounded-full",
                        "{download.format_duration()}"
                    }
                }

                // Quality badge
                div { class: "absolute bottom-2 left-2 bg-gray-800 bg-opacity-75 text-white text-xs px-2 py-1 rounded-full",
                    "{download.quality}"
                }

                // Play overlay button (only if file exists)
                if file_exists {
                    button {
                        class: "absolute inset-0 flex items-center justify-center bg-black bg-opacity-0 hover:bg-opacity-50 transition-opacity duration-300",
                        onclick: move |_| show_player.set(!show_player()),
                        div { class: "w-14 h-14 bg-white bg-opacity-90 rounded-full flex items-center justify-center shadow-lg transform hover:scale-110 transition-transform",
                            // Text-based play icon for simplicity
                            span { class: "text-red-600 text-xl font-bold",
                                if show_player() {
                                    "X"
                                } else {
                                    "▶"
                                }
                            }
                        }
                    }
                }
            }

            // Embedded player (shown when play is clicked)
            if show_player() && file_exists {
                if is_video {
                    div { class: "w-full bg-black",
                        video {
                            class: "w-full",
                            controls: true,
                            src: "file://{download.file_path}",
                        }
                    }
                } else if is_audio {
                    div { class: "w-full bg-gray-100 p-3 border-t border-b border-gray-200",
                        audio {
                            class: "w-full",
                            controls: true,
                            src: "file://{download.file_path}",
                        }
                    }
                }
            }

            // Details section
            div { class: "p-4",
                // Title
                h3 { class: "font-medium text-lg mb-2 line-clamp-2 text-gray-800",
                    if let Some(ref title) = download.title {
                        "{title}"
                    } else {
                        "Untitled download"
                    }
                }

                // Info row
                div { class: "flex justify-between text-sm text-gray-500 mb-4",
                    div { class: "flex items-center",
                        Icon {
                            icon: FaCalendar,
                            width: 12,
                            height: 12,
                            class: "mr-1.5",
                        }
                        span { "{download.format_date()}" }
                    }
                    div { class: "flex items-center",
                        Icon {
                            icon: FaDatabase,
                            width: 12,
                            height: 12,
                            class: "mr-1.5",
                        }
                        span { "{download.format_file_size()}" }
                    }
                }

                // Action buttons
                div { class: "flex space-x-2 mt-3",
                    if file_exists {
                        // Play button
                        button {
                            class: "flex-1 bg-blue-600 hover:bg-blue-700 text-white py-2 px-3 rounded-lg text-sm transition-colors duration-200 flex items-center justify-center shadow-sm",
                            onclick: move |_| show_player.set(!show_player()),
                            {if show_player() { "Close Player" } else { "Play Media" }}
                        }

                        // Open folder button
                        button {
                            class: "bg-gray-100 hover:bg-gray-200 text-gray-800 py-2 px-3 rounded-lg text-sm transition-colors duration-200 flex items-center justify-center shadow-sm",
                            onclick: {
                                let file_path = download.file_path.clone();
                                move |_| {
                                    #[cfg(target_os = "windows")]
                                    {
                                        use std::process::Command;
                                        let _ = Command::new("explorer").args(["/select,", &file_path]).spawn();
                                    }
                                    #[cfg(target_os = "macos")]
                                    {
                                        use std::process::Command;
                                        let parent = std::path::Path::new(&file_path)
                                            .parent()
                                            .unwrap_or(std::path::Path::new(""));
                                        let _ = Command::new("open").arg(parent).spawn();
                                    }
                                    #[cfg(target_os = "linux")]
                                    {
                                        use std::process::Command;
                                        let parent = std::path::Path::new(&file_path)
                                            .parent()
                                            .unwrap_or(std::path::Path::new(""));
                                        let _ = Command::new("xdg-open").arg(parent).spawn();
                                    }
                                }
                            },
                            "Open Folder"
                        }
                    } else {
                        div { class: "flex-1 bg-red-100 text-red-800 py-2 px-3 rounded-lg text-sm text-center flex items-center justify-center",
                            Icon {
                                icon: BsExclamationTriangleFill,
                                width: 12,
                                height: 12,
                                class: "mr-1.5",
                            }
                            "File not found"
                        }
                    }
                }
            }
        }
    }
}
