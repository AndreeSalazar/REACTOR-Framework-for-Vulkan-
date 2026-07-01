use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::reactor::Reactor;
use ash::vk;

impl Reactor {
    pub(super) fn prepare_frame(&mut self) -> ReactorResult<(u32, vk::CommandBuffer)> {
        if self.device_lost {
            return Err(ReactorError::new(ErrorCode::VulkanSynchronization, "Device lost"));
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
                    return Err(ReactorError::new(ErrorCode::VulkanSwapchain, "Out of date"));
                }
                Err(e) => return Err(ReactorError::with_source(ErrorCode::VulkanSwapchain, "acquire_next_image failed", e)),
            }
        };

        let command_buffer = self.command_buffers[self.current_frame];
        unsafe {
            self.context.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanCommandPool, "reset_command_buffer failed", e))?;
        }
        Ok((image_index, command_buffer))
    }

    pub(super) fn end_and_present(&mut self, command_buffer: vk::CommandBuffer, image_index: u32) -> ReactorResult<()> {
        let swapchain_image = self.swapchain.images[image_index as usize];

        let image_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .image(swapchain_image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1,
            });

        unsafe {
            self.context.device.cmd_pipeline_barrier(command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(), &[], &[], &[image_barrier]);

            self.context.device.end_command_buffer(command_buffer).map_err(|e| {
                ReactorError::with_source(ErrorCode::VulkanCommandPool, "end_command_buffer failed", e)
            })?;
        }

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let signal_semaphores = [self.render_finished_semaphores[image_index as usize]];
        let command_buffers_submit = [command_buffer];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers_submit)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.context.device.queue_submit(self.context.graphics_queue, &[submit_info], self.in_flight_fences[self.current_frame])
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanSynchronization, "queue_submit failed", e))?;
        }

        let swapchains = [self.swapchain.handle];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let result = unsafe { self.swapchain.loader.queue_present(self.context.graphics_queue, &present_info) };
        match result {
            Ok(_) => {}
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => { self.resized = true; }
            Err(e) => return Err(ReactorError::with_source(ErrorCode::VulkanSwapchain, "queue_present failed", e)),
        }

        Ok(())
    }
}
