// REACTOR Resources Module
// Contains game asset abstractions (meshes, materials, textures, vertices)

pub mod vertex;
pub mod mesh;
pub mod material;
pub mod texture;
pub mod model;
pub mod primitives;
pub mod asset_manager;
pub mod pbr_material;

pub use vertex::{Vertex, VertexPBR, InstanceData};
pub use mesh::Mesh;
pub use material::{Material, MaterialBuilder};
pub use texture::Texture;
pub use model::{Model, ModelBatch, ObjData, GltfData};
pub use primitives::Primitives;
pub use asset_manager::{AssetManager, AssetHandle, AssetState, AssetStats};
pub use pbr_material::{PBRMaterial, PBRUniformData, PBRTextures, IBLEnvironment, IBLUniformData};
