#[derive(Clone, Debug, PartialEq)]
pub struct DownloadItem {
    pub id: Option<i64>,
    pub title: String,
    pub filename: String,
    pub file_path: String,
    pub format_type: String,
    pub quality: String,
    pub file_size: Option<i64>,
    pub duration: Option<i64>,
    pub date_downloaded: String,
    pub thumbnail_url: Option<String>,
    pub file_exists: bool,
}

impl DownloadItem {
    pub fn format_duration(&self) -> String {
        if let Some(duration) = self.duration {
            let hours = duration / 3600;
            let minutes = (duration % 3600) / 60;
            let seconds = duration % 60;

            if hours > 0 {
                format!("{}:{:02}:{:02}", hours, minutes, seconds)
            } else {
                format!("{}:{:02}", minutes, seconds)
            }
        } else {
            "".to_string()
        }
    }

    pub fn format_file_size(&self) -> String {
        if let Some(size) = self.file_size {
            if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        } else {
            "Unknown".to_string()
        }
    }
}
