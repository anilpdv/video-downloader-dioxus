use dioxus::prelude::*;

#[component]
pub fn LoadingSpinner() -> Element {
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
