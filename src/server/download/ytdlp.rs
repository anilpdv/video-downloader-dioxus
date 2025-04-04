use dioxus::prelude::*;
use server_fn::error::NoCustomError;
use std::path::PathBuf;
use std::process::Command;
use tracing;

/// Check if yt-dlp is installed and download it if not found
#[cfg(feature = "server")]
pub async fn ensure_yt_dlp_available() -> Result<PathBuf, ServerFnError<NoCustomError>> {
    let app_data_dir = get_app_data_dir()?;
    let bin_dir = app_data_dir.join("bin");

    // Create the bin directory if it doesn't exist
    std::fs::create_dir_all(&bin_dir).map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!(
            "Failed to create bin directory: {}",
            e
        ))
    })?;

    // Path to the yt-dlp executable in our bin directory
    let yt_dlp_path = bin_dir.join(get_yt_dlp_binary_name());

    // Check if we already have yt-dlp in our bin directory
    if yt_dlp_path.exists() {
        // Verify it works
        let check = Command::new(&yt_dlp_path).arg("--version").output();
        if let Ok(output) = check {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                tracing::info!("Found bundled yt-dlp: {}", version.trim());
                return Ok(yt_dlp_path);
            }
        }

        // If we reach here, the existing binary doesn't work, so we'll remove it
        let _ = std::fs::remove_file(&yt_dlp_path);
    }

    tracing::info!("Bundled yt-dlp not found or not working, extracting...");

    // Extract or download yt-dlp
    #[cfg(feature = "desktop")]
    {
        // For desktop, extract the bundled binary
        extract_bundled_yt_dlp(&bin_dir)?;

        // Make it executable on Unix-like systems
        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&yt_dlp_path)?.permissions();
            perms.set_mode(0o755); // rwx r-x r-x
            std::fs::set_permissions(&yt_dlp_path, perms).map_err(|e| {
                ServerFnError::<NoCustomError>::ServerError(format!(
                    "Failed to set executable permissions: {}",
                    e
                ))
            })?;
        }
    }

    // For non-desktop or as fallback, download yt-dlp
    if !yt_dlp_path.exists() {
        match youtube_dl::download_yt_dlp(&bin_dir).await {
            Ok(path) => {
                tracing::info!("Downloaded yt-dlp to {:?}", path);
                // Verify it works
                let downloaded_check = Command::new(&path).arg("--version").output();

                match downloaded_check {
                    Ok(output) if output.status.success() => {
                        tracing::info!("Downloaded yt-dlp is working");
                        return Ok(path);
                    }
                    _ => {
                        tracing::error!("Downloaded yt-dlp failed verification");
                        return Err(ServerFnError::<NoCustomError>::ServerError(
                            "Downloaded yt-dlp failed verification. Make sure it has executable permissions.".to_string(),
                        ));
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to download yt-dlp: {}", e);
                return Err(ServerFnError::<NoCustomError>::ServerError(format!(
                    "Failed to download yt-dlp: {}",
                    e
                )));
            }
        }
    }

    // Final check to make sure we have a working binary
    let final_check = Command::new(&yt_dlp_path).arg("--version").output();
    match final_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            tracing::info!("Bundled yt-dlp ready: {}", version.trim());
            Ok(yt_dlp_path)
        }
        _ => Err(ServerFnError::<NoCustomError>::ServerError(
            "Failed to get a working yt-dlp binary".to_string(),
        )),
    }
}

/// Get the appropriate app data directory for storing our bundled binaries
#[cfg(feature = "server")]
fn get_app_data_dir() -> Result<PathBuf, ServerFnError<NoCustomError>> {
    // Use dirs crate to get platform-specific app data directory
    let base_dir = dirs::data_local_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .ok_or_else(|| {
            ServerFnError::<NoCustomError>::ServerError(
                "Could not determine app data directory".to_string(),
            )
        })?;

    // Create our app's directory
    let app_dir = base_dir.join("youtube_downloader");
    std::fs::create_dir_all(&app_dir).map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!(
            "Failed to create app data directory: {}",
            e
        ))
    })?;

    Ok(app_dir)
}

/// Get platform-specific yt-dlp binary name
#[cfg(feature = "server")]
fn get_yt_dlp_binary_name() -> String {
    #[cfg(target_os = "windows")]
    {
        "yt-dlp.exe".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "yt-dlp".to_string()
    }
}

/// Extract bundled yt-dlp binary for desktop builds
#[cfg(all(feature = "server", feature = "desktop"))]
fn extract_bundled_yt_dlp(bin_dir: &PathBuf) -> Result<(), ServerFnError<NoCustomError>> {
    // Path to the bundled binary (embedded in the executable at compile time)
    // We'll use different binaries for different platforms
    let binary_data = {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            include_bytes!("../../../resources/yt-dlp-windows-x64.exe")
        }
        #[cfg(all(target_os = "windows", not(target_arch = "x86_64")))]
        {
            include_bytes!("../../../resources/yt-dlp-windows-x86.exe")
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            include_bytes!("../../../resources/yt-dlp-macos-x64")
        }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            include_bytes!("../../../resources/yt-dlp-macos-arm64")
        }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            include_bytes!("../../../resources/yt-dlp-linux-x64")
        }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            include_bytes!("../../../resources/yt-dlp-linux-arm64")
        }
        #[cfg(not(any(
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "windows", not(target_arch = "x86_64")),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64")
        )))]
        {
            return Err(ServerFnError::<NoCustomError>::ServerError(
                "No bundled yt-dlp for this platform".to_string(),
            ));
        }
    };

    // Write the binary to disk
    let target_path = bin_dir.join(get_yt_dlp_binary_name());
    std::fs::write(&target_path, binary_data).map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to write yt-dlp binary: {}", e))
    })?;

    tracing::info!("Extracted bundled yt-dlp to {:?}", target_path);
    Ok(())
}
