//! `reactorapp` — The single high-level entry point for REACTOR.
//!
//! This module is the **one place to import** when building a REACTOR app.
//! It re-exports:
//!
//! - The core app traits: [`ReactorApp`], [`ReactorContext`], [`ReactorConfig`]
//! - The fluent builder: [`App`], [`CameraSetup`], [`LightingSetup`], [`MeshBuilder`]
//! - The common error type: [`ReactorResult`], [`ReactorError`], [`SpawnError`]
//! - The most-used resources: [`Material`], [`Mesh`], [`Vertex`], [`Texture`], [`Scene`]
//! - The math types from `glam`
//!
//! # Example
//!
//! ```ignore
//! use reactor_vulkan::reactorapp::*;
//!
//! struct MyGame { /* ... */ }
//!
//! impl ReactorApp for MyGame {
//!     fn init(&mut self, ctx: &mut ReactorContext) {
//!         let mut app = App::new(ctx);
//!         app.camera().look_at(Vec3::new(0, 2, 4), Vec3::ZERO, 60.0);
//!         app.lighting().default_three_point();
//!         app.mesh().vertices(&V).indices(&I).use_cookbook_forward_material().spawn();
//!     }
//! }
//!
//! fn main() { reactor_vulkan::run(MyGame::new()); }
//! ```
//!
//! The philosophy: if you need to **change how the engine is used by
//! applications**, this is the file to edit. All the surface area that
//! typical apps touch is curated and re-exported here.

pub use crate::app::ReactorContext;
pub use crate::app::{GltfBounds, GltfSpawn, ModelSpawnInfo, ReactorApp, RendererMode};
pub use crate::app_helpers::App;
pub use crate::app_helpers::camera_setup::CameraSetup;
pub use crate::app_helpers::exit_helpers::check_escape;
pub use crate::app_helpers::lighting_setup::LightingSetup;
pub use crate::app_helpers::mesh_builder::{MeshBuilder, SpawnError};
pub use crate::core::error::{ErrorCode, ReactorError, ReactorResult};

pub use crate::base_shader::{BaseShaderCookbook, BaseShaderPair, BaseShaderStage};
pub use crate::resources::decal::Decal;
pub use crate::resources::material::Material;
pub use crate::resources::mesh::Mesh;
pub use crate::resources::texture::Texture;
pub use crate::resources::vertex::Vertex;
pub use crate::scene::camera::Camera;
pub use crate::scene::transform::Transform;
pub use crate::systems::lighting::{Light, LightType, LightingSystem};
pub use crate::systems::scene::{Scene, SceneObject};
pub use crate::platform::input::Input;
pub use crate::platform::time::Time;
pub use crate::platform::window::ReactorWindow;
pub use crate::reactor::Reactor;
pub use crate::app::ReactorConfig;

pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

/// Helper: build a default REACTOR app and run it.
/// Equivalent to `reactor_vulkan::run(app)`.
pub fn launch<A: ReactorApp + 'static>(app: A) {
    crate::app::run(app);
}


