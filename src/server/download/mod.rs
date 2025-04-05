// Download functionality module

// Export our services for download operations
pub mod services;
pub use services::*;

// Web-specific services
#[cfg(feature = "web")]
pub mod web_services;
#[cfg(feature = "web")]
pub use web_services::*;

// Types
pub mod types;
pub use types::*;

// Utilities
pub mod utils;
pub use utils::*;

// yt-dlp handling
pub mod ytdlp;
pub use ytdlp::*;

// File storage handling
pub mod storage;
pub use storage::*;

// Server handlers
pub mod handlers;
pub use handlers::*;

// Integrate all modules
