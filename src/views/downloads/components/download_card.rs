use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::BsExclamationTriangleFill;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCalendar, FaDatabase, FaMusic, FaPause, FaPlay, FaVideo,
};
use dioxus_free_icons::Icon;

use crate::views::downloads::data_access;
use crate::views::downloads::types::DownloadItem;

#[component]
pub fn DownloadCard(download: DownloadItem) -> Element {
    let is_audio = &download.format_type == "audio";
    let mut play_video = use_signal(|| false);
    let file_path = use_hook(|| download.file_path.clone());

    rsx! {
        div { class: "bg-background-card rounded-xl shadow-md overflow-hidden hover:shadow-lg transition-all duration-300 border border-border transform hover:-translate-y-1 hover:border-border-light",
            div { class: "relative aspect-video bg-background-dark",
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

                div {
                    class: " absolute top-16 right-36 bg-background-darker bg-opacity-75 text-text-primary text-xs  rounded-full flex items-center cursor-pointer p-4",
                    onclick: move |_| play_video.set(!play_video()),
                    if play_video() {
                        Icon {
                            icon: FaPause,
                            width: 32,
                            height: 32,
                            class: "justify-center",
                        }
                    } else {
                        Icon {
                            icon: FaPlay,
                            width: 32,
                            height: 32,
                            class: "justify-center",
                        }
                    }
                }

                div { class: if play_video() { "hidden" } else { "absolute top-2 right-2 bg-accent-teal bg-opacity-90 text-text-invert text-xs px-2 py-1 rounded-full flex items-center" },
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
