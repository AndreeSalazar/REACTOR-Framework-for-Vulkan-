use bytemuck::{Pod, Zeroable};
use crate::core::VulkanContext;
use crate::graphics::Image;
use ash::vk;
use std::sync::{Arc, Mutex};
use gpu_allocator::vulkan::Allocator;

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

    // General
    pub time: f32,
    pub effect_mask: u32, // Bitflags for enabled effects

    pub _padding: [f32; 1],
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        let mut settings = Self {
            vignette_intensity: 0.35,
            vignette_smoothness: 0.6,
            chromatic_intensity: 0.0018,
            grain_intensity: 0.006,
            grain_speed: 1.0,
            bloom_threshold: 0.85,
            bloom_intensity: 0.35,
            bloom_blur_size: 4.0,
            exposure: 1.02,
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
            time: 0.0,
            effect_mask: 0,
            _padding: [0.0],
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
        settings.enable_effect(PostProcessEffect::SSGI);
        settings.enable_effect(PostProcessEffect::VolumetricFog);
        settings.enable_effect(PostProcessEffect::LutColorGrading);
        settings.enable_effect(PostProcessEffect::SSR);
        settings.enable_effect(PostProcessEffect::PathTracedLighting);
        settings.enable_effect(PostProcessEffect::AnamorphicFlares);
        settings.vignette_intensity = 0.4;
        settings.grain_intensity = 0.008;
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

    device: Option<crate::core::arc_handle::ArcDevice>,
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
            device: None,
        }
    }

    pub fn update_time(&mut self, time: f32) {
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
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();
        self.device = Some(ctx.device.clone());

        // 1. Create Descriptor Set Layout
        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(&binding));
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
        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        // 3. Create Pipeline
        let vert_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../shaders/post_process_vert.spv"))).unwrap();
        let frag_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../shaders/post_process_frag.spv"))).unwrap();

        let vert_module = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&vert_spv), None)? };
        let frag_module = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&frag_spv), None)? };

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
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&dynamic_states);

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
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, e)| e)?
        };
        let pipeline = pipelines[0];

        unsafe {
            device.destroy_shader_module(vert_module, None);
            device.destroy_shader_module(frag_module, None);
        }

        // 4. Create Descriptor Pool
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(image_count);
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(image_count);
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
        self.recreate_offscreen_images(ctx, allocator, width, height, image_count, swapchain_format)?;

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
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();
        
        // Clean old resources
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

            let writes = [vk::WriteDescriptorSet::default()
                .dst_set(self.descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&image_info))];

            unsafe {
                device.update_descriptor_sets(&writes, &[]);
            }

            self.offscreen_images.push(img);
        }

        Ok(())
    }
}

impl Drop for PostProcessPipeline {
    fn drop(&mut self) {
        if let Some(device) = &self.device {
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
