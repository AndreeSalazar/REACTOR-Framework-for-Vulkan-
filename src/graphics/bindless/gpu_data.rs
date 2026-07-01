use super::handle::TextureHandle;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GpuMeshData {
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_count: u32,
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub _pad0: f32,
    pub _pad1: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuMaterialData {
    pub albedo: [f32; 4],
    pub albedo_texture: u32,
    pub normal_texture: u32,
    pub metallic_roughness_texture: u32,
    pub ao_texture: u32,
    pub emissive: [f32; 4],
    pub metallic_roughness_ao_alpha: [f32; 4],
    pub flags: u32,
    pub sampler_index: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

impl Default for GpuMaterialData {
    fn default() -> Self {
        Self {
            albedo: [1.0, 1.0, 1.0, 1.0],
            albedo_texture: TextureHandle::INVALID.0,
            normal_texture: TextureHandle::INVALID.0,
            metallic_roughness_texture: TextureHandle::INVALID.0,
            ao_texture: TextureHandle::INVALID.0,
            emissive: [0.0; 4],
            metallic_roughness_ao_alpha: [0.0, 0.5, 1.0, 1.0],
            flags: 0,
            sampler_index: 0,
            _pad0: 0,
            _pad1: 0,
        }
    }
}
