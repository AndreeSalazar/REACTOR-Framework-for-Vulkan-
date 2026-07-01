use crate::core::error::ReactorResult;
use crate::core::VulkanContext;
use crate::graphics::ibl::image::IblImage;
use crate::graphics::ibl::{verr, RGBA16F};
use ash::vk;
use bytemuck;
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocationScheme, Allocator};
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};

pub fn upload_equirect_hdr(
    ctx: &VulkanContext,
    allocator: Arc<Mutex<Allocator>>,
    pool: vk::CommandPool,
    pixels: &[u16],
    width: u32,
    height: u32,
) -> ReactorResult<IblImage> {
    let device = ctx.ash_device();
    let extent = vk::Extent3D { width, height, depth: 1 };
    let img_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D).extent(extent).mip_levels(1)
        .array_layers(1).format(RGBA16F).tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
        .sharing_mode(vk::SharingMode::EXCLUSIVE).samples(vk::SampleCountFlags::TYPE_1);
    let image = unsafe { device.create_image(&img_info, None).map_err(verr)? };
    let req = unsafe { device.get_image_memory_requirements(image) };
    let alloc = allocator.lock().unwrap()
        .allocate(&AllocationCreateDesc {
            name: "ibl_equirect_hdr", requirements: req, location: MemoryLocation::GpuOnly,
            linear: false, allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        }).map_err(verr)?;
    unsafe { device.bind_image_memory(image, alloc.memory(), alloc.offset()).map_err(verr)? };
    let view = unsafe {
        device.create_image_view(
            &vk::ImageViewCreateInfo::default().image(image)
                .view_type(vk::ImageViewType::TYPE_2D).format(RGBA16F)
                .subresource_range(vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0).level_count(1)
                    .base_array_layer(0).layer_count(1)),
            None,
        ).map_err(verr)?
    };
    let staging_info = vk::BufferCreateInfo::default()
        .size(std::mem::size_of_val(pixels) as u64)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let staging = unsafe { device.create_buffer(&staging_info, None).map_err(verr)? };
    let sreq = unsafe { device.get_buffer_memory_requirements(staging) };
    let mut salloc = allocator.lock().unwrap()
        .allocate(&AllocationCreateDesc {
            name: "ibl_equirect_staging", requirements: sreq,
            location: MemoryLocation::CpuToGpu, linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        }).map_err(verr)?;
    unsafe { device.bind_buffer_memory(staging, salloc.memory(), salloc.offset()).map_err(verr)? };
    let dst_slice = salloc.mapped_slice_mut().expect("staging not mapped");
    dst_slice[..std::mem::size_of_val(pixels)].copy_from_slice(bytemuck::cast_slice(pixels));
    let cmd = super::helpers::begin_one_shot(ctx, pool)?;
    super::helpers::transition_2d(ctx, cmd, image, 1,
        vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE,
        vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);
    let region = vk::BufferImageCopy::default().buffer_offset(0)
        .image_subresource(vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR).mip_level(0)
            .base_array_layer(0).layer_count(1))
        .image_extent(extent);
    unsafe { device.cmd_copy_buffer_to_image(cmd, staging, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]); }
    super::helpers::transition_2d(ctx, cmd, image, 1,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::COMPUTE_SHADER);
    super::helpers::end_and_submit(ctx, pool, cmd)?;
    unsafe { device.destroy_buffer(staging, None); }
    let _ = allocator.lock().unwrap().free(salloc);
    Ok(IblImage {
        image, allocation: Some(alloc), view, mip_views: vec![],
        format: RGBA16F, extent, mip_levels: 1, layer_count: 1,
        device: device.clone(), allocator,
    })
}
