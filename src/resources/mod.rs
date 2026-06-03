//! Asset and resource management
//!
//! Loading, caching, and managing game assets.
//!
//! ## Fase 3: Asset Pipeline
//!
//! Los módulos `asset_id`, `handle`, `gltf_loader`, `asset_database`,
//! `asset_hot_reload` y `asset_loader_queue` implementan el pipeline
//! completo de assets con:
//! - Carga CPU-side (sin dependencia Vulkan en el loader)
//! - Cache persistente con sled (AssetDatabase)
//! - Hot-reload via filesystem watcher (AssetHotReloadManager)
//! - Cola asíncrona con prioridad (AssetLoaderQueue)

pub mod asset_cooker;
pub mod asset_database;
pub mod asset_hot_reload;
pub mod asset_id;
pub mod asset_loader_queue;
pub mod asset_manager;
pub mod font;
pub mod gltf_loader;
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
pub use handle::{AssetRef, Handle, WeakHandle};

// Fase 3: GltfLoader, AssetDatabase, HotReload y LoaderQueue
pub use asset_cooker::AssetCooker;
pub use asset_database::{AssetDatabase, AssetDbStats, AssetMetadata, AssetType};
pub use asset_hot_reload::{AssetHotReloadManager, HotReloadConfig, HotReloadStats};
pub use asset_loader_queue::{AssetLoaderQueue, LoadPriority, LoaderQueueConfig, LoaderStats};
pub use font::FontAsset;
pub use gltf_loader::{
    GltfCacheStats, GltfLoader, GltfMaterialData, GltfMeshData, GltfModel, GltfNode,
    GltfTextureData,
};

// Legacy / actualmente funcional
pub mod decal;
pub use decal::Decal;
pub use asset_manager::{AssetHandle, AssetManager, AssetState, AssetStats};
pub use material::{Material, MaterialBuilder};
pub use mesh::Mesh;
pub use model::{GltfData, Model, ModelBatch, ObjData};
pub use pbr_material::{IBLEnvironment, IBLUniformData, PBRMaterial, PBRTextures, PBRUniformData};
pub use primitives::Primitives;
pub use texture::Texture;
pub use vertex::{InstanceData, Vertex, VertexPBR};

