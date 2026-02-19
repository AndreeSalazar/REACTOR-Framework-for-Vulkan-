// =============================================================================
// PBR Materials — Physically Based Rendering materials
// =============================================================================

use glam::Vec3;

/// PBR material properties
#[derive(Clone, Debug)]
pub struct PBRMaterial {
    pub albedo: Vec3,
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
    pub emissive: Vec3,
    pub emissive_strength: f32,
    pub normal_scale: f32,
    pub alpha: f32,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
}

impl Default for PBRMaterial {
    fn default() -> Self {
        Self {
            albedo: Vec3::new(1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            ao: 1.0,
            emissive: Vec3::ZERO,
            emissive_strength: 1.0,
            normal_scale: 1.0,
            alpha: 1.0,
            alpha_cutoff: 0.5,
            double_sided: false,
        }
    }
}

impl PBRMaterial {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a metal material
    pub fn metal(albedo: Vec3, roughness: f32) -> Self {
        Self {
            albedo,
            metallic: 1.0,
            roughness,
            ..Default::default()
        }
    }

    /// Create a dielectric (non-metal) material
    pub fn dielectric(albedo: Vec3, roughness: f32) -> Self {
        Self {
            albedo,
            metallic: 0.0,
            roughness,
            ..Default::default()
        }
    }

    /// Create a plastic-like material
    pub fn plastic(albedo: Vec3) -> Self {
        Self {
            albedo,
            metallic: 0.0,
            roughness: 0.4,
            ..Default::default()
        }
    }

    /// Create a glass-like material
    pub fn glass(tint: Vec3) -> Self {
        Self {
            albedo: tint,
            metallic: 0.0,
            roughness: 0.0,
            alpha: 0.3,
            ..Default::default()
        }
    }

    /// Create an emissive material
    pub fn emissive(color: Vec3, strength: f32) -> Self {
        Self {
            albedo: color,
            emissive: color,
            emissive_strength: strength,
            ..Default::default()
        }
    }

    // Builder methods
    pub fn with_albedo(mut self, albedo: Vec3) -> Self {
        self.albedo = albedo;
        self
    }

    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic.clamp(0.0, 1.0);
        self
    }

    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness.clamp(0.04, 1.0);
        self
    }

    pub fn with_ao(mut self, ao: f32) -> Self {
        self.ao = ao.clamp(0.0, 1.0);
        self
    }

    pub fn with_emissive(mut self, emissive: Vec3, strength: f32) -> Self {
        self.emissive = emissive;
        self.emissive_strength = strength;
        self
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }
}

/// PBR uniform data for shaders
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PBRUniformData {
    pub albedo: [f32; 4],
    pub metallic_roughness_ao_alpha: [f32; 4],
    pub emissive: [f32; 4],
    pub flags: [f32; 4],
}

impl PBRUniformData {
    pub fn from_material(mat: &PBRMaterial) -> Self {
        Self {
            albedo: [mat.albedo.x, mat.albedo.y, mat.albedo.z, mat.alpha],
            metallic_roughness_ao_alpha: [mat.metallic, mat.roughness, mat.ao, mat.alpha_cutoff],
            emissive: [
                mat.emissive.x * mat.emissive_strength,
                mat.emissive.y * mat.emissive_strength,
                mat.emissive.z * mat.emissive_strength,
                mat.normal_scale,
            ],
            flags: [
                if mat.double_sided { 1.0 } else { 0.0 },
                0.0, 0.0, 0.0,
            ],
        }
    }
}

/// PBR texture set
#[derive(Clone, Debug, Default)]
pub struct PBRTextures {
    pub albedo_map: Option<u32>,
    pub normal_map: Option<u32>,
    pub metallic_roughness_map: Option<u32>,
    pub ao_map: Option<u32>,
    pub emissive_map: Option<u32>,
}

impl PBRTextures {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_albedo(mut self, handle: u32) -> Self {
        self.albedo_map = Some(handle);
        self
    }

    pub fn with_normal(mut self, handle: u32) -> Self {
        self.normal_map = Some(handle);
        self
    }

    pub fn with_metallic_roughness(mut self, handle: u32) -> Self {
        self.metallic_roughness_map = Some(handle);
        self
    }

    pub fn with_ao(mut self, handle: u32) -> Self {
        self.ao_map = Some(handle);
        self
    }

    pub fn with_emissive(mut self, handle: u32) -> Self {
        self.emissive_map = Some(handle);
        self
    }
}

// =============================================================================
// IBL (Image-Based Lighting) support
// =============================================================================

/// IBL environment data
#[derive(Clone, Debug)]
pub struct IBLEnvironment {
    pub irradiance_map: Option<u32>,
    pub prefilter_map: Option<u32>,
    pub brdf_lut: Option<u32>,
    pub intensity: f32,
    pub rotation: f32,
}

impl Default for IBLEnvironment {
    fn default() -> Self {
        Self {
            irradiance_map: None,
            prefilter_map: None,
            brdf_lut: None,
            intensity: 1.0,
            rotation: 0.0,
        }
    }
}

impl IBLEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

/// IBL uniform data for shaders
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct IBLUniformData {
    pub intensity: f32,
    pub rotation: f32,
    pub max_reflection_lod: f32,
    pub _padding: f32,
}

// =============================================================================
// PBR Helper Functions
// =============================================================================

/// Fresnel-Schlick approximation
pub fn fresnel_schlick(cos_theta: f32, f0: Vec3) -> Vec3 {
    f0 + (Vec3::ONE - f0) * (1.0 - cos_theta).clamp(0.0, 1.0).powf(5.0)
}

/// GGX/Trowbridge-Reitz normal distribution function
pub fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h2 = n_dot_h * n_dot_h;
    
    let denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    a2 / (std::f32::consts::PI * denom * denom)
}

/// Smith's geometry function (Schlick-GGX)
pub fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    n_dot_v / (n_dot_v * (1.0 - k) + k)
}

/// Smith's geometry function for both view and light directions
pub fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    geometry_schlick_ggx(n_dot_v, roughness) * geometry_schlick_ggx(n_dot_l, roughness)
}

/// Calculate F0 (base reflectivity) for a material
pub fn calculate_f0(albedo: Vec3, metallic: f32) -> Vec3 {
    let dielectric_f0 = Vec3::splat(0.04);
    dielectric_f0.lerp(albedo, metallic)
}

/// Cook-Torrance BRDF
pub fn cook_torrance_brdf(
    normal: Vec3,
    view_dir: Vec3,
    light_dir: Vec3,
    albedo: Vec3,
    metallic: f32,
    roughness: f32,
) -> Vec3 {
    let h = (view_dir + light_dir).normalize();
    
    let n_dot_v = normal.dot(view_dir).max(0.0);
    let n_dot_l = normal.dot(light_dir).max(0.0);
    let n_dot_h = normal.dot(h).max(0.0);
    let h_dot_v = h.dot(view_dir).max(0.0);
    
    let f0 = calculate_f0(albedo, metallic);
    
    // Cook-Torrance BRDF
    let d = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, roughness);
    let f = fresnel_schlick(h_dot_v, f0);
    
    let numerator = d * g * f;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
    let specular = numerator / denominator;
    
    // Diffuse (Lambert)
    let k_d = (Vec3::ONE - f) * (1.0 - metallic);
    let diffuse = k_d * albedo / std::f32::consts::PI;
    
    (diffuse + specular) * n_dot_l
}
