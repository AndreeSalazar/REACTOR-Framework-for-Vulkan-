use ash::vk;
use std::error::Error;

use crate::core::context::VulkanContext;

pub struct CommandManager {
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
    device: ash::Device,
}

impl CommandManager {
    pub fn new(ctx: &VulkanContext, buffer_count: u32) -> Result<Self, Box<dyn Error>> {
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(ctx.queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        
        let pool = unsafe { ctx.device.create_command_pool(&pool_create_info, None)? };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(buffer_count);
        
        let buffers = unsafe { ctx.device.allocate_command_buffers(&alloc_info)? };

        Ok(Self {
            pool,
            buffers,
            device: ctx.device.clone(),
        })
    }

    pub fn begin_single_time(&self, ctx: &VulkanContext) -> Result<vk::CommandBuffer, Box<dyn Error>> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.pool)
            .command_buffer_count(1);

        let command_buffer = unsafe { ctx.device.allocate_command_buffers(&alloc_info)?[0] };

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { ctx.device.begin_command_buffer(command_buffer, &begin_info)?; }

        Ok(command_buffer)
    }

    pub fn end_single_time(&self, ctx: &VulkanContext, command_buffer: vk::CommandBuffer) -> Result<(), Box<dyn Error>> {
        unsafe {
            ctx.device.end_command_buffer(command_buffer)?;

            let command_buffers = [command_buffer];
            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&command_buffers);

            ctx.device.queue_submit(ctx.graphics_queue, &[submit_info], vk::Fence::null())?;
            ctx.device.queue_wait_idle(ctx.graphics_queue)?;

            ctx.device.free_command_buffers(self.pool, &command_buffers);
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
