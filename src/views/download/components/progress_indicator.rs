use dioxus::prelude::*;

#[component]
pub fn ProgressIndicator(
    loading: Signal<bool>,
    progress_percent: Signal<i32>,
    progress_eta: Signal<String>,
    status: Signal<Option<String>>,
) -> Element {
    if !loading() || progress_percent() <= 0 {
        return rsx! {
            div {}
        };
    }

    // Get status message for display
    let status_text = match status() {
        Some(stat) => {
            // If status contains "Downloading", make sure we show the percentage
            if stat.contains("Downloading") {
                format!("{}", stat)
            } else {
                // Otherwise just show the status message
                stat
            }
        }
        None => "Downloading...".to_string(),
    };

    let eta_section = if !progress_eta().is_empty() {
        rsx! {
            div { class: "mt-1 text-sm text-text-muted flex justify-between",
                span { "Estimated time: {progress_eta()}" }
            }
        }
    } else {
        rsx! {}
    };

    rsx! {
        div { class: "mt-4",
            div { class: "mb-2 flex justify-between",
                span { class: "text-text-secondary", "{status_text}" }
                span { class: "text-text-secondary", "{progress_percent()}%" }
            }
            div { class: "w-full bg-background-medium rounded-full h-2.5",
                div {
                    class: "bg-accent-teal h-2.5 rounded-full transition-all duration-1000 ease-in-out",
                    style: "width: {progress_percent()}%",
                }
            }
            {eta_section}
        }
    }
}
