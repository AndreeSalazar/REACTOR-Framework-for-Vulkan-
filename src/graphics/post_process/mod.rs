//! Post-processing pipeline — module entry point
//!
//! The full pipeline was previously a single 3212-LOC file (`post_process.rs`).
//! That file now lives as `pipeline.rs` and is being incrementally broken into
//! focused submodules:
//! - `pipeline` — `PostProcessPipeline` orchestrator + all the existing effects
//! - `ssgi_hiz` — Hi-Z screen-space GI compute pass (newly added)
//!
//! New effect submodules will be added here as the original monolith is split.

pub mod clouds;
pub mod light_cull;
pub mod pipeline;
pub mod ssgi_hiz;

pub use clouds::{generate_value_noise_3d, VolumetricClouds};
pub use light_cull::{lights_to_gpu_buffer, PointLightGpu};
pub use pipeline::*;
pub use ssgi_hiz::SsgiHiZ;
