//! Application layer
//!
//! High-level application management and configuration.

#[allow(clippy::module_inception)]
pub mod app;
pub mod config;
pub mod pause_config;
pub mod runner;

// Re-export main types
pub use app::{
    call_init, call_update, quick, quick_with, ReactorApp, ReactorContext, RendererMode,
};
pub use config::ReactorConfig;
pub use pause_config::{
    PauseConfig, PauseConfigPage, PauseConfigResult, PauseConfiguracion, PauseConfiguration,
};
pub use runner::run;
