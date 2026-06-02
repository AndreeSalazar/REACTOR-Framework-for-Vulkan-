use crate::core::VulkanContext;
use crate::graphics::Image;
use ash::vk;
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
    pub _padding: u32,
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
            pause_overlay_alpha: 0.0,
            pause_page: 0.0,
            pause_selected: 0.0,
            pause_row_count: 0.0,
            time: 0.0,
            depth_near: 0.1,
            depth_far: 1000.0,
            effect_mask: 0,
            _padding: 0,
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
            bloom_downsample_pipeline: None,
            bloom_upsample_pipeline: None,
            bloom_descriptor_layout: None,
            bloom_descriptor_pool: None,
            bloom_images: Vec::new(),
            bloom_mip_views_sampled: Vec::new(),
            bloom_mip_views_storage: Vec::new(),
            bloom_downsample_sets: Vec::new(),
            bloom_upsample_sets: Vec::new(),
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
            bloom_downsample_pipeline: None,
            bloom_upsample_pipeline: None,
            bloom_descriptor_layout: None,
            bloom_descriptor_pool: None,
            bloom_images: Vec::new(),
            bloom_mip_views_sampled: Vec::new(),
            bloom_mip_views_storage: Vec::new(),
            bloom_downsample_sets: Vec::new(),
            bloom_upsample_sets: Vec::new(),
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
        depth_view: vk::ImageView,
        sample_depth: bool,
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();
        self.device = Some(ctx.device.clone());

        // 1. Create Descriptor Set Layout
        let pp_bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
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
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&pp_bindings);
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
            "../../shaders/post_process_vert.spv"
        )))
        .unwrap();
        let frag_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../shaders/post_process_frag.spv"
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
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(image_count * 3);
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
            let pool_size = vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count * 3);
            let pool_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(std::slice::from_ref(&pool_size))
                .max_sets(image_count);
            let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };
            let layouts = vec![descriptor_layout; image_count as usize];
            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&layouts);
            self.descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };
            self.descriptor_pool = Some(descriptor_pool);
        }

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

            let depth_or_fallback_view = if sample_depth { depth_view } else { img.view };
            let depth_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(depth_or_fallback_view)
                .sampler(sampler);

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
            ];

            unsafe {
                device.update_descriptor_sets(&writes, &[]);
            }

            self.offscreen_images.push(img);
        }

        // Bloom descriptors reference the offscreen image views and sampler, so
        // they must be rebuilt whenever swapchain-sized images are recreated.
        self.init_bloom(ctx, allocator, width, height, image_count)?;

        Ok(())
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
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&bloom_bindings);
        let bloom_desc_layout =
            unsafe { device.create_descriptor_set_layout(&bloom_layout_info, None)? };

        // 2. Create bloom compute pipelines from pre-compiled SPIR-V
        let down_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../shaders/post/bloom_downsample.spv"
        )))
        .unwrap();
        let up_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../shaders/post/bloom_upsample.spv"
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
            .max_sets(total_sets);
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
}

impl Drop for PostProcessPipeline {
    fn drop(&mut self) {
        if let Some(device) = self.device.clone() {
            self.destroy_bloom_resources(&device);
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
