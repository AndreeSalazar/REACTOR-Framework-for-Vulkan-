// =============================================================================
// REACTOR GPU Memory Allocator
// =============================================================================
// Thin wrapper around `gpu-allocator` 0.28+ providing:
// - Thread-safe access via Arc<Mutex<Allocator>>
// - Integration with ReactorContext (via ArcInstance/ArcDevice)
// - ReactorResult-based error handling (never Box<dyn std::error::Error + Send + Sync>)
//
// Future (Fase 1.2): Add per-pool allocation tracking and VRAM stats.
// =============================================================================

use ash::vk;
use gpu_allocator::vulkan::{
    Allocation, AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc,
};
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};

use crate::core::context::VulkanContext;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};

/// Thread-safe GPU memory allocator.
///
/// Wraps `gpu-allocator::vulkan::Allocator` and provides safe access
/// via `Arc<Mutex<_>>` so multiple subsystems can allocate concurrently.
#[derive(Clone)]
pub struct MemoryAllocator {
    pub allocator: Arc<Mutex<Allocator>>,
}

impl MemoryAllocator {
    /// Create a new allocator bound to the given Vulkan context.
    ///
    /// Uses `buffer_device_address = true` for Vulkan 1.2+ BDA features.
    pub fn new(ctx: &VulkanContext) -> ReactorResult<Self> {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: ctx.ash_instance().clone(),
            device: ctx.ash_device().clone(),
            physical_device: ctx.physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanMemoryAllocation,
                "Failed to initialize gpu-allocator",
                e,
            )
        })?;

        log::info!("💾 GPU memory allocator initialized");
        Ok(Self {
            allocator: Arc::new(Mutex::new(allocator)),
        })
    }

    /// Clone the inner Arc for sharing with subsystems.
    pub fn get(&self) -> Arc<Mutex<Allocator>> {
        self.allocator.clone()
    }

    /// Allocate memory for a buffer.
    pub fn allocate_buffer(
        &self,
        device: &ash::Device,
        buffer: vk::Buffer,
        location: MemoryLocation,
        name: &str,
    ) -> ReactorResult<Allocation> {
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        self.allocator
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name,
                requirements,
                location,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanMemoryAllocation,
                    format!("Buffer allocation '{}' failed", name),
                    e,
                )
            })
    }

    /// Allocate memory for an image.
    pub fn allocate_image(
        &self,
        device: &ash::Device,
        image: vk::Image,
        location: MemoryLocation,
        name: &str,
    ) -> ReactorResult<Allocation> {
        let requirements = unsafe { device.get_image_memory_requirements(image) };
        self.allocator
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name,
                requirements,
                location,
                linear: false, // images are non-linear
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanMemoryAllocation,
                    format!("Image allocation '{}' failed", name),
                    e,
                )
            })
    }

    /// Free a previous allocation.
    pub fn free(&self, allocation: Allocation) -> ReactorResult<()> {
        self.allocator
            .lock()
            .unwrap()
            .free(allocation)
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanMemoryAllocation,
                    "Failed to free allocation",
                    e,
                )
            })
    }
}
