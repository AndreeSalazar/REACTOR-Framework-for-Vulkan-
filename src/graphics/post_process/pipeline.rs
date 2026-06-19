use crate::core::VulkanContext;
use crate::graphics::{Buffer, Image};
use ash::vk;
use gpu_allocator::MemoryLocation;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::vulkan::Allocator;
use std::sync::{Arc, Mutex};

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
    /// Sin AA
    Off,
    /// FXAA básico - rápido
    Low,
    /// FXAA mejorado
    Medium,
    /// SMAA - alta calidad
    High,
    /// SMAA + TAA - máxima calidad
    Ultra,
    /// Cinematográfico - calidad de película
    Cinematic,
}

/// Configuración de Anti-Aliasing
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AASettings {
    /// Preset de calidad
    pub quality: AAQualityPreset,
    /// Ancho del borde de suavizado (1.0 - 3.0)
    pub edge_width: f32,
    /// Intensidad del suavizado (0.0 - 1.0)
    pub smoothness: f32,
    /// Umbral de detección de bordes (0.0 - 0.5)
    pub edge_threshold: f32,
    /// Umbral mínimo de bordes
    pub edge_threshold_min: f32,
    /// Factor de mezcla temporal (para TAA)
    pub temporal_blend: f32,
    /// Habilitar corrección de subpixel
    pub subpixel_aa: bool,
    /// Habilitar corrección de gamma
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
    /// Preset de baja calidad (máximo rendimiento)
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

    /// Preset de calidad media
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

    /// Preset de alta calidad
    pub fn high() -> Self {
        Self::default()
    }

    /// Preset ultra (máxima calidad)
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

    /// Preset cinematográfico
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

    // Screen-space lighting
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

    // General
    pub time: f32,
    pub depth_near: f32,
    pub depth_far: f32,
    pub effect_mask: u32, // Bitflags for enabled effects
    pub camera_proj_x: f32,
    pub camera_proj_y: f32,
    pub light_dir_x: f32,
    pub light_dir_y: f32,
    pub light_dir_z: f32,

    // Depth of Field (Feature A)
    pub dof_focus_distance: f32,
    pub dof_aperture: f32,

    // Motion Blur (Feature F)
    pub motion_blur_strength: f32,
}

const _: () = assert!(std::mem::size_of::<PostProcessSettings>() == 144);

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
            dof_focus_distance: 8.0,  // Focus 8 units away by default
            dof_aperture: 0.04,        // Subtle default blur strength
            motion_blur_strength: 0.6,  // Moderate motion blur strength
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

/// Post-processing pipeline manager
pub struct PostProcessPipeline {
    pub settings: PostProcessSettings,
    pub enabled: bool,

    pub pipeline: Option<vk::Pipeline>,
    pub layout: Option<vk::PipelineLayout>,
    pub descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub descriptor_pool: Option<vk::DescriptorPool>,
    pub descriptor_sets: Vec<vk::DescriptorSet>,

    pub offscreen_images: Vec<crate::graphics::Image>,
    pub sampler: Option<vk::Sampler>,
    pub depth_resolve_pipeline: Option<crate::compute::ComputePipeline>,
    pub depth_resolve_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub depth_resolve_descriptor_pool: Option<vk::DescriptorPool>,
    pub depth_resolve_sets: Vec<vk::DescriptorSet>,
    pub depth_resolved_images: Vec<crate::graphics::Image>,
    pub depth_resolved_initialized: Vec<bool>,

    // ── Bloom Mip-Chain (Compute) ──
    pub bloom_downsample_pipeline: Option<crate::compute::ComputePipeline>,
    pub bloom_upsample_pipeline: Option<crate::compute::ComputePipeline>,
    pub bloom_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub bloom_descriptor_pool: Option<vk::DescriptorPool>,
    pub bloom_images: Vec<crate::graphics::Image>,
    pub bloom_mip_views_sampled: Vec<Vec<vk::ImageView>>,
    pub bloom_mip_views_storage: Vec<Vec<vk::ImageView>>,
    pub bloom_downsample_sets: Vec<Vec<vk::DescriptorSet>>,
    pub bloom_upsample_sets: Vec<Vec<vk::DescriptorSet>>,

    // ── Auto-Exposure (Compute) ──
    pub auto_exposure_pipeline: Option<crate::compute::ComputePipeline>,
    pub exposure_buffers: Vec<crate::graphics::Buffer>,
    pub last_time: f32,
    pub delta_time: f32,

    // ── Temporal Anti-Aliasing (TAA Compute) ──
    pub taa_pipeline: Option<crate::compute::ComputePipeline>,
    pub taa_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub taa_descriptor_pool: Option<vk::DescriptorPool>,
    pub taa_descriptor_sets: Vec<vk::DescriptorSet>,

    // ── Volumetric Fog (Compute) ──
    pub fog_pipeline: Option<crate::compute::ComputePipeline>,
    pub fog_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub fog_descriptor_pool: Option<vk::DescriptorPool>,
    pub fog_descriptor_sets: Vec<vk::DescriptorSet>,
    pub fog_output_images: Vec<crate::graphics::Image>,

    pub lens_flare_pipeline: Option<crate::compute::ComputePipeline>,
    pub lens_flare_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub lens_flare_descriptor_pool: Option<vk::DescriptorPool>,
    pub lens_flare_descriptor_sets: Vec<vk::DescriptorSet>,
    pub lens_flare_output_images: Vec<crate::graphics::Image>,

    // ── GTAO (Compute) ──
    pub gtao_pipeline: Option<crate::compute::ComputePipeline>,
    pub gtao_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub gtao_descriptor_pool: Option<vk::DescriptorPool>,
    pub gtao_descriptor_sets: Vec<vk::DescriptorSet>,
    pub gtao_ao_images: Vec<crate::graphics::Image>,
    pub gtao_initialized: Vec<bool>,

    // ── Light Culling (Compute) ──
    pub light_cull_pipeline: Option<crate::compute::ComputePipeline>,
    pub light_cull_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub light_cull_descriptor_pool: Option<vk::DescriptorPool>,
    pub light_cull_descriptor_sets: Vec<vk::DescriptorSet>,
    pub light_cull_tile_buffer: Option<crate::graphics::Buffer>,
    pub light_cull_index_buffer: Option<crate::graphics::Buffer>,
    pub light_cull_atomic_buffer: Option<crate::graphics::Buffer>,
    pub light_cull_light_buffer: Option<crate::graphics::Buffer>,

    pub lut_texture: Option<crate::resources::texture::Texture>,
    pub device: Option<crate::core::arc_handle::ArcDevice>,
}

