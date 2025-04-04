// Platform-specific implementations
mod common;
#[cfg(feature = "desktop")]
mod desktop;
#[cfg(feature = "web")]
mod web;

// Export platform-specific functions
#[cfg(not(feature = "web"))]
pub use common::create_blob_url;
#[cfg(not(feature = "web"))]
pub use common::trigger_download;
#[cfg(feature = "desktop")]
pub use desktop::save_to_disk;
#[cfg(feature = "web")]
pub use web::{create_blob_url, trigger_download};

// Common formatting function
pub use common::format_eta;
