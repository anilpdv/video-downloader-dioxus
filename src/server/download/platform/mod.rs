//! Platform-specific functionality

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::*;

#[cfg(feature = "web")]
mod web;
#[cfg(feature = "web")]
pub use web::*;
