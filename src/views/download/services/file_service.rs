use crate::views::download::platforms::trigger_download;
use crate::views::download::types::FormatType;
use dioxus::prelude::*;

// Define platform-specific download handlers
pub fn handle_download_file(
    blob_url: Option<String>,
    download_data: Option<Vec<u8>>,
    filename: &str,
    format_type: &FormatType,
) -> Box<dyn Fn() + 'static> {
    let download_filename = format!("{}.{}", filename, format_type.get_extension());

    #[cfg(feature = "web")]
    {
        let url_clone = blob_url.clone();
        let filename_clone = download_filename.clone();
        return Box::new(move || {
            if let Some(url) = &url_clone {
                trigger_download(url, &filename_clone);
            }
        });
    }

    #[cfg(feature = "desktop")]
    {
        use crate::views::download::platforms::save_to_disk;
        let data_clone = download_data.clone();
        let filename_clone = download_filename.clone();
        return Box::new(move || {
            if let Some(data) = &data_clone {
                let mut status = Signal::new(None::<String>);
                let mut error = Signal::new(None::<String>);
                let _ = save_to_disk(data, &filename_clone, &status, &error);
            }
        });
    }

    #[cfg(not(any(feature = "web", feature = "desktop")))]
    {
        let url_clone = blob_url.clone();
        let filename_clone = download_filename.clone();
        return Box::new(move || {
            if let Some(url) = &url_clone {
                trigger_download(url, &filename_clone);
            }
        });
    }
}
