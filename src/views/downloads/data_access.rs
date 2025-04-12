use dioxus::prelude::{server_fn::error::NoCustomError, ServerFnError};

use super::types::DownloadItem;
use crate::{
    components::download_progress::{DownloadInfo, DownloadStatus},
    server::DeleteDownload,
};

#[cfg(not(feature = "web"))]
pub async fn fetch_downloads() -> Vec<DownloadItem> {
    use crate::server::core::fetch_downloads;

    #[cfg(feature = "server")]
    {
        fetch_downloads().await
    }
    #[cfg(not(feature = "server"))]
    {
        Vec::new()
    }
}

#[cfg(feature = "web")]
pub async fn fetch_downloads() -> Vec<DownloadItem> {
    Vec::new()
}

#[cfg(not(feature = "web"))]
pub fn open_file(path: &str) {
    #[cfg(feature = "server")]
    {
        use crate::views::downloads::media_player;
        media_player::open_with_best_player(path);
    }
}

#[cfg(feature = "web")]
pub fn open_file(path: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.open_with_url(path);
    }
}

#[cfg(not(feature = "web"))]
pub fn open_containing_folder(path: &str) {
    #[cfg(feature = "server")]
    {
        let _ = crate::server::download::platform::open_containing_folder(path);
    }
}

#[cfg(feature = "web")]
pub fn open_containing_folder(_: &str) {}

#[cfg(feature = "web")]
pub async fn download_with_progress<F>(
    url: &str,
    file_name: &str,
    on_progress: F,
) -> Result<String, String>
where
    F: Fn(DownloadInfo) + 'static,
{
    crate::server::download::web_services::download_with_progress_real(url, file_name, on_progress)
        .await
}

#[cfg(not(feature = "web"))]
pub async fn download_with_progress<F>(
    url: &str,
    file_name: &str,
    on_progress: F,
) -> Result<String, String>
where
    F: Fn(DownloadInfo) + 'static,
{
    #[cfg(feature = "server")]
    {
        let mut download_info = DownloadInfo {
            url: url.to_string(),
            file_name: file_name.to_string(),
            status: DownloadStatus::Downloading,
            ..Default::default()
        };

        use crate::server::download::api::video;

        on_progress(download_info.clone());

        let result = video::download_video(url.to_string())
            .await
            .map_err(|e| e.to_string());

        match result {
            Ok(_) => {
                download_info.status = DownloadStatus::Completed;
                download_info.blob_url = Some(format!("/downloads/{}", file_name));
                on_progress(download_info.clone());
                Ok(format!("/downloads/{}", file_name))
            }
            Err(err) => {
                download_info.status = DownloadStatus::Failed(err.clone());
                on_progress(download_info);
                Err(err)
            }
        }
    }

    #[cfg(not(feature = "server"))]
    {
        Err("Download not supported on this platform".to_string())
    }
}

pub fn get_media_url(path: &str) -> String {
    use crate::views::downloads::media_player;
    media_player::get_optimal_media_url(path)
}
