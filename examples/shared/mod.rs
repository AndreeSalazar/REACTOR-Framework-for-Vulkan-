//! Common helpers shared across REACTOR example applications.
//!
//! Each example is its own binary in `examples/<name>/main.rs` and declares
//! this module via:
//!
//! ```ignore
//! #[path = "../shared/mod.rs"]
//! mod shared;
//! ```
//!
//! This keeps the example binaries independent (no extra workspace crate) while
//! letting them share camera input, FPS counter, and scene-construction helpers.

pub mod camera_input;
pub mod fps_counter;
pub mod scene_helpers;
