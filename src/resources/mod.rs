//! Asset and resource management
//! 
//! Loading, caching, and managing game assets.
//! 
//! ## Fase 3: Asset Pipeline
//! - `asset_id`: AssetId estable basado en hash de contenido
//! - `handle`: Handle<T> genérico con reference counting
//! - `gltf_loader`: Cargador de modelos glTF 2.0 con PBR
//! - `asset_manager`: Sistema centralizado con cache y hot-reload

pub mod asset_id;
pub mod asset_manager;
pub mod asset_database;
pub mod asset_hot_reload;
pub mod asset_loader_queue;
pub mod gltf_loader;
pub mod handle;
pub mod material;
pub mod mesh;
pub mod model;
pub mod pbr_material;
pub mod primitives;
pub mod texture;
pub mod vertex;

// Fase 3 exports - Asset Pipeline completo
pub use asset_id::{AssetId, AssetPath};
pub use handle::{Handle, WeakHandle, AssetRef};
pub use asset_database::{AssetDatabase, AssetMetadata, AssetType, AssetDbStats};
pub use asset_hot_reload::{AssetHotReloadManager, HotReloadConfig, AssetReloadEvent, HotReloadStats};
pub use asset_loader_queue::{AssetLoaderQueue, LoadPriority, LoadState, LoaderStats, LoaderQueueConfig, AssetLoaders};
pub use gltf_loader::{GltfLoader, GltfModel, GltfNode, GltfAnimation, load_gltf_simple};

// Legacy exports
pub use asset_manager::{AssetHandle, AssetManager, AssetState, AssetStats};
pub use material::{Material, MaterialBuilder};
pub use mesh::Mesh;
pub use model::{GltfData, Model, ModelBatch, ObjData};
pub use pbr_material::{IBLEnvironment, IBLUniformData, PBRMaterial, PBRTextures, PBRUniformData};
pub use primitives::Primitives;
pub use texture::Texture;
pub use vertex::{InstanceData, Vertex, VertexPBR};
