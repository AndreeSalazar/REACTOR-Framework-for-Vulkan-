//! `Reactor::init` — construcción del runtime Vulkan a partir de una ventana.
//!
//! Responsable de:
//! 1. Crear el `VulkanContext` (instance, device, surface, queues + debug utils).
//! 2. Crear el `gpu-allocator`.
//! 3. Crear el swapchain inicial.
//! 4. Negociar MSAA y depth.
//! 5. Crear command pool + buffers + sync (triple buffering).
//! 6. Etiquetar todos los recursos con debug names.
//! 7. Intentar inicializar Ray Tracing si la GPU lo soporta.
//! 8. Pipeline warm-up (pre-compilar variantes comunes).

use super::{depth, msaa, Reactor, MAX_FRAMES_IN_FLIGHT};
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use crate::graphics::swapchain::Swapchain;
use crate::platform::input::Input;
use crate::raytracing::RayTracingContext;
use crate::scene::ecs::World;
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::sync::{Arc, Mutex};
use winit::window::Window;

impl Reactor {
    pub fn init(
        window: &Window,
        requested_msaa: u32,
        enable_ray_tracing: bool,
        vsync: bool,
    ) -> ReactorResult<Self> {
        let context = VulkanContext::new(window, enable_ray_tracing)?;

        // ── GPU allocator ──
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: context.ash_instance().clone(),
            device: context.ash_device().clone(),
            physical_device: context.physical_device,
            debug_settings: Default::default(),
            buffer_device_address: enable_ray_tracing, // habilitado para RT
            allocation_sizes: Default::default(),
        })
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanMemoryAllocation,
                "Failed to create GPU allocator",
                e,
            )
        })?;
        let allocator = Arc::new(Mutex::new(allocator));

        // ── Swapchain inicial ──
        let inner_size = window.inner_size();
        let swapchain = Swapchain::new(&context, inner_size.width, inner_size.height, vsync)?;

        // Label swapchain
        context
            .debug_namer()
            .name_swapchain(swapchain.handle, "Swapchain: Main Window");

        // Label swapchain image views
        for (i, view) in swapchain.image_views.iter().enumerate() {
            context
                .debug_namer()
                .name_image_view(*view, &format!("ImageView: Swapchain[{}]", i));
        }

        // ── MSAA ──
        let msaa_samples = msaa::msaa_from_u32(requested_msaa, &context);
        if msaa_samples == vk::SampleCountFlags::TYPE_1 {
            println!("🔷 MSAA: disabled (1 sample)");
        } else {
            println!("🔷 MSAA: {:?} enabled for anti-aliasing", msaa_samples);
        }

        let (msaa_image, msaa_image_view, msaa_memory) =
            if msaa_samples != vk::SampleCountFlags::TYPE_1 {
                let (img, view, mem) = msaa::create_msaa_resources(
                    &context,
                    swapchain.extent.width,
                    swapchain.extent.height,
                    swapchain.format,
                    msaa_samples,
                )?;
                // Label MSAA resources
                context
                    .debug_namer()
                    .name_image(img, "Image: MSAA Color Resolve");
                context
                    .debug_namer()
                    .name_image_view(view, "ImageView: MSAA Color");
                context
                    .debug_namer()
                    .name_device_memory(mem, "Memory: MSAA Color");
                (Some(img), Some(view), Some(mem))
            } else {
                (None, None, None)
            };

        // ── Depth Buffer ──
        let depth_format = depth::find_depth_format(&context)?;
        let (depth_image, depth_image_view, depth_memory) = depth::create_depth_resources(
            &context,
            swapchain.extent.width,
            swapchain.extent.height,
            depth_format,
            msaa_samples,
        )?;
        println!("🔹 Depth buffer created: {:?}", depth_format);

        // Label depth resources
        context
            .debug_namer()
            .name_image(depth_image, "Image: Depth Buffer");
        context
            .debug_namer()
            .name_image_view(depth_image_view, "ImageView: Depth Buffer");
        context
            .debug_namer()
            .name_device_memory(depth_memory, "Memory: Depth Buffer");

        // ── Command Pool ──
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(context.queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe {
            context
                .device
                .create_command_pool(&pool_create_info, None)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanCommandPool,
                        "Failed to create command pool",
                        e,
                    )
                })?
        };
        context
            .debug_namer()
            .name_command_pool(command_pool, "CommandPool: Graphics (Main)");

        // ── Command Buffers (uno por frame en vuelo) ──
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
        let command_buffers = unsafe {
            context
                .device
                .allocate_command_buffers(&alloc_info)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanCommandPool,
                        "Failed to allocate command buffers",
                        e,
                    )
                })?
        };

        // Label command buffers
        for (i, cmd) in command_buffers.iter().enumerate() {
            context
                .debug_namer()
                .name_command_buffer(*cmd, &format!("CmdBuf: Frame[{}]", i));
        }

        // ── Sincronización (semáforos + fences por frame) ──
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let img_sem = context
                    .device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanSynchronization,
                            "Failed to create semaphore",
                            e,
                        )
                    })?;
                context
                    .debug_namer()
                    .name_semaphore(img_sem, &format!("Semaphore: ImageAvailable[{}]", i));
                image_available_semaphores.push(img_sem);

                let render_sem = context
                    .device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanSynchronization,
                            "Failed to create semaphore",
                            e,
                        )
                    })?;
                context
                    .debug_namer()
                    .name_semaphore(render_sem, &format!("Semaphore: RenderFinished[{}]", i));
                render_finished_semaphores.push(render_sem);

                let fence = context
                    .device
                    .create_fence(&fence_info, None)
                    .map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanSynchronization,
                            "Failed to create fence",
                            e,
                        )
                    })?;
                context
                    .debug_namer()
                    .name_fence(fence, &format!("Fence: InFlight[{}]", i));
                in_flight_fences.push(fence);
            }
        }

        // ── Ray Tracing opcional ──
        let ray_tracing = if enable_ray_tracing {
            match RayTracingContext::new(&context) {
                Ok(rt) => {
                    println!("Ray Tracing initialized successfully!");
                    Some(rt)
                }
                Err(e) => {
                    println!("Ray Tracing not supported or failed to init: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // ── Log VRAM budget at startup ──
        let budget = context.get_vram_budget();
        log::info!(
            "💾 VRAM Budget: {}/{} MB ({})",
            budget.total_vram_usage_mb(),
            budget.total_vram_budget_mb(),
            if budget.has_dynamic_budget {
                "dynamic"
            } else {
                "static"
            }
        );

        // ── Log Async Queue status ──
        if context.has_async_compute() {
            log::info!(
                "⚡ Async Compute: family {} ready",
                context.compute_family()
            );
        } else {
            log::info!("⚡ Async Compute: not available (using graphics queue)");
        }
        if context.has_async_transfer() {
            log::info!(
                "📦 Async Transfer: family {} ready",
                context.transfer_family()
            );
        } else {
            log::info!("📦 Async Transfer: not available (using graphics queue)");
        }

        if context.supports_fragment_shading_rate() {
            log::info!("Pixel Inteligente: Vulkan VRS active (pipeline mode)");
        }

        let mut post_process = crate::graphics::post_process::PostProcessPipeline::new();
        post_process.init(
            &context,
            allocator.clone(),
            swapchain.extent.width,
            swapchain.extent.height,
            swapchain.images.len() as u32,
            swapchain.format,
            depth_image_view,
            msaa_samples == vk::SampleCountFlags::TYPE_1,
        )?;

        let gbuffer = crate::graphics::GBuffer::new(
            &context,
            allocator.clone(),
            swapchain.extent.width,
            swapchain.extent.height,
        )?;
        log::info!(
            "G-Buffer ready: 4 attachments @ {}x{} (~{:.1} MiB, storage writes: {})",
            swapchain.extent.width,
            swapchain.extent.height,
            gbuffer.estimated_bytes() as f32 / (1024.0 * 1024.0),
            gbuffer.storage_writes_supported
        );

        let temporal_history = crate::graphics::TemporalHistory::new(
            &context,
            allocator.clone(),
            swapchain.extent.width,
            swapchain.extent.height,
        )?;
        log::info!(
            "Temporal history ready: color/depth ping-pong @ {}x{} (~{:.1} MiB, storage writes: {})",
            swapchain.extent.width,
            swapchain.extent.height,
            temporal_history.estimated_bytes() as f32 / (1024.0 * 1024.0),
            temporal_history.storage_writes_supported
        );

        let mut reactor = Self {
            context,
            swapchain,
            allocator,
            command_pool,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
            input: Input::new(),
            world: World::new(),
            ray_tracing,
            resized: false,
            device_lost: false,
            exit_requested: false,
            vsync,
            camera_pos: glam::Vec3::ZERO,
            light_pos: glam::Vec4::new(5.0, 5.0, 5.0, 1.0),
            camera_view: glam::Mat4::IDENTITY,
            camera_proj: glam::Mat4::IDENTITY,
            camera_near: 0.1,
            camera_far: 1000.0,
            post_process,
            gbuffer: Some(gbuffer),
            temporal_history: Some(temporal_history),
            pixel_intelligent: crate::core::PixelIntelligent::default(),
            msaa_samples,
            msaa_image,
            msaa_image_view,
            msaa_memory,
            depth_image: Some(depth_image),
            depth_image_view: Some(depth_image_view),
            depth_memory: Some(depth_memory),
            depth_format,
            ibl_textures: None,
            shadow_map: None,
            shadow_image: None,
            shadow_image_views: Vec::new(),
            shadow_array_view: None,
            shadow_sampler: None,
            shadow_memory: None,
            shadow_pipeline: None,
            shadow_descriptor_layout: None,
            shadow_descriptor_pool: None,
            shadow_descriptor_sets: Vec::new(),
            shadow_uniform_buffers: Vec::new(),
            decals: Vec::new(),
            decal_pipeline: None,
            decal_descriptor_layout: None,
            decal_cube_mesh: None,
        };

        reactor.init_decals()?;
        Ok(reactor)
    }
}

impl Reactor {
    /// Inicializa el pipeline de proyección de decals en espacio de pantalla (MRT).
    pub fn init_decals(&mut self) -> ReactorResult<()> {
        let device = self.context.ash_device();

        // 1. Crear el Layout de Descriptores para Decals
        let bindings = [
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
            vk::DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let decal_descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        // 2. Cargar código de shaders desde el cookbook/asset manager
        let vert_words = crate::base_shader::BaseShaderAsset::ShadowVert.words();
        let frag_words = crate::base_shader::BaseShaderAsset::DecalFrag.words();

        let config = crate::graphics::pipeline::PipelineConfig {
            cull_mode: vk::CullModeFlags::NONE, // Procesar ambas caras del cubo de proyección
            depth_write: false,
            depth_test: true,
            blend_enable: true,
            ..Default::default()
        };

        // Renderizar directamente sobre el target de color offscreen (formato swapchain)
        let color_formats = [
            self.swapchain.format,
        ];

        let decal_pipeline = crate::graphics::pipeline::Pipeline::with_config_multi_color(
            &self.context.device,
            None,
            &vert_words,
            &frag_words,
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            &config,
            &[decal_descriptor_layout],
            &color_formats,
            Some(self.depth_format),
        )?;

        self.decal_descriptor_layout = Some(decal_descriptor_layout);
        self.decal_pipeline = Some(decal_pipeline);

        let (vertices, indices) = crate::resources::primitives::Primitives::cube();
        let decal_cube_mesh = self.create_mesh(&vertices, &indices)?;
        self.decal_cube_mesh = Some(decal_cube_mesh);

        log::info!("✅ Screen-Space MRT Decals pipeline initialized successfully");

        Ok(())
    }
}

impl Reactor {
    /// Inicializa toda la infraestructura para Cascaded Shadow Maps (CSM)
    pub fn init_shadows(&mut self) -> ReactorResult<()> {
        use ash::vk;

        let shadow_map = crate::graphics::shadows::ShadowMap::new(
            crate::graphics::shadows::ShadowConfig::default(),
        );

        let width = 2048;
        let height = 2048;
        let format = vk::Format::D32_SFLOAT;
        let device = self.context.ash_device();

        // 1. Crear Imagen Texture Array de 4 capas
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(4)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let shadow_image = unsafe { device.create_image(&image_info, None)? };
        let requirements = unsafe { device.get_image_memory_requirements(shadow_image) };

        let memory_props = unsafe {
            self.context
                .instance
                .get_physical_device_memory_properties(self.context.physical_device)
        };
        let memory_type_index = (0..memory_props.memory_type_count)
            .find(|&i| {
                let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
                let memory_type = memory_props.memory_types[i as usize];
                suitable
                    && memory_type
                        .property_flags
                        .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })
            .ok_or_else(|| {
                ReactorError::new(
                    ErrorCode::VulkanMemoryAllocation,
                    "Failed to find memory type for shadow map",
                )
            })?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);

        let shadow_memory = unsafe { device.allocate_memory(&alloc_info, None)? };
        unsafe { device.bind_image_memory(shadow_image, shadow_memory, 0)? };

        // 2. Crear Vistas (1 Array de 4 capas y 4 individuales)
        let array_view_info = vk::ImageViewCreateInfo::default()
            .image(shadow_image)
            .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 4,
            });
        let shadow_array_view = unsafe { device.create_image_view(&array_view_info, None)? };

        let mut shadow_image_views = Vec::with_capacity(4);
        for layer in 0..4 {
            let view_info = vk::ImageViewCreateInfo::default()
                .image(shadow_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: layer,
                    layer_count: 1,
                });
            let view = unsafe { device.create_image_view(&view_info, None)? };
            shadow_image_views.push(view);
        }

        // 3. Crear Sampler de sombras con filtrado comparativo
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .compare_enable(false)
            .compare_op(vk::CompareOp::LESS_OR_EQUAL);
        let shadow_sampler = unsafe { device.create_sampler(&sampler_info, None)? };

        // 4. Crear Layout de Descriptores de Sombras
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let shadow_descriptor_layout =
            unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        // 5. Crear Pool de Descriptores de Sombras
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32);
        let shadow_descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        // 6. Alloc Sets
        let layouts = vec![shadow_descriptor_layout; MAX_FRAMES_IN_FLIGHT];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(shadow_descriptor_pool)
            .set_layouts(&layouts);
        let shadow_descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        // 7. Crear Buffers de Uniformes y Actualizar Descriptores
        let mut shadow_uniform_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let size = std::mem::size_of::<crate::graphics::shadows::ShadowUniformData>() as u64;
            let buffer = crate::graphics::buffer::Buffer::new_uniform(
                &self.context,
                self.allocator.clone(),
                size,
            )?;

            let image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(shadow_array_view)
                .sampler(shadow_sampler);

            let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(buffer.handle)
                .offset(0)
                .range(size);

            let write_image = vk::WriteDescriptorSet::default()
                .dst_set(shadow_descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&image_info));

            let write_buffer = vk::WriteDescriptorSet::default()
                .dst_set(shadow_descriptor_sets[i])
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&buffer_info));

            unsafe {
                device.update_descriptor_sets(&[write_image, write_buffer], &[]);
            }
            shadow_uniform_buffers.push(buffer);
        }

        // 8. Cargar y Crear Pipeline de Sombras (Depth-only)
        let shadow_vert_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../shaders/shadow_vert.spv"
        )))
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanShaderCompilation,
                "Failed to load shadow_vert spv",
                e,
            )
        })?;

        let shadow_frag_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../shaders/shadow_frag.spv"
        )))
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanShaderCompilation,
                "Failed to load shadow_frag spv",
                e,
            )
        })?;

        let config = crate::graphics::pipeline::PipelineConfig {
            cull_mode: vk::CullModeFlags::BACK,
            depth_write: true,
            depth_test: true,
            ..Default::default()
        };

        let shadow_pipeline = crate::graphics::pipeline::Pipeline::with_config(
            &self.context.device,
            None,
            &shadow_vert_spv,
            &shadow_frag_spv,
            width,
            height,
            &config,
            &[shadow_descriptor_layout],
            vk::Format::UNDEFINED,
            Some(vk::Format::D32_SFLOAT),
        )?;

        // 9. Guardar recursos
        self.shadow_map = Some(shadow_map);
        self.shadow_image = Some(shadow_image);
        self.shadow_image_views = shadow_image_views;
        self.shadow_array_view = Some(shadow_array_view);
        self.shadow_sampler = Some(shadow_sampler);
        self.shadow_memory = Some(shadow_memory);
        self.shadow_pipeline = Some(shadow_pipeline);
        self.shadow_descriptor_layout = Some(shadow_descriptor_layout);
        self.shadow_descriptor_pool = Some(shadow_descriptor_pool);
        self.shadow_descriptor_sets = shadow_descriptor_sets;
        self.shadow_uniform_buffers = shadow_uniform_buffers;

        println!("✅ CSM Shadow Maps initialized: 4 cascades @ 2048x2048");

        Ok(())
    }
}
