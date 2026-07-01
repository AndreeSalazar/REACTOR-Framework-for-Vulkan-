use bytemuck::{Pod, Zeroable};
use std::mem;

// ═══════════════════════════════════════════════════════════════════════════
// Post-Process Effect Types
// ═══════════════════════════════════════════════════════════════════════════

/// Post-processing effect types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PostProcessEffect {
    None,
    Grayscale,
    Sepia,
    Invert,
    Vignette,
    ChromaticAberration,
    FilmGrain,
    Sharpen,
    Blur,
    Bloom,
    ToneMapping,
    FXAA,
    SMAA,
    TAA,
    SSGI,
    VolumetricFog,
    LutColorGrading,
    SSR,
    PathTracedLighting,
    AnamorphicFlares,
    ContactShadows,
    SSSDiffusion,
    DepthOfField,
    AutoExposure,
    MotionBlur,
    GTAO,
}

/// Anti-Aliasing quality presets
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AAQualityPreset {
    Off,
    Low,
    Medium,
    High,
    Ultra,
    Cinematic,
}

/// Configuración de Anti-Aliasing
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AASettings {
    pub quality: AAQualityPreset,
    pub edge_width: f32,
    pub smoothness: f32,
    pub edge_threshold: f32,
    pub edge_threshold_min: f32,
    pub temporal_blend: f32,
    pub subpixel_aa: bool,
    pub gamma_correct: bool,
}

impl Default for AASettings {
    fn default() -> Self {
        Self {
            quality: AAQualityPreset::High,
            edge_width: 1.5,
            smoothness: 1.0,
            edge_threshold: 0.125,
            edge_threshold_min: 0.0625,
            temporal_blend: 0.15,
            subpixel_aa: true,
            gamma_correct: true,
        }
    }
}

impl AASettings {
    pub fn low() -> Self {
        Self {
            quality: AAQualityPreset::Low,
            edge_width: 1.0,
            smoothness: 0.8,
            edge_threshold: 0.166,
            edge_threshold_min: 0.0833,
            temporal_blend: 0.0,
            subpixel_aa: false,
            gamma_correct: false,
        }
    }

    pub fn medium() -> Self {
        Self {
            quality: AAQualityPreset::Medium,
            edge_width: 1.2,
            smoothness: 1.0,
            edge_threshold: 0.125,
            edge_threshold_min: 0.0625,
            temporal_blend: 0.0,
            subpixel_aa: true,
            gamma_correct: true,
        }
    }

    pub fn high() -> Self {
        Self::default()
    }

    pub fn ultra() -> Self {
        Self {
            quality: AAQualityPreset::Ultra,
            edge_width: 2.0,
            smoothness: 1.5,
            edge_threshold: 0.1,
            edge_threshold_min: 0.05,
            temporal_blend: 0.2,
            subpixel_aa: true,
            gamma_correct: true,
        }
    }

