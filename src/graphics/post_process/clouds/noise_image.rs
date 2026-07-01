use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator};
use std::sync::{Arc, Mutex};

use super::generation::generate_value_noise_3d;

pub(crate) struct Noise3D {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub allocation: Allocation,
}

pub(crate) fn create_3d_noise_image(
    ctx: &VulkanContext,
    device: &ash::Device,
    allocator: Arc<Mutex<Allocator>>,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    size: u32,
    seed: u32,
) -> ReactorResult<Noise3D> {
    use gpu_allocator::MemoryLocation;

    let data = generate_value_noise_3d(size, seed);
    let extent = vk::Extent3D { width: size, height: size, depth: size };

    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_3D)
        .extent(extent)
        .mip_levels(1)
        .array_layers(1)
        .format(vk::Format::R8G8B8A8_UNORM)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1);
    let image = unsafe { device.create_image(&image_info, None) }.map_err(|e| {
        ReactorError::with_source(ErrorCode::VulkanImageCreation, "Clouds: create 3D noise image", e)
    })?;

    let req = unsafe { device.get_image_memory_requirements(image) };
    let allocation = allocator
        .lock()
        .unwrap()
        .allocate(&gpu_allocator::vulkan::AllocationCreateDesc {
            name: "clouds_noise_3d",
            requirements: req,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
        })
        .map_err(|e| ReactorError::with_source(ErrorCode::VulkanMemoryAllocation, "Clouds: allocate 3D noise memory", e))?;
    unsafe {
        device.bind_image_memory(image, allocation.memory(), allocation.offset()).map_err(|e| {
            ReactorError::with_source(ErrorCode::VulkanMemoryAllocation, "Clouds: bind 3D noise memory", e)
        })?;
    }

    let staging_size = data.len() as u64;
    let staging_buffer = crate::graphics::Buffer::new(
        ctx,
        allocator.clone(),
        staging_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    staging_buffer.write_slice(&data);

    let cmd_alloc = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let cmd = unsafe { device.allocate_command_buffers(&cmd_alloc) }.map_err(|e| {
        ReactorError::with_source(ErrorCode::VulkanCommandBuffer, "Clouds: allocate cmd buffer", e)
    })?[0];
    let begin = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(cmd, &begin).map_err(|e| {
        ReactorError::with_source(ErrorCode::VulkanCommandBuffer, "Clouds: begin cmd buffer", e)
    })?; }

    let to_transfer = vk::ImageMemoryBarrier::default()
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[], &[], &[to_transfer],
        );
    }

    let copy = vk::BufferImageCopy::default()
        .buffer_offset(0).buffer_row_length(0).buffer_image_height(0)
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1,
        })
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(extent);
    unsafe {
        device.cmd_copy_buffer_to_image(cmd, staging_buffer.handle, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[copy]);
    }

    let to_shader = vk::ImageMemoryBarrier::default()
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::SHADER_READ)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1,
        });
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[], &[], &[to_shader],
        );
    }
    unsafe { device.end_command_buffer(cmd).map_err(|e| {
        ReactorError::with_source(ErrorCode::VulkanCommandBuffer, "Clouds: end cmd buffer", e)
    })?; }

    let cbs = [cmd];
    let submit = vk::SubmitInfo::default().command_buffers(&cbs);
    unsafe {
        device.queue_submit(queue, &[submit], vk::Fence::null()).map_err(|e| {
            ReactorError::with_source(ErrorCode::VulkanSynchronization, "Clouds: submit", e)
        })?;
        device.queue_wait_idle(queue).map_err(|e| {
            ReactorError::with_source(ErrorCode::VulkanSynchronization, "Clouds: wait idle", e)
        })?;
    }
    unsafe { device.free_command_buffers(command_pool, &[cmd]); }
    drop(staging_buffer);

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_3D)
        .format(vk::Format::R8G8B8A8_UNORM)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1,
        });
    let view = unsafe { device.create_image_view(&view_info, None) }.map_err(|e| {
        ReactorError::with_source(ErrorCode::VulkanImageCreation, "Clouds: create 3D noise view", e)
    })?;

    Ok(Noise3D { image, view, allocation })
}
