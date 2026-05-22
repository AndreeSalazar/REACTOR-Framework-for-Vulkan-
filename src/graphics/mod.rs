//! Vulkan graphics rendering
//! 
//! Low-level rendering primitives and pipeline management.

pub mod buffer;
pub mod debug_renderer;
pub mod depth;
pub mod descriptors;
pub mod framebuffer;
pub mod image;
pub mod msaa;
pub mod pipeline;
pub mod post_process;
pub mod render_pass;
pub mod sampler;
pub mod shadows;
pub mod swapchain;
pub mod uniform_buffer;

pub use buffer::Buffer;
pub use debug_renderer::{DebugLine, DebugRenderer};
pub use depth::DepthBuffer;
pub use descriptors::{
    DescriptorBinding, DescriptorPool, DescriptorSet, DescriptorSetLayout, PoolSize,
};
pub use framebuffer::{Framebuffer, FramebufferSet};
pub use image::Image;
pub use msaa::MsaaTarget;
pub use pipeline::{Pipeline, PipelineConfig};
pub use post_process::{
    PostProcessEffect, PostProcessPipeline, PostProcessPreset, PostProcessSettings,
};
pub use render_pass::{RenderPass, RenderPassConfig};
pub use sampler::{FilterMode, Sampler, SamplerConfig, WrapMode};
pub use shadows::{ShadowCascade, ShadowConfig, ShadowMap, ShadowUniformData};
pub use swapchain::Swapchain;
pub use uniform_buffer::{
    GlobalUniformData, LightData, LightUniformData, MaterialUniformData, UniformBuffer,
};

// ═══ FASE 2 — Pipeline gráfico moderno ═══
pub mod bindless;
pub mod pso_hash;
pub mod pso_cache;
pub mod shader_compiler;
pub mod indirect;

pub use bindless::{BindlessRegistry, BindlessConfig, TextureHandle, BufferHandle, SamplerHandle};
pub use pso_hash::{PsoHash, PsoHashBuilder};
pub use pso_cache::{PsoCache, CachedPipeline, PsoCacheManager};
pub use shader_compiler::{ShaderCompiler, ShaderLanguage, ShaderStage, CompiledShader};
pub use indirect::{IndirectDrawBuffer, DrawIndexedIndirectCommand, IndirectCommandWithMaterial};
