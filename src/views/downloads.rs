#[cfg(feature = "server")]
use crate::database::models::Download;
use dioxus::prelude::*;

#[component]
pub fn Downloads() -> Element {
    rsx! {
        div { class: "container mx-auto py-6 px-4",
            h1 { class: "text-3xl font-bold mb-6", "Your Downloads" }
            p { class: "mb-4", "All your downloaded videos and audio files can be accessed here." }
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
        let available_downloads = use_signal(|| false);
        let downloads = use_signal(|| Vec::<Download>::new());
        let loading = use_signal(|| true);

        // Only fetch downloads when the component is first mounted
        use_effect(move || {
            if loading() {
                use crate::database::{get_database, schema::get_all_downloads};

                let mut available_downloads_clone = available_downloads.clone();
                let mut downloads_clone = downloads.clone();
                let mut loading_clone = loading.clone();

                use_future(move || async move {
                    tracing::info!("Loading downloads from database...");
                    if let Ok(pool) = get_database().await {
                        match get_all_downloads(&pool).await {
                            Ok(results) => {
                                tracing::info!("Found {} downloads", results.len());
                                if !results.is_empty() {
                                    available_downloads_clone.set(true);
                                }
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
                div { class: "flex justify-center py-12",
                    p { class: "text-gray-500", "Loading downloads..." }
                }
            };
        }

        if !available_downloads() {
            return rsx! {
                div { class: "text-center py-12 bg-gray-50 rounded-lg",
                    p { class: "text-xl font-medium text-gray-500", "No downloads yet" }
                    p { class: "text-gray-400 mt-2", "Your download history will appear here." }
                }
            };
        }

        // Show downloads
        return rsx! {
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                for download in downloads() {
                    DownloadCard { download: download.clone() }
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
    let file_type = if &download.format_type == "audio" {
        "Audio"
    } else {
        "Video"
    };

    rsx! {
        div { class: "bg-white rounded-lg shadow-md overflow-hidden hover:shadow-lg transition-shadow duration-300",
            // Thumbnail
            div { class: "relative aspect-video bg-gray-200",
                if let Some(ref thumbnail) = download.thumbnail_url {
                    img {
                        class: "w-full h-full object-cover",
                        src: "{thumbnail}",
                        alt: "Video thumbnail",
                    }
                } else {
                    div { class: "w-full h-full flex items-center justify-center bg-gray-800 text-white",
                        if &download.format_type == "audio" {
                            span { class: "text-3xl", "🎵" }
                        } else {
                            span { class: "text-3xl", "📹" }
                        }
                    }
                }

                // Format badge
                div { class: "absolute top-2 right-2 bg-blue-500 text-white text-xs px-2 py-1 rounded",
                    "{file_type}"
                }

                // Duration badge if available
                if let Some(duration_secs) = download.duration {
                    div { class: "absolute bottom-2 right-2 bg-black bg-opacity-70 text-white text-xs px-2 py-1 rounded",
                        "{download.format_duration()}"
                    }
                }
            }

            div { class: "p-4",
                // Title
                h3 { class: "font-medium text-lg mb-1 line-clamp-2",
                    if let Some(ref title) = download.title {
                        "{title}"
                    } else {
                        "Untitled download"
                    }
                }

                // Date and size
                div { class: "flex justify-between text-sm text-gray-500 mb-3",
                    span { "{download.format_date()}" }

                    span { "{download.format_file_size()}" }
                }

                // Actions
                div { class: "flex space-x-2 mt-2",
                    if file_exists {
                        button {
                            class: "flex-1 bg-blue-500 hover:bg-blue-600 text-white py-2 px-3 rounded text-sm transition-colors duration-200",
                            onclick: {
                                let file_path = download.file_path.clone();
                                move |_| {
                                    if let Err(e) = std::process::Command::new("xdg-open")
                                        .arg(&file_path)
                                        .spawn()
                                    {
                                        tracing::error!("Failed to open file: {}", e);
                                    }
                                }
                            },
                            "Play"
                        }

                        button {
                            class: "bg-gray-200 hover:bg-gray-300 text-gray-800 p-2 rounded text-sm transition-colors duration-200",
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
                            span { class: "material-icons", "folder" }
                        }
                    } else {
                        div { class: "flex-1 bg-red-100 text-red-800 py-2 px-3 rounded text-sm text-center",
                            "File not found"
                        }
                    }
                }
            }
        }
    }
}
