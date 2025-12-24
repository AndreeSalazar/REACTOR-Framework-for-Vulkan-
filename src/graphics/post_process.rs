use bytemuck::{Pod, Zeroable};

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
}

/// Post-processing settings passed to shaders
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PostProcessSettings {
    // Vignette
    pub vignette_intensity: f32,
    pub vignette_smoothness: f32,
    
    // Chromatic Aberration
    pub chromatic_intensity: f32,
    
    // Film Grain
    pub grain_intensity: f32,
    pub grain_speed: f32,
    
    // Bloom
    pub bloom_threshold: f32,
    pub bloom_intensity: f32,
    pub bloom_blur_size: f32,
    
    // Tone Mapping
    pub exposure: f32,
    pub gamma: f32,
    
    // Sharpen
    pub sharpen_intensity: f32,
    
    // General
    pub time: f32,
    pub effect_mask: u32, // Bitflags for enabled effects
    
    pub _padding: [f32; 2],
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            vignette_intensity: 0.3,
            vignette_smoothness: 0.5,
            chromatic_intensity: 0.005,
            grain_intensity: 0.05,
            grain_speed: 1.0,
            bloom_threshold: 1.0,
            bloom_intensity: 0.5,
            bloom_blur_size: 4.0,
            exposure: 1.0,
            gamma: 2.2,
            sharpen_intensity: 0.3,
            time: 0.0,
            effect_mask: 0,
            _padding: [0.0, 0.0],
        }
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
        settings.vignette_intensity = 0.4;
        settings.grain_intensity = 0.03;
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

/// Post-processing pipeline manager
pub struct PostProcessPipeline {
    pub settings: PostProcessSettings,
    pub enabled: bool,
}

impl PostProcessPipeline {
    pub fn new() -> Self {
        Self {
            settings: PostProcessSettings::default(),
            enabled: true,
        }
    }

    pub fn with_preset(preset: PostProcessPreset) -> Self {
        Self {
            settings: match preset {
                PostProcessPreset::None => PostProcessSettings::default(),
                PostProcessPreset::Cinematic => PostProcessSettings::cinematic(),
                PostProcessPreset::Vibrant => PostProcessSettings::vibrant(),
                PostProcessPreset::Retro => PostProcessSettings::retro(),
            },
            enabled: true,
        }
    }

    pub fn update_time(&mut self, time: f32) {
        self.settings.time = time;
    }
}

impl Default for PostProcessPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PostProcessPreset {
    None,
    Cinematic,
    Vibrant,
    Retro,
}
