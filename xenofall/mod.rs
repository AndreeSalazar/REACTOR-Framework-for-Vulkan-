//! XENOFALL gameplay/render lab modules.
//!
//! Phase 1 keeps the original `examples/xenofall.rs` playable while moving
//! reusable systems into this directory. The monolith is now only the launcher
//! and integration shell; professional render experiments should live here.

#![allow(dead_code)]

pub mod audio;
pub mod app_lifecycle;
pub mod cards;
pub mod cards_runtime;
pub mod combat;
pub mod constants;
pub mod core;
pub mod enemies;
pub mod helpers;
pub mod player;
pub mod render_lab;
pub mod scene;
pub mod types;
pub mod ui;
pub mod vfx;
pub mod visual_features;
pub mod waves;
pub mod world;
