use crate::core::debug_utils::DebugNamer;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::memory_budget::{self, GpuMemoryBudget};
use crate::core::vrs::VrsCapabilities;
use ash::vk;

use super::VulkanContext;

impl VulkanContext {
    pub fn wait_idle(&self) -> ReactorResult<()> {
        unsafe {
            self.device.get().device_wait_idle().map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanSynchronization,
                    "device_wait_idle failed",
                    e,
                )
            })
        }
    }

    #[inline]
    pub fn ash_instance(&self) -> &ash::Instance {
        self.instance.get()
    }

    #[inline]
    pub fn ash_device(&self) -> &ash::Device {
        self.device.get()
    }

    #[inline]
    pub fn surface_khr(&self) -> vk::SurfaceKHR {
        self.surface.handle()
    }

    #[inline]
    pub fn surface_loader(&self) -> &ash::khr::surface::Instance {
        self.surface.loader()
    }

    #[inline]
    pub fn surface_handle(&self) -> vk::SurfaceKHR {
        self.surface.handle()
    }

    pub fn ref_counts(&self) -> (usize, usize, usize) {
        (
            self.instance.ref_count(),
            self.device.ref_count(),
            self.surface.ref_count(),
        )
    }

    #[inline]
    pub fn debug_namer(&self) -> &DebugNamer {
        &self.debug_namer
    }

    pub fn get_vram_budget(&self) -> GpuMemoryBudget {
        memory_budget::query_memory_budget(
            self.ash_instance(),
            self.physical_device,
            self.has_memory_budget,
        )
    }

    #[inline]
    pub fn supports_fragment_shading_rate(&self) -> bool {
        self.fragment_shading_rate.is_some()
    }

    #[inline]
    pub fn vrs_capabilities(&self) -> &VrsCapabilities {
        &self.vrs_capabilities
    }

    #[inline]
    pub fn has_async_compute(&self) -> bool {
        self.compute_queue.is_some()
    }

    #[inline]
    pub fn has_async_transfer(&self) -> bool {
        self.transfer_queue.is_some()
    }

    pub fn submit_compute(
        &self,
        submit_info: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> ReactorResult<()> {
        let queue = self.compute_queue.unwrap_or(self.graphics_queue);
        unsafe {
            self.device
                .get()
                .queue_submit(queue, submit_info, fence)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSynchronization,
                        "Compute queue submit failed",
                        e,
                    )
                })
        }
    }

    pub fn submit_transfer(
        &self,
        submit_info: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> ReactorResult<()> {
        let queue = self.transfer_queue.unwrap_or(self.graphics_queue);
        unsafe {
            self.device
                .get()
                .queue_submit(queue, submit_info, fence)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSynchronization,
                        "Transfer queue submit failed",
                        e,
                    )
                })
        }
    }

    #[inline]
    pub fn compute_family(&self) -> u32 {
        self.compute_queue_family_index
            .unwrap_or(self.queue_family_index)
    }

    #[inline]
    pub fn transfer_family(&self) -> u32 {
        self.transfer_queue_family_index
            .unwrap_or(self.queue_family_index)
    }
}
