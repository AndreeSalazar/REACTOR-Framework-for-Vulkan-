//! Application configuration
//!
//! Re-exports configuration types from the app module.

// Re-export from app.rs to maintain backward compatibility
pub use crate::app::app::{ReactorConfig, RendererMode};
