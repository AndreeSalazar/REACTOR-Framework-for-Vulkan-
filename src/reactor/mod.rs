//! # Reactor — fachada del runtime Vulkan
//!
//! Este módulo agrupa la *estructura* principal `Reactor` que orquesta todo el
//! ciclo de vida Vulkan (contexto, swapchain, MSAA, depth, command pool,
//! sincronización, draw…). La implementación está **partida en sub-módulos
//! temáticos** para no caer en el monolito de 1.800 líneas de antaño:
//!
//! ```text
//! reactor/
//! ├── mod.rs                — struct + Drop + re-exports
//! ├── init.rs               — Reactor::init
//! ├── msaa.rs               — MSAA helpers
//! ├── depth.rs              — depth buffer helpers
//! ├── render_pass.rs        — render pass + framebuffer helpers
//! ├── swapchain_recreate.rs — recreate_swapchain
//! ├── resources.rs          — create_mesh / load_texture / materials
//! ├── events.rs             — handle_event + queries
//! └── draw.rs               — draw_scene + draw_frame
//! ```
//!
//! El usuario sigue viendo **un solo tipo `Reactor`** (monolito en la API),
//! pero internamente cada responsabilidad vive en su archivo (modular).

use crate::core::{PixelIntelligent, PixelIntelligentProfile, VrsRate, VulkanContext};
use crate::graphics::swapchain::Swapchain;
use crate::platform::input::Input;
use crate::raytracing::RayTracingContext;
use crate::scene::ecs::World;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use std::sync::{Arc, Mutex};

mod depth;
mod draw;
mod events;
mod init;
mod msaa;
mod render_pass;
mod resources;
mod swapchain_recreate;

/// Número máximo de frames en vuelo simultáneamente.
///
/// Triple buffering por defecto: balance entre latencia y throughput.
pub(crate) const MAX_FRAMES_IN_FLIGHT: usize = 3;

/// El runtime Vulkan central. Posee el contexto, swapchain, MSAA, depth,
/// command pool y sincronización. La capa de aplicación (`crate::app`)
/// construye uno por ventana y lo conduce vía `init` → `draw_*` → `Drop`.
pub struct Reactor {
    // ── Swapchain / GPU ──
    pub swapchain: Swapchain,
    pub allocator: Arc<Mutex<Allocator>>,

    // ── Command pool y buffers ──
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,

    // ── Sincronización (triple buffer) ──
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub current_frame: usize,

    // ── Subsistemas ──
    pub input: Input,
    pub world: World,
    pub ray_tracing: Option<RayTracingContext>,

    // ── Flags de estado del frame ──
    pub resized: bool,
    pub device_lost: bool,
    pub vsync: bool,
    pub camera_pos: glam::Vec3,
    pub post_process: crate::graphics::post_process::PostProcessPipeline,
    pub pixel_intelligent: PixelIntelligent,

    // ── Contexto Vulkan (al final: se libera al final por orden de Drop) ──
    pub context: VulkanContext,

    // ── MSAA Anti-Aliasing ──
    pub msaa_samples: vk::SampleCountFlags,
    pub msaa_image: Option<vk::Image>,
    pub msaa_image_view: Option<vk::ImageView>,
    pub msaa_memory: Option<vk::DeviceMemory>,

    // ── Depth Buffer ──
    pub depth_image: Option<vk::Image>,
    pub depth_image_view: Option<vk::ImageView>,
    pub depth_memory: Option<vk::DeviceMemory>,
    pub depth_format: vk::Format,
}

impl Reactor {
    pub fn set_pixel_intelligent_profile(&mut self, profile: PixelIntelligentProfile) {
        self.pixel_intelligent.set_profile(profile);
    }

    pub fn pixel_intelligent_rate(&self) -> VrsRate {
        self.pixel_intelligent.current_rate
    }

    pub fn pixel_intelligent_enabled(&self) -> bool {
        self.pixel_intelligent.enabled && self.context.supports_fragment_shading_rate()
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        unsafe {
            // Esperar a que la GPU termine cualquier trabajo pendiente.
            let _ = self.context.device.device_wait_idle();

            // ── Depth ──
            if let Some(depth_view) = self.depth_image_view.take() {
                self.context.device.destroy_image_view(depth_view, None);
            }
            if let Some(depth_image) = self.depth_image.take() {
                self.context.device.destroy_image(depth_image, None);
            }
            if let Some(depth_memory) = self.depth_memory.take() {
                self.context.device.free_memory(depth_memory, None);
            }

            // ── MSAA ──
            if let Some(msaa_view) = self.msaa_image_view.take() {
                self.context.device.destroy_image_view(msaa_view, None);
            }
            if let Some(msaa_image) = self.msaa_image.take() {
                self.context.device.destroy_image(msaa_image, None);
            }
            if let Some(msaa_memory) = self.msaa_memory.take() {
                self.context.device.free_memory(msaa_memory, None);
            }

            // ── Sincronización ──
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.context
                    .device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.context
                    .device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.context
                    .device
                    .destroy_fence(self.in_flight_fences[i], None);
            }

            // ── Command pool ──
            self.context
                .device
                .destroy_command_pool(self.command_pool, None);
        }

        // ── Swapchain ──
        self.swapchain.destroy(self.context.ash_device());

        // El allocator debe liberarse ANTES que el device (lo hace Arc<Mutex<_>>
        // automáticamente al perder el último Arc al salir de scope).
        if let Ok(allocator) = self.allocator.lock() {
            drop(allocator);
        }
    }
}
