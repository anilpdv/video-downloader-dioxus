use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{
        bs_icons::{BsExclamationTriangleFill, BsSearch},
        fa_solid_icons::{FaCalendar, FaDatabase, FaDownload, FaMusic, FaVideo},
        hi_outline_icons::{HiFilm, HiMusicNote, HiViewGrid},
    },
    Icon,
};

// Platform-agnostic download item model for UI
#[derive(Clone, Debug, PartialEq)]
pub struct DownloadItem {
    pub id: Option<i64>,
    pub title: String,
    pub filename: String,
    pub file_path: String,
    pub format_type: String,
    pub quality: String,
    pub file_size: Option<i64>,
    pub duration: Option<i64>,
    pub date_downloaded: String,
    pub thumbnail_url: Option<String>,
    pub file_exists: bool,
}

impl DownloadItem {
    pub fn format_duration(&self) -> String {
        if let Some(duration) = self.duration {
            let hours = duration / 3600;
            let minutes = (duration % 3600) / 60;
            let seconds = duration % 60;

            if hours > 0 {
                format!("{}:{:02}:{:02}", hours, minutes, seconds)
            } else {
                format!("{}:{:02}", minutes, seconds)
            }
        } else {
            "".to_string()
        }
    }

    pub fn format_file_size(&self) -> String {
        if let Some(size) = self.file_size {
            if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        } else {
            "Unknown".to_string()
        }
    }
}

// Interface for accessing download data - platform agnostic
pub mod data_access {
    use super::DownloadItem;

    #[cfg(feature = "server")]
    pub async fn fetch_downloads() -> Vec<DownloadItem> {
        crate::server::download::fetch_downloads().await
    }

    #[cfg(not(feature = "server"))]
    pub async fn fetch_downloads() -> Vec<DownloadItem> {
        Vec::new() // Empty implementation for non-server platforms
    }

    #[cfg(feature = "server")]
    pub fn open_file(path: &str) {
        crate::server::download::open_file(path);
    }

    #[cfg(feature = "server")]
    pub fn open_containing_folder(path: &str) {
        crate::server::download::open_containing_folder(path);
    }

    #[cfg(not(feature = "server"))]
    pub fn open_file(_: &str) {}

    #[cfg(not(feature = "server"))]
    pub fn open_containing_folder(_: &str) {}
}

// Main download view component
#[component]
pub fn Downloads() -> Element {
    rsx! {
        div { class: "container mx-auto py-6 px-4",
            h1 { class: "text-3xl font-bold mb-4 text-text-primary", "My Downloads" }
            p { class: "mb-6 text-text-secondary",
                "Access and play your downloaded videos and audio files."
            }
            DownloadsContent {}
        }
    }
}

