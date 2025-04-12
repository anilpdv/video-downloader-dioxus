#[cfg(feature = "server")]
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tracing;

#[cfg(all(feature = "server", feature = "vlc"))]
use std::sync::mpsc::channel;
#[cfg(all(feature = "server", feature = "vlc"))]
use vlc::{Instance, Media, MediaPlayer};

/// Represents available media player types
#[cfg(feature = "server")]
pub enum PlayerType {
    /// External VLC player installed on the system
    ExternalVlc,
    /// Embedded VLC player using vlc-rs
    EmbeddedVlc,
    /// Web-based player (for smaller files)
    WebPlayer,
}

/// Media player interface for playing downloaded content
#[cfg(feature = "server")]
pub struct MediaPlayer;

#[cfg(feature = "server")]
impl MediaPlayer {
    /// Create a new media player instance
    pub fn new() -> Self {
        Self {}
    }

    /// Play a media file
    pub fn play_media(&self, file_path: &Path) -> Result<(), String> {
        // First try to use VLC-RS directly
        #[cfg(feature = "vlc")]
        {
            if let Ok(()) = self.play_with_vlc_rs(file_path) {
                return Ok(());
            }
        }

        // If VLC-RS isn't available or fails, try using external VLC
        if self.is_vlc_installed() {
            return self.play_with_external_vlc(file_path);
        }

        // Finally fall back to system default player
        let file_path_str = file_path.to_string_lossy();
        tracing::info!("Opening with system default player: {}", file_path_str);

        match open::that(file_path) {
            Ok(_) => {
                tracing::info!("Successfully opened with system default player");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to open with system default player: {}", e);
                Err(format!("Failed to open with system default player: {}", e))
            }
        }
    }

    /// Check if VLC is installed on the system
    fn is_vlc_installed(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            // Check common installation paths on Windows
            let program_files =
                std::env::var("ProgramFiles").unwrap_or("C:\\Program Files".to_string());
            let program_files_x86 =
                std::env::var("ProgramFiles(x86)").unwrap_or("C:\\Program Files (x86)".to_string());

            let possible_paths = [
                format!("{}\\VideoLAN\\VLC\\vlc.exe", program_files),
                format!("{}\\VideoLAN\\VLC\\vlc.exe", program_files_x86),
            ];

            for path in possible_paths {
                if Path::new(&path).exists() {
                    return true;
                }
            }

            // Try using where command
            match Command::new("where").arg("vlc.exe").output() {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Check common installation paths on macOS
            let possible_paths = ["/Applications/VLC.app/Contents/MacOS/VLC"];

            for path in possible_paths {
                if Path::new(path).exists() {
                    return true;
                }
            }

            // Try using which command
            match Command::new("which").arg("vlc").output() {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Try using which command on Linux
            match Command::new("which").arg("vlc").output() {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            false
        }
    }

    /// Play media using external VLC application
    fn play_with_external_vlc(&self, file_path: &Path) -> Result<(), String> {
        let file_path_str = file_path.to_string_lossy();

        #[cfg(target_os = "windows")]
        {
            // Find vlc.exe path
            let program_files =
                std::env::var("ProgramFiles").unwrap_or("C:\\Program Files".to_string());
            let program_files_x86 =
                std::env::var("ProgramFiles(x86)").unwrap_or("C:\\Program Files (x86)".to_string());

            let possible_paths = [
                format!("{}\\VideoLAN\\VLC\\vlc.exe", program_files),
                format!("{}\\VideoLAN\\VLC\\vlc.exe", program_files_x86),
                "vlc.exe".to_string(),
            ];

            let vlc_path = possible_paths
                .iter()
                .find(|&path| Path::new(path).exists() || path == "vlc.exe")
                .ok_or_else(|| "Could not find VLC executable".to_string())?;

            tracing::info!("Playing media with external VLC: {}", file_path_str);

            match open::with(vlc_path, file_path_str.to_string()) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to open VLC: {}", e)),
            }
        }

        #[cfg(target_os = "macos")]
        {
            tracing::info!("Playing media with external VLC: {}", file_path_str);

            match open::with("VLC", file_path_str.to_string()) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to open VLC: {}", e)),
            }
        }

        #[cfg(target_os = "linux")]
        {
            tracing::info!("Playing media with external VLC: {}", file_path_str);

            match open::with("vlc", file_path_str.to_string()) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to open VLC: {}", e)),
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err("Unsupported platform".to_string())
        }
    }

    /// Play media using VLC-RS (similar to the example provided)
    #[cfg(feature = "vlc")]
    fn play_with_vlc_rs(&self, file_path: &Path) -> Result<(), String> {
        let file_path_str = file_path.to_string_lossy();
        tracing::info!("Attempting to play with embedded vlc-rs: {}", file_path_str);

        // Create the VLC instance
        let instance = match Instance::new() {
            Some(instance) => instance,
            None => {
                tracing::error!("Failed to create VLC instance");
                return Err("Failed to create VLC instance".to_string());
            }
        };

        // Create media from path
        let media = match Media::new_path(&instance, file_path) {
            Some(media) => media,
            None => {
                tracing::error!("Failed to create media from path: {}", file_path_str);
                return Err("Failed to create media".to_string());
            }
        };

        // Create media player
        let media_player = match MediaPlayer::new(&instance) {
            Some(player) => player,
            None => {
                tracing::error!("Failed to create media player");
                return Err("Failed to create media player".to_string());
            }
        };

        // Create channel for event handling
        let (tx, rx) = channel::<()>();

        // Attach event manager to handle completion
        let event_manager = media.event_manager();
        if let Err(e) = event_manager.attach(vlc::EventType::MediaStateChanged, move |e, _| {
            if let vlc::Event::MediaStateChanged(s) = e {
                tracing::info!("VLC media state: {:?}", s);
                if s == vlc::State::Ended || s == vlc::State::Error {
                    let _ = tx.send(());
                }
            }
        }) {
            tracing::warn!("Failed to attach event handler: {:?}", e);
        }

        // Set the media and play
        media_player.set_media(&media);

        // Play the media
        match media_player.play() {
            Ok(_) => {
                tracing::info!("Started playback with vlc-rs successfully");

                // Don't wait for completion - just return success
                // This allows the UI to remain responsive

                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to play media with vlc-rs: {:?}", e);
                Err(format!("Failed to play media: {:?}", e))
            }
        }
    }

    /// Get info about what player is being used
    pub fn get_player_info(&self) -> String {
        #[cfg(feature = "vlc")]
        {
            if Instance::new().is_some() {
                return "Embedded VLC".to_string();
            }
        }

        if self.is_vlc_installed() {
            "External VLC".to_string()
        } else {
            "System Default".to_string()
        }
    }
}
