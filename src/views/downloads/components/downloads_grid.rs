use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::{BsExclamationTriangleFill, BsSearch};
use dioxus_free_icons::icons::fa_solid_icons::{FaMusic, FaTrash, FaVideo};
use dioxus_free_icons::icons::hi_outline_icons::{HiFilm, HiMusicNote, HiViewGrid};
use dioxus_free_icons::Icon;

use super::download_card::DownloadCard;
use super::loading_spinner::LoadingSpinner;
use crate::server::delete_download;
use crate::views::downloads::data_access;
use crate::views::downloads::types::DownloadItem;

#[component]
pub fn DownloadsGrid(
    downloads: Signal<Vec<DownloadItem>>,
    active_tab: Signal<String>,
    search_query: Signal<String>,
) -> Element {
    let filtered_downloads = {
        let query = search_query().to_lowercase();

        let tab_filtered = if active_tab() == "all" {
            downloads().clone()
        } else {
            downloads()
                .iter()
                .filter(|d| d.format_type == active_tab())
                .cloned()
                .collect()
        };

        if query.is_empty() {
            tab_filtered
        } else {
            tab_filtered
                .into_iter()
                .filter(|d| {
                    d.title.to_lowercase().contains(&query)
                        || d.filename.to_lowercase().contains(&query)
                })
                .collect::<Vec<DownloadItem>>()
        }
    };

    let audio_count = downloads()
        .iter()
        .filter(|d| d.format_type == "audio")
        .count();
    let video_count = downloads()
        .iter()
        .filter(|d| d.format_type == "video")
        .count();
    let total_count = audio_count + video_count;

    // Add state for deletion
    let mut is_deleting = use_signal(|| false);
    let mut deletion_error = use_signal(|| None::<String>);

    // Handle delete event
    let handle_delete = move |id: i64| {
        tracing::info!("Handling delete for ID: {}", id);

        is_deleting.set(true);
        deletion_error.set(None);

        let mut downloads_clone = downloads.clone();
        let mut is_deleting_clone = is_deleting.clone();
        let mut deletion_error_clone = deletion_error.clone();

        spawn(async move {
            match delete_download(id).await {
                Ok(true) => {
                    tracing::info!("Successfully deleted download with ID: {}", id);
                    downloads_clone.with_mut(|list| {
                        list.retain(|item| item.id != Some(id));
                    });
                }
                Ok(false) => {
                    tracing::warn!("Delete operation failed, item not found: {}", id);
                    deletion_error_clone.set(Some("Item not found or already deleted".to_string()));
                }
                Err(e) => {
                    tracing::error!("Error deleting download: {}", e);
                    deletion_error_clone.set(Some(format!("Error deleting: {}", e)));
                }
            }
            is_deleting_clone.set(false);
        });
    };

    rsx! {
        div { class: "mb-6 relative",
            div { class: "relative",
                span { class: "absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none",
                    Icon {
                        icon: BsSearch,
                        width: 16,
                        height: 16,
                        class: "text-text-muted",
                    }
                }
                input {
                    class: "bg-background-card border border-border text-text-primary text-sm rounded-lg focus:ring-accent-teal focus:border-accent-teal block w-full pl-10 p-2.5",
                    r#type: "text",
                    placeholder: "Search downloads...",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value().clone()),
                }
            }
        }
        // Error message for deletion
        if let Some(error) = deletion_error() {
            div { class: "mb-4 p-4 bg-background-card border border-accent-rose rounded-lg shadow-md",
                div { class: "flex items-start justify-between",
                    div { class: "flex items-start",
                        Icon {
                            icon: BsExclamationTriangleFill,
                            width: 16,
                            height: 16,
                            class: "text-accent-rose mt-0.5 mr-2 flex-shrink-0",
                        }
                        div { class: "text-sm text-text-primary", "{error}" }
                    }
                    button {
                        class: "text-text-muted hover:text-text-primary ml-4",
                        onclick: move |_| deletion_error.set(None),
                        "×"
                    }
                }
            }
        }

        div { class: "mb-6 border-b border-border",
            div { class: "flex flex-wrap -mb-px",
                button {
                    class: if active_tab() == "all" { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-accent-teal border-b-2 border-accent-teal rounded-t-lg" } else { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-text-muted border-b-2 border-transparent hover:text-text-secondary hover:border-border rounded-t-lg" },
                    onclick: move |_| active_tab.set("all".to_string()),
                    Icon {
                        icon: HiViewGrid,
                        width: 18,
                        height: 18,
                        class: "mr-2",
                    }
                    "All ({total_count})"
                }

                button {
                    class: if active_tab() == "audio" { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-accent-teal border-b-2 border-accent-teal rounded-t-lg" } else { "inline-flex items-center py-3 px-4 mr-4 text-sm font-medium text-text-muted border-b-2 border-transparent hover:text-text-secondary hover:border-border rounded-t-lg" },
                    onclick: move |_| active_tab.set("audio".to_string()),
                    Icon {
                        icon: HiMusicNote,
                        width: 18,
                        height: 18,
                        class: "mr-2",
                    }
                    "Audio ({audio_count})"
                }

                button {
                    class: if active_tab() == "video" { "inline-flex items-center py-3 px-4 text-sm font-medium text-accent-teal border-b-2 border-accent-teal rounded-t-lg" } else { "inline-flex items-center py-3 px-4 text-sm font-medium text-text-muted border-b-2 border-transparent hover:text-text-secondary hover:border-border rounded-t-lg" },
                    onclick: move |_| active_tab.set("video".to_string()),
                    Icon {
                        icon: HiFilm,
                        width: 18,
                        height: 18,
                        class: "mr-2",
                    }
                    "Video ({video_count})"
                }
            }
        }
        // Loading overlay for deletion
        if is_deleting() {
            div { class: "fixed inset-0 z-50 bg-background-darker bg-opacity-50 backdrop-blur-sm flex items-center justify-center",
                div { class: "bg-background-card p-4 rounded-lg shadow-lg border border-border flex items-center",
                    div { class: "animate-spin w-5 h-5 border-2 border-accent-teal border-t-transparent rounded-full mr-3" }
                    p { class: "text-text-primary text-sm", "Deleting..." }
                }
            }
        }

        if filtered_downloads.is_empty() {
            div { class: "text-center py-12 bg-background-card rounded-xl border border-border shadow-md",
                if !search_query().is_empty() {
                    div { class: "flex flex-col items-center",
                        Icon {
                            icon: BsSearch,
                            width: 40,
                            height: 40,
                            class: "text-text-muted mb-4",
                        }
                        p { class: "text-lg font-medium text-text-primary",
                            "No results found for \"{search_query()}\""
                        }
                        p { class: "text-text-secondary mt-2",
                            "Try different keywords or clear your search"
                        }
                        button {
                            class: "mt-4 px-4 py-2 bg-accent-teal text-text-primary rounded-lg text-sm hover:bg-opacity-80 transition-colors",
                            onclick: move |_| search_query.set(String::new()),
                            "Clear Search"
                        }
                    }
                } else {
                    div { class: "flex flex-col items-center",
                        if active_tab() == "audio" {
                            Icon {
                                icon: FaMusic,
                                width: 40,
                                height: 40,
                                class: "text-text-muted mb-4",
                            }
                        } else {
                            Icon {
                                icon: FaVideo,
                                width: 40,
                                height: 40,
                                class: "text-text-muted mb-4",
                            }
                        }
                        p { class: "text-lg font-medium text-text-primary",
                            "No {active_tab()} files found"
                        }
                        p { class: "text-text-secondary mt-2",
                            "Try switching to a different category or download some {active_tab()} files."
                        }
                    }
                }
            }
        } else {
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                for download in filtered_downloads {
                    DownloadCard { download: download.clone(), on_delete: handle_delete }
                }
            }
        }
    }
}