// Content component - handles UI logic separate from data fetching
#[component]
fn DownloadsContent() -> Element {
    let mut active_tab = use_signal(|| "all".to_string());
    let mut search_query = use_signal(|| String::new());

    // State for downloads
    let downloads = use_signal(|| Vec::<DownloadItem>::new());
    let mut loading = use_signal(|| true);

    // Fetch downloads when component mounts
    use_effect(move || {
        if loading() {
            let mut downloads_clone = downloads.clone();
            let mut loading_clone = loading.clone();

            use_future(move || async move {
                let results = data_access::fetch_downloads().await;
                downloads_clone.set(results);
                loading_clone.set(false);
            });
        }
    });

    // Show loading state
    if loading() {
        return rsx! {
            LoadingSpinner {}
        };
    }

    // Determine if we have downloads to show
    let has_downloads = !downloads().is_empty();

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
                .collect()
        };

        // Apply search filter if query is not empty
        if query.is_empty() {
            tab_filtered
        } else {
            tab_filtered
                .into_iter()
                .filter(|d| {
                    d.title.to_lowercase().contains(&query)
                        || d.filename.to_lowercase().contains(&query)
                })
                .collect::<Vec<DownloadItem>>()
        }
    };

    // Count items by type
    let audio_count = downloads()
        .iter()
        .filter(|d| d.format_type == "audio")
        .count();
    let video_count = downloads()
        .iter()
        .filter(|d| d.format_type == "video")
        .count();
    let total_count = audio_count + video_count;

    if !has_downloads {
        return rsx! {
            // Show an informative message based on platform
            div { class: "text-center py-16 bg-background-card rounded-xl border border-border shadow-md",
                div { class: "flex justify-center mb-6",
                    Icon {
                        icon: FaDownload,
                        width: 52,
                        height: 52,
                        class: "text-text-muted",
                    }
                }
                {
                    if cfg!(feature = "server") {
                        rsx! {
                            p { class: "text-xl font-medium text-text-primary", "No downloads yet" }
                            p { class: "text-text-secondary mt-2 max-w-md mx-auto",
                                "Your downloaded files will appear here. Try downloading a video or audio file from the home page."
                            }
                        }
                    } else {
                        rsx! {
                            p { class: "text-xl font-medium text-text-secondary", "Download history is not available in web mode" }
                            p { class: "text-text-muted mt-2",
                                "For full functionality including download history, please use the desktop app."
                            }
                        }
                    }
                }
            }
        };
    }

    // Show downloads with tabs
    rsx! {
        // Search bar
        div { class: "mb-6 relative",
            div { class: "relative",
                span { class: "absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none",
                    Icon {
                        icon: BsSearch,
                        width: 16,
                        height: 16,
                        class: "text-text-muted",
                    }
                }
                input {
                    class: "bg-background-card border border-border text-text-primary text-sm rounded-lg focus:ring-accent-teal focus:border-accent-teal block w-full pl-10 p-2.5",
                    r#type: "text",
                    placeholder: "Search downloads...",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value().clone()),
                }
            }
        }

        // Tab navigation
        div { class: "mb-6 border-b border-border",
            div { class: "flex flex-wrap -mb-px",
                // All tab
                button {
                    class: if active_tab() == "all" { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-accent-teal border-b-2 border-accent-teal rounded-t-lg" } else { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-text-muted border-b-2 border-transparent hover:text-text-secondary hover:border-border rounded-t-lg" },
                    onclick: move |_| active_tab.set("all".to_string()),
                    Icon {
                        icon: HiViewGrid,
                        width: 18,
                        height: 18,
                        class: "mr-2",
                    }
                    "All ({total_count})"
                }

                // Audio tab
                button {
                    class: if active_tab() == "audio" { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-accent-teal border-b-2 border-accent-teal rounded-t-lg" } else { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-text-muted border-b-2 border-transparent hover:text-text-secondary hover:border-border rounded-t-lg" },
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
                    class: if active_tab() == "video" { "inline-flex items-center py-3 px-4 text-sm font-medium text-accent-teal border-b-2 border-accent-teal rounded-t-lg" } else { "inline-flex items-center py-3 px-4 text-sm font-medium text-text-muted border-b-2 border-transparent hover:text-text-secondary hover:border-border rounded-t-lg" },
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
            div { class: "text-center py-12 bg-background-card rounded-xl border border-border shadow-md",
                if !search_query().is_empty() {
                    div { class: "flex flex-col items-center",
                        Icon {
                            icon: BsSearch,
                            width: 40,
                            height: 40,
                            class: "text-text-muted mb-4",
                        }
                        p { class: "text-lg font-medium text-text-primary",
                            "No results found for \"{search_query()}\""
                        }
                        p { class: "text-text-secondary mt-2",
                            "Try different keywords or clear your search"
                        }
                        button {
                            class: "mt-4 px-4 py-2 bg-accent-teal text-text-primary rounded-lg text-sm hover:bg-opacity-80 transition-colors",
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
                                class: "text-text-muted mb-4",
                            }
                        } else {
                            Icon {
                                icon: FaVideo,
                                width: 40,
                                height: 40,
                                class: "text-text-muted mb-4",
                            }
                        }
                        p { class: "text-lg font-medium text-text-primary",
                            "No {active_tab()} files found"
                        }
                        p { class: "text-text-secondary mt-2",
                            "Try switching to a different category or download some {active_tab()} files."
                        }
                    }
                }
            }
        } else {
            // Grid display of downloads
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                for download in filtered_downloads {
                    DownloadCard { download }
                }
            }
        }
    }
}

