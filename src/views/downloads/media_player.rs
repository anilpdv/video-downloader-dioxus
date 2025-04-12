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
    tracing::info!("Opening file with VLC player: {}", file_path);

    // Determine if this is a large file
    let is_large_file = match path.metadata() {
        Ok(metadata) => metadata.len() > 50 * 1024 * 1024, // 50MB
        Err(_) => false,
    };

    // If it's a large file, log that info
    if is_large_file {
        tracing::info!(
            "Large file detected ({}MB)",
            match path.metadata() {
                Ok(metadata) => metadata.len() / (1024 * 1024),
                Err(_) => 0,
            }
        );
    }

    // Try to open with VLC player
    if open_with_vlc(file_path) {
        tracing::info!("Successfully opened file with VLC");
        return true;
    }

    // Fall back to system default player if VLC fails
    tracing::info!("VLC failed, trying system default player");
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
        // Try to actually create a VLC instance with different options
        #[cfg(vlc_linking)]
        {
            // Try different argument sets
            let arg_sets = [
                vec!["--no-video-title-show", "--quiet", "--no-xlib"],
                vec!["--quiet", "--no-xlib"],
                vec!["--no-video-title-show"],
                vec![], // Try with no args as last resort
            ];

            for args in &arg_sets {
                tracing::debug!("Checking VLC availability with args: {:?}", args);

                let result = if args.is_empty() {
                    vlc::Instance::new()
                } else {
                    vlc::Instance::with_args(args.as_slice())
                };

                if let Ok(_) = result {
                    tracing::info!(
                        "VLC is available - successfully created instance with args: {:?}",
                        args
                    );
                    return true;
                }
            }

            tracing::warn!("Failed to create VLC instance with any argument set");
        }

        // Fallback to checking for the VLC executable
        #[cfg(target_os = "macos")]
        {
            let possible_paths = [
                "/Applications/VLC.app/Contents/MacOS/VLC",
                "/Applications/VLC Media Player.app/Contents/MacOS/VLC",
            ];
            for path in possible_paths {
                if Path::new(path).exists() {
                    tracing::info!("VLC binary found at: {}", path);
                    return true;
                }
            }
            tracing::warn!("VLC binary not found in common macOS locations");
        }

        #[cfg(target_os = "windows")]
        {
            let possible_paths = [
                "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",
                "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe",
            ];
            for path in possible_paths {
                if Path::new(path).exists() {
                    tracing::info!("VLC binary found at: {}", path);
                    return true;
                }
            }
            tracing::warn!("VLC binary not found in common Windows locations");
        }

        #[cfg(target_os = "linux")]
        {
            // Try to find vlc in PATH
            if let Ok(output) = std::process::Command::new("which").arg("vlc").output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    tracing::info!("VLC binary found at: {}", path);
                    return true;
                }
            }
            tracing::warn!("VLC binary not found in PATH");
        }

        tracing::debug!("VLC is not available on this system");
        false
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
