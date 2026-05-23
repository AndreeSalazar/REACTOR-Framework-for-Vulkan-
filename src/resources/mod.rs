//! Asset and resource management
//!
//! Loading, caching, and managing game assets.
//!
//! ## Fase 3: Asset Pipeline (en progreso)
//!
//! Los módulos `asset_id`, `handle`, `gltf_loader`, `asset_database`,
//! `asset_hot_reload` y `asset_loader_queue` están **temporalmente
//! deshabilitados** porque su API todavía no encaja con el resto del
//! engine (varios cientos de errores de compilación). Cuando se resuelvan
//! las firmas reales de `Material::new`, `ReactorError::AssetLoad`,
//! `Texture::from_rgba8`, `Mesh::from_vertices_and_indices`, etc., se
//! volverán a habilitar uno a uno.
//!
//! Mientras tanto, los juegos siguen pudiendo cargar modelos a través de
//! `model.rs` (que sí compila) y `asset_manager.rs`.

pub mod asset_id;
pub mod asset_manager;
pub mod handle;
pub mod material;
pub mod mesh;
pub mod model;
pub mod pbr_material;
pub mod primitives;
pub mod texture;
pub mod vertex;

// Re-exports foundacionales
pub use asset_id::{AssetId, AssetPath};
pub use handle::{Handle, WeakHandle, AssetRef};

// Legacy / actualmente funcional
pub use asset_manager::{AssetHandle, AssetManager, AssetState, AssetStats};
pub use material::{Material, MaterialBuilder};
pub use mesh::Mesh;
pub use model::{GltfData, Model, ModelBatch, ObjData};
pub use pbr_material::{IBLEnvironment, IBLUniformData, PBRMaterial, PBRTextures, PBRUniformData};
pub use primitives::Primitives;
pub use texture::Texture;
pub use vertex::{InstanceData, Vertex, VertexPBR};
