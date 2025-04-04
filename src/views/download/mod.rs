// Export all parts of the download module
mod handlers;
mod platforms;
mod types;
mod ui;

// Re-export the main component
pub use ui::Download;
// Re-export types for external use
pub use types::{FormatType, Quality};