    pub fn cinematic() -> Self {
        Self {
            quality: AAQualityPreset::Cinematic,
            edge_width: 2.5,
            smoothness: 2.0,
            edge_threshold: 0.08,
            edge_threshold_min: 0.04,
            temporal_blend: 0.25,
            subpixel_aa: true,
            gamma_correct: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PostProcessSettings (layout-compatible with GPU push constants)
// ═══════════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PostProcessSettings {
    pub vignette_intensity: f32,
    pub vignette_smoothness: f32,
    pub chromatic_intensity: f32,
    pub grain_intensity: f32,
    pub grain_speed: f32,
    pub bloom_threshold: f32,
    pub bloom_intensity: f32,
    pub bloom_blur_size: f32,
    pub exposure: f32,
    pub gamma: f32,
    pub sharpen_intensity: f32,
    pub ssgi_intensity: f32,
    pub ssgi_radius: f32,
    pub fog_density: f32,
    pub fog_scatter: f32,
    pub lut_strength: f32,
    pub ssr_strength: f32,
    pub pathtrace_intensity: f32,
    pub flare_intensity: f32,
    pub highlight_recovery: f32,
    pub pause_overlay_alpha: f32,
    pub pause_page: f32,
    pub pause_selected: f32,
    pub pause_row_count: f32,
    pub time: f32,
    pub depth_near: f32,
    pub depth_far: f32,
    pub effect_mask: u32,
    pub camera_proj_x: f32,
    pub camera_proj_y: f32,
    pub light_dir_x: f32,
    pub light_dir_y: f32,
    pub light_dir_z: f32,
    pub dof_focus_distance: f32,
    pub dof_aperture: f32,
    pub motion_blur_strength: f32,
}

const _: () = assert!(mem::size_of::<PostProcessSettings>() == 144);

impl Default for PostProcessSettings {
    fn default() -> Self {
        let mut settings = Self {
            vignette_intensity: 0.35,
            vignette_smoothness: 0.6,
            chromatic_intensity: 0.0018,
            grain_intensity: 0.006,
            grain_speed: 1.0,
            bloom_threshold: 1.2,
            bloom_intensity: 0.30,
            bloom_blur_size: 4.0,
            exposure: 0.85,
            gamma: 2.2,
            sharpen_intensity: 0.25,
            ssgi_intensity: 0.26,
            ssgi_radius: 8.0,
            fog_density: 0.18,
            fog_scatter: 0.45,
            lut_strength: 0.72,
            ssr_strength: 0.35,
            pathtrace_intensity: 0.58,
            flare_intensity: 0.42,
            highlight_recovery: 0.62,
            pause_overlay_alpha: 0.0,
            pause_page: 0.0,
            pause_selected: 0.0,
            pause_row_count: 0.0,
            time: 0.0,
            depth_near: 0.1,
            depth_far: 1000.0,
            effect_mask: 0,
            camera_proj_x: 1.0,
            camera_proj_y: -1.0,
            light_dir_x: 0.0,
            light_dir_y: 1.0,
            light_dir_z: 0.0,
            dof_focus_distance: 8.0,
            dof_aperture: 0.04,
            motion_blur_strength: 0.6,
        };
        settings.enable_effect(PostProcessEffect::ToneMapping);
        settings.enable_effect(PostProcessEffect::Vignette);
        settings.enable_effect(PostProcessEffect::FilmGrain);
        settings.enable_effect(PostProcessEffect::ChromaticAberration);
        settings.enable_effect(PostProcessEffect::FXAA);
        settings.enable_effect(PostProcessEffect::SSGI);
        settings.enable_effect(PostProcessEffect::VolumetricFog);
        settings.enable_effect(PostProcessEffect::LutColorGrading);
        settings.enable_effect(PostProcessEffect::SSR);
        settings.enable_effect(PostProcessEffect::PathTracedLighting);
        settings.enable_effect(PostProcessEffect::AnamorphicFlares);
        settings.enable_effect(PostProcessEffect::Bloom);
        settings
    }
}

impl PostProcessSettings {
    pub fn enable_effect(&mut self, effect: PostProcessEffect) {
        self.effect_mask |= 1 << (effect as u32);
    }

    pub fn disable_effect(&mut self, effect: PostProcessEffect) {
        self.effect_mask &= !(1 << (effect as u32));
    }

    pub fn is_effect_enabled(&self, effect: PostProcessEffect) -> bool {
        (self.effect_mask & (1 << (effect as u32))) != 0
    }

    pub fn cinematic() -> Self {
        let mut settings = Self::default();
        settings.enable_effect(PostProcessEffect::Vignette);
        settings.enable_effect(PostProcessEffect::ToneMapping);
        settings.enable_effect(PostProcessEffect::FilmGrain);
        settings.enable_effect(PostProcessEffect::Bloom);
        settings.enable_effect(PostProcessEffect::SSGI);
        settings.enable_effect(PostProcessEffect::VolumetricFog);
        settings.enable_effect(PostProcessEffect::LutColorGrading);
        settings.enable_effect(PostProcessEffect::SSR);
        settings.enable_effect(PostProcessEffect::PathTracedLighting);
        settings.enable_effect(PostProcessEffect::AnamorphicFlares);
        settings.enable_effect(PostProcessEffect::ContactShadows);
        settings.enable_effect(PostProcessEffect::SSSDiffusion);
        settings.enable_effect(PostProcessEffect::DepthOfField);
        settings.enable_effect(PostProcessEffect::AutoExposure);
        settings.enable_effect(PostProcessEffect::MotionBlur);
        settings.enable_effect(PostProcessEffect::GTAO);
        settings.vignette_intensity = 0.4;
        settings.grain_intensity = 0.008;
        settings.bloom_threshold = 0.75;
        settings.bloom_intensity = 0.4;
        settings.fog_density = 0.22;
        settings.lut_strength = 0.82;
        settings.flare_intensity = 0.52;
        settings.highlight_recovery = 0.68;
        settings
    }

    pub fn vibrant() -> Self {
        let mut settings = Self::default();
        settings.enable_effect(PostProcessEffect::ToneMapping);
        settings.enable_effect(PostProcessEffect::Bloom);
        settings.enable_effect(PostProcessEffect::Sharpen);
        settings.exposure = 1.2;
        settings.bloom_intensity = 0.3;
        settings
    }

    pub fn retro() -> Self {
        let mut settings = Self::default();
        settings.enable_effect(PostProcessEffect::Sepia);
        settings.enable_effect(PostProcessEffect::Vignette);
        settings.enable_effect(PostProcessEffect::FilmGrain);
        settings.vignette_intensity = 0.5;
        settings.grain_intensity = 0.1;
        settings
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PostProcessPreset & AutoExposureParams
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug)]
pub enum PostProcessPreset {
    None,
    Cinematic,
    Vibrant,
    Retro,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct AutoExposureParams {
    pub dt: f32,
    pub speed: f32,
    pub target_luminance: f32,
    pub max_exposure: f32,
    pub min_exposure: f32,
}
