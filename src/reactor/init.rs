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
        )?;

        Ok(Self {
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
            post_process,
            pixel_intelligent: crate::core::PixelIntelligent::default(),
            msaa_samples,
            msaa_image,
            msaa_image_view,
            msaa_memory,
            depth_image: Some(depth_image),
            depth_image_view: Some(depth_image_view),
            depth_memory: Some(depth_memory),
            depth_format,
        })
    }
}
