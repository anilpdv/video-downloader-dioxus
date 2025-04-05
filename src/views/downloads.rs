use crate::common::Toaster;
use crate::components::download_progress::{DownloadInfo, DownloadStatus};
use dioxus::prelude::Signal;
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{
        bs_icons::{BsExclamationTriangleFill, BsSearch},
        fa_solid_icons::{FaCalendar, FaDatabase, FaDownload, FaMusic, FaVideo},
        hi_outline_icons::{HiFilm, HiMusicNote, HiViewGrid},
    },
    Icon,
};
use std::collections::HashMap;

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
    use crate::components::download_progress::{DownloadInfo, DownloadStatus};

    // For non-web platforms (desktop, iOS, etc.)
    #[cfg(not(feature = "web"))]
    pub async fn fetch_downloads() -> Vec<DownloadItem> {
        #[cfg(feature = "server")]
        {
            // Call the server-side function directly
            crate::server::download::services::fetch_downloads().await
        }
        #[cfg(not(feature = "server"))]
        {
            Vec::new()
        }
    }

    // Web platform implementation
    #[cfg(feature = "web")]
    pub async fn fetch_downloads() -> Vec<DownloadItem> {
        // Web doesn't support persistent storage in the same way as desktop
        // Return an empty list or fetch from browser's IndexedDB if implemented
        Vec::new()
    }

    // Open file on non-web platforms
    #[cfg(not(feature = "web"))]
    pub fn open_file(path: &str) {
        #[cfg(feature = "server")]
        {
            let _ = crate::server::download::services::open_file(path);
        }
    }

    // Create and open a blob URL on web platform
    #[cfg(feature = "web")]
    pub fn open_file(path: &str) {
        // In a real implementation, you would:
        // 1. Find the downloaded blob URL associated with this path
        // 2. Open it in a new tab or trigger browser download
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url(path);
        }
    }

    // Open containing folder on non-web platforms
    #[cfg(not(feature = "web"))]
    pub fn open_containing_folder(path: &str) {
        #[cfg(feature = "server")]
        {
            let _ = crate::server::download::services::open_containing_folder(path);
        }
    }

    // Web doesn't have folders in the same way
    #[cfg(feature = "web")]
    pub fn open_containing_folder(_: &str) {
        // No-op for web
    }

    // Download file with progress tracking for web
    #[cfg(feature = "web")]
    pub async fn download_with_progress<F>(
        url: &str,
        file_name: &str,
        on_progress: F,
    ) -> Result<String, String>
    where
        F: Fn(DownloadInfo) + 'static,
    {
        crate::server::download::web_services::download_with_progress_real(
            url,
            file_name,
            on_progress,
        )
        .await
    }

    // Non-web implementation for download with progress
    #[cfg(not(feature = "web"))]
    pub async fn download_with_progress<F>(
        url: &str,
        file_name: &str,
        on_progress: F,
    ) -> Result<String, String>
    where
        F: Fn(DownloadInfo) + 'static,
    {
        #[cfg(feature = "server")]
        {
            // Create initial download info
            let mut download_info = DownloadInfo {
                url: url.to_string(),
                file_name: file_name.to_string(),
                status: DownloadStatus::Downloading,
                ..Default::default()
            };

            // Use video handler for downloads
            use crate::server::download::handlers::video;

            // Call the progress callback with initial status
            on_progress(download_info.clone());

            // Do a simple download without progress for now
            // In a real app, you would connect to the progress events
            let result = video::download_video(url.to_string())
                .await
                .map_err(|e| e.to_string());

            match result {
                Ok(_) => {
                    // Update download info with completed status
                    download_info.status = DownloadStatus::Completed;
                    download_info.blob_url = Some(format!("/downloads/{}", file_name));
                    on_progress(download_info.clone());
                    Ok(format!("/downloads/{}", file_name))
                }
                Err(err) => {
                    // Update download info with error status
                    download_info.status = DownloadStatus::Failed(err.clone());
                    on_progress(download_info);
                    Err(err)
                }
            }
        }

        #[cfg(not(feature = "server"))]
        {
            Err("Download not supported on this platform".to_string())
        }
    }
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

    // New state for active downloads with progress
    let active_downloads = use_signal(|| HashMap::<String, DownloadInfo>::new());
    let toaster = use_signal(|| None::<Toaster>);

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

    // Function to handle download requests using atomic references
    let handle_download = move |url: String, filename: String| {
        let downloads_clone = active_downloads.clone();

        use_future(move || {
            let mut downloads_ref = downloads_clone.clone();
            let url_clone = url.clone();
            let filename_clone = filename.clone();

            async move {
                let download_key = format!("{}-{}", url_clone, filename_clone);

                // Create initial download info
                let initial_info = DownloadInfo {
                    url: url_clone.clone(),
                    file_name: filename_clone.clone(),
                    status: DownloadStatus::NotStarted,
                    ..Default::default()
                };

                // Add to active downloads
                downloads_ref
                    .write()
                    .insert(download_key.clone(), initial_info);

                // Define a callback for progress updates
                let callback_ref =
                    std::sync::Arc::new(std::sync::Mutex::new(downloads_ref.clone()));
                let key_ref = download_key.clone();

                let progress_callback = move |info: DownloadInfo| {
                    let info_copy = info.clone();
                    let key_clone = key_ref.clone();
                    let callback = callback_ref.clone();

                    // Use a thread-safe approach
                    use dioxus::prelude::spawn;
                    spawn(async move {
                        if let Ok(mut guard) = callback.lock() {
                            guard.with_mut(|map| {
                                map.insert(key_clone, info_copy);
                            });
                        }
                    });
                };

                // Call the appropriate download function based on platform
                let _ = data_access::download_with_progress(
                    &url_clone,
                    &filename_clone,
                    progress_callback,
                )
                .await;
            }
        });
    };

    // Show loading state
    if loading() {
        return rsx! {
            LoadingSpinner {}
        };
    }

    // Determine if we have downloads to show
    let has_downloads = !downloads().is_empty();

    // If we're on web, show a demo section even if no downloads
    #[cfg(feature = "web")]
    {
        return rsx! {
            // Demo section for web
            div { class: "mb-10",
                // Show active downloads if any
                {
                    if !active_downloads().is_empty() {
                        rsx! {
                            div { class: "mb-8",
                                h2 { class: "text-xl font-semibold mb-4", "Active Downloads" }
                            
                                {
                                    let downloads_map = active_downloads();
                                    downloads_map
                                        .iter()
                                        .map(|(_, info)| {
                                            let info_clone = info.clone();
                                            let url_clone = info.url.clone();
                                            let filename_clone = info.file_name.clone();
                                            rsx! {
                                                crate::components::DownloadProgress {
                                                    download_info: Signal::new(info_clone),
                                                    on_download_click: move |_| { handle_download(url_clone.clone(), filename_clone.clone()) },
                                                }
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                        .into_iter()
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "text-center py-12 bg-background-card rounded-xl border border-border shadow-md mb-10",
                                div { class: "flex justify-center mb-6",
                                    Icon {
                                        icon: FaDownload,
                                        width: 52,
                                        height: 52,
                                        class: "text-text-muted",
                                    }
                                }
                                p { class: "text-xl font-medium text-text-primary", "Try downloading a sample file" }
                                p { class: "text-text-secondary mt-2 max-w-md mx-auto",
                                    "This web demo can download and save files to your device. Try one of the samples below:"
                                }
                            
                                div { class: "mt-6 flex flex-col md:flex-row justify-center gap-3",
                                    // Short video sample
                                    button {
                                        class: "bg-accent-teal hover:bg-opacity-80 text-text-invert py-2 px-6 rounded-lg text-base transition-colors flex items-center",
                                        onclick: move |_| handle_download(
                                            "https://download.samplelib.com/mp4/sample-5s.mp4".to_string(),
                                            "sample-video-short.mp4".to_string(),
                                        ),
                                        Icon {
                                            icon: FaVideo,
                                            width: 18,
                                            height: 18,
                                            class: "mr-2",
                                        }
                                        "Short Video (5s)"
                                    }
                            
                                    // Medium video sample
                                    button {
                                        class: "bg-accent-teal hover:bg-opacity-80 text-text-invert py-2 px-6 rounded-lg text-base transition-colors flex items-center",
                                        onclick: move |_| handle_download(
                                            "https://download.samplelib.com/mp4/sample-15s.mp4".to_string(),
                                            "sample-video-medium.mp4".to_string(),
                                        ),
                                        Icon {
                                            icon: FaVideo,
                                            width: 18,
                                            height: 18,
                                            class: "mr-2",
                                        }
                                        "Medium Video (15s)"
                                    }
                            
                                    // Audio sample
                                    button {
                                        class: "bg-accent-amber hover:bg-opacity-80 text-text-invert py-2 px-6 rounded-lg text-base transition-colors flex items-center",
                                        onclick: move |_| handle_download(
                                            "https://download.samplelib.com/mp3/sample-3s.mp3".to_string(),
                                            "sample-audio.mp3".to_string(),
                                        ),
                                        Icon {
                                            icon: FaMusic,
                                            width: 18,
                                            height: 18,
                                            class: "mr-2",
                                        }
                                        "Audio Sample (3s)"
                                    }
                                }
                            
                                p { class: "text-text-secondary text-xs mt-4",
                                    "Sample files from SampleLib.com - https://samplelib.com/"
                                }
                            }
                        }
                    }
                }

                // Regular downloads section (if any)
                {
                    if has_downloads {
                        rsx! {
                            DownloadsGrid {
                                downloads: downloads.clone(),
                                active_tab: active_tab.clone(),
                                search_query: search_query.clone(),
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }
        };
    }

    // For non-web platforms
    #[cfg(not(feature = "web"))]
    {
        if !has_downloads {
            return rsx! {
                // Show an informative message for non-web platforms
                div { class: "text-center py-16 bg-background-card rounded-xl border border-border shadow-md",
                    div { class: "flex justify-center mb-6",
                        Icon {
                            icon: FaDownload,
                            width: 52,
                            height: 52,
                            class: "text-text-muted",
                        }
                    }
                    p { class: "text-xl font-medium text-text-primary", "No downloads yet" }
                    p { class: "text-text-secondary mt-2 max-w-md mx-auto",
                        "Your downloaded files will appear here. Try downloading a video or audio file from the home page."
                    }
                }
            };
        }

        return rsx! {
            // Show downloads with tabs
            DownloadsGrid {
                downloads: downloads.clone(),
                active_tab: active_tab.clone(),
                search_query: search_query.clone(),
            }
        };
    }
}

// Downloads grid component - separated for reuse
#[component]
fn DownloadsGrid(
    downloads: Signal<Vec<DownloadItem>>,
    active_tab: Signal<String>,
    search_query: Signal<String>,
) -> Element {
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
