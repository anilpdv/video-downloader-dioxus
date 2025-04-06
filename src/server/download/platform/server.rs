// File operations for server platform
pub fn open_file(path: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("cmd").args(["/c", "start", "", path]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let _ = Command::new("xdg-open").arg(path).spawn();
    }
}

pub fn open_containing_folder(path: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("explorer").args(["/select,", path]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let parent = std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        let _ = Command::new("open").arg(parent).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let parent = std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        let _ = Command::new("xdg-open").arg(parent).spawn();
    }
}
