use std::path::Path;

// Player status structure for tracking VLC playback
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerStatus {
    pub is_playing: bool,
    pub position: f32, // 0.0 to 1.0
    pub duration_ms: i64,
    pub time_ms: i64,
    pub volume: i32, // 0 to 100
    pub state: String,
}

#[cfg(feature = "server")]
pub use vlc_player_impl::*;

#[cfg(not(feature = "server"))]
pub mod vlc_player_impl {
    use super::PlayerStatus;

    pub struct VlcPlayer;

    impl VlcPlayer {
        pub fn new(_path: &str) -> Result<Self, String> {
            Err("VLC player is not supported in this environment".to_string())
        }

        pub fn play(&mut self) -> Result<(), String> {
            Err("VLC player is not supported in this environment".to_string())
        }

        pub fn pause(&mut self) -> Result<(), String> {
            Err("VLC player is not supported in this environment".to_string())
        }

        pub fn toggle_play(&mut self) -> Result<(), String> {
            Err("VLC player is not supported in this environment".to_string())
        }

        pub fn stop(&mut self) -> Result<(), String> {
            Err("VLC player is not supported in this environment".to_string())
        }

        pub fn set_position(&mut self, _pos: f32) -> Result<(), String> {
            Err("VLC player is not supported in this environment".to_string())
        }

        pub fn set_volume(&mut self, _volume: i32) -> Result<(), String> {
            Err("VLC player is not supported in this environment".to_string())
        }

        pub fn get_status(&self) -> PlayerStatus {
            PlayerStatus {
                is_playing: false,
                position: 0.0,
                duration_ms: 0,
                time_ms: 0,
                volume: 0,
                state: "Unsupported".to_string(),
            }
        }
    }
}

#[cfg(feature = "server")]
mod vlc_player_impl {
    use super::PlayerStatus;
    use std::path::Path;
    use std::process::{Child, Command};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    pub struct VlcPlayer {
        // External player process
        process: Option<Child>,
        // State tracking for the player
        status: Arc<Mutex<PlayerStatus>>,
        // File being played
        file_path: String,
        // Start time
        start_time: Instant,
        // Manually tracked position (0.0 - 1.0)
        position: f32,
        // Duration in ms (estimated)
        duration_ms: i64,
    }

    impl VlcPlayer {
        pub fn new(path: &str) -> Result<Self, String> {
            tracing::info!("Creating external VLC player for: {}", path);

            // Get duration estimate from file metadata if possible
            let duration_ms = match Path::new(path).metadata() {
                Ok(metadata) => {
                    // Rough estimate - 1MB = 10 seconds for video
                    let size_mb = metadata.len() / (1024 * 1024);
                    (size_mb * 10 * 1000) as i64
                }
                Err(_) => 0, // Unknown
            };

            // Create empty player with initial status
            let player = VlcPlayer {
                process: None,
                status: Arc::new(Mutex::new(PlayerStatus {
                    is_playing: false,
                    position: 0.0,
                    duration_ms,
                    time_ms: 0,
                    volume: 80,
                    state: "Ready".to_string(),
                })),
                file_path: path.to_string(),
                start_time: Instant::now(),
                position: 0.0,
                duration_ms,
            };

            Ok(player)
        }

        // We need a helper method to safely try_wait on a process
        fn try_wait_process(
            process: &mut Child,
        ) -> Result<Option<std::process::ExitStatus>, std::io::Error> {
            process.try_wait()
        }

        // Immutable version that clones the process status
        fn check_process_running(process: &Child) -> bool {
            // We need to use try_wait() but it requires a mutable reference
            // In this context we can't mutate the process, so we'll just
            // check if the process is still alive by another method
            #[cfg(target_os = "unix")]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Ok(status) = std::process::Command::new("ps")
                    .arg("-p")
                    .arg(process.id().to_string())
                    .output()
                {
                    // If ps shows the process is running
                    return status.status.success();
                }
            }

