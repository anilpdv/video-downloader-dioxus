//! API handlers for download functionality

pub mod database;
pub mod echo;
pub mod info;
pub mod player;
pub mod progress;
pub mod search;
pub mod video;

// Re-export all handlers
pub use database::*;
pub use echo::*;
pub use info::*;
pub use player::*;
pub use progress::*;
pub use search::*;
pub use video::*;
