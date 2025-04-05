// Export all parts of the download module
mod platforms;
mod types;

// Re-export types for external use
pub use types::{FormatType, Quality};

pub mod components;
pub mod data;
pub mod page;
pub mod services;

pub use page::DownloadPage;
