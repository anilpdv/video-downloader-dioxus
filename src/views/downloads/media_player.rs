use std::fs;
use std::path::Path;
use std::process::Command;
use tracing;

#[cfg(feature = "server")]
pub fn open_with_best_player(file_path: &str) -> bool {
    // First try to check if the file exists
    let path = Path::new(file_path);
    if !path.exists() {
        tracing::error!("File does not exist: {}", file_path);
        return false;
    }

    // Log the attempt
    tracing::info!("Opening file with best available player: {}", file_path);

    // Use our new MediaPlayer implementation that has vlc-rs fallback
    let player = crate::server::download::player::MediaPlayer::new();

    match player.play_media(path) {
        Ok(_) => {
            tracing::info!("Successfully opened file with player");
            true
        }
        Err(e) => {
            tracing::error!("Failed to open with player: {}", e);

            // Fall back to system default player if all else fails
            tracing::info!("Player failed, trying system default player");
            match open::that(file_path) {
                Ok(_) => {
                    tracing::info!("Successfully opened file with system default player");
                    true
                }
                Err(e) => {
                    tracing::error!("Failed to open with default player: {}", e);
                    false
                }
            }
        }
    }
}

#[cfg(feature = "server")]
fn open_with_vlc(file_path: &str) -> bool {
    // Different VLC commands for different platforms
    #[cfg(target_os = "macos")]
    {
        // Common VLC paths on macOS
        let vlc_paths = [
            "/Applications/VLC.app/Contents/MacOS/VLC",
            "/Applications/VLC Media Player.app/Contents/MacOS/VLC",
        ];

        for vlc_path in vlc_paths.iter() {
            if Path::new(vlc_path).exists() {
                match std::process::Command::new(vlc_path).arg(file_path).spawn() {
                    Ok(_) => return true,
                    Err(e) => tracing::error!("Failed to start VLC at {}: {}", vlc_path, e),
                }
            }
        }

        // Try using the 'open' command with VLC as the app
        match std::process::Command::new("open")
            .args(["-a", "VLC", file_path])
            .spawn()
        {
            Ok(_) => return true,
            Err(e) => tracing::error!("Failed to open with VLC via 'open' command: {}", e),
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Common VLC paths on Windows
        let vlc_paths = [
            "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",
            "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe",
        ];

        for vlc_path in vlc_paths.iter() {
            if Path::new(vlc_path).exists() {
                match std::process::Command::new(vlc_path).arg(file_path).spawn() {
                    Ok(_) => return true,
                    Err(e) => tracing::error!("Failed to start VLC at {}: {}", vlc_path, e),
                }
            }
        }

        // Try using the vlc command directly (if it's in PATH)
        match std::process::Command::new("vlc").arg(file_path).spawn() {
            Ok(_) => return true,
            Err(e) => tracing::error!("Failed to open with VLC via PATH: {}", e),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try using vlc directly (common on Linux systems)
        match std::process::Command::new("vlc").arg(file_path).spawn() {
            Ok(_) => return true,
            Err(e) => tracing::error!("Failed to open with VLC: {}", e),
        }
    }

    false
}

#[cfg(not(feature = "server"))]
pub fn open_with_best_player(file_path: &str) -> bool {
    // In web environments, try to open in a new tab
    #[cfg(feature = "web")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url(file_path);
            return true;
        }
    }

    false
}

pub fn get_optimal_media_url(file_path: &str) -> String {
    // Simple function to ensure proper URL encoding of spaces and special characters
    let encoded_path = file_path
        .replace(" ", "%20")
        .replace("#", "%23")
        .replace("(", "%28")
        .replace(")", "%29")
        .replace("&", "%26");

    // Simple function to determine if a path is local or remote
    let is_remote = encoded_path.starts_with("http://") || encoded_path.starts_with("https://");

    if is_remote {
        encoded_path
    } else {
        // For local files, add the file:// scheme
        format!("file://{}", encoded_path)
    }
}

/// Check if VLC is available on the system
pub fn is_vlc_available() -> bool {
    #[cfg(not(feature = "server"))]
    return false;

    #[cfg(feature = "server")]
    {
        // Use our new MediaPlayer to check if VLC is available
        let player = crate::server::download::player::MediaPlayer::new();
        let player_info = player.get_player_info();

        // Check if the player info indicates VLC is available
        if player_info == "External VLC" || player_info == "Embedded VLC" {
            tracing::info!("VLC is available: {}", player_info);
            true
        } else {
            tracing::debug!("VLC is not available on this system");
            false
        }
    }
}

/// Get thumbnail image for a media file
pub fn get_media_thumbnail(_file_path: &str) -> Option<String> {
    // In a real implementation, this would generate a thumbnail from the video
    // For this demo, we'll just return None
    None
}

/// Check if a file is a media file
pub fn is_media_file(file_path: &str) -> bool {
    let media_extensions = [
        ".mp4", ".mkv", ".avi", ".mov", ".webm", ".flv", ".wmv", ".mp3", ".wav", ".flac", ".ogg",
        ".m4a",
    ];

    if let Some(ext) = Path::new(file_path).extension() {
        if let Some(ext_str) = ext.to_str() {
            let ext_lower = ext_str.to_lowercase();
            return media_extensions
                .iter()
                .any(|&media_ext| format!(".{}", ext_lower) == media_ext);
        }
    }

    false
}

/// Check if a file is considered a large file (over 50MB)
pub fn is_large_file(file_path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(file_path) {
        return metadata.len() > 50 * 1024 * 1024; // 50MB
    }
    false
}

// Add a dedicated function for opening files in an external player
#[cfg(feature = "server")]
pub fn open_external_player(file_path: &str) -> Result<(), String> {
    // First try to check if the file exists
    let path = Path::new(file_path);
    if !path.exists() {
        let error = format!("File does not exist: {}", file_path);
        tracing::error!("{}", error);
        return Err(error);
    }

    // Log the attempt
    tracing::info!("Opening file with external player: {}", file_path);

    // Try to open with VLC first if available
    if is_vlc_available() {
        if open_with_vlc(file_path) {
            tracing::info!("Successfully opened file with VLC");
            return Ok(());
        }
    }

    // Fall back to system default player if VLC fails or isn't available
    tracing::info!("Trying system default player");
    match open::that(file_path) {
        Ok(_) => {
            tracing::info!("Successfully opened file with system default player");
            Ok(())
        }
        Err(e) => {
            let error = format!("Failed to open with default player: {}", e);
            tracing::error!("{}", error);
            Err(error)
        }
    }
}

#[cfg(not(feature = "server"))]
pub fn open_external_player(file_path: &str) -> Result<(), String> {
    // In web environments, try to open in a new tab
    #[cfg(feature = "web")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url(file_path);
            return Ok(());
        }
    }

    Err("Cannot open external player in this environment".to_string())
}
