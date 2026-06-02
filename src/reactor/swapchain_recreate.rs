//! Recreación del swapchain ante resize / out-of-date.
//!
//! Libera y reconstruye también los attachments MSAA y depth, manteniendo
//! la coherencia de extents con el nuevo tamaño de la superficie.

use super::{depth, msaa, Reactor};
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::graphics::swapchain::Swapchain;
use ash::vk;

impl Reactor {
    /// Espera GPU, destruye los recursos dependientes del tamaño y los
    /// reconstruye contra el nuevo extent reportado por la surface.
    pub fn recreate_swapchain(&mut self) -> ReactorResult<()> {
        unsafe {
            self.context.device.device_wait_idle().map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanSynchronization,
                    "device_wait_idle failed",
                    e,
                )
            })?;
        }

        let capabilities = unsafe {
            self.context
                .surface_loader()
                .get_physical_device_surface_capabilities(
                    self.context.physical_device,
                    self.context.surface_khr(),
                )?
        };

        // Ventana minimizada → no recreamos hasta que vuelva a tener tamaño.
        if capabilities.current_extent.width == 0 || capabilities.current_extent.height == 0 {
            return Ok(());
        }

        // ── Destruir depth previo ──
        if let Some(view) = self.depth_image_view.take() {
            unsafe { self.context.device.destroy_image_view(view, None) };
        }
        if let Some(image) = self.depth_image.take() {
            unsafe { self.context.device.destroy_image(image, None) };
        }
        if let Some(memory) = self.depth_memory.take() {
            unsafe { self.context.device.free_memory(memory, None) };
        }

        // ── Destruir MSAA previo ──
        if let Some(view) = self.msaa_image_view.take() {
            unsafe { self.context.device.destroy_image_view(view, None) };
        }
        if let Some(image) = self.msaa_image.take() {
            unsafe { self.context.device.destroy_image(image, None) };
        }
        if let Some(memory) = self.msaa_memory.take() {
            unsafe { self.context.device.free_memory(memory, None) };
        }

        // ── Swapchain ──
        self.swapchain.destroy(self.context.ash_device());
        self.swapchain = Swapchain::new(
            &self.context,
            capabilities.current_extent.width,
            capabilities.current_extent.height,
            self.vsync,
        )?;

        // ── Recrear MSAA si estaba habilitado ──
        if self.msaa_samples != vk::SampleCountFlags::TYPE_1 {
            let (img, view, mem) = msaa::create_msaa_resources(
                &self.context,
                self.swapchain.extent.width,
                self.swapchain.extent.height,
                self.swapchain.format,
                self.msaa_samples,
            )?;
            self.msaa_image = Some(img);
            self.msaa_image_view = Some(view);
            self.msaa_memory = Some(mem);
        }

        // ── Recrear depth ──
        let (depth_img, depth_view, depth_mem) = depth::create_depth_resources(
            &self.context,
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            self.depth_format,
            self.msaa_samples,
        )?;
        self.depth_image = Some(depth_img);
        self.depth_image_view = Some(depth_view);
        self.depth_memory = Some(depth_mem);

        // ── Recrear offscreen images de post-proceso ──
        self.post_process.recreate_offscreen_images(
            &self.context,
            self.allocator.clone(),
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            self.swapchain.images.len() as u32,
            self.swapchain.format,
            self.depth_image_view.unwrap(),
        )?;

        Ok(())
    }
}
