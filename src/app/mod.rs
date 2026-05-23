//! Application layer
//! 
//! High-level application management and configuration.

pub mod app;
pub mod config;
pub mod runner;

// Re-export main types
pub use app::{ReactorApp, ReactorContext, RendererMode, quick, quick_with, call_init, call_update};
pub use config::ReactorConfig;
pub use runner::run;
