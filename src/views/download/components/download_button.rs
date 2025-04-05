use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_solid_icons::FaDownload, Icon};

#[component]
pub fn DownloadButton(
    loading: Signal<bool>,
    is_url_valid: bool,
    download_ready: Signal<bool>,
    onclick: EventHandler<()>,
) -> Element {
    let get_button_class = move || {
        if loading() {
            "w-full text-text-invert bg-accent-teal cursor-not-allowed font-medium rounded-lg text-sm px-5 py-3 text-center shadow-sm"
        } else if !is_url_valid {
            "w-full text-text-muted bg-background-medium cursor-not-allowed rounded-lg text-sm px-5 py-3 text-center border border-border"
        } else if download_ready() {
            "w-full text-text-invert bg-accent-green hover:bg-opacity-80 font-medium rounded-lg text-sm px-5 py-3 text-center transition-colors shadow-sm"
        } else {
            "w-full text-text-invert bg-accent-teal hover:bg-opacity-80 font-medium rounded-lg text-sm px-5 py-3 text-center transition-colors shadow-sm"
        }
    };

    let render_button_content = move || {
        if loading() {
            rsx! {
                span { class: "flex items-center justify-center",
                    svg {
                        class: "animate-spin -ml-1 mr-3 h-5 w-5 text-text-invert",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        circle {
                            class: "opacity-25",
                            cx: "12",
                            cy: "12",
                            r: "10",
                            stroke: "currentColor",
                            stroke_width: "4",
                        }
                        path {
                            class: "opacity-75",
                            fill: "currentColor",
                            d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                        }
                    }
                    "Processing..."
                }
            }
        } else if download_ready() {
            rsx! {
                span { "Download Ready" }
            }
        } else {
            rsx! {
                span { class: "flex items-center justify-center",
                    Icon {
                        icon: FaDownload,
                        width: 16,
                        height: 16,
                        class: "mr-2",
                    }
                    "Download Now"
                }
            }
        }
    };

    rsx! {
        button {
            class: get_button_class(),
            disabled: loading() || !is_url_valid,
            onclick: move |_| onclick.call(()),
            {render_button_content()}
        }
    }
}
