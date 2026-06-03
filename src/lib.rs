//! # REACTOR Framework
//!
//! A high-performance, memory-safe game engine built with Rust and Vulkan.
//!
//! ## Quick Start
//!
//! ```rust
//! use reactor_vulkan::prelude::*;
//!
//! struct MyGame;
//!
//! impl ReactorApp for MyGame {
//!     fn config(&self) -> ReactorConfig {
//!         ReactorConfig::new("My Game")
//!             .with_size(1920, 1080)
//!             .with_vsync(true)
//!     }
//!     
//!     fn init(&mut self, ctx: &mut ReactorContext) {
//!         println!("Game initialized!");
//!     }
//!     
//!     fn update(&mut self, ctx: &mut ReactorContext) {
//!         // Game loop
//!     }
//! }
//!
//! fn main() {
//!     reactor_vulkan::run(MyGame);
//! }
//! ```

#![allow(unused_variables, dead_code)]

// Core modules
pub mod app;
pub mod base_shader;
pub mod builtin_shaders;
pub mod compute;
pub mod core;
pub mod graphics;
pub mod platform;
pub mod raytracing;
pub mod renderer;
pub mod resources;
pub mod scene;
pub mod systems;
pub mod utils;

// Main reactor module
pub mod reactor;

// Re-export the main run function
pub use app::run;

// Re-export commonly used types at the crate root
pub use app::app::{quick, quick_with, ReactorApp};
pub use app::config::{ReactorConfig, RendererMode};
pub use app::pause_config::{
    PauseConfig, PauseConfigPage, PauseConfigResult, PauseConfiguracion, PauseConfiguration,
};
pub use base_shader::{
    BaseMaterialDefaults, BaseShaderAsset, BaseShaderCookbook, BaseShaderPair, BaseShaderStage,
};
pub use core::context::VulkanContext;
pub use core::debug_utils::DebugNamer;
pub use core::error::{ErrorCode, ReactorError, ReactorResult};
pub use core::memory_budget::{GpuMemoryBudget, HeapBudget};
pub use core::vrs::{
    PixelIntelligent, PixelIntelligentProfile, VrsCapabilities, VrsContext, VrsRate,
    VrsSupportedRate,
};
pub use reactor::Reactor;
pub use resources::font::FontAsset;
pub use resources::material::Material;
pub use resources::mesh::Mesh;
pub use resources::decal::Decal;
pub use resources::vertex::Vertex;
pub use scene::camera::Camera;
pub use scene::transform::Transform;
pub use systems::audio::AudioClip;

// Re-export system types
pub use systems::lighting::{Light, LightType, LightingSystem};
pub use systems::physics::{PhysicsWorld, Ray, RigidBody, Sphere, AABB};
pub use systems::scene::Scene;

// Re-export utility types
pub use utils::{CPUDetector, ResolutionDetector};

// Re-export glam types for convenience
pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

/// Prelude module - import everything you need with `use reactor_vulkan::prelude::*;`
pub mod prelude {
    pub use crate::platform::{Gamepad, GamepadAxis, GamepadButton};
    pub use crate::systems::audio::{
        AudioClipId, AudioListener, AudioSource, AudioSourceId, AudioSystem,
    };
    pub use crate::systems::console::{color, GameBanner, Log, ReactorBanner};
    pub use crate::systems::event_bus::{EventBus, Observer};
    pub use crate::{
        quick,
        quick_with,

        run,
        // Audio
        AudioClip,

        BaseMaterialDefaults,
        BaseShaderAsset,
        BaseShaderCookbook,
        BaseShaderPair,
        BaseShaderStage,
        // Utilities
        CPUDetector,
        // Scene / Systems
        Camera,
        // Core
        DebugNamer,
        ErrorCode,
        FontAsset,

        GpuMemoryBudget,
        HeapBudget,
        Light,
        LightType,
        LightingSystem,
        Mat3,
        Mat4,
        Material,
        // Resources
        Mesh,
        Decal,
        PauseConfig,
        PauseConfigPage,
        PauseConfigResult,
        PauseConfiguracion,
        PauseConfiguration,
        PhysicsWorld,
        PixelIntelligent,
        PixelIntelligentProfile,
        Quat,
        Ray,
        Reactor,

        ReactorApp,
        // App
        ReactorConfig,
        ReactorError,
        ReactorResult,
        RendererMode,
        ResolutionDetector,

        RigidBody,
        Scene,
        Sphere,
        Transform,
        // Math
        Vec2,
        Vec3,
        Vec4,
        VrsCapabilities,
        VrsRate,
        VrsSupportedRate,
        VulkanContext,
        AABB,
    };

    // Re-export the ReactorContext type alias if it exists
    pub use crate::app::app::{GltfBounds, GltfSpawn, ModelSpawnInfo};
    pub use crate::app::ReactorContext;
}

/// Macro for declarative game definition
#[macro_export]
macro_rules! game {
    (
        title: $title:expr,
        size: ($w:expr, $h:expr),
        vsync: $vsync:expr,
        msaa: $msaa:expr,
        init: $init:expr,
        update: $update:expr
    ) => {{
        struct Game;

        impl $crate::ReactorApp for Game {
            fn config(&self) -> $crate::ReactorConfig {
                $crate::ReactorConfig::new($title)
                    .with_size($w, $h)
                    .with_vsync($vsync)
                    .with_msaa($msaa)
            }

            fn init(&mut self, ctx: &mut $crate::app::ReactorContext) {
                $crate::app::call_init($init, ctx);
            }

            fn update(&mut self, ctx: &mut $crate::app::ReactorContext) {
                $crate::app::call_update($update, ctx);
            }
        }

        $crate::run(Game);
    }};
}
