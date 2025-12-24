use ash::vk;
use gpu_allocator::vulkan::Allocator;
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};
use std::error::Error;
use crate::buffer::Buffer;
use crate::vertex::Vertex;
use crate::vulkan_context::VulkanContext;

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(
        ctx: &VulkanContext,
        allocator: &Arc<Mutex<Allocator>>,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<Self, Box<dyn Error>> {
        let vertex_size = (vertices.len() * std::mem::size_of::<Vertex>()) as u64;
        let index_size = (indices.len() * std::mem::size_of::<u32>()) as u64;

        // Create Staging Buffers
        let staging_vertex = Buffer::new(
            ctx,
            allocator.clone(),
            vertex_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;
        
        let staging_index = Buffer::new(
            ctx,
            allocator.clone(),
            index_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;

        // Copy data to staging
        unsafe {
            let data_ptr = staging_vertex.allocation.as_ref().unwrap().mapped_ptr().unwrap().as_ptr() as *mut Vertex;
            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());

            let data_ptr = staging_index.allocation.as_ref().unwrap().mapped_ptr().unwrap().as_ptr() as *mut u32;
            data_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());
        }

        // Create GPU Buffers
        let vertex_buffer = Buffer::new(
            ctx,
            allocator.clone(),
            vertex_size,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )?;

        let index_buffer = Buffer::new(
            ctx,
            allocator.clone(),
            index_size,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )?;

        // Copy from Staging to GPU (Simplified immediate submit)
        // Note: In production, use a dedicated transfer command buffer
        Self::copy_buffer(ctx, staging_vertex.handle, vertex_buffer.handle, vertex_size)?;
        Self::copy_buffer(ctx, staging_index.handle, index_buffer.handle, index_size)?;

        // Staging buffers are dropped here automatically

        Ok(Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        })
    }

    fn copy_buffer(ctx: &VulkanContext, src: vk::Buffer, dst: vk::Buffer, size: u64) -> Result<(), Box<dyn Error>> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(ctx.queue_family_index)
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);
        let command_pool = unsafe { ctx.device.create_command_pool(&pool_info, None)? };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffer = unsafe { ctx.device.allocate_command_buffers(&alloc_info)?[0] };

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            ctx.device.begin_command_buffer(command_buffer, &begin_info)?;
            
            let copy_region = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size,
            };
            
            ctx.device.cmd_copy_buffer(command_buffer, src, dst, &[copy_region]);
            
            ctx.device.end_command_buffer(command_buffer)?;
            
            let command_buffers = [command_buffer];
            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&command_buffers);
                
            ctx.device.queue_submit(ctx.graphics_queue, &[submit_info], vk::Fence::null())?;
            ctx.device.queue_wait_idle(ctx.graphics_queue)?;
            
            ctx.device.destroy_command_pool(command_pool, None);
        }
        
        Ok(())
    }
}
