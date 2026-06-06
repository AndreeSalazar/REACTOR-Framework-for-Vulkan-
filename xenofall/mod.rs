//! XENOFALL gameplay/render lab modules.
//!
//! Phase 1 keeps the original `examples/xenofall.rs` playable while moving
//! reusable systems into this directory. The monolith is now only the launcher
//! and integration shell; professional render experiments should live here.

#![allow(dead_code)]

pub mod audio;
pub mod cards;
pub mod constants;
pub mod helpers;
pub mod render_lab;
pub mod types;
pub mod visual_features;
