// =============================================================================
// REACTOR Logging — Structured, layered, engine-grade
// =============================================================================
// Provides a single `init_logger()` entry point that configures:
//   - `tracing` as the event system (spans + events)
//   - `tracing-subscriber` with fmt layer (console) and optional file layer
//   - Env-filter for granular control (REACTOR_LOG env var)
//
// Usage:
//   reactor::core::logging::init_logger();                 // defaults
//   reactor::core::logging::init_logger_with(LogLevel::Debug, None);
//
// =============================================================================

use std::sync::Once;

pub use tracing::Level as LogLevel;
pub use tracing::{debug, error, info, trace, warn};

static INIT: Once = Once::new();

/// Initialize the global logger with sensible defaults.
///
/// - Console output with colors and targets
/// - Default level: `info`
/// - Override via `REACTOR_LOG` env var (e.g. `debug`, `reactor=trace,winit=warn`)
/// - Safe to call multiple times (idempotent).
pub fn init_logger() {
    init_logger_with(LogLevel::INFO, None);
}

/// Initialize the logger with a specific level and optional log file path.
pub fn init_logger_with(level: LogLevel, log_file: Option<&str>) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    INIT.call_once(|| {
        let env_filter = EnvFilter::try_from_env("REACTOR_LOG")
            .unwrap_or_else(|_| EnvFilter::new(level.as_str()));

        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_ansi(true)
            .compact();

        // Optional: write logs to a file as JSON for analysis
        if let Some(_path) = log_file {
            // Reserved for future: JSON file layer
            // let file = std::fs::File::create(path).ok();
            // let json_layer = fmt::layer().json().with_writer(file);
        }

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();

        tracing::info!(
            target: "reactor::init",
            version = env!("CARGO_PKG_VERSION"),
            level = %level,
            "REACTOR engine initialized"
        );
    });
}

/// Log engine shutdown with final statistics.
pub fn log_shutdown(stats: &crate::utils::time::Time) {
    tracing::info!(
        target: "reactor::shutdown",
        elapsed_secs = stats.elapsed(),
        "REACTOR engine shutting down"
    );
}

// =============================================================================
// Convenience macros
// =============================================================================

/// Log a message at `info` level with `reactor` target.
#[macro_export]
macro_rules! r_info {
    ($($arg:tt)*) => { tracing::info!(target: "reactor", $($arg)*); };
}

/// Log a message at `warn` level with `reactor` target.
#[macro_export]
macro_rules! r_warn {
    ($($arg:tt)*) => { tracing::warn!(target: "reactor", $($arg)*); };
}

/// Log a message at `error` level with `reactor` target.
#[macro_export]
macro_rules! r_error {
    ($($arg:tt)*) => { tracing::error!(target: "reactor", $($arg)*); };
}

/// Log a message at `debug` level with `reactor` target.
#[macro_export]
macro_rules! r_debug {
    ($($arg:tt)*) => { tracing::debug!(target: "reactor", $($arg)*); };
}

/// Log a message at `trace` level with `reactor` target.
#[macro_export]
macro_rules! r_trace {
    ($($arg:tt)*) => { tracing::trace!(target: "reactor", $($arg)*); };
}
