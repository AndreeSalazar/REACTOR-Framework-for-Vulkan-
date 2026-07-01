use std::path::PathBuf;
use glam::Mat4;
use crate::resources::vertex::Vertex;

#[derive(Clone, Debug)]
pub struct GltfMeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub name: String,
    pub material_index: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct GltfTextureData {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GltfAlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}

#[derive(Clone, Debug)]
pub struct GltfMaterialData {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub base_color_texture_index: Option<usize>,
    pub normal_texture_index: Option<usize>,
    pub metallic_roughness_texture_index: Option<usize>,
    pub occlusion_texture_index: Option<usize>,
    pub emissive_texture_index: Option<usize>,
    pub emissive_factor: [f32; 3],
    pub alpha_mode: GltfAlphaMode,
    pub double_sided: bool,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct GltfModel {
    pub meshes: Vec<GltfMeshData>,
    pub materials: Vec<GltfMaterialData>,
    pub textures: Vec<GltfTextureData>,
    pub root_node: GltfNode,
    pub animations: Vec<GltfAnimation>,
    pub source_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct GltfNode {
    pub name: String,
    pub transform: Mat4,
    pub mesh_index: Option<usize>,
    pub material_index: Option<usize>,
    pub children: Vec<GltfNode>,
}

#[derive(Clone, Debug)]
pub struct GltfAnimation {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
    pub samplers: Vec<AnimationSampler>,
}

#[derive(Clone, Debug)]
pub struct AnimationChannel {
    pub node_index: usize,
    pub sampler_index: usize,
    pub path: AnimationPath,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationPath {
    Translation,
    Rotation,
    Scale,
    Weights,
}

#[derive(Clone, Debug)]
pub struct AnimationSampler {
    pub input: Vec<f32>,
    pub output: Vec<f32>,
    pub interpolation: AnimationInterpolation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationInterpolation {
    Linear,
    Step,
    CubicSpline,
}

#[derive(Clone, Debug)]
pub struct GltfCacheStats {
    pub models_cached: usize,
}
