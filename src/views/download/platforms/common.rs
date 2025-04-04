// Common functionality across platforms

// Format the ETA in human readable format
pub fn format_eta(seconds: u64) -> String {
    if seconds == 0 {
        return "calculating...".to_string();
    }

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

// Non-web fallback implementation using base64
#[cfg(not(feature = "web"))]
pub fn create_blob_url(data: &[u8], mime_type: &str) -> Option<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let base64_data = STANDARD.encode(data);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);
    Some(data_url)
}

// No-op for trigger_download on non-web platforms
#[cfg(not(feature = "web"))]
pub fn trigger_download(_url: &str, _filename: &str) {
    // This is a no-op for non-web platforms
    // Downloads happen via data URLs in the anchor element
}
