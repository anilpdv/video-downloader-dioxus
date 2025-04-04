// Desktop-specific implementations for the download view
use crate::views::download::types::FormatType;
use dioxus::prelude::*;

/// Save downloaded data to disk
///
/// This implementation:
/// 1. Tries to save to the user's Downloads folder
/// 2. Falls back to a temporary directory if Downloads isn't available
/// 3. Returns a result with the path of the saved file or an error message
pub fn save_to_disk(
    data: &[u8],
    filename: &str,
    status_signal: &Signal<Option<String>>,
    error_signal: &Signal<Option<String>>,
) -> Result<String, String> {
    // Clone signals to make them mutable
    let mut status_signal = status_signal.clone();
    let mut error_signal = error_signal.clone();

    // Try to get the user's Downloads folder
    let home_dir = dirs::home_dir();
    let downloads_dir = home_dir.and_then(|dir| Some(dir.join("Downloads")));

    // If we have a Downloads folder, save there
    if let Some(downloads_path) = downloads_dir {
        // Create the full path with filename
        let file_path = downloads_path.join(filename);

        // Try to save the file directly
        match std::fs::write(&file_path, data) {
            Ok(_) => {
                let path_str = file_path.to_string_lossy().to_string();
                status_signal.set(Some(format!("File saved successfully to {}", path_str)));
                error_signal.set(None);
                Ok(path_str)
            }
            Err(e) => {
                let error_msg = format!("Failed to save file: {}", e);
                error_signal.set(Some(error_msg.clone()));
                status_signal.set(Some(
                    "Error saving file. Check permissions and try again.".to_string(),
                ));
                Err(error_msg)
            }
        }
    } else {
        // Fall back to a temporary directory if we can't find Downloads
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(filename);

        match std::fs::write(&file_path, data) {
            Ok(_) => {
                let path_str = file_path.to_string_lossy().to_string();
                status_signal.set(Some(format!(
                    "File saved to temporary location: {}",
                    path_str
                )));
                error_signal.set(None);
                Ok(path_str)
            }
            Err(e) => {
                let error_msg = format!("Failed to save file: {}", e);
                error_signal.set(Some(error_msg.clone()));
                status_signal.set(Some(
                    "Error saving file. Check permissions and try again.".to_string(),
                ));
                Err(error_msg)
            }
        }
    }
}
