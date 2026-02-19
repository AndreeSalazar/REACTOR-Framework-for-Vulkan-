// =============================================================================
// Shadow Mapping — Directional light shadow maps
// =============================================================================

use glam::{Mat4, Vec3};

/// Shadow map configuration
#[derive(Clone, Debug)]
pub struct ShadowConfig {
    pub resolution: u32,
    pub cascade_count: u32,
    pub cascade_splits: Vec<f32>,
    pub bias: f32,
    pub normal_bias: f32,
    pub pcf_samples: u32,
    pub soft_shadows: bool,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            resolution: 2048,
            cascade_count: 4,
            cascade_splits: vec![0.1, 0.3, 0.6, 1.0],
            bias: 0.005,
            normal_bias: 0.02,
            pcf_samples: 16,
            soft_shadows: true,
        }
    }
}

impl ShadowConfig {
    pub fn low_quality() -> Self {
        Self {
            resolution: 1024,
            cascade_count: 2,
            cascade_splits: vec![0.3, 1.0],
            pcf_samples: 4,
            soft_shadows: false,
            ..Default::default()
        }
    }

    pub fn high_quality() -> Self {
        Self {
            resolution: 4096,
            cascade_count: 4,
            cascade_splits: vec![0.05, 0.15, 0.4, 1.0],
            pcf_samples: 32,
            soft_shadows: true,
            ..Default::default()
        }
    }
}

/// Shadow cascade data for cascaded shadow maps
#[derive(Clone, Debug)]
pub struct ShadowCascade {
    pub view_proj: Mat4,
    pub split_depth: f32,
    pub near: f32,
    pub far: f32,
}

impl ShadowCascade {
    pub fn new(split_depth: f32, near: f32, far: f32) -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
            split_depth,
            near,
            far,
        }
    }
}

/// Shadow map manager
#[derive(Clone, Debug)]
pub struct ShadowMap {
    pub config: ShadowConfig,
    pub cascades: Vec<ShadowCascade>,
    pub light_direction: Vec3,
    pub light_view: Mat4,
    pub enabled: bool,
}

impl ShadowMap {
    pub fn new(config: ShadowConfig) -> Self {
        let cascade_count = config.cascade_count as usize;
        let mut cascades = Vec::with_capacity(cascade_count);
        
        for i in 0..cascade_count {
            let split = config.cascade_splits.get(i).copied().unwrap_or(1.0);
            cascades.push(ShadowCascade::new(split, 0.0, 1.0));
        }
        
        Self {
            config,
            cascades,
            light_direction: Vec3::new(-0.5, -1.0, -0.3).normalize(),
            light_view: Mat4::IDENTITY,
            enabled: true,
        }
    }

    /// Update shadow cascades for a camera frustum
    pub fn update(&mut self, _camera_view: Mat4, _camera_proj: Mat4, near: f32, far: f32) {
        self.light_view = Mat4::look_at_rh(
            -self.light_direction * 50.0,
            Vec3::ZERO,
            Vec3::Y,
        );

        let range = far - near;
        
        for (i, cascade) in self.cascades.iter_mut().enumerate() {
            let prev_split = if i == 0 { 0.0 } else { 
                self.config.cascade_splits.get(i - 1).copied().unwrap_or(0.0) 
            };
            let curr_split = self.config.cascade_splits.get(i).copied().unwrap_or(1.0);
            
            cascade.near = near + range * prev_split;
            cascade.far = near + range * curr_split;
            cascade.split_depth = cascade.far;
            
            // Calculate orthographic projection for this cascade
            let cascade_proj = Mat4::orthographic_rh(-50.0, 50.0, -50.0, 50.0, 0.1, 100.0);
            cascade.view_proj = cascade_proj * self.light_view;
        }
    }

    /// Set light direction
    pub fn set_light_direction(&mut self, direction: Vec3) {
        self.light_direction = direction.normalize();
    }

    /// Get cascade index for a given depth
    pub fn get_cascade_index(&self, depth: f32) -> usize {
        for (i, cascade) in self.cascades.iter().enumerate() {
            if depth < cascade.split_depth {
                return i;
            }
        }
        self.cascades.len().saturating_sub(1)
    }

    /// Get shadow bias for a cascade
    pub fn get_bias(&self, cascade_index: usize) -> f32 {
        // Increase bias for farther cascades
        self.config.bias * (1.0 + cascade_index as f32 * 0.5)
    }
}

impl Default for ShadowMap {
    fn default() -> Self {
        Self::new(ShadowConfig::default())
    }
}

/// Shadow uniform data for shaders
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ShadowUniformData {
    pub cascade_view_proj: [[[f32; 4]; 4]; 4],
    pub cascade_splits: [f32; 4],
    pub light_direction: [f32; 4],
    pub shadow_bias: f32,
    pub normal_bias: f32,
    pub pcf_radius: f32,
    pub enabled: u32,
}

impl ShadowUniformData {
    pub fn from_shadow_map(shadow_map: &ShadowMap) -> Self {
        let mut data = Self::default();
        
        for (i, cascade) in shadow_map.cascades.iter().enumerate().take(4) {
            data.cascade_view_proj[i] = cascade.view_proj.to_cols_array_2d();
            data.cascade_splits[i] = cascade.split_depth;
        }
        
        data.light_direction = [
            shadow_map.light_direction.x,
            shadow_map.light_direction.y,
            shadow_map.light_direction.z,
            0.0,
        ];
        data.shadow_bias = shadow_map.config.bias;
        data.normal_bias = shadow_map.config.normal_bias;
        data.pcf_radius = 1.0 / shadow_map.config.resolution as f32;
        data.enabled = if shadow_map.enabled { 1 } else { 0 };
        
        data
    }
}

/// PCF (Percentage Closer Filtering) helper
pub fn pcf_sample_offsets(samples: u32) -> Vec<(f32, f32)> {
    let mut offsets = Vec::new();
    let sqrt_samples = (samples as f32).sqrt() as i32;
    let half = sqrt_samples / 2;
    
    for y in -half..=half {
        for x in -half..=half {
            offsets.push((x as f32, y as f32));
        }
    }
    
    offsets
}

/// Calculate shadow factor (0 = in shadow, 1 = lit)
pub fn calculate_shadow_factor(
    world_pos: Vec3,
    normal: Vec3,
    shadow_map: &ShadowMap,
    cascade_index: usize,
) -> f32 {
    if !shadow_map.enabled {
        return 1.0;
    }
    
    let cascade = &shadow_map.cascades[cascade_index];
    let _bias = shadow_map.get_bias(cascade_index);
    
    // Apply normal bias
    let biased_pos = world_pos + normal * shadow_map.config.normal_bias;
    
    // Transform to light space
    let light_space = cascade.view_proj * biased_pos.extend(1.0);
    let _ndc = light_space.truncate() / light_space.w;
    
    // In a real implementation, we would sample the shadow map texture here
    // This is a placeholder that returns fully lit
    1.0
}
