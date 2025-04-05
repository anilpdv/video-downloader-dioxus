use crate::views::download::types::FormatType;
use dioxus::prelude::*;

#[component]
pub fn DownloadReady(
    download_ready: Signal<bool>,
    format_type: Signal<FormatType>,
    filename: String,
    on_save_click: EventHandler<()>,
) -> Element {
    if !download_ready() {
        return rsx! {
            div {}
        };
    }

    // Define button text based on platform
    let save_button_text = if cfg!(feature = "desktop") {
        "Choose Where to Save"
    } else {
        "Save to Device"
    };

    rsx! {
        div { class: "mt-6 p-6 bg-background-card rounded-lg border border-accent-green",
            p { class: "text-accent-green font-medium mb-4", "✓ Your file is ready to download!" }

            // Separate components for the two format types
            match format_type() {
                FormatType::Video => rsx! {
                    p { class: "text-text-secondary mb-4",
                        "File format: "
                        span { class: "font-bold text-accent-teal", "Video (MP4)" }
                    }
                },
                FormatType::Audio => rsx! {
                    p { class: "text-text-secondary mb-4",
                        "File format: "
                        span { class: "font-bold text-accent-amber", "Audio (MP3)" }
                    }
                },
            }

            div { class: "text-center",
                button {
                    class: "inline-block w-full sm:w-auto px-6 py-3 bg-accent-green bg-opacity-80 hover:bg-opacity-100 rounded-lg font-medium text-text-primary transition-colors duration-200 shadow-sm",
                    onclick: move |_| on_save_click.call(()),
                    "{save_button_text}"
                }
            }
        }
    }
}
