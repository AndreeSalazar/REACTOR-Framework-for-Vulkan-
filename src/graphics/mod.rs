// REACTOR Graphics Module
// Contains all rendering-related abstractions

pub mod swapchain;
pub mod pipeline;
pub mod render_pass;
pub mod framebuffer;
pub mod buffer;
pub mod image;
pub mod sampler;
pub mod descriptors;
pub mod depth;
pub mod msaa;
pub mod uniform_buffer;
pub mod debug_renderer;
pub mod post_process;
pub mod aa_pipeline;

pub use swapchain::Swapchain;
pub use pipeline::{Pipeline, PipelineConfig};
pub use render_pass::{RenderPass, RenderPassConfig};
pub use framebuffer::{Framebuffer, FramebufferSet};
pub use buffer::Buffer;
pub use image::Image;
pub use sampler::{Sampler, SamplerConfig, FilterMode, WrapMode};
pub use descriptors::{DescriptorPool, DescriptorSetLayout, DescriptorSet, DescriptorBinding, PoolSize};
pub use depth::DepthBuffer;
pub use msaa::MsaaTarget;
pub use uniform_buffer::{UniformBuffer, GlobalUniformData, LightUniformData, LightData, MaterialUniformData};
pub use debug_renderer::{DebugRenderer, DebugLine};
pub use post_process::{PostProcessPipeline, PostProcessSettings, PostProcessEffect, PostProcessPreset};
pub use aa_pipeline::{AAGlobalPipeline, AAStats, sdf_edge_coverage, sdf_blend_colors, sdf_edge_alpha, smooth_normal, smooth_fresnel, aa_preset_ui, aa_preset_realtime_3d, aa_preset_offline};