            #[cfg(target_os = "windows")]
            {
                // On Windows we'll assume it's still running
                // since we can't check without a mutable reference
                true
            }

            #[cfg(not(any(target_os = "unix", target_os = "windows")))]
            {
                // Default for other platforms
                true
            }
        }

        // Start playback using external VLC process
        pub fn play(&mut self) -> Result<(), String> {
            // Don't start if already playing
            if let Some(process) = &mut self.process {
                if let Ok(None) = Self::try_wait_process(process) {
                    // Process is still running
                    if let Ok(mut status) = self.status.lock() {
                        status.is_playing = true;
                        status.state = "Playing".to_string();
                    }
                    return Ok(());
                }
            }

            // Start VLC based on platform
            #[cfg(target_os = "macos")]
            let process_result = {
                // Common VLC paths on macOS
                let vlc_paths = [
                    "/Applications/VLC.app/Contents/MacOS/VLC",
                    "/Applications/VLC Media Player.app/Contents/MacOS/VLC",
                ];

                let mut result = None;

                for &vlc_path in &vlc_paths {
                    if Path::new(vlc_path).exists() {
                        match Command::new(vlc_path)
                            .arg("--play-and-exit")
                            .arg("--quiet")
                            .arg(&self.file_path)
                            .spawn()
                        {
                            Ok(child) => {
                                result = Some(Ok(child));
                                break;
                            }
                            Err(e) => {
                                result = Some(Err(format!("Failed to start VLC: {}", e)));
                            }
                        }
                    }
                }

                // If we haven't found a VLC path or starting it failed, try with 'open'
                if result.is_none() {
                    match Command::new("open")
                        .args(["-a", "VLC", &self.file_path])
                        .spawn()
                    {
                        Ok(child) => result = Some(Ok(child)),
                        Err(e) => result = Some(Err(format!("Failed to open VLC: {}", e))),
                    }
                }

                result.unwrap_or(Err("No VLC installation found".to_string()))
            };

            #[cfg(target_os = "windows")]
            let process_result = {
                // Common VLC paths on Windows
                let vlc_paths = [
                    "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",
                    "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe",
                ];

                let mut result = None;

                for &vlc_path in &vlc_paths {
                    if Path::new(vlc_path).exists() {
                        match Command::new(vlc_path)
                            .arg("--play-and-exit")
                            .arg("--quiet")
                            .arg(&self.file_path)
                            .spawn()
                        {
                            Ok(child) => {
                                result = Some(Ok(child));
                                break;
                            }
                            Err(e) => {
                                result = Some(Err(format!("Failed to start VLC: {}", e)));
                            }
                        }
                    }
                }

                // If we haven't found a VLC path, try with just 'vlc'
                if result.is_none() {
                    match Command::new("vlc")
                        .arg("--play-and-exit")
                        .arg("--quiet")
                        .arg(&self.file_path)
                        .spawn()
                    {
                        Ok(child) => result = Some(Ok(child)),
                        Err(e) => result = Some(Err(format!("Failed to start VLC: {}", e))),
                    }
                }

                result.unwrap_or(Err("No VLC installation found".to_string()))
            };

            #[cfg(target_os = "linux")]
            let process_result = {
                let mut result = None;

                match Command::new("vlc")
                    .arg("--play-and-exit")
                    .arg("--quiet")
                    .arg(&self.file_path)
                    .spawn()
                {
                    Ok(child) => result = Some(Ok(child)),
                    Err(e) => result = Some(Err(format!("Failed to start VLC: {}", e))),
                }

                result.unwrap_or(Err("No VLC installation found".to_string()))
            };

            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
            let process_result: Result<Child, String> = Err("Unsupported platform".to_string());

            match process_result {
                Ok(process) => {
                    // Store the process and update status
                    self.process = Some(process);
                    self.start_time = Instant::now();
                    if let Ok(mut status) = self.status.lock() {
                        status.is_playing = true;
                        status.state = "Playing".to_string();
                        status.time_ms = 0;
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }

        pub fn pause(&mut self) -> Result<(), String> {
            // We can't actually pause the external process easily,
            // so we'll just update the status
            if let Ok(mut status) = self.status.lock() {
                status.is_playing = false;
                status.state = "Paused".to_string();
            }
            Ok(())
        }

        pub fn toggle_play(&mut self) -> Result<(), String> {
            let is_playing = {
                if let Ok(status) = self.status.lock() {
                    status.is_playing
                } else {
                    false
                }
            };

            if is_playing {
                self.pause()
            } else {
                self.play()
            }
        }

        pub fn stop(&mut self) -> Result<(), String> {
            // Kill the process if it exists - we take() it to get ownership
            if let Some(mut process) = self.process.take() {
                // Now we own the process and can call kill()
                if let Err(e) = process.kill() {
                    tracing::warn!("Failed to kill VLC process: {}", e);
                }
            }

            // Update status
            if let Ok(mut status) = self.status.lock() {
                status.is_playing = false;
                status.state = "Stopped".to_string();
                status.time_ms = 0;
                status.position = 0.0;
            }

            Ok(())
        }

        pub fn set_position(&mut self, pos: f32) -> Result<(), String> {
            // Store the position (we can't actually seek in external process)
            self.position = pos;

            // Update status
            if let Ok(mut status) = self.status.lock() {
                status.position = pos;
                status.time_ms = (pos * status.duration_ms as f32) as i64;
            }

            Ok(())
        }

        pub fn set_volume(&mut self, volume: i32) -> Result<(), String> {
            // Update status (we can't control external volume)
            if let Ok(mut status) = self.status.lock() {
                status.volume = volume;
            }

            Ok(())
        }

        pub fn get_status(&self) -> PlayerStatus {
            // Check if process is still running
            let process_running = if let Some(process) = &self.process {
                Self::check_process_running(process)
            } else {
                false
            };

            // Calculate current position and time based on start time
            let elapsed = self.start_time.elapsed();
            let time_ms = elapsed.as_millis() as i64;
            let position = if self.duration_ms > 0 {
                (time_ms as f32) / (self.duration_ms as f32)
            } else {
                // Simulate some progress if we don't know duration
                (elapsed.as_secs() as f32) / 300.0 // Assume 5 minutes
            }
            .min(1.0);

            // Get a copy of the current status
            let mut status = if let Ok(status) = self.status.lock() {
                status.clone()
            } else {
                // Fallback status if mutex is poisoned
                PlayerStatus {
                    is_playing: process_running,
                    position: position,
                    duration_ms: self.duration_ms,
                    time_ms,
                    volume: 80,
                    state: if process_running {
                        "Playing"
                    } else {
                        "Stopped"
                    }
                    .to_string(),
                }
            };

            // Update with latest calculated values if playing
            if process_running && status.is_playing {
                status.time_ms = time_ms;
                status.position = position;

                // If we hit the end, mark as stopped
                if position >= 1.0 {
                    status.is_playing = false;
                    status.state = "Finished".to_string();
                }
            }

            status
        }
    }
}

// Check if VLC is available on the system
pub fn is_vlc_available() -> bool {
    #[cfg(feature = "server")]
    {
        #[cfg(vlc_linking)]
        {
            tracing::debug!("Checking if VLC is available by attempting to create an instance");
            match vlc::Instance::new() {
                Ok(_) => {
                    tracing::info!("VLC is available - successfully created instance");
                    return true;
                }
                Err(e) => {
                    tracing::warn!("VLC instance creation failed: {:?}", e);
                    return false;
                }
            }
        }

        tracing::debug!("VLC linking not enabled");
    }

    tracing::debug!("VLC is not available (server feature not enabled)");
    false
}