impl PostProcessPipeline {
    pub fn new() -> Self {
        Self {
            settings: PostProcessSettings::default(),
            enabled: true,
            pipeline: None,
            layout: None,
            descriptor_layout: None,
            descriptor_pool: None,
            descriptor_sets: Vec::new(),
            offscreen_images: Vec::new(),
            sampler: None,
            depth_resolve_pipeline: None,
            depth_resolve_descriptor_layout: None,
            depth_resolve_descriptor_pool: None,
            depth_resolve_sets: Vec::new(),
            depth_resolved_images: Vec::new(),
            depth_resolved_initialized: Vec::new(),
            bloom_downsample_pipeline: None,
            bloom_upsample_pipeline: None,
            bloom_descriptor_layout: None,
            bloom_descriptor_pool: None,
            bloom_images: Vec::new(),
            bloom_mip_views_sampled: Vec::new(),
            bloom_mip_views_storage: Vec::new(),
            bloom_downsample_sets: Vec::new(),
            bloom_upsample_sets: Vec::new(),
            auto_exposure_pipeline: None,
            exposure_buffers: Vec::new(),
            last_time: 0.0,
            delta_time: 0.0166,
            taa_pipeline: None,
            taa_descriptor_layout: None,
            taa_descriptor_pool: None,
            taa_descriptor_sets: Vec::new(),
            fog_pipeline: None,
            fog_descriptor_layout: None,
            fog_descriptor_pool: None,
            fog_descriptor_sets: Vec::new(),
            fog_output_images: Vec::new(),
            lens_flare_pipeline: None,
            lens_flare_descriptor_layout: None,
            lens_flare_descriptor_pool: None,
            lens_flare_descriptor_sets: Vec::new(),
            lens_flare_output_images: Vec::new(),
            gtao_pipeline: None,
            gtao_descriptor_layout: None,
            gtao_descriptor_pool: None,
            gtao_descriptor_sets: Vec::new(),
            gtao_ao_images: Vec::new(),
            gtao_initialized: Vec::new(),
            light_cull_pipeline: None,
            light_cull_descriptor_layout: None,
            light_cull_descriptor_pool: None,
            light_cull_descriptor_sets: Vec::new(),
            light_cull_tile_buffer: None,
            light_cull_index_buffer: None,
            light_cull_atomic_buffer: None,
            light_cull_light_buffer: None,
            lut_texture: None,
            device: None,
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
            pipeline: None,
            layout: None,
            descriptor_layout: None,
            descriptor_pool: None,
            descriptor_sets: Vec::new(),
            offscreen_images: Vec::new(),
            sampler: None,
            depth_resolve_pipeline: None,
            depth_resolve_descriptor_layout: None,
            depth_resolve_descriptor_pool: None,
            depth_resolve_sets: Vec::new(),
            depth_resolved_images: Vec::new(),
            depth_resolved_initialized: Vec::new(),
            bloom_downsample_pipeline: None,
            bloom_upsample_pipeline: None,
            bloom_descriptor_layout: None,
            bloom_descriptor_pool: None,
            bloom_images: Vec::new(),
            bloom_mip_views_sampled: Vec::new(),
            bloom_mip_views_storage: Vec::new(),
            bloom_downsample_sets: Vec::new(),
            bloom_upsample_sets: Vec::new(),
            auto_exposure_pipeline: None,
            exposure_buffers: Vec::new(),
            last_time: 0.0,
            delta_time: 0.0166,
            taa_pipeline: None,
            taa_descriptor_layout: None,
            taa_descriptor_pool: None,
            taa_descriptor_sets: Vec::new(),
            fog_pipeline: None,
            fog_descriptor_layout: None,
            fog_descriptor_pool: None,
            fog_descriptor_sets: Vec::new(),
            fog_output_images: Vec::new(),
            lens_flare_pipeline: None,
            lens_flare_descriptor_layout: None,
            lens_flare_descriptor_pool: None,
            lens_flare_descriptor_sets: Vec::new(),
            lens_flare_output_images: Vec::new(),
            gtao_pipeline: None,
            gtao_descriptor_layout: None,
            gtao_descriptor_pool: None,
            gtao_descriptor_sets: Vec::new(),
            gtao_ao_images: Vec::new(),
            gtao_initialized: Vec::new(),
            light_cull_pipeline: None,
            light_cull_descriptor_layout: None,
            light_cull_descriptor_pool: None,
            light_cull_descriptor_sets: Vec::new(),
            light_cull_tile_buffer: None,
            light_cull_index_buffer: None,
            light_cull_atomic_buffer: None,
            light_cull_light_buffer: None,
            lut_texture: None,
            device: None,
        }
    }

    pub fn update_time(&mut self, time: f32) {
        let raw_dt = time - self.last_time;
        self.delta_time = if raw_dt > 0.0 && raw_dt < 2.0 { raw_dt } else { 0.0166 };
        self.last_time = time;
        self.settings.time = time;
    }

