mod extract;
mod loader;
mod model;
mod types;

pub use loader::GltfLoader;
pub use types::{
    AnimationChannel, AnimationInterpolation, AnimationPath, AnimationSampler,
    GltfAlphaMode, GltfAnimation, GltfCacheStats, GltfMaterialData, GltfMeshData, GltfModel,
    GltfNode, GltfTextureData,
};

use std::path::Path;
use crate::core::error::ReactorResult;

pub fn load_gltf_simple<P: AsRef<Path>>(path: P) -> ReactorResult<GltfModel> {
    let base_path = path.as_ref().parent().unwrap_or(Path::new("."));
    let mut loader = GltfLoader::new(base_path);
    loader.load(path)
}
