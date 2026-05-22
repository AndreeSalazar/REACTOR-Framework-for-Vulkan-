//! Application layer
//! 
//! High-level application management and configuration.

pub mod app;
pub mod config;
pub mod runner;

// Re-export main types
pub use app::{ReactorApp, ReactorContext, RendererMode};
pub use config::ReactorConfig;
pub use runner::run;
