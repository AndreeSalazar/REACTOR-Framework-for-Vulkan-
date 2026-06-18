//! `Reactor` initialization orchestration
//!
//! This submodule contains the high-level `Reactor::init` orchestrator plus
//! the sub-initializer entry points split out by responsibility:
//! - `shadows` — Cascaded Shadow Maps
//! - `decals`  — Screen-Space Decals
//!
//! Future sub-initializers (e.g. Hi-Z, light cull, volumetric clouds) live
//! in their own files alongside this one.

use super::depth;
use super::msaa;
use super::{Reactor, MAX_FRAMES_IN_FLIGHT};
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

mod decals;
mod shadows;

impl Reactor {
    pub fn init(
        window: &Window,
        requested_msaa: u32,
        enable_ray_tracing: bool,
        vsync: bool,
    ) -> ReactorResult<Self> {
        let context = VulkanContext::new(window, enable_ray_tracing)?;

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: context.ash_instance().clone(),
            device: context.ash_device().clone(),
            physical_device: context.physical_device,
            debug_settings: Default::default(),
            buffer_device_address: enable_ray_tracing,
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

        let inner_size = window.inner_size();
        let swapchain = Swapchain::new(&context, inner_size.width, inner_size.height, vsync)?;

        context
            .debug_namer()
            .name_swapchain(swapchain.handle, "Swapchain: Main Window");

        for (i, view) in swapchain.image_views.iter().enumerate() {
            context
                .debug_namer()
                .name_image_view(*view, &format!("ImageView: Swapchain[{}]", i));
        }

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

        let depth_format = depth::find_depth_format(&context)?;
        let (depth_image, depth_image_view, depth_memory) = depth::create_depth_resources(
            &context,
            swapchain.extent.width,
            swapchain.extent.height,
            depth_format,
            msaa_samples,
        )?;
        println!("🔹 Depth buffer created: {:?}", depth_format);

        context
            .debug_namer()
            .name_image(depth_image, "Image: Depth Buffer");
        context
            .debug_namer()
            .name_image_view(depth_image_view, "ImageView: Depth Buffer");
        context
            .debug_namer()
            .name_device_memory(depth_memory, "Memory: Depth Buffer");

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

        for (i, cmd) in command_buffers.iter().enumerate() {
            context
                .debug_namer()
                .name_command_buffer(*cmd, &format!("CmdBuf: Frame[{}]", i));
        }

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

        if context.has_async_compute() {
            log::info!("⚡ Async Compute: family {} ready", context.compute_family());
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

        let hiz_pyramid = crate::graphics::HiZPyramid::new(
            &context,
            allocator.clone(),
            swapchain.extent.width,
            swapchain.extent.height,
            swapchain.images.len() as u32,
        )?;
        log::info!(
            "Hi-Z pyramid ready: mip-chain ({} levels) @ {}x{}",
            hiz_pyramid.mip_levels,
            hiz_pyramid.width,
            hiz_pyramid.height,
        );

        let ssgi_hiz = crate::graphics::post_process::SsgiHiZ::new(
            &context,
            allocator.clone(),
            swapchain.extent.width,
            swapchain.extent.height,
            swapchain.images.len() as u32,
        )?;
        log::info!("SSGI Hi-Z compute pipeline ready");

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
            prev_view_projection: glam::Mat4::IDENTITY,
            camera_near: 0.1,
            camera_far: 1000.0,
            post_process,
            gbuffer: Some(gbuffer),
            temporal_history: Some(temporal_history),
            hiz_pyramid: Some(hiz_pyramid),
            ssgi_hiz: Some(ssgi_hiz),
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
        reactor.init_shadows()?;
        Ok(reactor)
    }
}
