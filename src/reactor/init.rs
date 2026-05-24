//! `Reactor::init` — construcción del runtime Vulkan a partir de una ventana.
//!
//! Responsable de:
//! 1. Crear el `VulkanContext` (instance, device, surface, queues).
//! 2. Crear el `gpu-allocator`.
//! 3. Crear el swapchain inicial.
//! 4. Negociar MSAA y depth.
//! 5. Crear command pool + buffers + sync (triple buffering).
//! 6. Intentar inicializar Ray Tracing si la GPU lo soporta.

use super::{msaa, depth, Reactor, MAX_FRAMES_IN_FLIGHT};
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
    /// Inicializa Vulkan completo a partir de una ventana de `winit`.
    ///
    /// `requested_msaa` ∈ {1, 2, 4, 8}. Si el valor pedido no está soportado,
    /// se reduce automáticamente al máximo disponible.
    pub fn init(window: &Window, requested_msaa: u32) -> ReactorResult<Self> {
        let context = VulkanContext::new(window)?;

        // ── GPU allocator ──
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: context.ash_instance().clone(),
            device: context.ash_device().clone(),
            physical_device: context.physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true, // habilitado para RT
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
        let swapchain = Swapchain::new(&context, inner_size.width, inner_size.height)?;

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

        // ── Command Buffers (uno por frame en vuelo) ──
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
        let command_buffers = unsafe {
            context.device.allocate_command_buffers(&alloc_info).map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanCommandPool,
                    "Failed to allocate command buffers",
                    e,
                )
            })?
        };

        // ── Sincronización (semáforos + fences por frame) ──
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                image_available_semaphores.push(
                    context.device.create_semaphore(&semaphore_info, None).map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanSynchronization,
                            "Failed to create semaphore",
                            e,
                        )
                    })?,
                );
                render_finished_semaphores.push(
                    context.device.create_semaphore(&semaphore_info, None).map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanSynchronization,
                            "Failed to create semaphore",
                            e,
                        )
                    })?,
                );
                in_flight_fences.push(
                    context.device.create_fence(&fence_info, None).map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanSynchronization,
                            "Failed to create fence",
                            e,
                        )
                    })?,
                );
            }
        }

        // ── Ray Tracing opcional ──
        let ray_tracing = match RayTracingContext::new(&context) {
            Ok(rt) => {
                println!("Ray Tracing initialized successfully!");
                Some(rt)
            }
            Err(e) => {
                println!("Ray Tracing not supported or failed to init: {}", e);
                None
            }
        };

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
