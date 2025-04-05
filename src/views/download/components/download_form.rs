use crate::views::download::types::{FormatType, Quality};
use dioxus::prelude::*;

#[component]
pub fn DownloadForm(
    url: Signal<String>,
    filename: Signal<String>,
    format_type: Signal<FormatType>,
    quality: Signal<Quality>,
    loading: Signal<bool>,
    on_format_change: EventHandler<FormatType>,
) -> Element {
    rsx! {
        // Format Selection Buttons
        div { class: "mb-6",
            label { class: "block mb-2 text-sm font-medium text-text-primary", "Download Format" }
            div { class: "grid grid-cols-2 gap-2",
                // Audio option
                button {
                    key: "audio",
                    class: if format_type() == FormatType::Audio { "bg-accent-amber bg-opacity-20 text-accent-amber border border-accent-amber text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                    onclick: move |_| on_format_change.call(FormatType::Audio),
                    disabled: loading(),
                    "🎵 Audio (MP3)"
                }
                // Video option
                button {
                    key: "video",
                    class: if format_type() == FormatType::Video { "bg-accent-teal bg-opacity-20 text-accent-teal border border-accent-teal text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                    onclick: move |_| on_format_change.call(FormatType::Video),
                    disabled: loading(),
                    "🎬 Video (MP4)"
                }
            }
        }

        // Quality selection
        div { class: "mb-6",
            label { class: "block mb-2 text-sm font-medium text-text-primary",
                if format_type() == FormatType::Audio {
                    "Audio Quality"
                } else {
                    "Video Quality"
                }
            }
            div { class: "grid grid-cols-3 gap-2",
                button {
                    class: if quality() == Quality::Highest { "bg-accent-teal bg-opacity-20 text-accent-teal border border-accent-teal text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                    onclick: move |_| quality.set(Quality::Highest),
                    disabled: loading(),
                    "High"
                }
                button {
                    class: if quality() == Quality::Medium { "bg-accent-teal bg-opacity-20 text-accent-teal border border-accent-teal text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                    onclick: move |_| quality.set(Quality::Medium),
                    disabled: loading(),
                    "Medium"
                }
                button {
                    class: if quality() == Quality::Lowest { "bg-accent-teal bg-opacity-20 text-accent-teal border border-accent-teal text-sm rounded-lg px-4 py-2.5 focus:outline-none" } else { "bg-background-medium hover:bg-background-hover text-text-primary border border-border text-sm rounded-lg px-4 py-2.5 focus:outline-none" },
                    onclick: move |_| quality.set(Quality::Lowest),
                    disabled: loading(),
                    "Low"
                }
            }
        }

        // URL input group
        div { class: "mb-6",
            label { class: "block mb-2 text-sm font-medium text-text-primary", "Video URL" }
            div { class: "flex",
                input {
                    class: "flex-1 bg-background-medium border border-border text-text-primary text-sm rounded-l-lg focus:ring-accent-teal focus:border-accent-teal block w-full p-2.5",
                    r#type: "text",
                    placeholder: "Enter video URL (YouTube, Vimeo, etc.)",
                    value: "{url}",
                    oninput: move |e| url.set(e.value().clone()),
                    disabled: loading(),
                }
                button {
                    class: "bg-background-medium hover:bg-background-hover text-text-primary border border-l-0 border-border font-medium rounded-r-lg text-sm px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-accent-teal",
                    r#type: "button",
                    onclick: move |_| {
                        url.set("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string());
                    },
                    disabled: loading(),
                    "Paste"
                }
            }
        }

        // Filename input
        div { class: "mb-6",
            label { class: "block mb-2 text-sm font-medium text-text-primary",
                "Custom filename (optional)"
            }
            input {
                class: "bg-background-medium border border-border text-text-primary text-sm rounded-lg focus:ring-accent-teal focus:border-accent-teal block w-full p-2.5",
                r#type: "text",
                placeholder: "Enter custom filename (without extension)",
                value: "{filename}",
                oninput: move |e| filename.set(e.value().clone()),
                disabled: loading(),
            }
        }
    }
}
