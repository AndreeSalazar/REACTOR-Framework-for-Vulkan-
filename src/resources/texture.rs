use ash::vk;
use crate::vulkan_context::VulkanContext;
use crate::graphics::buffer::Buffer;
use crate::graphics::image::Image;
use crate::graphics::sampler::Sampler;
use gpu_allocator::vulkan::Allocator;
use gpu_allocator::MemoryLocation;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Texture {
    pub image: Image,
    pub sampler: Sampler,
    pub width: u32,
    pub height: u32,
    device: ash::Device,
}

impl Texture {
    /// Load texture from file (PNG, JPG, BMP, etc.)
    pub fn from_file<P: AsRef<Path>>(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        path: P,
        generate_mipmaps: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();
        
        Self::from_rgba(ctx, allocator, &data, width, height, generate_mipmaps)
    }

    /// Load texture from embedded bytes (PNG, JPG, etc.)
    pub fn from_bytes(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        bytes: &[u8],
        generate_mipmaps: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();
        
        Self::from_rgba(ctx, allocator, &data, width, height, generate_mipmaps)
    }

    /// Create a solid color texture (useful for defaults)
    pub fn solid_color(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        r: u8, g: u8, b: u8, a: u8,
    ) -> Result<Self, Box<dyn Error>> {
        let data = [r, g, b, a];
        Self::from_rgba(ctx, allocator, &data, 1, 1, false)
    }

    /// Create white texture (default diffuse)
    pub fn white(ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>) -> Result<Self, Box<dyn Error>> {
        Self::solid_color(ctx, allocator, 255, 255, 255, 255)
    }

    /// Create black texture
    pub fn black(ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>) -> Result<Self, Box<dyn Error>> {
        Self::solid_color(ctx, allocator, 0, 0, 0, 255)
    }

    /// Create normal map default (flat surface pointing up)
    pub fn default_normal(ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>) -> Result<Self, Box<dyn Error>> {
        Self::solid_color(ctx, allocator, 128, 128, 255, 255)
    }

    pub fn from_rgba(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        data: &[u8],
        width: u32,
        height: u32,
        generate_mipmaps: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let mip_levels = if generate_mipmaps {
            ((width.max(height) as f32).log2().floor() as u32) + 1
        } else {
            1
        };

        let image = Image::new_texture(ctx, allocator.clone(), width, height, mip_levels)?;

        // Create staging buffer
        let buffer_size = (width * height * 4) as u64;
        let staging = Buffer::new(
            ctx,
            allocator.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;

        staging.write(data);

        // Copy buffer to image
        Self::copy_buffer_to_image(ctx, staging.handle, image.handle, width, height)?;

        // Generate mipmaps or transition to shader read
        if generate_mipmaps && mip_levels > 1 {
            Self::generate_mipmaps(ctx, image.handle, width, height, mip_levels)?;
        } else {
            Self::transition_to_shader_read(ctx, image.handle, mip_levels)?;
        }

        let sampler = Sampler::linear(ctx)?;

        Ok(Self {
            image,
            sampler,
            width,
            height,
            device: ctx.device.clone(),
        })
    }

    /// Get the image view for binding to descriptor sets
    pub fn view(&self) -> vk::ImageView {
        self.image.view
    }

    /// Get the sampler handle
    pub fn sampler_handle(&self) -> vk::Sampler {
        self.sampler.handle
    }

    fn copy_buffer_to_image(
        ctx: &VulkanContext,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn Error>> {
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

            // Transition to transfer dst
            let barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);

            ctx.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );

            // Copy buffer to image
            let region = vk::BufferImageCopy::default()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D { width, height, depth: 1 });

            ctx.device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );

            ctx.device.end_command_buffer(command_buffer)?;

            let command_buffers = [command_buffer];
            let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
            ctx.device.queue_submit(ctx.graphics_queue, &[submit_info], vk::Fence::null())?;
            ctx.device.queue_wait_idle(ctx.graphics_queue)?;
            ctx.device.destroy_command_pool(command_pool, None);
        }

        Ok(())
    }

    fn transition_to_shader_read(
        ctx: &VulkanContext,
        image: vk::Image,
        mip_levels: u32,
    ) -> Result<(), Box<dyn Error>> {
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

            let barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(mip_levels)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ);

            ctx.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );

            ctx.device.end_command_buffer(command_buffer)?;

            let command_buffers = [command_buffer];
            let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
            ctx.device.queue_submit(ctx.graphics_queue, &[submit_info], vk::Fence::null())?;
            ctx.device.queue_wait_idle(ctx.graphics_queue)?;
            ctx.device.destroy_command_pool(command_pool, None);
        }

        Ok(())
    }

    fn generate_mipmaps(
        ctx: &VulkanContext,
        image: vk::Image,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> Result<(), Box<dyn Error>> {
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

            // First, transition ALL mip levels (except 0 which is already TRANSFER_DST) to TRANSFER_DST
            // Mip level 0 is already in TRANSFER_DST_OPTIMAL from copy_buffer_to_image
            // Mip levels 1+ are in UNDEFINED, need to transition them to TRANSFER_DST_OPTIMAL
            if mip_levels > 1 {
                let barrier = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(1)
                            .level_count(mip_levels - 1)
                            .base_array_layer(0)
                            .layer_count(1),
                    )
                    .src_access_mask(vk::AccessFlags::empty())
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);

                ctx.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );
            }

            let mut mip_width = width as i32;
            let mut mip_height = height as i32;

            for i in 1..mip_levels {
                // Transition previous level to transfer src
                let barrier = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(i - 1)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    )
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ);

                ctx.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );

                // Blit
                let blit = vk::ImageBlit::default()
                    .src_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D { x: mip_width, y: mip_height, z: 1 },
                    ])
                    .src_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .mip_level(i - 1)
                            .base_array_layer(0)
                            .layer_count(1),
                    )
                    .dst_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D {
                            x: if mip_width > 1 { mip_width / 2 } else { 1 },
                            y: if mip_height > 1 { mip_height / 2 } else { 1 },
                            z: 1,
                        },
                    ])
                    .dst_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .mip_level(i)
                            .base_array_layer(0)
                            .layer_count(1),
                    );

                ctx.device.cmd_blit_image(
                    command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[blit],
                    vk::Filter::LINEAR,
                );

                // Transition to shader read
                let barrier = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(i - 1)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    )
                    .src_access_mask(vk::AccessFlags::TRANSFER_READ)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ);

                ctx.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );

                if mip_width > 1 { mip_width /= 2; }
                if mip_height > 1 { mip_height /= 2; }
            }

            // Transition last mip level
            let barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(mip_levels - 1)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ);

            ctx.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );

            ctx.device.end_command_buffer(command_buffer)?;

            let command_buffers = [command_buffer];
            let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
            ctx.device.queue_submit(ctx.graphics_queue, &[submit_info], vk::Fence::null())?;
            ctx.device.queue_wait_idle(ctx.graphics_queue)?;
            ctx.device.destroy_command_pool(command_pool, None);
        }

        Ok(())
    }
}
