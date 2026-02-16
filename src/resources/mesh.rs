use ash::vk;
use gpu_allocator::vulkan::Allocator;
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};
use std::error::Error;
use crate::graphics::buffer::Buffer;
use crate::resources::vertex::Vertex;
use crate::vulkan_context::VulkanContext;

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub vertex_count: u32,
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
        staging_vertex.write(vertices);
        staging_index.write(indices);

        // Create GPU Buffers
        let vertex_buffer = Buffer::new(
            ctx,
            allocator.clone(),
            vertex_size,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::GpuOnly,
        )?;

        let index_buffer = Buffer::new(
            ctx,
            allocator.clone(),
            index_size,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::GpuOnly,
        )?;

        // Copy from Staging to GPU
        Self::copy_buffer(ctx, staging_vertex.handle, vertex_buffer.handle, vertex_size)?;
        Self::copy_buffer(ctx, staging_index.handle, index_buffer.handle, index_size)?;

        Ok(Self {
            vertex_buffer,
            index_buffer,
            vertex_count: vertices.len() as u32,
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

    pub fn bind(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer.handle], &[0]);
            device.cmd_bind_index_buffer(command_buffer, self.index_buffer.handle, 0, vk::IndexType::UINT32);
        }
    }

    pub fn draw(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            device.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0);
        }
    }

    pub fn draw_instanced(&self, device: &ash::Device, command_buffer: vk::CommandBuffer, instance_count: u32) {
        unsafe {
            device.cmd_draw_indexed(command_buffer, self.index_count, instance_count, 0, 0, 0);
        }
    }
}

// Primitive mesh generators
impl Mesh {
    pub fn cube(ctx: &VulkanContext, allocator: &Arc<Mutex<Allocator>>) -> Result<Self, Box<dyn Error>> {
        let vertices = [
            // Front
            Vertex::new(glam::Vec3::new(-0.5, -0.5,  0.5), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec2::new(0.0, 0.0)),
            Vertex::new(glam::Vec3::new( 0.5, -0.5,  0.5), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec2::new(1.0, 0.0)),
            Vertex::new(glam::Vec3::new( 0.5,  0.5,  0.5), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec2::new(1.0, 1.0)),
            Vertex::new(glam::Vec3::new(-0.5,  0.5,  0.5), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec2::new(0.0, 1.0)),
            // Back
            Vertex::new(glam::Vec3::new( 0.5, -0.5, -0.5), glam::Vec3::new(0.0, 0.0, -1.0), glam::Vec2::new(0.0, 0.0)),
            Vertex::new(glam::Vec3::new(-0.5, -0.5, -0.5), glam::Vec3::new(0.0, 0.0, -1.0), glam::Vec2::new(1.0, 0.0)),
            Vertex::new(glam::Vec3::new(-0.5,  0.5, -0.5), glam::Vec3::new(0.0, 0.0, -1.0), glam::Vec2::new(1.0, 1.0)),
            Vertex::new(glam::Vec3::new( 0.5,  0.5, -0.5), glam::Vec3::new(0.0, 0.0, -1.0), glam::Vec2::new(0.0, 1.0)),
        ];

        let indices = [
            0, 1, 2, 2, 3, 0, // Front
            4, 5, 6, 6, 7, 4, // Back
            5, 0, 3, 3, 6, 5, // Left
            1, 4, 7, 7, 2, 1, // Right
            3, 2, 7, 7, 6, 3, // Top
            5, 4, 1, 1, 0, 5, // Bottom
        ];

        Self::new(ctx, allocator, &vertices, &indices)
    }

    pub fn quad(ctx: &VulkanContext, allocator: &Arc<Mutex<Allocator>>) -> Result<Self, Box<dyn Error>> {
        let vertices = [
            Vertex::new(glam::Vec3::new(-0.5, -0.5, 0.0), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec2::new(0.0, 0.0)),
            Vertex::new(glam::Vec3::new( 0.5, -0.5, 0.0), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec2::new(1.0, 0.0)),
            Vertex::new(glam::Vec3::new( 0.5,  0.5, 0.0), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec2::new(1.0, 1.0)),
            Vertex::new(glam::Vec3::new(-0.5,  0.5, 0.0), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec2::new(0.0, 1.0)),
        ];

        let indices = [0, 1, 2, 2, 3, 0];

        Self::new(ctx, allocator, &vertices, &indices)
    }
}