    pub fn init(
        &mut self,
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
        swapchain_format: vk::Format,
        depth_view: vk::ImageView,
        sample_depth: bool,
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();
        self.device = Some(ctx.device.clone());

        // Initialize neutral LUT texture
        let lut_texture = crate::resources::texture::Texture::neutral_lut(ctx, allocator.clone())?;
        self.lut_texture = Some(lut_texture);

        // 1. Create Descriptor Set Layout
        let pp_bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(4)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(5)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(6)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(7)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(8)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&pp_bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        // 2. Create Pipeline Layout
        let push_range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: std::mem::size_of::<PostProcessSettings>() as u32,
        };
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_layout))
            .push_constant_ranges(std::slice::from_ref(&push_range));
        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        // 3. Create Pipeline
        let vert_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post_process_vert.spv"
        )))
        .unwrap();
        let frag_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post_process_frag.spv"
        )))
        .unwrap();

        let vert_module = unsafe {
            device.create_shader_module(
                &vk::ShaderModuleCreateInfo::default().code(&vert_spv),
                None,
            )?
        };
        let frag_module = unsafe {
            device.create_shader_module(
                &vk::ShaderModuleCreateInfo::default().code(&frag_spv),
                None,
            )?
        };

        let entry_point = std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap();
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_module)
                .name(entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_module)
                .name(entry_point),
        ];

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width, height },
        };
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0);

        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(false)
            .depth_write_enable(false);

        let blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);
        let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&blend_attachment));

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(std::slice::from_ref(&swapchain_format));

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .push_next(&mut rendering_info);

        let pipelines = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, e)| e)?
        };
        let pipeline = pipelines[0];

        unsafe {
            device.destroy_shader_module(vert_module, None);
            device.destroy_shader_module(frag_module, None);
        }

        // 4. Create Descriptor Pool
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count * 7),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        // 5. Allocate Descriptor Sets
        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        self.pipeline = Some(pipeline);
        self.layout = Some(pipeline_layout);
        self.descriptor_layout = Some(descriptor_layout);
        self.descriptor_pool = Some(descriptor_pool);
        self.descriptor_sets = descriptor_sets;

        // 6. Create Offscreen Color Images
        self.recreate_offscreen_images(
            ctx,
            allocator.clone(),
            width,
            height,
            image_count,
            swapchain_format,
            depth_view,
            sample_depth,
        )?;

        Ok(())
    }

    pub fn recreate_offscreen_images(
        &mut self,
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
        format: vk::Format,
        depth_view: vk::ImageView,
        sample_depth: bool,
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();

        // Clean old resources
        self.destroy_bloom_resources(device);
        self.destroy_depth_resolve_resources(device);
        self.destroy_gtao_resources(device);
        self.destroy_light_cull_resources(device);
        self.exposure_buffers.clear();
        self.auto_exposure_pipeline = None;
        self.offscreen_images.clear();
        if let Some(sampler) = self.sampler.take() {
            unsafe {
                device.destroy_sampler(sampler, None);
            }
        }

        // Create linear sampler for offscreen texture
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .anisotropy_enable(false)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR);
        let sampler = unsafe { device.create_sampler(&sampler_info, None)? };
        self.sampler = Some(sampler);

        if !sample_depth {
            self.init_depth_resolve(
                ctx,
                allocator.clone(),
                width,
                height,
                image_count,
                depth_view,
                sampler,
            )?;
        }

        if self.descriptor_sets.len() != image_count as usize {
            if let Some(pool) = self.descriptor_pool.take() {
                unsafe {
                    device.destroy_descriptor_pool(pool, None);
                }
            }

            let descriptor_layout = self.descriptor_layout.ok_or_else(|| {
                crate::core::error::ReactorError::new(
                    crate::core::error::ErrorCode::VulkanDescriptorSet,
                    "post-process descriptor layout is not initialized",
                )
            })?;
            let pool_sizes = [
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(image_count * 9),
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(image_count),
            ];
            let pool_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                .max_sets(image_count)
                .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
            let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };
            let layouts = vec![descriptor_layout; image_count as usize];
            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&layouts);
            self.descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };
            self.descriptor_pool = Some(descriptor_pool);
        }

        // Allocate exposure buffers
        self.exposure_buffers.clear();
        for _ in 0..image_count as usize {
            let buf = Buffer::new(
                ctx,
                allocator.clone(),
                4, // 4 bytes for float
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                MemoryLocation::CpuToGpu,
            )?;
            buf.write(&[1.0f32]); // Write initial exposure 1.0
            self.exposure_buffers.push(buf);
        }

        // Initialize Auto-Exposure compute pipeline
        let ae_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/auto_exposure.spv"
        )))
        .unwrap();
        let ae_pipeline = crate::compute::ComputePipeline::new(
            ctx,
            &ae_spv,
            &[self.descriptor_layout.unwrap()],
            Some(20), // 20 bytes push constant (AutoExposureParams)
        )?;
        self.auto_exposure_pipeline = Some(ae_pipeline);

        // Initialize TAA resolve compute pipeline
        self.destroy_taa_resources(device);
        self.init_taa(ctx, image_count)?;

        for i in 0..image_count as usize {
            let img = Image::new(
                ctx,
                allocator.clone(),
                width,
                height,
                format,
                vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                vk::ImageAspectFlags::COLOR,
                1,
            )?;

            // Update descriptor set
            let image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(img.view)
                .sampler(sampler);

            let depth_or_fallback_view = if sample_depth {
                depth_view
            } else {
                self.depth_resolved_images[i].view
            };
            let depth_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            let depth_info = vk::DescriptorImageInfo::default()
                .image_layout(depth_layout)
                .image_view(depth_or_fallback_view)
                .sampler(sampler);

            let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(self.exposure_buffers[i].handle)
                .offset(0)
                .range(4);

            let lut_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(self.lut_texture.as_ref().unwrap().view())
                .sampler(self.lut_texture.as_ref().unwrap().sampler_handle());

            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&image_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(2)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&depth_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(3)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(&buffer_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(4)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&lut_info)),
            ];

            unsafe {
                device.update_descriptor_sets(&writes, &[]);
            }

            self.offscreen_images.push(img);
        }

        // Bloom descriptors reference the offscreen image views and sampler, so
        // they must be rebuilt whenever swapchain-sized images are recreated.
        self.init_bloom(ctx, allocator.clone(), width, height, image_count)?;

        // Initialize Volumetric Fog compute resources
        self.init_fog(ctx, allocator.clone(), width, height, image_count)?;

        // Initialize Lens Flare compute resources
        self.init_lens_flare(ctx, allocator.clone(), width, height, image_count)?;

        // Initialize Light Culling compute resources (GTAO deferred to first dispatch)
        self.init_light_cull(ctx, allocator.clone(), width, height, image_count, 1024)?;

        // Write binding 6 (fog texture) to each descriptor set
        if !self.fog_output_images.is_empty() {
            let fog_sampler = self.sampler.unwrap();
            for i in 0..image_count as usize {
                let fog_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(self.fog_output_images[i].view)
                    .sampler(fog_sampler);
                let fog_write = vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(6)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&fog_info));
                unsafe {
                    device.update_descriptor_sets(&[fog_write], &[]);
                }
            }
        }

        // Write binding 7 (GTAO AO texture) — placeholder until GTAO is lazily initialized
        {
            let ao_sampler = self.sampler.unwrap();
            for i in 0..image_count as usize {
                let ao_view = if !self.gtao_ao_images.is_empty() {
                    self.gtao_ao_images[i].view
                } else {
                    // Use offscreen image as placeholder
                    self.offscreen_images[i].view
                };
                let ao_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(ao_view)
                    .sampler(ao_sampler);
                let ao_write = vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(7)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&ao_info));
                unsafe {
                    device.update_descriptor_sets(&[ao_write], &[]);
                }
            }
        }

        // Write binding 8 (lens flare) — placeholder until lens_flare_output_images is ready
        {
            let flare_sampler = self.sampler.unwrap();
            for i in 0..image_count as usize {
                let flare_view = if !self.lens_flare_output_images.is_empty() {
                    self.lens_flare_output_images[i].view
                } else {
                    self.offscreen_images[i].view
                };
                let flare_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(flare_view)
                    .sampler(flare_sampler);
                let flare_write = vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(8)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&flare_info));
                unsafe {
                    device.update_descriptor_sets(&[flare_write], &[]);
                }
            }
        }

        Ok(())
    }

    fn init_depth_resolve(
        &mut self,
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
        depth_view: vk::ImageView,
        sampler: vk::Sampler,
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/depth_resolve.spv"
        )))
        .unwrap();
        let pipeline =
            crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(4))?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        let mut resolved_images = Vec::with_capacity(image_count as usize);
        for i in 0..image_count as usize {
            let resolved = Image::new(
                ctx,
                allocator.clone(),
                width,
                height,
                vk::Format::R32_SFLOAT,
                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
                vk::ImageAspectFlags::COLOR,
                1,
            )?;

            let input_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(depth_view)
                .sampler(sampler);
            let output_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(resolved.view);

            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i])
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&input_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i])
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .image_info(std::slice::from_ref(&output_info)),
            ];
            unsafe {
                device.update_descriptor_sets(&writes, &[]);
            }

            resolved_images.push(resolved);
        }

        self.depth_resolve_pipeline = Some(pipeline);
        self.depth_resolve_descriptor_layout = Some(descriptor_layout);
        self.depth_resolve_descriptor_pool = Some(descriptor_pool);
        self.depth_resolve_sets = descriptor_sets;
        self.depth_resolved_images = resolved_images;
        self.depth_resolved_initialized = vec![false; image_count as usize];

        Ok(())
    }

    fn destroy_depth_resolve_resources(&mut self, device: &ash::Device) {
        self.depth_resolve_sets.clear();
        self.depth_resolve_pipeline = None;

        unsafe {
            if let Some(pool) = self.depth_resolve_descriptor_pool.take() {
                device.destroy_descriptor_pool(pool, None);
            }
            if let Some(layout) = self.depth_resolve_descriptor_layout.take() {
                device.destroy_descriptor_set_layout(layout, None);
            }
        }

        self.depth_resolved_images.clear();
        self.depth_resolved_initialized.clear();
    }

    fn destroy_bloom_resources(&mut self, device: &ash::Device) {
        self.bloom_downsample_sets.clear();
        self.bloom_upsample_sets.clear();
        self.bloom_downsample_pipeline = None;
        self.bloom_upsample_pipeline = None;

        unsafe {
            for views in self.bloom_mip_views_sampled.drain(..) {
                for view in views {
                    device.destroy_image_view(view, None);
                }
            }
            for views in self.bloom_mip_views_storage.drain(..) {
                for view in views {
                    device.destroy_image_view(view, None);
                }
            }
            if let Some(pool) = self.bloom_descriptor_pool.take() {
                device.destroy_descriptor_pool(pool, None);
            }
            if let Some(layout) = self.bloom_descriptor_layout.take() {
                device.destroy_descriptor_set_layout(layout, None);
            }
        }

        self.bloom_images.clear();
    }

    pub fn dispatch_depth_resolve(
        &mut self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        width: u32,
        height: u32,
        sample_count: vk::SampleCountFlags,
    ) {
        let Some(pipeline) = self.depth_resolve_pipeline.as_ref() else {
            return;
        };
        let Some(resolved) = self.depth_resolved_images.get(image_index) else {
            return;
        };
        let Some(descriptor_set) = self.depth_resolve_sets.get(image_index) else {
            return;
        };

        let initialized = self
            .depth_resolved_initialized
            .get(image_index)
            .copied()
            .unwrap_or(false);

        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(if initialized {
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            } else {
                vk::ImageLayout::UNDEFINED
            })
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(if initialized {
                vk::AccessFlags::SHADER_READ
            } else {
                vk::AccessFlags::empty()
            })
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(resolved.handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                if initialized {
                    vk::PipelineStageFlags::FRAGMENT_SHADER
                } else {
                    vk::PipelineStageFlags::TOP_OF_PIPE
                },
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_general],
            );

            pipeline.bind(command_buffer, device);
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout,
                0,
                &[*descriptor_set],
                &[],
            );

            let sample_count = sample_count.as_raw();
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                &sample_count.to_ne_bytes(),
            );
            device.cmd_dispatch(command_buffer, (width + 15) / 16, (height + 15) / 16, 1);

            let ready_for_sampling = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(resolved.handle)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );

            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[ready_for_sampling],
            );
        }

        if let Some(initialized) = self.depth_resolved_initialized.get_mut(image_index) {
            *initialized = true;
        }
    }

    /// Initialize bloom compute pipeline with mip-chain for physical bloom.
    /// Creates bloom images with multiple mip levels, per-mip image views,
    /// compute pipelines for downsample/upsample, and all descriptor sets.
    pub fn init_bloom(
        &mut self,
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();
        let sampler = match self.sampler {
            Some(s) => s,
            None => return Ok(()), // No sampler = post-process not ready
        };

        // Calculate bloom base resolution (half of scene) and mip count
        let bloom_w = (width / 2).max(1);
        let bloom_h = (height / 2).max(1);
        let mip_count = ((bloom_w.min(bloom_h) as f32).log2().floor() as u32)
            .max(1)
            .min(6);

        // 1. Create bloom compute descriptor set layout
        //    binding 0: COMBINED_IMAGE_SAMPLER (input texture)
        //    binding 1: STORAGE_IMAGE (output image)
        let bloom_bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let bloom_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&bloom_bindings)
                .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let bloom_desc_layout =
            unsafe { device.create_descriptor_set_layout(&bloom_layout_info, None)? };

        // 2. Create bloom compute pipelines from pre-compiled SPIR-V
        let down_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/bloom_downsample.spv"
        )))
        .unwrap();
        let up_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/bloom_upsample.spv"
        )))
        .unwrap();

        let down_pipeline = crate::compute::ComputePipeline::new(
            ctx,
            &down_spv,
            &[bloom_desc_layout],
            Some(16), // BloomParams: vec2 + int + float = 16 bytes
        )?;
        let up_pipeline = crate::compute::ComputePipeline::new(
            ctx,
            &up_spv,
            &[bloom_desc_layout],
            Some(12), // UpsampleParams: vec2 + float = 12 bytes
        )?;

        // 3. Create bloom images (one per swapchain image) with mip levels
        let mut bloom_images = Vec::with_capacity(image_count as usize);
        let mut mip_views_sampled: Vec<Vec<vk::ImageView>> =
            Vec::with_capacity(image_count as usize);
        let mut mip_views_storage: Vec<Vec<vk::ImageView>> =
            Vec::with_capacity(image_count as usize);

        for _ in 0..image_count {
            let bloom_img = Image::new(
                ctx,
                allocator.clone(),
                bloom_w,
                bloom_h,
                vk::Format::R16G16B16A16_SFLOAT,
                vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE,
                vk::ImageAspectFlags::COLOR,
                mip_count,
            )?;

            // Create per-mip image views for sampling and storage access
            let mut sampled_views = Vec::with_capacity(mip_count as usize);
            let mut storage_views = Vec::with_capacity(mip_count as usize);

            for mip in 0..mip_count {
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(bloom_img.handle)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(vk::Format::R16G16B16A16_SFLOAT)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(mip)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    );

                let sampled_view = unsafe { device.create_image_view(&view_info, None)? };
                let storage_view = unsafe { device.create_image_view(&view_info, None)? };

                sampled_views.push(sampled_view);
                storage_views.push(storage_view);
            }

            mip_views_sampled.push(sampled_views);
            mip_views_storage.push(storage_views);
            bloom_images.push(bloom_img);
        }

        // 4. Create bloom descriptor pool
        let total_sets = (2 * mip_count - 1) * image_count;
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(total_sets),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(total_sets),
        ];
        let bloom_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(total_sets)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let bloom_desc_pool = unsafe { device.create_descriptor_pool(&bloom_pool_info, None)? };

        // 5. Allocate and write bloom descriptor sets
        let mut downsample_sets: Vec<Vec<vk::DescriptorSet>> =
            Vec::with_capacity(image_count as usize);
        let mut upsample_sets: Vec<Vec<vk::DescriptorSet>> =
            Vec::with_capacity(image_count as usize);

        for img_idx in 0..image_count as usize {
            // ── Downsample descriptor sets (one per mip level) ──
            let down_layouts = vec![bloom_desc_layout; mip_count as usize];
            let down_alloc = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(bloom_desc_pool)
                .set_layouts(&down_layouts);
            let down_ds = unsafe { device.allocate_descriptor_sets(&down_alloc)? };

            for mip in 0..mip_count as usize {
                // Binding 0: input texture (scene for mip 0, previous bloom mip otherwise)
                let input_info = if mip == 0 {
                    vk::DescriptorImageInfo::default()
                        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(self.offscreen_images[img_idx].view)
                        .sampler(sampler)
                } else {
                    vk::DescriptorImageInfo::default()
                        .image_layout(vk::ImageLayout::GENERAL)
                        .image_view(mip_views_sampled[img_idx][mip - 1])
                        .sampler(sampler)
                };

                // Binding 1: output storage image (current bloom mip)
                let output_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::GENERAL)
                    .image_view(mip_views_storage[img_idx][mip]);

                let writes = [
                    vk::WriteDescriptorSet::default()
                        .dst_set(down_ds[mip])
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&input_info)),
                    vk::WriteDescriptorSet::default()
                        .dst_set(down_ds[mip])
                        .dst_binding(1)
                        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                        .image_info(std::slice::from_ref(&output_info)),
                ];
                unsafe {
                    device.update_descriptor_sets(&writes, &[]);
                }
            }
            downsample_sets.push(down_ds);

            // ── Upsample descriptor sets (N-1 passes: smallest → largest) ──
            if mip_count > 1 {
                let up_count = (mip_count - 1) as usize;
                let up_layouts = vec![bloom_desc_layout; up_count];
                let up_alloc = vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(bloom_desc_pool)
                    .set_layouts(&up_layouts);
                let up_ds = unsafe { device.allocate_descriptor_sets(&up_alloc)? };

                for pass in 0..up_count {
                    let src_mip = mip_count as usize - 1 - pass;
                    let dst_mip = src_mip - 1;

                    let input_info = vk::DescriptorImageInfo::default()
                        .image_layout(vk::ImageLayout::GENERAL)
                        .image_view(mip_views_sampled[img_idx][src_mip])
                        .sampler(sampler);

                    let output_info = vk::DescriptorImageInfo::default()
                        .image_layout(vk::ImageLayout::GENERAL)
                        .image_view(mip_views_storage[img_idx][dst_mip]);

                    let writes = [
                        vk::WriteDescriptorSet::default()
                            .dst_set(up_ds[pass])
                            .dst_binding(0)
                            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .image_info(std::slice::from_ref(&input_info)),
                        vk::WriteDescriptorSet::default()
                            .dst_set(up_ds[pass])
                            .dst_binding(1)
                            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                            .image_info(std::slice::from_ref(&output_info)),
                    ];
                    unsafe {
                        device.update_descriptor_sets(&writes, &[]);
                    }
                }
                upsample_sets.push(up_ds);
            } else {
                upsample_sets.push(Vec::new());
            }
        }

        // 6. Write bloom result (mip 0) to post-process descriptor sets at binding 1
        for img_idx in 0..image_count as usize {
            let bloom_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(mip_views_sampled[img_idx][0])
                .sampler(sampler);

            let write = vk::WriteDescriptorSet::default()
                .dst_set(self.descriptor_sets[img_idx])
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&bloom_info));

            unsafe {
                device.update_descriptor_sets(&[write], &[]);
            }
        }

        // Store all bloom resources
        self.bloom_downsample_pipeline = Some(down_pipeline);
        self.bloom_upsample_pipeline = Some(up_pipeline);
        self.bloom_descriptor_layout = Some(bloom_desc_layout);
        self.bloom_descriptor_pool = Some(bloom_desc_pool);
        self.bloom_images = bloom_images;
        self.bloom_mip_views_sampled = mip_views_sampled;
        self.bloom_mip_views_storage = mip_views_storage;
        self.bloom_downsample_sets = downsample_sets;
        self.bloom_upsample_sets = upsample_sets;

        Ok(())
    }

    /// Record bloom compute commands: downsample mip-chain then upsample with
    /// progressive additive blending (COD: Advanced Warfare technique).
    ///
    /// Call after the offscreen scene image has been transitioned to SHADER_READ_ONLY_OPTIMAL.
    pub fn dispatch_bloom(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        scene_width: u32,
        scene_height: u32,
    ) {
        let mip_count = match self.bloom_mip_views_sampled.get(image_index) {
            Some(v) if !v.is_empty() => v.len() as u32,
            _ => return,
        };

        let bloom_img = &self.bloom_images[image_index];

        // Transition entire bloom image from UNDEFINED → GENERAL for compute
        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(bloom_img.handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(mip_count)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_general],
            );
        }

        // ── Downsample passes (full-res → smallest mip) ──
        let down = self.bloom_downsample_pipeline.as_ref().unwrap();
        down.bind(command_buffer, device);

        let mut out_w = (scene_width / 2).max(1);
        let mut out_h = (scene_height / 2).max(1);

        for mip in 0..mip_count as usize {
            unsafe {
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    down.layout,
                    0,
                    &[self.bloom_downsample_sets[image_index][mip]],
                    &[],
                );
            }

            // Push constants: vec2 texel_size, int mip_level, float threshold
            let input_w = if mip == 0 { scene_width } else { out_w * 2 };
            let input_h = if mip == 0 { scene_height } else { out_h * 2 };
            let texel_x = 1.0f32 / input_w.max(1) as f32;
            let texel_y = 1.0f32 / input_h.max(1) as f32;
            let mip_level = mip as i32;
            let threshold = self.settings.bloom_threshold;

            let mut push = [0u8; 16];
            push[0..4].copy_from_slice(&texel_x.to_ne_bytes());
            push[4..8].copy_from_slice(&texel_y.to_ne_bytes());
            push[8..12].copy_from_slice(&mip_level.to_ne_bytes());
            push[12..16].copy_from_slice(&threshold.to_ne_bytes());

            unsafe {
                device.cmd_push_constants(
                    command_buffer,
                    down.layout,
                    vk::ShaderStageFlags::COMPUTE,
                    0,
                    &push,
                );
                device.cmd_dispatch(command_buffer, (out_w + 15) / 16, (out_h + 15) / 16, 1);
            }

            // Memory barrier: this mip write → next pass read
            let barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::GENERAL)
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(bloom_img.handle)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(mip as u32)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );
            }

            out_w = (out_w / 2).max(1);
            out_h = (out_h / 2).max(1);
        }

        // ── Upsample passes (smallest mip → mip 0) ──
        if mip_count > 1 {
            let up = self.bloom_upsample_pipeline.as_ref().unwrap();
            up.bind(command_buffer, device);

            let upsample_count = (mip_count - 1) as usize;
            for pass in 0..upsample_count {
                let dst_mip = mip_count as usize - 2 - pass;

                unsafe {
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::COMPUTE,
                        up.layout,
                        0,
                        &[self.bloom_upsample_sets[image_index][pass]],
                        &[],
                    );
                }

                // Push constants: vec2 texel_size (of input/source), float filter_radius
                let src_mip = dst_mip + 1;
                let src_w = ((scene_width / 2) >> src_mip).max(1);
                let src_h = ((scene_height / 2) >> src_mip).max(1);
                let texel_x = 1.0f32 / src_w as f32;
                let texel_y = 1.0f32 / src_h as f32;
                let filter_radius = 1.0f32;

                let mut push = [0u8; 12];
                push[0..4].copy_from_slice(&texel_x.to_ne_bytes());
                push[4..8].copy_from_slice(&texel_y.to_ne_bytes());
                push[8..12].copy_from_slice(&filter_radius.to_ne_bytes());

                let dst_w = ((scene_width / 2) >> dst_mip).max(1);
                let dst_h = ((scene_height / 2) >> dst_mip).max(1);

                unsafe {
                    device.cmd_push_constants(
                        command_buffer,
                        up.layout,
                        vk::ShaderStageFlags::COMPUTE,
                        0,
                        &push,
                    );
                    device.cmd_dispatch(command_buffer, (dst_w + 15) / 16, (dst_h + 15) / 16, 1);
                }

                // Barrier for next upsample pass
                let barrier = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::GENERAL)
                    .new_layout(vk::ImageLayout::GENERAL)
                    .src_access_mask(vk::AccessFlags::SHADER_WRITE | vk::AccessFlags::SHADER_READ)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ)
                    .image(bloom_img.handle)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(dst_mip as u32)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    );

                unsafe {
                    device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::COMPUTE_SHADER,
                        vk::PipelineStageFlags::COMPUTE_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[barrier],
                    );
                }
            }
        }

        // Final barrier: bloom mip 0 ready for fragment shader sampling
        let final_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(bloom_img.handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[final_barrier],
            );
        }
    }

    /// Record auto-exposure compute commands to estimate screen average luminance and dynamically update exposure factor.
    pub fn dispatch_auto_exposure(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        dt: f32,
    ) {
        let Some(pipeline) = self.auto_exposure_pipeline.as_ref() else {
            return;
        };

        pipeline.bind(command_buffer, device);

        // Bind the post-process descriptor set containing bindings 0 (input texture) and 3 (exposure buffer)
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout,
                0,
                &[self.descriptor_sets[image_index]],
                &[],
            );
        }

        // Push constants: dt, speed, target_luminance, max_exposure, min_exposure
        let params = AutoExposureParams {
            dt,
            speed: 1.5,             // Speed of eye adaptation (larger = faster)
            target_luminance: 0.18,  // Target average gray (18% reflectance standard)
            max_exposure: 3.5,       // Max exposure limit to prevent darkness being too bright
            min_exposure: 0.2,       // Min exposure limit to prevent light being too dim
        };

        let push_bytes = bytemuck::bytes_of(&params);
        unsafe {
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                push_bytes,
            );

            // Dispatch a single workgroup of 16x16 threads (256 total threads)
            device.cmd_dispatch(command_buffer, 1, 1, 1);
        }

        // Memory barrier to ensure compute buffer writes to exposure buffer are visible to fragment shader reads
        let buffer_barrier = vk::BufferMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(self.exposure_buffers[image_index].handle)
            .offset(0)
            .size(4);

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[buffer_barrier],
                &[],
            );
        }
    }

    /// Initialize TAA resolve compute pipeline, descriptor set layout, and descriptor pool.
    fn init_taa(&mut self, ctx: &VulkanContext, image_count: u32) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();

        // 1. Create Descriptor Set Layout
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(4)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(5)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(6)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        // 2. Create Compute Pipeline (16 bytes push constants for TaaParams)
        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/taa_resolve.spv"
        )))
        .unwrap();
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(16))?;

        // 3. Create Descriptor Pool
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count * 5),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(image_count * 2),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        // 4. Allocate Descriptor Sets
        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        self.taa_pipeline = Some(pipeline);
        self.taa_descriptor_layout = Some(descriptor_layout);
        self.taa_descriptor_pool = Some(descriptor_pool);
        self.taa_descriptor_sets = descriptor_sets;

        Ok(())
    }

    /// Clean up TAA resources
    fn destroy_taa_resources(&mut self, device: &ash::Device) {
        self.taa_descriptor_sets.clear();
        self.taa_pipeline = None;
        unsafe {
            if let Some(pool) = self.taa_descriptor_pool.take() {
                device.destroy_descriptor_pool(pool, None);
            }
            if let Some(layout) = self.taa_descriptor_layout.take() {
                device.destroy_descriptor_set_layout(layout, None);
            }
        }
    }

    fn init_fog(
        &mut self,
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();

        // 1. Create Descriptor Set Layout
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        // 2. Create Compute Pipeline (152 bytes push constants for FogParams)
        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/volumetric_fog.spv"
        )))
        .unwrap();
            let pipeline =
            crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(156))?;

        // 3. Create Descriptor Pool
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count * 2),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        // 4. Allocate Descriptor Sets
        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        // 5. Create Fog Output Images (RGBA16F for scattered light + transmittance)
        self.fog_output_images.clear();
        for _ in 0..image_count as usize {
            let img = Image::new(
                ctx,
                allocator.clone(),
                width,
                height,
                vk::Format::R16G16B16A16_SFLOAT,
                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
                vk::ImageAspectFlags::COLOR,
                1,
            )?;
            self.fog_output_images.push(img);
        }

        self.fog_pipeline = Some(pipeline);
        self.fog_descriptor_layout = Some(descriptor_layout);
        self.fog_descriptor_pool = Some(descriptor_pool);
        self.fog_descriptor_sets = descriptor_sets;

        Ok(())
    }

    fn destroy_fog_resources(&mut self, device: &ash::Device) {
        self.fog_descriptor_sets.clear();
        self.fog_pipeline = None;
        self.fog_output_images.clear();
        unsafe {
            if let Some(pool) = self.fog_descriptor_pool.take() {
                device.destroy_descriptor_pool(pool, None);
            }
            if let Some(layout) = self.fog_descriptor_layout.take() {
                device.destroy_descriptor_set_layout(layout, None);
            }
        }
    }

    pub fn dispatch_volumetric_fog(
        &mut self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        camera_view: glam::Mat4,
        camera_proj: glam::Mat4,
        camera_pos: glam::Vec3,
        sun_direction: glam::Vec3,
        sun_color: glam::Vec3,
        near: f32,
        far: f32,
        time: f32,
    ) {
        let Some(pipeline) = self.fog_pipeline.as_ref() else {
            return;
        };
        let Some(descriptor_set) = self.fog_descriptor_sets.get(image_index) else {
            return;
        };
        let sampler = match self.sampler {
            Some(s) => s,
            None => return,
        };

        let width = self.fog_output_images.first().map(|i| i.extent.width).unwrap_or(1);
        let height = self.fog_output_images.first().map(|i| i.extent.height).unwrap_or(1);

        // Transition fog output image to GENERAL for compute write
        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(self.fog_output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_general],
            );
        }

        // Update descriptor set: binding 0 = depth, binding 1 = scene, binding 2 = fog output
        let depth_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.depth_resolved_images.first().map(|i| i.view).unwrap_or(
                self.offscreen_images.first().map(|i| i.view).unwrap_or(vk::ImageView::null()),
            ))
            .sampler(sampler);

        let scene_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.offscreen_images[image_index].view)
            .sampler(sampler);

        let fog_output_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(self.fog_output_images[image_index].view);

        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(*descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&depth_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(*descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&scene_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(*descriptor_set)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&fog_output_info)),
        ];

        unsafe {
            device.update_descriptor_sets(&writes, &[]);
        }

        // Bind and dispatch
        pipeline.bind(command_buffer, device);
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout,
                0,
                &[*descriptor_set],
                &[],
            );
        }

        let inv_view_proj = (camera_proj * camera_view).inverse();
        let light_dir_view = camera_view.transform_vector3(-sun_direction.normalize()).normalize();

        let mut push_bytes = [0u8; 156];
        let mut offset = 0;

        let write_f32 = |push: &mut [u8; 156], offset: &mut usize, val: f32| {
            push[*offset..*offset + 4].copy_from_slice(&val.to_ne_bytes());
            *offset += 4;
        };
        let write_vec4 = |push: &mut [u8; 156], offset: &mut usize, v: glam::Vec4| {
            write_f32(push, offset, v.x);
            write_f32(push, offset, v.y);
            write_f32(push, offset, v.z);
            write_f32(push, offset, v.w);
        };
        let write_mat4 = |push: &mut [u8; 156], offset: &mut usize, m: glam::Mat4| {
            for col in m.to_cols_array() {
                write_f32(push, offset, col);
            }
        };

        write_mat4(&mut push_bytes, &mut offset, inv_view_proj);
        write_vec4(&mut push_bytes, &mut offset, glam::Vec4::new(camera_pos.x, camera_pos.y, camera_pos.z, 0.0));
        write_vec4(&mut push_bytes, &mut offset, glam::Vec4::new(light_dir_view.x, light_dir_view.y, light_dir_view.z, 1.0));
        write_vec4(&mut push_bytes, &mut offset, glam::Vec4::new(sun_color.x, sun_color.y, sun_color.z, 1.0));
        write_f32(&mut push_bytes, &mut offset, width as f32);
        write_f32(&mut push_bytes, &mut offset, height as f32);
        write_f32(&mut push_bytes, &mut offset, self.settings.fog_density);
        write_f32(&mut push_bytes, &mut offset, self.settings.fog_scatter);
        write_f32(&mut push_bytes, &mut offset, time);
        write_f32(&mut push_bytes, &mut offset, near);
        write_f32(&mut push_bytes, &mut offset, far);
        write_f32(&mut push_bytes, &mut offset, 0.35);
        write_f32(&mut push_bytes, &mut offset, -1.0);
        push_bytes[offset..offset + 4].copy_from_slice(&48u32.to_ne_bytes()); offset += 4;
        push_bytes[offset..offset + 4].copy_from_slice(&0u32.to_ne_bytes());
        unsafe {
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                &push_bytes,
            );

            let dispatch_x = (width + 7) / 8;
            let dispatch_y = (height + 7) / 8;
            device.cmd_dispatch(command_buffer, dispatch_x, dispatch_y, 1);
        }

        // Transition fog output to SHADER_READ_ONLY for fragment sampling
        let to_read = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(self.fog_output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_read],
            );
        }
    }

    fn init_lens_flare(
        &mut self,
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/lens_flare.spv"
        )))
        .unwrap();
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(48))?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        self.lens_flare_output_images.clear();
        for _ in 0..image_count as usize {
            let img = Image::new(
                ctx,
                allocator.clone(),
                width,
                height,
                vk::Format::R16G16B16A16_SFLOAT,
                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
                vk::ImageAspectFlags::COLOR,
                1,
            )?;
            self.lens_flare_output_images.push(img);
        }

        self.lens_flare_pipeline = Some(pipeline);
        self.lens_flare_descriptor_layout = Some(descriptor_layout);
        self.lens_flare_descriptor_pool = Some(descriptor_pool);
        self.lens_flare_descriptor_sets = descriptor_sets;

        Ok(())
    }

    fn destroy_lens_flare_resources(&mut self, device: &ash::Device) {
        self.lens_flare_descriptor_sets.clear();
        self.lens_flare_pipeline = None;
        self.lens_flare_output_images.clear();
        unsafe {
            if let Some(pool) = self.lens_flare_descriptor_pool.take() {
                device.destroy_descriptor_pool(pool, None);
            }
            if let Some(layout) = self.lens_flare_descriptor_layout.take() {
                device.destroy_descriptor_set_layout(layout, None);
            }
        }
    }

    pub fn dispatch_lens_flare(
        &mut self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        width: u32,
        height: u32,
        time: f32,
    ) {
        let Some(pipeline) = self.lens_flare_pipeline.as_ref() else {
            return;
        };
        let Some(descriptor_set) = self.lens_flare_descriptor_sets.get(image_index) else {
            return;
        };
        let sampler = match self.sampler {
            Some(s) => s,
            None => return,
        };

        let bloom_mip = self
            .bloom_mip_views_sampled
            .get(image_index)
            .and_then(|v| v.get(2))
            .copied()
            .unwrap_or_else(|| {
                self.offscreen_images
                    .get(image_index)
                    .map(|i| i.view)
                    .unwrap_or(vk::ImageView::null())
            });

        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(self.lens_flare_output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_general],
            );
        }

        let bloom_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(bloom_mip)
            .sampler(sampler);
        let flare_output_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(self.lens_flare_output_images[image_index].view);

        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(*descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&bloom_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(*descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&flare_output_info)),
        ];
        unsafe {
            device.update_descriptor_sets(&writes, &[]);
        }

        pipeline.bind(command_buffer, device);
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout,
                0,
                &[*descriptor_set],
                &[],
            );
        }

        let mut push_bytes = [0u8; 48];
        push_bytes[0..4].copy_from_slice(&(width as f32).to_ne_bytes());
        push_bytes[4..8].copy_from_slice(&(height as f32).to_ne_bytes());
        push_bytes[8..12].copy_from_slice(&0.37f32.to_ne_bytes());
        push_bytes[12..16].copy_from_slice(&6.0f32.to_ne_bytes());
        push_bytes[16..20].copy_from_slice(&0.85f32.to_ne_bytes());
        push_bytes[20..24].copy_from_slice(&0.45f32.to_ne_bytes());
        push_bytes[24..28].copy_from_slice(&0.06f32.to_ne_bytes());
        push_bytes[28..32].copy_from_slice(&0.02f32.to_ne_bytes());
        push_bytes[32..36].copy_from_slice(&0.5f32.to_ne_bytes());
        push_bytes[36..40].copy_from_slice(&self.settings.flare_intensity.to_ne_bytes());
        push_bytes[40..44].copy_from_slice(&time.to_ne_bytes());
        push_bytes[44..48].copy_from_slice(&0.0f32.to_ne_bytes());

        unsafe {
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                &push_bytes,
            );
            let dispatch_x = (width + 15) / 16;
            let dispatch_y = (height + 15) / 16;
            device.cmd_dispatch(command_buffer, dispatch_x, dispatch_y, 1);
        }

        let to_read = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(self.lens_flare_output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_read],
            );
        }
    }

    /// Record TAA compute resolve dispatch. Reprojects current frame against history color/depth using motion vectors.
    pub fn dispatch_taa(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        history: &crate::graphics::TemporalHistory,
        gbuffer: &crate::graphics::GBuffer,
        depth_view: vk::ImageView,
        reset_history: bool,
    ) {
        let Some(pipeline) = self.taa_pipeline.as_ref() else {
            return;
        };

        // ── 1. Transition layouts dynamically for history ping-pong images ──
        //
        // After each TAA dispatch:
        //   - current color is transitioned GENERAL → SHADER_READ_ONLY (post_resolve_barrier below)
        //   - current depth stays in GENERAL (no post-dispatch barrier)
        //   - previous color/depth stay in SHADER_READ_ONLY (sampled only, no barrier after read)
        //
        // So when roles swap via advance():
        //   new previous (was current): color = SHADER_READ_ONLY, depth = GENERAL
        //   new current  (was previous): both = SHADER_READ_ONLY
        let prev_color_old_layout = if history.frame_index == 0 { vk::ImageLayout::UNDEFINED } else { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL };
        let prev_depth_old_layout = if history.frame_index == 0 { vk::ImageLayout::UNDEFINED } else { vk::ImageLayout::GENERAL };
        let curr_old_layout = if history.frame_index == 0 { vk::ImageLayout::UNDEFINED } else { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL };
        let prev_color_src_access = if history.frame_index == 0 { vk::AccessFlags::empty() } else { vk::AccessFlags::SHADER_READ };
        let prev_depth_src_access = if history.frame_index == 0 { vk::AccessFlags::empty() } else { vk::AccessFlags::SHADER_WRITE };
        let curr_src_access = if history.frame_index == 0 { vk::AccessFlags::empty() } else { vk::AccessFlags::SHADER_READ };
        let src_stage = if history.frame_index == 0 { vk::PipelineStageFlags::TOP_OF_PIPE } else { vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER };

        let prev_color_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(prev_color_old_layout)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(prev_color_src_access)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(history.previous_color().handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        let prev_depth_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(prev_depth_old_layout)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(prev_depth_src_access)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(history.previous_depth().handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR) // R32_SFLOAT storage uses Color aspect
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        let curr_color_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(curr_old_layout)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(curr_src_access)
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(history.current_color().handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        let curr_depth_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(curr_old_layout)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(curr_src_access)
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(history.current_depth().handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                src_stage,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[prev_color_barrier, prev_depth_barrier, curr_color_barrier, curr_depth_barrier],
            );
        }

        // ── 2. Update Descriptor Set dynamically for the current frame views ──
        let sampler = self.sampler.unwrap();

        let current_color_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.offscreen_images[image_index].view)
            .sampler(sampler);

        let history_color_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(history.previous_color().view)
            .sampler(sampler);

        let motion_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(gbuffer.motion_depth_flags.view)
            .sampler(sampler);

        let current_depth_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(depth_view)
            .sampler(sampler);

        let history_depth_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(history.previous_depth().view)
            .sampler(sampler);

        let output_color_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(history.current_color().view);

        let output_depth_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(history.current_depth().view);

        let ds = self.taa_descriptor_sets[image_index];

        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(ds)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&current_color_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(ds)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&history_color_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(ds)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&motion_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(ds)
                .dst_binding(3)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&current_depth_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(ds)
                .dst_binding(4)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&history_depth_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(ds)
                .dst_binding(5)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&output_color_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(ds)
                .dst_binding(6)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&output_depth_info)),
        ];

        unsafe {
            device.update_descriptor_sets(&writes, &[]);
        }

        // ── 3. Bind and dispatch TAA compute shader ──
        pipeline.bind(command_buffer, device);

        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout,
                0,
                &[ds],
                &[],
            );
        }

        // Push constants: history_weight, depth_rejection, motion_scale, reset_history
        let pc = [
            0.90f32,                              // history_weight
            0.003f32,                             // depth_rejection
            1.0f32,                               // motion_scale
            if reset_history { 1.0f32 } else { 0.0f32 }, // reset_history
        ];

        let push_bytes = bytemuck::cast_slice(&pc);
        unsafe {
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                push_bytes,
            );

            let dispatch_x = (history.width + 15) / 16;
            let dispatch_y = (history.height + 15) / 16;
            device.cmd_dispatch(command_buffer, dispatch_x, dispatch_y, 1);
        }

        // ── 4. Transition current color output back to SHADER_READ_ONLY_OPTIMAL for post_process fragment sampling
        let post_resolve_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(history.current_color().handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[post_resolve_barrier],
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════════
    // GTAO — Ground Truth Ambient Occlusion
    // ══════════════════════════════════════════════════════════════════════════

    fn destroy_gtao_resources(&mut self, device: &ash::Device) {
        self.gtao_descriptor_sets.clear();
        self.gtao_pipeline = None;
        self.gtao_ao_images.clear();
        self.gtao_initialized.clear();
        unsafe {
            if let Some(pool) = self.gtao_descriptor_pool.take() {
                device.destroy_descriptor_pool(pool, None);
            }
            if let Some(layout) = self.gtao_descriptor_layout.take() {
                device.destroy_descriptor_set_layout(layout, None);
            }
        }
    }

    pub fn init_gtao(
        &mut self,
        ctx: &crate::core::VulkanContext,
        allocator: std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
        depth_view: vk::ImageView,
        gbuffer_normal_view: vk::ImageView,
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout =
            unsafe { device.create_descriptor_set_layout(&layout_info, None)? };
        self.gtao_descriptor_layout = Some(descriptor_layout);

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/gtao.spv"
        )))
        .unwrap();
        let pipeline =
            crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(32))?;
        self.gtao_pipeline = Some(pipeline);

        let sampler = self.sampler.unwrap();
        self.gtao_ao_images.clear();
        self.gtao_initialized.clear();
        for _ in 0..image_count as usize {
            let img = crate::graphics::Image::new(
                ctx,
                allocator.clone(),
                width,
                height,
                vk::Format::R16_SFLOAT,
                vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE,
                vk::ImageAspectFlags::COLOR,
                1,
            )?;
            self.gtao_ao_images.push(img);
            self.gtao_initialized.push(false);
        }

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count * 2),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };
        self.gtao_descriptor_pool = Some(descriptor_pool);

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        self.gtao_descriptor_sets =
            unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        for i in 0..image_count as usize {
            let depth_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(depth_view)
                .sampler(sampler);
            let normal_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(gbuffer_normal_view)
                .sampler(sampler);
            let ao_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(self.gtao_ao_images[i].view);

            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(self.gtao_descriptor_sets[i])
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&depth_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(self.gtao_descriptor_sets[i])
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&normal_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(self.gtao_descriptor_sets[i])
                    .dst_binding(2)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .image_info(std::slice::from_ref(&ao_info)),
            ];
            unsafe { device.update_descriptor_sets(&writes, &[]); }
        }

        Ok(())
    }

    pub fn dispatch_gtao(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        width: u32,
        height: u32,
        proj_x: f32,
        proj_y: f32,
        near: f32,
        far: f32,
        frame_index: f32,
    ) {
        let Some(pipeline) = self.gtao_pipeline.as_ref() else { return; };
        let Some(descriptor_set) = self.gtao_descriptor_sets.get(image_index) else { return; };
        let Some(ao_image) = self.gtao_ao_images.get(image_index) else { return; };
        let initialized = self.gtao_initialized.get(image_index).copied().unwrap_or(false);

        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(if initialized { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL } else { vk::ImageLayout::UNDEFINED })
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(if initialized { vk::AccessFlags::SHADER_READ } else { vk::AccessFlags::empty() })
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(ao_image.handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));

        unsafe {
            device.cmd_pipeline_barrier(command_buffer,
                if initialized { vk::PipelineStageFlags::FRAGMENT_SHADER } else { vk::PipelineStageFlags::TOP_OF_PIPE },
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(), &[], &[], &[to_general]);

            pipeline.bind(command_buffer, device);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE,
                pipeline.layout, 0, &[*descriptor_set], &[]);

            let push_bytes = [
                proj_x.to_ne_bytes(), proj_y.to_ne_bytes(),
                near.to_ne_bytes(), far.to_ne_bytes(),
                (width as f32 * 0.05).to_ne_bytes(),
                0.5f32.to_ne_bytes(), 1.5f32.to_ne_bytes(),
                frame_index.to_ne_bytes(),
            ].concat();

            device.cmd_push_constants(command_buffer, pipeline.layout,
                vk::ShaderStageFlags::COMPUTE, 0, &push_bytes);
            device.cmd_dispatch(command_buffer, (width + 7) / 8, (height + 7) / 8, 1);

            let ready = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(ao_image.handle)
                .subresource_range(vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));
            device.cmd_pipeline_barrier(command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[], &[], &[ready]);
        }
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Light Culling — Clustered/Tiled Light Culling
    // ══════════════════════════════════════════════════════════════════════════

    fn destroy_light_cull_resources(&mut self, device: &ash::Device) {
        self.light_cull_descriptor_sets.clear();
        self.light_cull_pipeline = None;
        unsafe {
            if let Some(mut buf) = self.light_cull_tile_buffer.take() { buf.destroy(); }
            if let Some(mut buf) = self.light_cull_index_buffer.take() { buf.destroy(); }
            if let Some(mut buf) = self.light_cull_atomic_buffer.take() { buf.destroy(); }
            if let Some(mut buf) = self.light_cull_light_buffer.take() { buf.destroy(); }
            if let Some(pool) = self.light_cull_descriptor_pool.take() {
                device.destroy_descriptor_pool(pool, None);
            }
            if let Some(layout) = self.light_cull_descriptor_layout.take() {
                device.destroy_descriptor_set_layout(layout, None);
            }
        }
    }

    pub fn init_light_cull(
        &mut self,
        ctx: &crate::core::VulkanContext,
        allocator: std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
        max_lights: u32,
    ) -> crate::core::error::ReactorResult<()> {
        use crate::graphics::Buffer;
        let device = ctx.ash_device();
        let tile_count_x = (width + 15) / 16;
        let tile_count_y = (height + 15) / 16;
        let tile_count = (tile_count_x * tile_count_y) as usize;

        let bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(2).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(3).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(4).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };
        self.light_cull_descriptor_layout = Some(descriptor_layout);

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/compute/light_cull.spv"
        ))).unwrap();
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(224))?;
        self.light_cull_pipeline = Some(pipeline);

        let light_buffer = Buffer::new(ctx, allocator.clone(), (max_lights as usize * 32) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER, gpu_allocator::MemoryLocation::CpuToGpu)?;
        self.light_cull_light_buffer = Some(light_buffer);

        let tile_buffer = Buffer::new(ctx, allocator.clone(), (tile_count * 8) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            gpu_allocator::MemoryLocation::GpuOnly)?;
        self.light_cull_tile_buffer = Some(tile_buffer);

        let index_buffer = Buffer::new(ctx, allocator.clone(), (tile_count * 256 * 4) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            gpu_allocator::MemoryLocation::GpuOnly)?;
        self.light_cull_index_buffer = Some(index_buffer);

        let atomic_buffer = Buffer::new(ctx, allocator.clone(), 4,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            gpu_allocator::MemoryLocation::GpuOnly)?;
        self.light_cull_atomic_buffer = Some(atomic_buffer);

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(image_count * 4),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };
        self.light_cull_descriptor_pool = Some(descriptor_pool);

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);
        self.light_cull_descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        // Write initial descriptors so sets are valid before first dispatch
        let light_buf = self.light_cull_light_buffer.as_ref().unwrap().handle;
        let tile_buf = self.light_cull_tile_buffer.as_ref().unwrap().handle;
        let index_buf = self.light_cull_index_buffer.as_ref().unwrap().handle;
        let atomic_buf = self.light_cull_atomic_buffer.as_ref().unwrap().handle;

        for i in 0..image_count as usize {
            let light_info = vk::DescriptorBufferInfo::default().buffer(light_buf).offset(0).range(vk::WHOLE_SIZE);
            let tile_info = vk::DescriptorBufferInfo::default().buffer(tile_buf).offset(0).range(vk::WHOLE_SIZE);
            let index_info = vk::DescriptorBufferInfo::default().buffer(index_buf).offset(0).range(vk::WHOLE_SIZE);
            let atomic_info = vk::DescriptorBufferInfo::default().buffer(atomic_buf).offset(0).range(4);

            // Use SHADER_READ_ONLY_OPTIMAL — matches depth image layout after depth_resolve
            let dummy_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(self.offscreen_images.first().map(|i| i.view).unwrap_or(vk::ImageView::null()))
                .sampler(self.sampler.unwrap_or(vk::Sampler::null()));

            let writes = [
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(std::slice::from_ref(&light_info)),
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(std::slice::from_ref(&tile_info)),
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(2)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(std::slice::from_ref(&index_info)),
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(3)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(std::slice::from_ref(&atomic_info)),
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(4)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&dummy_info)),
            ];
            unsafe { device.update_descriptor_sets(&writes, &[]); }
        }

        Ok(())
    }

    pub fn dispatch_light_cull(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        width: u32,
        height: u32,
        view: glam::Mat4,
        projection: glam::Mat4,
        inv_projection: glam::Mat4,
        light_count: u32,
        depth_view: vk::ImageView,
    ) {
        let light_count = light_count.min(1024);
        self.dispatch_light_cull_inner(
            device,
            command_buffer,
            image_index,
            width,
            height,
            view,
            projection,
            inv_projection,
            light_count,
            depth_view,
        );
    }

    pub fn update_lights(&mut self, lights: &[crate::graphics::post_process::PointLightGpu]) {
        if let Some(buf) = self.light_cull_light_buffer.as_mut() {
            if lights.is_empty() {
                return;
            }
            let max_count = (buf.size as usize) / std::mem::size_of::<crate::graphics::post_process::PointLightGpu>();
            let to_write = lights.len().min(max_count);
            buf.write(&lights[..to_write]);
        }
    }

    fn dispatch_light_cull_inner(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        width: u32,
        height: u32,
        view: glam::Mat4,
        projection: glam::Mat4,
        inv_projection: glam::Mat4,
        light_count: u32,
        depth_view: vk::ImageView,
    ) {
        let Some(pipeline) = self.light_cull_pipeline.as_ref() else { return; };
        let Some(descriptor_set) = self.light_cull_descriptor_sets.get(image_index) else { return; };
        let sampler = match self.sampler { Some(s) => s, None => return };
        let tile_count_x = (width + 15) / 16;
        let tile_count_y = (height + 15) / 16;

        unsafe {
            pipeline.bind(command_buffer, device);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE,
                pipeline.layout, 0, &[*descriptor_set], &[]);

            let mut pb = [0u8; 224];
            let mut o = 0usize;
            for &v in &view.to_cols_array() { pb[o..o+4].copy_from_slice(&v.to_ne_bytes()); o += 4; }
            for &v in &projection.to_cols_array() { pb[o..o+4].copy_from_slice(&v.to_ne_bytes()); o += 4; }
            for &v in &inv_projection.to_cols_array() { pb[o..o+4].copy_from_slice(&v.to_ne_bytes()); o += 4; }
            pb[o..o+4].copy_from_slice(&light_count.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&tile_count_x.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&tile_count_y.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&width.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&height.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&0.1f32.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&1000f32.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&256u32.to_ne_bytes());

            device.cmd_push_constants(command_buffer, pipeline.layout,
                vk::ShaderStageFlags::COMPUTE, 0, &pb);
            device.cmd_dispatch(command_buffer, tile_count_x, tile_count_y, 1);
        }

        let barriers = [
            vk::BufferMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .buffer(self.light_cull_tile_buffer.as_ref().unwrap().handle).offset(0).size(vk::WHOLE_SIZE),
            vk::BufferMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .buffer(self.light_cull_index_buffer.as_ref().unwrap().handle).offset(0).size(vk::WHOLE_SIZE),
        ];
        unsafe {
            device.cmd_pipeline_barrier(command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[], &barriers, &[]);
        }
    }
}

