// REACTOR Resources Module
// Contains game asset abstractions (meshes, materials, textures, vertices)

pub mod asset_manager;
pub mod material;
pub mod mesh;
pub mod model;
pub mod pbr_material;
pub mod primitives;
pub mod texture;
pub mod vertex;

pub use asset_manager::{AssetHandle, AssetManager, AssetState, AssetStats};
pub use material::{Material, MaterialBuilder};
pub use mesh::Mesh;
pub use model::{GltfData, Model, ModelBatch, ObjData};
pub use pbr_material::{IBLEnvironment, IBLUniformData, PBRMaterial, PBRTextures, PBRUniformData};
pub use primitives::Primitives;
pub use texture::Texture;
pub use vertex::{InstanceData, Vertex, VertexPBR};
