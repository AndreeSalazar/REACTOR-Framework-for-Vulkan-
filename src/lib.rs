// =============================================================================
// REACTOR Framework for Vulkan (Rust Edition) — v1.2.0
// =============================================================================
// Architecture: A (Vulkan/Ash) → B (Reactor Core) → C (Game)
//   A: Raw Vulkan bindings (unsafe)
//   B: Safe RAII wrappers + UE5-style subsystems (this framework)
//   C: High-level game API (ReactorApp trait)
// =============================================================================

// Application trait system (the user-facing API)
pub mod app;

// Modular architecture (production code)
pub mod compute;
pub mod core;
pub mod graphics;
pub mod raytracing;
pub mod resources;
pub mod systems;
pub mod utils;

// Platform abstraction (window, config)
pub mod platform;

// Built-in SPIR-V shaders (embedded for plug-and-play materials)
pub mod builtin_shaders;

// =============================================================================
// Legacy Modules (temporary — will be migrated in Fase 0.2)
// =============================================================================
// These modules use the old flat structure and are kept for backward compatibility
// with app.rs and examples. Will be removed once migration is complete.
pub mod buffer;
pub mod cpu_detector;
pub mod ecs;
pub mod gpu_detector;
pub mod input;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod ray_tracing;
pub mod reactor;
pub mod resolution_detector;
pub mod scene;
pub mod swapchain;
pub mod vertex;

// =============================================================================
// App Trait System Re-exports
// =============================================================================
pub use app::{quick, quick_with, run, ReactorApp, ReactorConfig, ReactorContext, RendererMode};

// =============================================================================
// `reactor::game!` macro — ultra-short game declaration
// =============================================================================

/// Macro de una sola línea para crear y lanzar un juego REACTOR.
///
/// # Ejemplos
///
/// Mínimo:
/// ```rust,no_run
/// reactor::game! {
///     title: "Mi Juego",
///     update: |ctx| {
///         let _ = ctx.time.delta();
///     }
/// }
/// ```
///
/// Completo:
/// ```rust,no_run
/// reactor::game! {
///     title: "Mi Juego",
///     size: (1920, 1080),
///     vsync: true,
///     msaa: 4,
///     init: |ctx| {
///         ctx.camera.position = reactor::prelude::Vec3::new(0.0, 2.0, 5.0);
///     },
///     update: |ctx| {
///         let _ = ctx.time.delta();
///     }
/// }
/// ```
#[macro_export]
macro_rules! game {
    (
        title: $title:expr
        $(, size: ($w:expr, $h:expr))?
        $(, vsync: $vsync:expr)?
        $(, msaa: $msaa:expr)?
        $(, fullscreen: $fs:expr)?
        $(, init: $init:expr)?
        , update: $update:expr
        $(,)?
    ) => {{
        fn __reactor_main() {
            #[allow(unused_mut)]
            let mut __cfg = $crate::ReactorConfig::new($title);
            $( __cfg = __cfg.with_size($w, $h); )?
            $( __cfg = __cfg.with_vsync($vsync); )?
            $( __cfg = __cfg.with_msaa($msaa); )?
            $( __cfg = __cfg.with_fullscreen($fs); )?

            #[allow(unused_assignments, unused_mut)]
            let mut __init_fn: Option<fn(&mut $crate::ReactorContext)> = None;
            $( __init_fn = Some($init); )?

            if let Some(init) = __init_fn {
                $crate::quick_with(__cfg, init, $update);
            } else {
                $crate::quick_with(__cfg, |_ctx: &mut $crate::ReactorContext| {}, $update);
            }
        }
        __reactor_main();
    }};
}

// =============================================================================
// Core Re-exports (canonical names, no *New suffix)
// =============================================================================

pub use core::allocator::MemoryAllocator;
pub use core::command::CommandManager;
pub use core::context::VulkanContext;
pub use core::device::DeviceInfo;
pub use core::error::{
    clear_last_error, get_last_error_code, get_last_error_message, has_error, set_last_error,
    ErrorCode, ReactorError, ReactorResult,
};

// FrameGraph (Deterministic Render Graph)
pub use core::frame_graph::{
    create_deferred_graph, create_forward_graph, Barrier, FrameGraph, FrameGraphStats, PassDesc,
    PassId, ResourceFormat, ResourceId, ResourceType,
};

// Importance Map (Universal Importance System)
pub use core::importance_map::{
    ImportanceMap, ImportanceMapConfig, ImportanceMapStats, ImportanceTileData, ImportanceType,
};

// Profiler & Logging (UE5-style observability)
pub use core::jobs::{init_job_system, join, par_iter, par_iter_mut, parallel_for};
pub use core::linear_allocator::{BumpArena, LinearAllocator};
pub use core::logging::{init_logger, init_logger_with, LogLevel};
pub use core::profiler::{begin_frame, get_frame_id, CpuTimer, PerfCounter};