impl Drop for PostProcessPipeline {
    fn drop(&mut self) {
        self.exposure_buffers.clear();
        self.auto_exposure_pipeline = None;
            if let Some(device) = self.device.clone() {
            self.destroy_depth_resolve_resources(&device);
            self.destroy_bloom_resources(&device);
            self.destroy_taa_resources(&device);
            self.destroy_fog_resources(&device);
            self.destroy_lens_flare_resources(&device);
            self.destroy_gtao_resources(&device);
            self.destroy_light_cull_resources(&device);
            unsafe {
                if let Some(sampler) = self.sampler.take() {
                    device.destroy_sampler(sampler, None);
                }
                if let Some(pool) = self.descriptor_pool.take() {
                    device.destroy_descriptor_pool(pool, None);
                }
                if let Some(layout) = self.descriptor_layout.take() {
                    device.destroy_descriptor_set_layout(layout, None);
                }
                if let Some(layout) = self.layout.take() {
                    device.destroy_pipeline_layout(layout, None);
                }
                if let Some(pipe) = self.pipeline.take() {
                    device.destroy_pipeline(pipe, None);
                }
                // ── Bloom Compute Resources ──
                for views in &self.bloom_mip_views_sampled {
                    for &view in views {
                        device.destroy_image_view(view, None);
                    }
                }
                for views in &self.bloom_mip_views_storage {
                    for &view in views {
                        device.destroy_image_view(view, None);
                    }
                }
                if let Some(pool) = self.bloom_descriptor_pool.take() {
                    device.destroy_descriptor_pool(pool, None);
                }
                if let Some(layout) = self.bloom_descriptor_layout.take() {
                    device.destroy_descriptor_set_layout(layout, None);
                }
            }
        }
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct AutoExposureParams {
    pub dt: f32,
    pub speed: f32,
    pub target_luminance: f32,
    pub max_exposure: f32,
    pub min_exposure: f32,
}

