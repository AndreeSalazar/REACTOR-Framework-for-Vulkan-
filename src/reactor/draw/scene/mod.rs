use crate::reactor::Reactor;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::systems::scene::Scene;
use ash::vk;

mod geometry;
mod postprocess;
mod shadow;
mod sync;

impl Reactor {
    pub fn draw_scene(&mut self, scene: &Scene, view_projection: &glam::Mat4) -> ReactorResult<()> {
        if self.device_lost {
            return Ok(());
        }
        if self.resized {
            self.recreate_swapchain()?;
            self.resized = false;
        }

        unsafe {
            match self.context.device.wait_for_fences(
                &[self.in_flight_fences[self.current_frame]], true, u64::MAX,
            ) {
                Ok(_) => {}
                Err(vk::Result::ERROR_DEVICE_LOST) => {
                    eprintln!("REACTOR FATAL: Dispositivo Vulkan perdido (wait_for_fences).");
                    self.device_lost = true;
                    return Err(ReactorError::new(ErrorCode::VulkanSynchronization, "Device lost"));
                }
                Err(e) => return Err(ReactorError::with_source(ErrorCode::VulkanSynchronization, "wait_for_fences failed", e)),
            }
        }

        let (image_index, _) = unsafe {
            match self.swapchain.loader.acquire_next_image(
                self.swapchain.handle, u64::MAX, self.image_available_semaphores[self.current_frame], vk::Fence::null(),
            ) {
                Ok(result) => {
                    self.context.device.reset_fences(&[self.in_flight_fences[self.current_frame]])
                        .map_err(|e| ReactorError::with_source(ErrorCode::VulkanSynchronization, "reset_fences failed", e))?;
                    result
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain()?;
                    return Ok(());
                }
                Err(e) => return Err(ReactorError::with_source(ErrorCode::VulkanSwapchain, "acquire_next_image failed", e)),
            }
        };

        let command_buffer = self.command_buffers[self.current_frame];
        unsafe {
            self.context.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanCommandPool, "reset_command_buffer failed", e))?;
        }

        let begin_info = vk::CommandBufferBeginInfo::default();

        let (use_post_process, taa_enabled) = self.update_post_descriptors(image_index);

        unsafe {
            self.context.device.begin_command_buffer(command_buffer, &begin_info)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanCommandPool, "begin_command_buffer failed", e))?;

            self.render_shadow_cascades(scene, command_buffer);

            let local_vp = self.render_geometry(scene, command_buffer, image_index, view_projection, use_post_process, taa_enabled);

            if use_post_process && !self.decals.is_empty() {
                self.draw_screen_space_decals(command_buffer, image_index as usize, &local_vp)?;
            }

            let swapchain_view = self.swapchain.image_views[image_index as usize];
            let swapchain_image = self.swapchain.images[image_index as usize];

            self.render_post_process(scene, command_buffer, image_index, use_post_process, taa_enabled, &local_vp, swapchain_view, swapchain_image);
        }

        self.end_and_present(command_buffer, image_index)
    }
}