// =============================================================================
// Graphics Re-exports
// =============================================================================

pub use graphics::buffer::Buffer;
pub use graphics::depth::DepthBuffer;
pub use graphics::descriptors::{
    DescriptorBinding, DescriptorPool, DescriptorSet, DescriptorSetLayout, PoolSize,
};
pub use graphics::framebuffer::{Framebuffer, FramebufferSet};
pub use graphics::image::Image;
pub use graphics::msaa::MsaaTarget;
pub use graphics::pipeline::{Pipeline, PipelineConfig};
pub use graphics::render_pass::{RenderPass, RenderPassConfig};
pub use graphics::sampler::{FilterMode, Sampler, SamplerConfig, WrapMode};
pub use graphics::swapchain::Swapchain;

// Uniform Buffers
pub use graphics::uniform_buffer::{
    GlobalUniformData, LightData, LightUniformData, MaterialUniformData, UniformBuffer, MAX_LIGHTS,
};

// Debug Renderer
pub use graphics::debug_renderer::{DebugAABB, DebugLine, DebugRay, DebugRenderer, DebugSphere};

// Post-processing
pub use graphics::post_process::{
    AAQualityPreset, AASettings, PostProcessEffect, PostProcessPipeline, PostProcessPreset,
    PostProcessSettings,
};

// =============================================================================
// Ray Tracing Re-exports
// =============================================================================

pub use raytracing::acceleration_structure::{AccelerationStructure, Blas, Tlas};
pub use raytracing::context::RayTracingContext;
pub use raytracing::pipeline::{RayTracingPipeline, ShaderGroup, ShaderStage};
pub use raytracing::shader_binding_table::ShaderBindingTable;

// =============================================================================
// Compute Re-exports
// =============================================================================

pub use compute::dispatch::ComputeDispatch;
pub use compute::pipeline::ComputePipeline;

// =============================================================================
// Resources Re-exports
// =============================================================================

pub use resources::material::{Material, MaterialBuilder};
pub use resources::mesh::Mesh;
pub use resources::model::{Model, ModelBatch};
pub use resources::primitives::Primitives;
pub use resources::texture::Texture;
pub use resources::vertex::{InstanceData, Vertex, VertexPBR};

// =============================================================================
// Systems Re-exports
// =============================================================================

pub use systems::animation::{
    AnimationClip, AnimationPlayer, AnimationTrack, EasingFunction, Keyframe, LoopMode, Tween,
};
pub use systems::audio::{AudioClipId, AudioListener, AudioSource, AudioSourceId, AudioSystem};
pub use systems::camera::{Camera, Camera2D};
pub use systems::ecs::{Component, Entity, World};
pub use systems::frustum::{CullingSystem, Frustum, FrustumTestResult, Plane};
pub use systems::input::Input;
pub use systems::lighting::{Light, LightType, LightingSystem};
pub use systems::particles::{EmitterShape, Particle, ParticleSystem, ParticleSystemConfig};
pub use systems::physics::{PhysicsWorld, Ray, RigidBody, Sphere, AABB};
pub use systems::scene::{Scene, SceneObject};
pub use systems::transform::Transform;

// =============================================================================
// Utils Re-exports
// =============================================================================

pub use utils::cpu_detector::{CPUDetector, CPUInfo};
pub use utils::gpu_detector::{GPUDetector, GPUInfo};
pub use utils::resolution_detector::{MonitorInfo, ResolutionDetector};
pub use utils::time::{FixedTimestep, Time};

// =============================================================================
// Prelude - Import everything commonly needed
// =============================================================================
pub mod prelude {
    // App Trait System
    pub use crate::app::{run, ReactorApp, ReactorConfig, ReactorContext, RendererMode};

    // Core (Vulkan context)
    pub use crate::core::context::VulkanContext;

    // Resources
    pub use crate::resources::material::{Material, MaterialBuilder};
    pub use crate::resources::mesh::Mesh;
    pub use crate::resources::primitives::Primitives;
    pub use crate::resources::texture::Texture;
    pub use crate::resources::vertex::Vertex;

    // Systems
    pub use crate::systems::animation::{AnimationPlayer, EasingFunction, Tween};
    pub use crate::systems::camera::{Camera, Camera2D};
    pub use crate::systems::input::Input;
    pub use crate::systems::lighting::{Light, LightType, LightingSystem};
    pub use crate::systems::particles::ParticleSystem;
    pub use crate::systems::physics::{Ray, RigidBody, Sphere, AABB};
    pub use crate::systems::scene::Scene;
    pub use crate::systems::transform::Transform;

    // Graphics
    pub use crate::graphics::debug_renderer::DebugRenderer;
    pub use crate::graphics::post_process::{PostProcessPreset, PostProcessSettings};

    // Utils
    pub use crate::utils::cpu_detector::CPUDetector;
    pub use crate::utils::resolution_detector::ResolutionDetector;
    pub use crate::utils::time::Time;

    // Math
    pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
}
