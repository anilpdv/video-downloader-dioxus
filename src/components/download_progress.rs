use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{
        bs_icons::{BsArrowDownCircle, BsCloudDownload, BsDownload},
        fa_solid_icons::{FaDownload, FaSpinner},
    },
    Icon,
};

// Struct to hold download progress information
#[derive(Debug, Clone, Default)]
pub struct DownloadInfo {
    pub url: String,
    pub file_name: String,
    pub progress: f64,
    pub downloaded_size: String,
    pub total_size: String,
    pub speed: String,
    pub eta: String,
    pub status: DownloadStatus,
    pub blob_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    NotStarted,
    Downloading,
    Completed,
    Failed(String),
}

impl Default for DownloadStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

// Download progress component - shows different UI based on platform
#[component]
pub fn DownloadProgress(
    download_info: Signal<DownloadInfo>,
    on_download_click: EventHandler<()>,
) -> Element {
    // Render platform-specific progress UI
    #[cfg(feature = "web")]
    {
        // Web-specific UI with download blob URL
        rsx! {
            div { class: "download-progress-container bg-background-card p-4 rounded-lg shadow-sm mb-4",
                match download_info().status {
                    DownloadStatus::NotStarted => rsx! {
                        div { class: "flex items-center",
                            button {
                                class: "flex items-center bg-accent-teal hover:bg-opacity-80 text-text-invert py-2 px-4 rounded-lg transition-colors",
                                onclick: move |_| on_download_click.call(()),
                                Icon {
                                    icon: BsCloudDownload,
                                    width: 18,
                                    height: 18,
                                    class: "mr-2",
                                }
                                "Download {download_info().file_name}"
                            }
                        }
                    },
                    DownloadStatus::Downloading => rsx! {
                        div { class: "flex flex-col",
                            div { class: "flex justify-between mb-2",
                                span { class: "text-sm font-medium text-text-primary", "Downloading {download_info().file_name}" }
                                span { class: "text-xs text-text-secondary",
                                    "{download_info().downloaded_size} of {download_info().total_size} ({(download_info().progress * 100.0) as i32}%)"
                                }
                            }
                            div { class: "w-full bg-background-dark rounded-full h-2.5",
                                div {
                                    class: "bg-accent-teal h-2.5 rounded-full transition-all duration-300",
                                    style: "width: {(download_info().progress * 100.0) as i32}%",
                                }
                            }
                            div { class: "flex justify-between mt-2 text-xs text-text-muted",
                                span { "{download_info().speed}" }
                                span { "ETA: {download_info().eta}" }
                            }
                        }
                    },
                    DownloadStatus::Completed => rsx! {
                        div { class: "flex flex-col",
                            div { class: "flex justify-between items-center mb-2",
                                span { class: "text-green-500 font-medium", "Download Complete!" }
                                span { class: "text-xs text-text-secondary", "{download_info().file_name}" }
                            }
                        
                            if let Some(blob_url) = &download_info().blob_url {
                                div { class: "flex space-x-2 mt-2",
                                    a {
                                        class: "flex-1 flex items-center justify-center bg-accent-teal hover:bg-opacity-80 text-text-invert py-2 px-3 rounded-lg transition-colors text-sm",
                                        href: "{blob_url}",
                                        download: "{download_info().file_name}",
                                        Icon {
                                            icon: BsArrowDownCircle,
                                            width: 16,
                                            height: 16,
                                            class: "mr-2",
                                        }
                                        "Save File"
                                    }
                        
                                    a {
                                        class: "flex items-center justify-center bg-background-medium hover:bg-background-hover text-text-primary py-2 px-3 rounded-lg transition-colors text-sm",
                                        href: "{blob_url}",
                                        target: "_blank",
                                        "Play Media"
                                    }
                                }
                            }
                        }
                    },
                    DownloadStatus::Failed(ref error) => rsx! {
                        div { class: "text-accent-rose",
                            p { "Download failed: {error}" }
                            button {
                                class: "mt-2 bg-accent-teal hover:bg-opacity-80 text-text-invert py-1 px-3 rounded-lg text-sm transition-colors",
                                onclick: move |_| on_download_click.call(()),
                                "Retry"
                            }
                        }
                    },
                }
            }
        }
    }

    // Desktop/mobile UI with file system paths
    #[cfg(not(feature = "web"))]
    {
        rsx! {
            div { class: "download-progress-container bg-background-card p-4 rounded-lg shadow-sm mb-4",
                match download_info().status {
                    DownloadStatus::NotStarted => rsx! {
                        div { class: "flex items-center",
                            button {
                                class: "flex items-center bg-accent-teal hover:bg-opacity-80 text-text-invert py-2 px-4 rounded-lg transition-colors",
                                onclick: move |_| on_download_click.call(()),
                                Icon {
                                    icon: FaDownload,
                                    width: 18,
                                    height: 18,
                                    class: "mr-2",
                                }
                                "Download {download_info().file_name}"
                            }
                        }
                    },
                    DownloadStatus::Downloading => rsx! {
                        div { class: "flex flex-col",
                            div { class: "flex justify-between mb-2",
                                span { class: "text-sm font-medium text-text-primary", "Downloading to your device..." }
                                span { class: "text-xs text-text-secondary",
                                    "{download_info().downloaded_size} of {download_info().total_size} ({(download_info().progress * 100.0) as i32}%)"
                                }
                            }
                            div { class: "w-full bg-background-dark rounded-full h-2.5",
                                div {
                                    class: "bg-accent-teal h-2.5 rounded-full transition-all duration-300",
                                    style: "width: {(download_info().progress * 100.0) as i32}%",
                                }
                            }
                            div { class: "flex justify-between mt-2 text-xs text-text-muted",
                                span { "{download_info().speed}" }
                                span { "ETA: {download_info().eta}" }
                            }
                        }
                    },
                    DownloadStatus::Completed => rsx! {
                        div { class: "flex flex-col",
                            p { class: "text-green-500 mb-2", "Download Complete!" }
                            p { class: "text-text-secondary text-sm",
                                "Saved to: {download_info().blob_url.as_ref().unwrap_or(&String::new())}"
                            }
                        }
                    },
                    DownloadStatus::Failed(ref error) => rsx! {
                        div { class: "text-accent-rose",
                            p { "Download failed: {error}" }
                            button {
                                class: "mt-2 bg-accent-teal hover:bg-opacity-80 text-text-invert py-1 px-3 rounded-lg text-sm transition-colors",
                                onclick: move |_| on_download_click.call(()),
                                "Retry"
                            }
                        }
                    },
                }
            }
        }
    }
}
