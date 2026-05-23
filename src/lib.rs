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
pub mod core;
pub mod platform;
pub mod graphics;
pub mod resources;
pub mod scene;
pub mod renderer;
pub mod raytracing;
pub mod compute;
pub mod systems;
pub mod app;
pub mod utils;
pub mod builtin_shaders;

// Main reactor module
pub mod reactor;

// Re-export the main run function
pub use app::run;

// Re-export commonly used types at the crate root
pub use core::error::{ReactorError, ReactorResult, ErrorCode};
pub use core::context::VulkanContext;
pub use app::config::{ReactorConfig, RendererMode};
pub use app::app::{ReactorApp, quick, quick_with};
pub use scene::camera::Camera;
pub use scene::transform::Transform;
pub use resources::mesh::Mesh;
pub use resources::material::Material;
pub use resources::font::FontAsset;
pub use systems::audio::AudioClip;
pub use resources::vertex::Vertex;
pub use reactor::Reactor;

// Re-export system types
pub use systems::lighting::{Light, LightType, LightingSystem};
pub use systems::physics::{PhysicsWorld, RigidBody, Ray, Sphere, AABB};
pub use systems::scene::Scene;

// Re-export utility types
pub use utils::{CPUDetector, ResolutionDetector};

// Re-export glam types for convenience
pub use glam::{Vec2, Vec3, Vec4, Mat3, Mat4, Quat};

/// Prelude module - import everything you need with `use reactor_vulkan::prelude::*;`
pub mod prelude {
    pub use crate::{
        // App
        ReactorConfig, ReactorApp, RendererMode, run, quick, quick_with,
        
        // Core
        ReactorError, ReactorResult, ErrorCode, VulkanContext,
        Reactor,
        
        // Scene / Systems
        Camera, Transform, Scene,
        Light, LightType, LightingSystem,
        PhysicsWorld, RigidBody, Ray, Sphere, AABB,
        
        // Utilities
        CPUDetector, ResolutionDetector,
        
        // Resources
        Mesh, Material, FontAsset,
        
        // Audio
        AudioClip,

        // Math
        Vec2, Vec3, Vec4, Mat3, Mat4, Quat,
    };
    pub use crate::systems::audio::{AudioSystem, AudioClipId, AudioSourceId, AudioListener, AudioSource};
    pub use crate::systems::event_bus::{EventBus, Observer};
    
    // Re-export the ReactorContext type alias if it exists
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
