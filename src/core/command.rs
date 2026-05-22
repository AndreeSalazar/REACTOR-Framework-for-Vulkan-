use ash::vk;

use crate::core::arc_handle::ArcDevice;
use crate::core::context::VulkanContext;
use crate::core::error::{ReactorResult, ReactorError, ErrorCode};

pub struct CommandManager {
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
    device: ArcDevice,
}

impl CommandManager {
    pub fn new(ctx: &VulkanContext, buffer_count: u32) -> ReactorResult<Self> {
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(ctx.queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let pool = unsafe {
            ctx.ash_device()
                .create_command_pool(&pool_create_info, None)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanCommandPool, "create_command_pool failed", e))?
        };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(buffer_count);

        let buffers = unsafe {
            ctx.ash_device()
                .allocate_command_buffers(&alloc_info)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanCommandPool, "allocate_command_buffers failed", e))?
        };

        Ok(Self {
            pool,
            buffers,
            device: ctx.device.clone(),
        })
    }

    pub fn begin_single_time(
        &self,
        ctx: &VulkanContext,
    ) -> ReactorResult<vk::CommandBuffer> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.pool)
            .command_buffer_count(1);

        let command_buffer = unsafe {
            ctx.ash_device()
                .allocate_command_buffers(&alloc_info)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanCommandPool, "allocate_command_buffers failed", e))?[0]
        };

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            ctx.ash_device()
                .begin_command_buffer(command_buffer, &begin_info)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanCommandPool, "begin_command_buffer failed", e))?;
        }

        Ok(command_buffer)
    }

    pub fn end_single_time(
        &self,
        ctx: &VulkanContext,
        command_buffer: vk::CommandBuffer,
    ) -> ReactorResult<()> {
        unsafe {
            ctx.ash_device()
                .end_command_buffer(command_buffer)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanCommandPool, "end_command_buffer failed", e))?;

            let command_buffers = [command_buffer];
            let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);

            ctx.ash_device()
                .queue_submit(ctx.graphics_queue, &[submit_info], vk::Fence::null())
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanSynchronization, "queue_submit failed", e))?;
            ctx.ash_device()
                .queue_wait_idle(ctx.graphics_queue)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanSynchronization, "queue_wait_idle failed", e))?;

            ctx.ash_device().free_command_buffers(self.pool, &command_buffers);
        }
        Ok(())
    }
}

impl Drop for CommandManager {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.pool, None);
        }
    }
}
