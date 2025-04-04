use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Represents a downloaded video in the database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Download {
    /// Unique identifier
    pub id: Option<i64>,
    /// Original YouTube URL
    pub url: String,
    /// Video title
    pub title: Option<String>,
    /// Filename given to the download
    pub filename: String,
    /// Full path to the saved file
    pub file_path: String,
    /// Format type (video or audio)
    pub format_type: String,
    /// Quality setting used
    pub quality: String,
    /// File size in bytes
    pub file_size: Option<i64>,
    /// When the file was downloaded
    #[serde(with = "time::serde::timestamp::option")]
    pub download_date: Option<OffsetDateTime>,
    /// URL to video thumbnail
    pub thumbnail_url: Option<String>,
    /// YouTube video ID
    pub video_id: Option<String>,
    /// Duration in seconds
    pub duration: Option<i64>,
}

impl Download {
    /// Create a new download record
    pub fn new(
        url: String,
        title: Option<String>,
        filename: String,
        file_path: String,
        format_type: String,
        quality: String,
        file_size: Option<i64>,
        thumbnail_url: Option<String>,
        video_id: Option<String>,
        duration: Option<i64>,
    ) -> Self {
        Self {
            id: None,
            url,
            title,
            filename,
            file_path,
            format_type,
            quality,
            file_size,
            download_date: Some(OffsetDateTime::now_utc()),
            thumbnail_url,
            video_id,
            duration,
        }
    }

    /// Extract the video ID from a YouTube URL
    pub fn extract_video_id(url: &str) -> Option<String> {
        let parsed_url = url::Url::parse(url).ok()?;

        // Handle youtube.com/watch?v=VIDEO_ID
        if parsed_url.host_str() == Some("www.youtube.com")
            || parsed_url.host_str() == Some("youtube.com")
        {
            if parsed_url.path() == "/watch" {
                let pairs = parsed_url.query_pairs();
                for (key, value) in pairs {
                    if key == "v" {
                        return Some(value.to_string());
                    }
                }
            }
        }

        // Handle youtu.be/VIDEO_ID
        if parsed_url.host_str() == Some("youtu.be") {
            let path = parsed_url.path();
            if path.len() > 1 {
                return Some(path[1..].to_string());
            }
        }

        None
    }

    /// Generate a thumbnail URL from a video ID
    pub fn generate_thumbnail_url(video_id: &str) -> String {
        format!("https://i.ytimg.com/vi/{}/mqdefault.jpg", video_id)
    }

    /// Format the duration as a human-readable string
    pub fn format_duration(&self) -> String {
        if let Some(seconds) = self.duration {
            let hours = seconds / 3600;
            let minutes = (seconds % 3600) / 60;
            let remaining_seconds = seconds % 60;

            if hours > 0 {
                format!("{}:{:02}:{:02}", hours, minutes, remaining_seconds)
            } else {
                format!("{}:{:02}", minutes, remaining_seconds)
            }
        } else {
            "Unknown".to_string()
        }
    }

    /// Format the file size as a human-readable string
    pub fn format_file_size(&self) -> String {
        if let Some(size) = self.file_size {
            if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        } else {
            "Unknown size".to_string()
        }
    }

    /// Format the download date as a readable string
    pub fn format_date(&self) -> String {
        if let Some(date) = self.download_date {
            // Format as "Mon, 01 Jan 2023 14:30"
            time::format_description::parse(
                "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]",
            )
            .map(|fmt| {
                date.format(&fmt)
                    .unwrap_or_else(|_| "Invalid date".to_string())
            })
            .unwrap_or_else(|_| "Invalid date".to_string())
        } else {
            "Unknown date".to_string()
        }
    }

    /// Check if the file exists on disk
    pub fn file_exists(&self) -> bool {
        std::path::Path::new(&self.file_path).exists()
    }
}
