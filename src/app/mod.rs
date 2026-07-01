//! Application layer
//!
//! High-level application management and configuration.

#[allow(clippy::module_inception)]
pub mod app;
pub mod pause_config;

// Re-export main types
pub use app::{
    call_init, call_update, quick, quick_with, ReactorApp, ReactorContext, RendererMode,
};
pub use app::ReactorConfig;
pub use pause_config::{
    PauseConfig, PauseConfigPage, PauseConfigResult, PauseConfiguracion, PauseConfiguration,
};
pub use app::run;
