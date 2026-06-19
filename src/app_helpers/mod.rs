//! High-level convenience API for building REACTOR applications.
//!
//! See the top-level `reactorapp` module for the unified entry point that
//! re-exports these helpers.
//!
//! NOTE: We declare submodules via `#[path = "..."]` because the Windows
//! build environment has trouble auto-discovering `.rs` files in
//! subdirectories in some cases. `#[path]` makes the resolution explicit.

#[path = "app.rs"]
pub mod app;
#[path = "camera_setup.rs"]
pub mod camera_setup;
#[path = "exit_helpers.rs"]
pub mod exit_helpers;
#[path = "lighting_setup.rs"]
pub mod lighting_setup;
#[path = "mesh_builder.rs"]
pub mod mesh_builder;

pub use app::App;
