use dioxus::prelude::use_effect;
use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::BsExclamationTriangleFill;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCalendar, FaDatabase, FaMusic, FaPause, FaPlay, FaTrash, FaVideo,
};
use dioxus_free_icons::icons::hi_solid_icons::HiRefresh;
use dioxus_free_icons::Icon;

use crate::views::downloads::components::EmbeddedPlayer;
use crate::views::downloads::data_access;
use crate::views::downloads::media_player;
use crate::views::downloads::types::DownloadItem;

#[component]
pub fn DownloadCard(download: DownloadItem, on_delete: EventHandler<i64>) -> Element {
    let is_audio = &download.format_type == "audio";
    let mut play_video = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let mut load_error = use_signal(|| false);
    let mut show_embedded_player = use_signal(|| false);
    let file_path = use_hook(|| download.file_path.clone());
    let is_large_file = media_player::is_large_file(&file_path);
    let vlc_available = media_player::is_vlc_available();
    let mut confirm_delete = use_signal(|| false);

    // Clone file_path for each closure that needs it BEFORE any moves happen
    let file_path_for_media = file_path.clone();
    let file_path_for_toggle = file_path.clone();
    let file_path_for_error = file_path.clone();
    let file_path_for_main = file_path.clone();
    let file_path_for_folder = file_path.clone();

    tracing::info!(
        "File path: {}, is large: {}, VLC available: {}",
        file_path,
        is_large_file,
        vlc_available
    );

    // Function to handle the delete confirmation
    let handle_confirm_delete = move |_| {
        if let Some(id) = download.id {
            // Call the on_delete handler with the download ID
            on_delete.call(id);
        }
        confirm_delete.set(false);
    };

    // Function to get the optimized media source URL
    let get_media_src = move || data_access::get_media_url(&file_path_for_media);

    // Function to open file in external player
    let handle_open_file = move || {
        data_access::open_file(&file_path_for_main);
    };

    // Just use a hardcoded player type for now until we fix the async issues
    let player_type = if vlc_available {
        "External VLC"
    } else if cfg!(feature = "vlc") {
        "Embedded VLC"
    } else {
        "System Default"
    };

    // Toggle player visibility with improved error handling
    let toggle_player = move |_| {
        if show_embedded_player() {
            tracing::info!("Closing embedded player");
            show_embedded_player.set(false);
        } else {
            // Only show embedded player for large files with VLC available
            if is_large_file && vlc_available {
                tracing::info!("Opening embedded VLC player for large file");
                show_embedded_player.set(true);
            } else if is_large_file {
                tracing::info!(
                    "VLC not available, opening file externally: {}",
                    file_path_for_toggle
                );
                handle_open_file();
            } else {
                tracing::info!("Playing small file in browser");
                if !play_video() {
                    is_loading.set(true);
                }
                play_video.set(!play_video());
            }
        }
    };

    // Error handler for media loading errors
    let handle_error = move |_| {
        tracing::error!("Error loading video: path={}", file_path_for_error);
        is_loading.set(false);
        load_error.set(true);
        play_video.set(false);
    };

    // Handler for error message "Open External Player" button
    let error_external_player = {
        let file_path = file_path.clone();
        move |_| {
            load_error.set(false);
            if vlc_available {
                show_embedded_player.set(true);
            } else {
                data_access::open_file(&file_path);
            }
        }
    };

    // Handler for the main "Open External Player" button
    let main_external_player = {
        let file_path = file_path.clone();
        move |_| {
            if vlc_available {
                show_embedded_player.set(true);
            } else {
                data_access::open_file(&file_path);
            }
        }
    };

    // Handler for "Open Folder" button
    let open_folder = move |_| {
        data_access::open_containing_folder(&file_path_for_folder);
    };

    // Close embedded player handler
    let close_embedded_player = move |_| {
        show_embedded_player.set(false);
    };

    rsx! {
        // Show embedded player modal
        if show_embedded_player() {
            EmbeddedPlayer {
                file_path: file_path.clone(),
                title: download.title.clone(),
                thumbnail_url: download.thumbnail_url.clone(),
                on_close: close_embedded_player,
            }
        }

        // Confirmation dialog
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

                // Large file indicator
                if is_large_file {
                    div { class: "absolute top-2 left-2 bg-accent-amber bg-opacity-75 text-text-invert text-xs px-2 py-1 rounded-full",
                        if vlc_available {
                            "Large File • Embedded VLC Available"
                        } else if player_type == "External VLC" {
                            "Large File • Embedded Player Available"
                        } else {
                            "Large File • External Player Recommended"
                        }
                    }
                }

                // Add player info message when it's not available
                if !vlc_available && is_large_file && player_type == "Web Player" {
                    div { class: "absolute top-12 left-2 bg-accent-rose bg-opacity-75 text-text-invert text-xs px-2 py-1 rounded-full z-10",
                        "VLC Not Found - Install for Better Playback"
                    }
                }

                // Video/Thumbnail content
                if let Some(ref thumbnail) = download.thumbnail_url {
                    // Show thumbnail when not playing video
                    if !play_video() {
                        img {
                            class: "w-full h-full object-cover",
                            src: "{thumbnail}",
                            alt: "Thumbnail",
                        }
                    }
                    // Show video when playing
                    if play_video() {
                        if is_audio {
                            // For audio files, use an audio element with a nicer container
                            div { class: "flex flex-col items-center justify-center w-full h-full bg-gradient-to-b from-background-dark to-background-darker",
                                // Audio player title
                                div { class: "mb-6 text-center",
                                    Icon {
                                        icon: FaMusic,
                                        width: 48,
                                        height: 48,
                                        class: "text-accent-amber mb-3",
                                    }
                                    p { class: "text-text-primary text-lg font-medium",
                                        "{download.title}"
                                    }
                                }

                                // Audio player
                                div { class: "w-4/5 bg-background-card p-4 rounded-lg shadow-md",
                                    audio {
                                        class: "w-full",
                                        src: "{get_media_src()}",
                                        controls: true,
                                        preload: "metadata",
                                        autoplay: true,
                                        onloadstart: move |_| {
                                            is_loading.set(true);
                                        },
                                        oncanplay: move |_| {
                                            is_loading.set(false);
                                        },
                                        onerror: handle_error,
                                    }
                                }
                            }
                        } else {
                            // For video files, use a video element
                            video {
                                class: "w-full h-full object-cover",
                                src: "{get_media_src()}",
                                alt: "Thumbnail",
                                controls: true,
                                preload: "metadata", // Only load metadata initially
                                autoplay: true,
                                onloadstart: move |_| {
                                    is_loading.set(true);
                                },
                                oncanplay: move |_| {
                                    is_loading.set(false);
                                },
                                onerror: handle_error,
                            }
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

                // Loading spinner overlay
                if is_loading() {
                    div { class: "absolute inset-0 flex items-center justify-center bg-black bg-opacity-50 z-20",
                        div { class: "w-12 h-12 animate-spin rounded-full border-4 border-background-card border-t-accent-teal" }
                    }
                }

                // Error message overlay
                if load_error() {
                    div { class: "absolute inset-0 flex items-center justify-center bg-black bg-opacity-50 z-20",
                        div { class: "bg-background-dark p-4 rounded-lg max-w-xs text-center",
                            div { class: "text-accent-rose mb-2 flex justify-center",
                                Icon {
                                    icon: BsExclamationTriangleFill,
                                    width: 24,
                                    height: 24,
                                }
                            }
                            p { class: "text-white text-sm mb-3",
                                if vlc_available {
                                    "This video couldn't be played in the browser. Try the embedded VLC player."
                                } else {
                                    "This video couldn't be played in the browser. Please use an external player."
                                }
                            }
                            button {
                                class: "bg-accent-teal hover:bg-opacity-80 text-text-invert py-1 px-2 rounded-md text-xs flex items-center mx-auto",
                                onclick: error_external_player,
                                if vlc_available {
                                    "Use Embedded VLC"
                                } else {
                                    "Open External Player"
                                }
                            }
                        }
                    }
                }

                // Only show play button when video is not playing
                if !play_video() {
                    div { class: "absolute inset-0 flex items-center justify-center",
                        div {
                            class: "bg-black bg-opacity-40 text-white rounded-full flex items-center justify-center cursor-pointer w-14 h-14 transition-all duration-200 hover:bg-opacity-60 hover:scale-110 shadow-lg",
                            onclick: toggle_player.clone(),
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

                // Show a small pause button when video is playing
                if play_video() && !is_loading() && !load_error() {
                    div { class: "absolute bottom-4 right-4",
                        div {
                            class: "bg-black bg-opacity-40 text-white rounded-full flex items-center justify-center cursor-pointer p-2 transition-all duration-200 hover:bg-opacity-60",
                            onclick: toggle_player.clone(),
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
                        // For large files, different options based on VLC availability
                        if is_large_file {
                            button {
                                class: "flex-1 bg-accent-amber hover:bg-opacity-80 text-text-invert py-2 px-3 rounded-lg text-sm transition-colors duration-200 flex items-center justify-center shadow-sm",
                                onclick: main_external_player,
                                if vlc_available {
                                    "Play with Embedded VLC"
                                } else {
                                    "Open External Player"
                                }
                            }
                        } else {
                            // For smaller files, show a regular play button
                            button {
                                class: "flex-1 bg-accent-teal hover:bg-opacity-80 text-text-invert py-2 px-3 rounded-lg text-sm transition-colors duration-200 flex items-center justify-center shadow-sm",
                                onclick: toggle_player.clone(),
                                "Play in Browser"
                            }
                        }

                        button {
                            class: "bg-background-medium hover:bg-background-hover text-text-primary py-2 px-3 rounded-lg text-sm transition-colors duration-200 flex items-center justify-center shadow-sm",
                            onclick: open_folder,
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
