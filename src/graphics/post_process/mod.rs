//! Post-processing pipeline module — split from the original 3252-line `pipeline.rs`.
//!
//! - `pipeline.rs` — `PostProcessPipeline` struct + constructors + Drop
//! - `types.rs` — shared types (PostProcessSettings, PostProcessEffect, etc.)
//! - `init.rs` — `init()` and `recreate_offscreen_images()` orchestrator
//! - Each effect has its own file (bloom, taa, fog, lens_flare, gtao, etc.)
//! - `light_cull.rs` — `PointLightGpu` type + conversion helpers
//! - `light_dispatch.rs` — light culling compute dispatch methods
//! - `clouds.rs` — VolumetricClouds (separate, not part of PostProcessPipeline)
//! - `ssgi_hiz.rs` — SsgiHiZ (separate)

mod auto_exposure;
mod bloom;
mod clouds;
mod depth_resolve;
mod fog;
mod gtao;
mod init;
mod lens_flare;
mod light_cull;
mod light_dispatch;
mod ssgi_hiz;
mod taa;

mod pipeline;
mod types;

pub use clouds::{generate_value_noise_3d, VolumetricClouds};
pub use light_cull::{lights_to_gpu_buffer, PointLightGpu};
pub use pipeline::PostProcessPipeline;
pub use ssgi_hiz::SsgiHiZ;
pub use types::{
    AASettings, AAQualityPreset, AutoExposureParams, PostProcessEffect, PostProcessPreset,
    PostProcessSettings,
};
