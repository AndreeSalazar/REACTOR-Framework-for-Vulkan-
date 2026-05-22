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
pub use app::app::ReactorApp;
pub use scene::camera::Camera;
pub use scene::transform::Transform;
pub use resources::mesh::Mesh;
pub use resources::material::Material;
pub use resources::vertex::Vertex;

// Re-export glam types for convenience
pub use glam::{Vec2, Vec3, Vec4, Mat3, Mat4, Quat};

/// Prelude module - import everything you need with `use reactor_vulkan::prelude::*;`
pub mod prelude {
    pub use crate::{
        // App
        ReactorConfig, ReactorApp, RendererMode, run,
        
        // Core
        ReactorError, ReactorResult, ErrorCode, VulkanContext,
        
        // Scene
        Camera, Transform,
        
        // Resources
        Mesh, Material,
        
        // Math
        Vec2, Vec3, Vec4, Mat3, Mat4, Quat,
    };
    
    // Re-export the ReactorContext type alias if it exists
    pub use crate::app::ReactorContext;
}

/// Quick start function - create a game in one line
pub fn quick<F>(title: &str, width: u32, height: u32, update_fn: F)
where
    F: FnMut(&mut app::ReactorContext) + 'static,
{
    struct QuickGame<F> {
        title: String,
        width: u32,
        height: u32,
        update_fn: F,
    }
    
    impl<F> ReactorApp for QuickGame<F>
    where
        F: FnMut(&mut app::ReactorContext),
    {
        fn config(&self) -> ReactorConfig {
            ReactorConfig::new(&self.title)
                .with_size(self.width, self.height)
                .with_vsync(true)
        }
        
        fn init(&mut self, _ctx: &mut app::ReactorContext) {}
        
        fn update(&mut self, ctx: &mut app::ReactorContext) {
            (self.update_fn)(ctx);
        }
    }
    
    run(QuickGame {
        title: title.to_string(),
        width,
        height,
        update_fn,
    });
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
                $init(ctx);
            }
            
            fn update(&mut self, ctx: &mut $crate::app::ReactorContext) {
                $update(ctx);
            }
        }
        
        $crate::run(Game);
    }};
}
