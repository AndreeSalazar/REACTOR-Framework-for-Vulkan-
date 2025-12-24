// =============================================================================
// REACTOR Framework for Vulkan (Rust Edition)
// =============================================================================
// Architecture: A (Vulkan/Ash) → B (Reactor) → C (Game)
//   A: Raw Vulkan bindings (unsafe)
//   B: Safe RAII wrappers (this framework)
//   C: High-level game API
// =============================================================================

// Legacy modules (for backwards compatibility)
pub mod vulkan_context;
pub mod reactor;
pub mod swapchain;
pub mod pipeline;
pub mod buffer;
pub mod vertex;
pub mod mesh;
pub mod material;
pub mod input;
pub mod ecs;
pub mod ray_tracing;
pub mod resolution_detector;
pub mod scene;
pub mod gpu_detector;
pub mod cpu_detector;

// New modular structure
pub mod core;
pub mod graphics;
pub mod raytracing;
pub mod compute;
pub mod resources;
pub mod systems;
pub mod utils;

// =============================================================================
// Legacy Re-exports (backwards compatibility)
// =============================================================================
pub use reactor::Reactor;
pub use pipeline::Pipeline;
pub use buffer::Buffer;
pub use vertex::Vertex;
pub use mesh::Mesh;
pub use material::Material;
pub use input::Input;
pub use ecs::{World, Entity, Component};
pub use ray_tracing::RayTracingContext;
pub use resolution_detector::ResolutionDetector;
pub use scene::Scene;
pub use gpu_detector::GPUDetector;
pub use cpu_detector::CPUDetector;

// =============================================================================
// New Modular Re-exports
// =============================================================================

// Core
pub use core::context::VulkanContext as VulkanContextNew;
pub use core::allocator::MemoryAllocator;
pub use core::command::CommandManager;
pub use core::device::DeviceInfo;

// Graphics
pub use graphics::swapchain::Swapchain as SwapchainNew;
pub use graphics::pipeline::{Pipeline as PipelineNew, PipelineConfig};
pub use graphics::render_pass::{RenderPass, RenderPassConfig};
pub use graphics::framebuffer::{Framebuffer, FramebufferSet};
pub use graphics::buffer::Buffer as BufferNew;
pub use graphics::image::Image;
pub use graphics::sampler::{Sampler, SamplerConfig, FilterMode, WrapMode};
pub use graphics::descriptors::{DescriptorPool, DescriptorSetLayout, DescriptorSet, DescriptorBinding, PoolSize};
pub use graphics::depth::DepthBuffer;
pub use graphics::msaa::MsaaTarget;

// Ray Tracing
pub use raytracing::context::RayTracingContext as RayTracingContextNew;
pub use raytracing::acceleration_structure::{AccelerationStructure, Blas, Tlas};
pub use raytracing::pipeline::{RayTracingPipeline, ShaderGroup, ShaderStage};
pub use raytracing::shader_binding_table::ShaderBindingTable;

// Compute
pub use compute::pipeline::ComputePipeline;
pub use compute::dispatch::ComputeDispatch;

// Resources
pub use resources::vertex::{Vertex as VertexNew, VertexPBR, InstanceData};
pub use resources::mesh::Mesh as MeshNew;
pub use resources::material::{Material as MaterialNew, MaterialBuilder};
pub use resources::texture::Texture;
pub use resources::model::{Model, ModelBatch};

// Systems
pub use systems::input::Input as InputNew;
pub use systems::ecs::{World as WorldNew, Entity as EntityNew, Component as ComponentNew};
pub use systems::scene::{Scene as SceneNew, SceneObject};
pub use systems::camera::{Camera, Camera2D};
pub use systems::transform::Transform;

// Utils
pub use utils::gpu_detector::{GPUDetector as GPUDetectorNew, GPUInfo};
pub use utils::cpu_detector::{CPUDetector as CPUDetectorNew, CPUInfo};
pub use utils::resolution_detector::{ResolutionDetector as ResolutionDetectorNew, MonitorInfo};
pub use utils::time::{Time, FixedTimestep};

// =============================================================================
// Prelude - Import everything commonly needed
// =============================================================================
pub mod prelude {
    pub use crate::Reactor;
    pub use crate::resources::vertex::Vertex;
    pub use crate::resources::mesh::Mesh;
    pub use crate::resources::material::Material;
    pub use crate::resources::texture::Texture;
    pub use crate::systems::scene::Scene;
    pub use crate::systems::camera::Camera;
    pub use crate::systems::input::Input;
    pub use crate::systems::transform::Transform;
    pub use crate::utils::time::Time;
    pub use crate::utils::cpu_detector::CPUDetector;
    pub use crate::utils::resolution_detector::ResolutionDetector;
    pub use glam::{Vec2, Vec3, Vec4, Mat4, Quat};
}