// Simple loading spinner component
#[component]
fn LoadingSpinner() -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center py-16",
            div { class: "animate-spin w-12 h-12 mb-4 text-text-secondary",
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
            p { class: "text-text-muted", "Loading your downloads..." }
        }
    }
}

// Download card component - separated from server logic
#[component]
fn DownloadCard(download: DownloadItem) -> Element {
    let is_video = &download.format_type == "video";
    let is_audio = &download.format_type == "audio";

    rsx! {
        div { class: "bg-background-card rounded-xl shadow-md overflow-hidden hover:shadow-lg transition-all duration-300 border border-border transform hover:-translate-y-1 hover:border-border-light",
            // Thumbnail area
            div { class: "relative aspect-video bg-background-dark",
                if let Some(ref thumbnail) = download.thumbnail_url {
                    img {
                        class: "w-full h-full object-cover",
                        src: "{thumbnail}",
                        alt: "Thumbnail",
                    }
                } else {
                    div { class: "w-full h-full flex items-center justify-center bg-gradient-to-r from-background-darker to-background",
                        if is_audio {
                            Icon {
                                icon: FaMusic,
                                width: 48,
                                height: 48,
                                class: "text-accent-amber opacity-50",
                            }
                        } else {
                            Icon {
                                icon: FaVideo,
                                width: 48,
                                height: 48,
                                class: "text-accent-teal opacity-50",
                            }
                        }
                    }
                }

                // Format badge
                div { class: if is_audio { "absolute top-2 right-2 bg-accent-amber bg-opacity-90 text-text-invert text-xs px-2 py-1 rounded-full flex items-center" } else { "absolute top-2 right-2 bg-accent-teal bg-opacity-90 text-text-invert text-xs px-2 py-1 rounded-full flex items-center" },
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
                    div { class: "absolute bottom-2 right-2 bg-background-darker bg-opacity-75 text-text-primary text-xs px-2 py-1 rounded-full",
                        "{download.format_duration()}"
                    }
                }

                // Quality badge
                div { class: "absolute bottom-2 left-2 bg-background-darker bg-opacity-75 text-text-primary text-xs px-2 py-1 rounded-full",
                    "{download.quality}"
                }
            }

            // Details section
            div { class: "p-4",
                // Title
                h3 { class: "font-medium text-lg mb-2 line-clamp-2 text-text-primary",
                    "{download.title}"
                }

                // Info row
                div { class: "flex justify-between text-sm text-text-muted mb-4",
                    div { class: "flex items-center",
                        Icon {
                            icon: FaCalendar,
                            width: 12,
                            height: 12,
                            class: "mr-1.5",
                        }
                        span { "{download.date_downloaded}" }
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
                    if download.file_exists {
                        // Play button
                        button {
                            class: "flex-1 bg-accent-teal hover:bg-opacity-80 text-text-invert py-2 px-3 rounded-lg text-sm transition-colors duration-200 flex items-center justify-center shadow-sm",
                            onclick: {
                                let file_path = download.file_path.clone();
                                move |_| data_access::open_file(&file_path)
                            },
                            "Play Media"
                        }

                        // Open folder button
                        button {
                            class: "bg-background-medium hover:bg-background-hover text-text-primary py-2 px-3 rounded-lg text-sm transition-colors duration-200 flex items-center justify-center shadow-sm",
                            onclick: {
                                let file_path = download.file_path.clone();
                                move |_| data_access::open_containing_folder(&file_path)
                            },
                            "Open Folder"
                        }
                    } else {
                        div { class: "flex-1 bg-accent-rose bg-opacity-20 text-accent-rose py-2 px-3 rounded-lg text-sm text-center flex items-center justify-center",
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
