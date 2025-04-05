use dioxus::prelude::*;
use std::collections::HashMap;

use crate::common::Toaster;
use crate::components::download_progress::DownloadInfo;

use super::components::downloads_grid::DownloadsGrid;
use super::components::loading_spinner::LoadingSpinner;
use super::data_access;
use super::types::DownloadItem;

#[component]
pub fn Downloads() -> Element {
    rsx! {
        div { class: "container mx-auto py-6 px-4",
            h1 { class: "text-3xl font-bold mb-4 text-text-primary", "My Downloads" }
            p { class: "mb-6 text-text-secondary",
                "Access and play your downloaded videos and audio files."
            }
            DownloadsContent {}
        }
    }
}

#[component]
fn DownloadsContent() -> Element {
    let active_tab = use_signal(|| "all".to_string());
    let search_query = use_signal(|| String::new());

    let downloads = use_signal(|| Vec::<DownloadItem>::new());
    let loading = use_signal(|| true);

    let active_downloads = use_signal(|| HashMap::<String, DownloadInfo>::new());
    let _toaster = use_signal(|| None::<Toaster>);

    use_effect(move || {
        if loading() {
            let mut downloads_clone = downloads.clone();
            let mut loading_clone = loading.clone();

            use_future(move || async move {
                let results = data_access::fetch_downloads().await;
                downloads_clone.set(results);
                loading_clone.set(false);
            });
        }
    });

    let _handle_download = move |url: String, filename: String| {
        let downloads_clone = active_downloads.clone();

        use_future(move || {
            let mut downloads_ref = downloads_clone.clone();
            let url_clone = url.clone();
            let filename_clone = filename.clone();

            async move {
                let download_key = format!("{}-{}", url_clone, filename_clone);

                let initial_info = DownloadInfo {
                    url: url_clone.clone(),
                    file_name: filename_clone.clone(),
                    status: crate::components::download_progress::DownloadStatus::NotStarted,
                    ..Default::default()
                };

                downloads_ref
                    .write()
                    .insert(download_key.clone(), initial_info);

                let callback_ref =
                    std::sync::Arc::new(std::sync::Mutex::new(downloads_ref.clone()));
                let key_ref = download_key.clone();

                let progress_callback = move |info: DownloadInfo| {
                    let info_copy = info.clone();
                    let key_clone = key_ref.clone();
                    let callback = callback_ref.clone();

                    use dioxus::prelude::spawn;
                    spawn(async move {
                        if let Ok(mut guard) = callback.lock() {
                            guard.with_mut(|map| {
                                map.insert(key_clone, info_copy);
                            });
                        }
                    });
                };

                let _ = data_access::download_with_progress(
                    &url_clone,
                    &filename_clone,
                    progress_callback,
                )
                .await;
            }
        });
    };

    if loading() {
        return rsx! {
            LoadingSpinner {}
        };
    }

    let has_downloads = !downloads().is_empty();

    #[cfg(not(feature = "web"))]
    {
        if !has_downloads {
            return rsx! {
                div { class: "text-center py-16 bg-background-card rounded-xl border border-border shadow-md",
                    div { class: "flex justify-center mb-6",
                        dioxus_free_icons::Icon {
                            icon: dioxus_free_icons::icons::fa_solid_icons::FaDownload,
                            width: 52,
                            height: 52,
                            class: "text-text-muted",
                        }
                    }
                    p { class: "text-xl font-medium text-text-primary", "No downloads yet" }
                    p { class: "text-text-secondary mt-2 max-w-md mx-auto",
                        "Your downloaded files will appear here. Try downloading a video or audio file from the home page."
                    }
                }
            };
        }

        return rsx! {
            DownloadsGrid {
                downloads: downloads.clone(),
                active_tab: active_tab.clone(),
                search_query: search_query.clone(),
            }
        };
    }
}
