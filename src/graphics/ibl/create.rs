use crate::core::error::ReactorResult;
use crate::core::VulkanContext;
use crate::graphics::ibl::image::IblImage;
use crate::graphics::ibl::{verr, RG16F, RGBA16F};
use ash::vk;
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocationScheme, Allocator};
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};

pub fn create_cubemap(
    ctx: &VulkanContext,
    allocator: Arc<Mutex<Allocator>>,
    size: u32,
    mip_levels: u32,
    usage: vk::ImageUsageFlags,
) -> ReactorResult<IblImage> {
    let device = ctx.ash_device();
    let extent = vk::Extent3D { width: size, height: size, depth: 1 };
    let img_info = vk::ImageCreateInfo::default()
        .flags(vk::ImageCreateFlags::CUBE_COMPATIBLE)
        .image_type(vk::ImageType::TYPE_2D).extent(extent).mip_levels(mip_levels)
        .array_layers(6).format(RGBA16F).tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED).usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE).samples(vk::SampleCountFlags::TYPE_1);
    let image = unsafe { device.create_image(&img_info, None).map_err(verr)? };
    let req = unsafe { device.get_image_memory_requirements(image) };
    let alloc = allocator.lock().unwrap()
        .allocate(&AllocationCreateDesc {
            name: "ibl_cubemap", requirements: req, location: MemoryLocation::GpuOnly,
            linear: false, allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        }).map_err(verr)?;
    unsafe { device.bind_image_memory(image, alloc.memory(), alloc.offset()).map_err(verr)? };
    let view = unsafe {
        device.create_image_view(
            &vk::ImageViewCreateInfo::default().image(image)
                .view_type(vk::ImageViewType::CUBE).format(RGBA16F)
                .subresource_range(vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0).level_count(mip_levels)
                    .base_array_layer(0).layer_count(6)),
            None,
        ).map_err(verr)?
    };
    let mut mip_views = Vec::with_capacity(mip_levels as usize);
    for mip in 0..mip_levels {
        let v = unsafe {
            device.create_image_view(
                &vk::ImageViewCreateInfo::default().image(image)
                    .view_type(vk::ImageViewType::TYPE_2D_ARRAY).format(RGBA16F)
                    .subresource_range(vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(mip).level_count(1)
                        .base_array_layer(0).layer_count(6)),
                None,
            ).map_err(verr)?
        };
        mip_views.push(v);
    }
    Ok(IblImage {
        image, allocation: Some(alloc), view, mip_views,
        format: RGBA16F, extent, mip_levels, layer_count: 6,
        device: device.clone(), allocator,
    })
}

pub fn create_2d_lut(
    ctx: &VulkanContext,
    allocator: Arc<Mutex<Allocator>>,
    size: u32,
    usage: vk::ImageUsageFlags,
) -> ReactorResult<IblImage> {
    let device = ctx.ash_device();
    let extent = vk::Extent3D { width: size, height: size, depth: 1 };
    let img_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D).extent(extent).mip_levels(1)
        .array_layers(1).format(RG16F).tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED).usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE).samples(vk::SampleCountFlags::TYPE_1);
    let image = unsafe { device.create_image(&img_info, None).map_err(verr)? };
    let req = unsafe { device.get_image_memory_requirements(image) };
    let alloc = allocator.lock().unwrap()
        .allocate(&AllocationCreateDesc {
            name: "ibl_brdf_lut", requirements: req, location: MemoryLocation::GpuOnly,
            linear: false, allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        }).map_err(verr)?;
    unsafe { device.bind_image_memory(image, alloc.memory(), alloc.offset()).map_err(verr)? };
    let view = unsafe {
        device.create_image_view(
            &vk::ImageViewCreateInfo::default().image(image)
                .view_type(vk::ImageViewType::TYPE_2D).format(RG16F)
                .subresource_range(vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0).level_count(1)
                    .base_array_layer(0).layer_count(1)),
            None,
        ).map_err(verr)?
    };
    Ok(IblImage {
        image, allocation: Some(alloc), view,
        mip_views: vec![view], format: RG16F, extent,
        mip_levels: 1, layer_count: 1,
        device: device.clone(), allocator,
    })
}
