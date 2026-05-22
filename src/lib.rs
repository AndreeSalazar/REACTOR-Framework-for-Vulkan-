//! REACTOR Framework for Vulkan — 100% Rust Puro
//!
//! Un framework de juegos de alto rendimiento sobre Vulkan 1.3.

#![allow(unused_variables, dead_code)]

// ── Módulos principales (nueva arquitectura) ─────────────────────────────
pub mod core;
pub mod graphics;
pub mod raytracing;
pub mod compute;
pub mod resources;
pub mod systems;
pub mod platform;
pub mod utils;
pub mod builtin_shaders;

// ── Módulos de la fachada principal ──────────────────────────────────────
pub mod reactor;
pub mod app;

// ── Re-exports canónicos (API pública estable) ───────────────────────────
pub use reactor::Reactor;
pub use app::{ReactorApp, ReactorConfig, ReactorContext, RendererMode, run};
pub use core::VulkanContext;
pub use core::error::{ReactorError, ReactorResult, ErrorCode};

pub use resources::vertex::Vertex;
pub use resources::mesh::Mesh;
pub use resources::material::Material;

pub use systems::input::Input;
pub use systems::scene::Scene;
pub use systems::camera::Camera;

pub use raytracing::RayTracingContext;

// ── Prelude: `use reactor_vulkan::prelude::*;` ──────────────────────────
pub mod prelude {
    pub use crate::{
        ReactorApp, ReactorConfig, ReactorContext, RendererMode, run,
        Vertex, Mesh, Material, Scene, Camera, Input,
        VulkanContext, ReactorError, ReactorResult,
    };
    pub use glam::{Vec2, Vec3, Vec4, Mat3, Mat4, Quat};
}