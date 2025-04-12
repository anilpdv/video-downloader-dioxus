use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::{BsBoxArrowUpRight, BsVolumeDown, BsVolumeUp};
use dioxus_free_icons::icons::fa_solid_icons::{FaPause, FaPlay};
use dioxus_free_icons::Icon;
use std::time::Duration;

use crate::views::downloads::media_player;
use crate::views::downloads::vlc_player::{PlayerStatus, VlcPlayer};

#[component]
pub fn EmbeddedPlayer(
    file_path: String,
    title: String,
    thumbnail_url: Option<String>,
    on_close: EventHandler<()>,
) -> Element {
    // Create shared state between component renders
    let mut player_status = use_signal(|| PlayerStatus {
        is_playing: false,
        position: 0.0,
        duration_ms: 0,
        time_ms: 0,
        volume: 80,
        state: "Stopped".to_string(),
    });
    let mut error_message = use_signal(|| None::<String>);
    let mut vlc_player = use_signal(|| Option::<VlcPlayer>::None);
    let mut attempted_init = use_signal(|| false);

    // Store file_path in a signal to access it across effects
    let file_path_signal = use_signal(|| file_path);

    // Use an effect for one-time initialization
    use_effect(move || {
        if *attempted_init.read() {
            return;
        }

        attempted_init.set(true);
        let path = file_path_signal.read().clone();

        // Attempt to create and initialize VLC player
        tracing::info!("Attempting to create VLC player for: {}", path);

        #[cfg(not(target_arch = "wasm32"))]
        {
            match VlcPlayer::new(&path) {
                Ok(player) => {
                    vlc_player.set(Some(player));
                    tracing::info!("Successfully created VLC player for: {}", path);
                }
                Err(e) => {
                    let err_msg = format!("Failed to create VLC instance: {}", e);
                    tracing::error!("{}", err_msg);
                    error_message.set(Some(err_msg));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            tracing::info!("VLC player not available in web mode");
            error_message.set(Some(String::from("VLC player not available in web mode")));
        }
    });

    // Set up polling for status updates in a separate effect to prevent infinite loops
    use_effect(move || {
        let vlc_player_for_polling = vlc_player.clone();
        let mut player_status_for_polling = player_status.clone();

        // Use use_coroutine for polling without using a channel
        use_coroutine(move |_: UnboundedReceiver<()>| {
            let vlc_player = vlc_player_for_polling;
            let mut player_status = player_status_for_polling;

            async move {
                let mut interval = tokio::time::interval(Duration::from_millis(200));

                // Continue polling until dropped
                loop {
                    interval.tick().await;
                    if let Some(player) = &vlc_player.read().as_ref() {
                        let status = player.get_status();
                        player_status.set(status);
                    }
                }
            }
        });

        // No explicit cleanup needed as coroutine will be dropped when effect is cleaned up
    });

    // Set up cleanup for player when component unmounts
    let mut vlc_player_for_drop = vlc_player.clone();
    use_drop(move || {
        if let Some(mut player) = vlc_player_for_drop.take() {
            let _ = player.stop();
        }
    });

    // Format time for display (mm:ss)
    let format_time = |ms: i64| {
        let total_seconds = ms / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    };

    // Handler for opening in external player
    let open_external = move |_| {
        let path = file_path_signal.read().clone();
        tracing::info!("Opening file in external player: {}", path);
        if let Err(e) = media_player::open_external_player(&path) {
            tracing::error!("Failed to open external player: {}", e);
        }
        on_close.call(());
    };

    // Create a second copy for the error section button
    let open_external_error = move |_| {
        let path = file_path_signal.read().clone();
        tracing::info!(
            "Opening file in external player from error section: {}",
            path
        );
        if let Err(e) = media_player::open_external_player(&path) {
            tracing::error!("Failed to open external player from error section: {}", e);
        }
        on_close.call(());
    };

    // Create a third copy for the bottom button
    let open_external_bottom = move |_| {
        let path = file_path_signal.read().clone();
        tracing::info!(
            "Opening file in external player from bottom button: {}",
            path
        );
        if let Err(e) = media_player::open_external_player(&path) {
            tracing::error!("Failed to open external player from bottom button: {}", e);
        }
        on_close.call(());
    };

    // Handler for play/pause button
    let toggle_play = move |_| {
        if let Some(player) = &mut vlc_player.write().as_mut() {
            if let Err(e) = player.toggle_play() {
                error_message.set(Some(format!("Failed to toggle playback: {}", e)));
            }
        }
    };

    // Handler for seeking
    let seek = move |ev: Event<FormData>| {
        if let Some(player) = &mut vlc_player.write().as_mut() {
            if let Ok(pos) = ev.value().parse::<f32>() {
                if let Err(e) = player.set_position(pos) {
                    error_message.set(Some(format!("Failed to seek: {}", e)));
                }
            }
        }
    };

    // Handler for volume
    let change_volume = move |ev: Event<FormData>| {
        if let Some(player) = &mut vlc_player.write().as_mut() {
            if let Ok(vol) = ev.value().parse::<i32>() {
                if let Err(e) = player.set_volume(vol) {
                    error_message.set(Some(format!("Failed to change volume: {}", e)));
                }
            }
        }
    };

    // Close handler
    let handle_close = move |_| {
        if let Some(player) = &mut vlc_player.write().as_mut() {
            let _ = player.stop();
        }
        on_close.call(());
    };

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-80",
            div { class: "bg-background-darker rounded-lg shadow-xl overflow-hidden max-w-4xl w-full max-h-[90vh] flex flex-col",
                // Header with title and close button
                div { class: "flex justify-between items-center p-4 border-b border-border",
                    h3 { class: "text-xl font-medium text-text-primary truncate", "{title}" }
                    button {
                        class: "text-text-muted hover:text-text-primary text-2xl",
                        onclick: handle_close,
                        "×"
                    }
                }

                // Main content area
                div { class: "relative aspect-video bg-black flex-1",
                    // Show thumbnail as background
                    if let Some(thumbnail) = thumbnail_url.as_ref() {
                        div {
                            class: "absolute inset-0 opacity-30",
                            style: "background-image: url('{thumbnail}'); background-size: cover; background-position: center; filter: blur(10px);",
                        }
                    }

                    // Main content
                    div { class: "absolute inset-0 flex flex-col items-center justify-center",
                        // Show error message if any
                        if let Some(error) = error_message() {
                            div { class: "bg-accent-rose bg-opacity-20 text-accent-rose p-6 rounded-lg max-w-md text-center",
                                p { class: "text-lg font-medium mb-2", "VLC Player Error" }
                                p { class: "mb-6", "{error}" }

                                // External player button with more prominence
                                div { class: "flex flex-col items-center",
                                    p { class: "text-white text-sm mb-3",
                                        "VLC works best in a separate window outside the browser."
                                    }
                                    button {
                                        class: "px-6 py-3 bg-accent-teal hover:bg-opacity-80 text-text-invert rounded-lg flex items-center justify-center mx-auto font-medium",
                                        onclick: open_external_error,
                                        Icon {
                                            icon: BsBoxArrowUpRight,
                                            width: 16,
                                            height: 16,
                                            class: "mr-2",
                                        }
                                        "Open in External Player"
                                    }
                                }
                            }
                        } else {
                            // Show player status and controls or a prominent button for external player
                            div { class: "flex flex-col items-center w-full",
                                // Player info
                                p { class: "text-text-primary text-lg mb-4", "Now Playing: {title}" }

                                // External player button (always shown on web, optionally for desktop)
                                button {
                                    class: "mb-4 px-6 py-3 bg-accent-teal hover:bg-opacity-80 text-text-invert rounded-lg flex items-center justify-center mx-auto font-medium",
                                    onclick: open_external,
                                    Icon {
                                        icon: BsBoxArrowUpRight,
                                        width: 16,
                                        height: 16,
                                        class: "mr-2",
                                    }
                                    "Open in External VLC"
                                }

                                // Only show standard player controls if we have a player and aren't in web mode
                                if vlc_player.read().is_some() {
                                    // Player status
                                    p { class: "text-text-muted text-sm mb-4",
                                        "State: {player_status().state}"
                                    }

                                    // Big play/pause button
                                    div {
                                        class: "bg-background-card bg-opacity-40 hover:bg-opacity-60 rounded-full p-8 mb-4 cursor-pointer",
                                        onclick: toggle_play,
                                        if player_status().is_playing {
                                            Icon {
                                                icon: FaPause,
                                                width: 48,
                                                height: 48,
                                                class: "text-accent-teal",
                                            }
                                        } else {
                                            Icon {
                                                icon: FaPlay,
                                                width: 48,
                                                height: 48,
                                                class: "text-accent-teal",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Controls bar
                div { class: "p-4 bg-background-card",
                    if vlc_player.read().is_some() {
                        // Progress bar
                        div { class: "mb-4",
                            div { class: "flex justify-between text-xs text-text-muted mb-1",
                                span { "{format_time(player_status().time_ms)}" }
                                span { "{format_time(player_status().duration_ms)}" }
                            }
                            input {
                                class: "w-full h-2 bg-background-medium rounded-lg appearance-none cursor-pointer",
                                r#type: "range",
                                min: "0",
                                max: "1",
                                step: "0.01",
                                value: "{player_status().position}",
                                oninput: seek,
                            }
                        }

                        // Control buttons and volume
                        div { class: "flex justify-between items-center",
                            // Play/Pause button
                            button {
                                class: "bg-accent-teal hover:bg-opacity-80 text-text-invert rounded-full p-3 flex items-center justify-center transition-colors",
                                onclick: toggle_play,
                                if player_status().is_playing {
                                    Icon {
                                        icon: FaPause,
                                        width: 16,
                                        height: 16,
                                    }
                                } else {
                                    Icon { icon: FaPlay, width: 16, height: 16 }
                                }
                            }

                            // Volume control
                            div { class: "flex items-center",
                                Icon {
                                    icon: BsVolumeDown,
                                    width: 16,
                                    height: 16,
                                    class: "text-text-muted mr-2",
                                }
                                input {
                                    class: "w-24 h-2 bg-background-medium rounded-lg appearance-none cursor-pointer",
                                    r#type: "range",
                                    min: "0",
                                    max: "100",
                                    step: "1",
                                    value: "{player_status().volume}",
                                    oninput: change_volume,
                                }
                                Icon {
                                    icon: BsVolumeUp,
                                    width: 16,
                                    height: 16,
                                    class: "text-text-muted ml-2",
                                }
                            }
                        }
                    } else if error_message().is_some() {
                        // If there's an error, show option to use external player
                        div { class: "flex justify-center",
                            button {
                                class: "px-4 py-2 bg-accent-amber hover:bg-opacity-80 text-text-invert rounded-lg flex items-center justify-center",
                                onclick: open_external_bottom,
                                "Try External Player"
                            }
                        }
                    } else {
                        // Loading state
                        div { class: "flex justify-center",
                            p { class: "text-text-muted", "Initializing player..." }
                        }
                    }
                }
            }
        }
    }
}
