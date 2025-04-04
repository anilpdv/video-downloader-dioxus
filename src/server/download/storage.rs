use std::path::{Path, PathBuf};
use tracing;

#[cfg(feature = "server")]
pub fn ensure_media_directory() -> Option<PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        let media_dir = home_dir
            .join("Documents")
            .join("youtube_downloader")
            .join("media");

        // Create directory with proper permissions
        if let Err(e) = std::fs::create_dir_all(&media_dir) {
            tracing::error!("Failed to create media directory: {}", e);
            return None;
        }

        // Set directory permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) =
                std::fs::set_permissions(&media_dir, std::fs::Permissions::from_mode(0o755))
            {
                tracing::error!("Failed to set media directory permissions: {}", e);
            }
        }

        return Some(media_dir);
    }

    None
}

#[cfg(feature = "server")]
pub fn save_file_with_permissions(path: &Path, content: &[u8]) -> bool {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::error!("Failed to create parent directories: {}", e);
                return false;
            }

            // Set directory permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Err(e) =
                    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o755))
                {
                    tracing::error!("Failed to set directory permissions: {}", e);
                }
            }
        }
    }

    // Write the file
    if let Err(e) = std::fs::write(path, content) {
        tracing::error!("Failed to write file: {}", e);
        return false;
    }

    // Set file permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644)) {
            tracing::error!("Failed to set file permissions: {}", e);
        }
    }

    true
}

#[cfg(feature = "server")]
pub fn create_clean_filename(title: &str, extension: &str) -> String {
    let clean_title = title
        .replace("/", "_")
        .replace("\\", "_")
        .replace(":", "_")
        .replace("*", "_")
        .replace("?", "_")
        .replace("\"", "_")
        .replace("<", "_")
        .replace(">", "_")
        .replace("|", "_");

    if clean_title.is_empty() {
        format!("video.{}", extension)
    } else {
        format!("{}.{}", clean_title, extension)
    }
}
