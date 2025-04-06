use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::BsExclamationTriangleFill;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCalendar, FaDatabase, FaMusic, FaPause, FaPlay, FaTrash, FaVideo,
};
use dioxus_free_icons::Icon;

use crate::views::downloads::data_access;
use crate::views::downloads::types::DownloadItem;
#[component]
pub fn DownloadCard(download: DownloadItem, on_delete: EventHandler<i64>) -> Element {
    let is_audio = &download.format_type == "audio";
    let mut play_video = use_signal(|| false);
    let file_path = use_hook(|| download.file_path.clone());
    let mut confirm_delete = use_signal(|| false);

    // Function to handle the delete confirmation
    let handle_confirm_delete = move |_| {
        if let Some(id) = download.id {
            // Call the on_delete handler with the download ID
            on_delete.call(id);
        }
        confirm_delete.set(false);
    };

    rsx! {
        if confirm_delete() {
            div { class: "fixed inset-0 z-50 flex items-center justify-center bg-background-darker bg-opacity-75",
                div { class: "bg-background-card p-4 rounded-lg shadow-md border border-border max-w-xs",
                    p { class: "text-text-primary text-sm mb-3",
                        "Are you sure you want to delete this download?"
                    }
                    div { class: "flex justify-end space-x-2",
                        button {
                            class: "px-3 py-1.5 text-xs bg-background-medium rounded-md text-text-secondary",
                            onclick: move |_| confirm_delete.set(false),
                            "Cancel"
                        }
                        button {
                            class: "px-3 py-1.5 text-xs bg-accent-rose rounded-md text-text-primary",
                            onclick: handle_confirm_delete,
                            "Delete"
                        }
                    }
                }
            }
        }
        div { class: "bg-background-card rounded-xl shadow-md overflow-hidden hover:shadow-lg transition-all duration-300 border border-border transform hover:-translate-y-1 hover:border-border-light relative",
            div { class: "relative aspect-video bg-background-dark",
                // Add a delete button in top right corner with better visibility
                button {
                    class: "absolute top-2 right-2 z-10 bg-background-darker bg-opacity-75 text-accent-rose p-2 rounded-md transition-colors duration-200 hover:bg-accent-rose hover:text-text-primary",
                    onclick: move |_| {
                        tracing::info!("Delete button clicked");
                        confirm_delete.set(true);
                    },
                    title: "Delete download",
                    Icon { icon: FaTrash, width: 14, height: 14 }
                }
                // Rest of your component remains unchanged
                if let Some(ref thumbnail) = download.thumbnail_url {
                    if play_video() {
                        video {
                            class: "w-full h-full object-cover",
                            src: "{file_path}",
                            alt: "Thumbnail",
                            controls: true,
                            autoplay: true,
                            onerror: move |e| {
                                tracing::error!("Error loading video: {:?}", e);
                            },
                        }
                    } else {
                        img {
                            class: "w-full h-full object-cover",
                            src: "{thumbnail}",
                            alt: "Thumbnail",
                        }
                    }
                } else {
                    div { class: if !play_video() { "w-full h-full flex items-center justify-center bg-gradient-to-r from-background-darker to-background" } else { "hidden" },
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

                // Only show play button when video is not playing
                if !play_video() {
                    div { class: "absolute inset-0 flex items-center justify-center",
                        div {
                            class: "bg-black bg-opacity-40 text-white rounded-full flex items-center justify-center cursor-pointer w-14 h-14 transition-all duration-200 hover:bg-opacity-60 hover:scale-110 shadow-lg",
                            onclick: move |_| play_video.set(true),
                            div { class: "ml-1", // Slight offset for play icon (visual centering)
                                Icon {
                                    icon: FaPlay,
                                    width: 24,
                                    height: 24,
                                    class: "text-white",
                                }
                            }
                        }
                    }
                }

                // Show a small pause button when video is playing (less intrusive)
                if play_video() {
                    div { class: "absolute bottom-4 right-4",
                        div {
                            class: "bg-black bg-opacity-40 text-white rounded-full flex items-center justify-center cursor-pointer p-2 transition-all duration-200 hover:bg-opacity-60",
                            onclick: move |_| play_video.set(false),
                            Icon {
                                icon: FaPause,
                                width: 16,
                                height: 16,
                                class: "text-white",
                            }
                        }
                    }
                }

                div { class: if play_video() { "hidden" } else { "absolute top-2 left-2 bg-accent-teal bg-opacity-90 text-text-invert text-xs px-2 py-1 rounded-full flex items-center" },
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

                if let Some(_) = download.duration {
                    div { class: "absolute bottom-2 right-2 bg-background-darker bg-opacity-75 text-text-primary text-xs px-2 py-1 rounded-full",
                        "{download.format_duration()}"
                    }
                }

                div { class: if play_video() { "hidden" } else { "absolute bottom-2 left-2 bg-background-darker bg-opacity-75 text-text-primary text-xs px-2 py-1 rounded-full" },
                    "{download.quality}"
                }
            }

            // Rest of the component...
            div { class: "p-4",
                h3 { class: "font-medium text-lg mb-2 line-clamp-2 text-text-primary",
                    "{download.title}"
                }

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

                div { class: "flex space-x-2 mt-3",
                    if download.file_exists {
                        button {
                            class: "flex-1 bg-accent-teal hover:bg-opacity-80 text-text-invert py-2 px-3 rounded-lg text-sm transition-colors duration-200 flex items-center justify-center shadow-sm",
                            onclick: {
                                let file_path = download.file_path.clone();
                                move |_| data_access::open_file(&file_path)
                            },
                            "Play"
                        }

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
