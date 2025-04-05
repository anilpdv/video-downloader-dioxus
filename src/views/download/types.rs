use dioxus::prelude::*;

/// Format type for download (audio or video)
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum FormatType {
    Audio,
    Video,
}

/// Quality options for media downloads
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Quality {
    Highest,
    Medium,
    Lowest,
}

// Helper functions for converting types to/from strings
impl FormatType {
    pub fn to_string(&self) -> String {
        match self {
            FormatType::Video => "video".to_string(),
            FormatType::Audio => "audio".to_string(),
        }
    }

    pub fn get_extension(&self) -> &'static str {
        match self {
            FormatType::Video => "mp4",
            FormatType::Audio => "mp3",
        }
    }

    pub fn get_mime_type(&self) -> &'static str {
        match self {
            FormatType::Video => "video/mp4",
            FormatType::Audio => "audio/mpeg",
        }
    }

    // Add helper to check extension
    pub fn has_valid_extension(&self, filename: &str) -> bool {
        let extension = self.get_extension();
        filename
            .to_lowercase()
            .ends_with(&format!(".{}", extension))
    }
}

impl Quality {
    pub fn to_string(&self) -> String {
        match self {
            Quality::Highest => "highest".to_string(),
            Quality::Medium => "medium".to_string(),
            Quality::Lowest => "lowest".to_string(),
        }
    }
}
