use dioxus::prelude::*;

// Enum for format type selection
#[derive(Clone, PartialEq)]
pub enum FormatType {
    Video,
    Audio,
}

// Enum for quality selection
#[derive(Clone, PartialEq)]
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
