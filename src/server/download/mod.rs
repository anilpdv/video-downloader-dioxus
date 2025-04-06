//! Download functionality module
//! Provides functionality for downloading media from various sources

pub(crate) mod core;
pub use core::types::*;

pub(crate) mod api;
pub(crate) mod platform;
pub(crate) mod provider;
pub(crate) mod storage;

// Re-export the main public API
pub use api::*;
