#[derive(Debug, Clone, Copy)]
pub struct BindlessConfig {
    pub max_textures: u32,
    pub max_buffers: u32,
    pub max_samplers: u32,
    pub max_meshes: u32,
    pub max_materials: u32,
}

impl Default for BindlessConfig {
    fn default() -> Self {
        Self {
            max_textures: 8192,
            max_buffers: 4096,
            max_samplers: 16,
            max_meshes: 4096,
            max_materials: 4096,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BindlessStats {
    pub textures_used: u32,
    pub textures_max: u32,
    pub buffers_used: u32,
    pub buffers_max: u32,
    pub meshes_used: u32,
    pub meshes_max: u32,
    pub materials_used: u32,
    pub materials_max: u32,
}
