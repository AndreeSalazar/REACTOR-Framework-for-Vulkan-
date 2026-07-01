use ash::vk;
use std::ffi::CStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
    SpirV,
}

impl ShaderLanguage {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wgsl" => Some(Self::Wgsl),
            "vert" | "frag" | "comp" | "geom" | "tesc" | "tese" | "glsl" => Some(Self::Glsl),
            "spv" => Some(Self::SpirV),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl ShaderStage {
    pub fn to_vk(&self) -> vk::ShaderStageFlags {
        match self {
            Self::Vertex => vk::ShaderStageFlags::VERTEX,
            Self::Fragment => vk::ShaderStageFlags::FRAGMENT,
            Self::Compute => vk::ShaderStageFlags::COMPUTE,
        }
    }
    pub fn to_naga(&self) -> naga::ShaderStage {
        match self {
            Self::Vertex => naga::ShaderStage::Vertex,
            Self::Fragment => naga::ShaderStage::Fragment,
            Self::Compute => naga::ShaderStage::Compute,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingType {
    UniformBuffer,
    StorageBuffer { read_only: bool },
    Sampler,
    SampledImage,
    CombinedImageSampler,
    StorageImage { read_only: bool },
    InputAttachment,
    AccelerationStructure,
}

impl BindingType {
    pub fn to_vk_descriptor_type(&self) -> vk::DescriptorType {
        match self {
            Self::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            Self::StorageBuffer { .. } => vk::DescriptorType::STORAGE_BUFFER,
            Self::Sampler => vk::DescriptorType::SAMPLER,
            Self::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
            Self::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            Self::StorageImage { .. } => vk::DescriptorType::STORAGE_IMAGE,
            Self::InputAttachment => vk::DescriptorType::INPUT_ATTACHMENT,
            Self::AccelerationStructure => vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectedBinding {
    pub name: String,
    pub group: u32,
    pub binding: u32,
    pub ty: BindingType,
    pub stages: vk::ShaderStageFlags,
    pub size: u32,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectedPushConstant {
    pub name: String,
    pub stages: vk::ShaderStageFlags,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct ReflectedEntryPoint {
    pub name: String,
    pub stage: ShaderStage,
    pub workgroup_size: Option<[u32; 3]>,
}

#[derive(Debug, Clone, Default)]
pub struct ShaderReflection {
    pub entry_points: Vec<ReflectedEntryPoint>,
    pub bindings: Vec<ReflectedBinding>,
    pub push_constants: Vec<ReflectedPushConstant>,
}

#[derive(Debug, Clone)]
pub struct CompiledShader {
    pub spirv: Vec<u32>,
    pub stage: ShaderStage,
    pub entry_point: String,
    pub reflection: ShaderReflection,
    pub spirv_hash: u64,
}
