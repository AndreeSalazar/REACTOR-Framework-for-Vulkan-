//! MSAA — selección del nivel de muestreo y creación del buffer multi-sample.
//!
//! Funciones internas usadas por `init.rs` y `swapchain_recreate.rs`.

use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use ash::vk;

/// Devuelve el máximo MSAA soportado por la GPU (color ∩ depth).
///
/// Prefiere 8x > 4x > 2x > 1x.
#[allow(dead_code)]
pub(super) fn max_supported(context: &VulkanContext) -> vk::SampleCountFlags {
    let props = unsafe {
        context
            .instance
            .get_physical_device_properties(context.physical_device)
    };
    let counts = props.limits.framebuffer_color_sample_counts
        & props.limits.framebuffer_depth_sample_counts;

    if counts.contains(vk::SampleCountFlags::TYPE_8) {
        vk::SampleCountFlags::TYPE_8
    } else if counts.contains(vk::SampleCountFlags::TYPE_4) {
        vk::SampleCountFlags::TYPE_4
    } else if counts.contains(vk::SampleCountFlags::TYPE_2) {
        vk::SampleCountFlags::TYPE_2
    } else {
        vk::SampleCountFlags::TYPE_1
    }
}

/// Convierte una petición de usuario (1/2/4/8) a `vk::SampleCountFlags`,
/// degradando al máximo soportado si la GPU no llega.
pub(super) fn msaa_from_u32(requested: u32, context: &VulkanContext) -> vk::SampleCountFlags {
    let props = unsafe {
        context
            .instance
            .get_physical_device_properties(context.physical_device)
    };
    let counts = props.limits.framebuffer_color_sample_counts
        & props.limits.framebuffer_depth_sample_counts;

    let requested_flag = match requested {
        1 => vk::SampleCountFlags::TYPE_1,
        2 => vk::SampleCountFlags::TYPE_2,
        4 => vk::SampleCountFlags::TYPE_4,
        8 => vk::SampleCountFlags::TYPE_8,
        _ => vk::SampleCountFlags::TYPE_1,
    };

    if counts.contains(requested_flag) {
        requested_flag
    } else if counts.contains(vk::SampleCountFlags::TYPE_4) {
        vk::SampleCountFlags::TYPE_4
    } else if counts.contains(vk::SampleCountFlags::TYPE_2) {
        vk::SampleCountFlags::TYPE_2
    } else {
        vk::SampleCountFlags::TYPE_1
    }
}

/// Crea la imagen multi-sample + su view + su memoria, para uso como
/// color attachment transitorio (no se almacena en disco, sólo se resuelve
/// al swapchain en el draw).
pub(super) fn create_msaa_resources(
    context: &VulkanContext,
    width: u32,
    height: u32,
    format: vk::Format,
    samples: vk::SampleCountFlags,
) -> ReactorResult<(vk::Image, vk::ImageView, vk::DeviceMemory)> {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D { width, height, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
        )
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples);

    let image = unsafe {
        context.device.create_image(&image_info, None).map_err(|e| {
            ReactorError::with_source(ErrorCode::VulkanImageCreation, "Failed to create MSAA image", e)
        })?
    };
    let requirements = unsafe { context.device.get_image_memory_requirements(image) };

    let memory_props = unsafe {
        context
            .instance
            .get_physical_device_memory_properties(context.physical_device)
    };
    let memory_type_index = (0..memory_props.memory_type_count)
        .find(|&i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory_props.memory_types[i as usize];
            suitable
                && memory_type
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
        })
        .ok_or_else(|| {
            ReactorError::new(
                ErrorCode::VulkanMemoryAllocation,
                "Failed to find suitable memory type for MSAA",
            )
        })?;

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);

    let memory = unsafe {
        context.device.allocate_memory(&alloc_info, None).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanMemoryAllocation,
                "Failed to allocate MSAA memory",
                e,
            )
        })?
    };
    unsafe {
        context.device.bind_image_memory(image, memory, 0).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanImageCreation,
                "Failed to bind MSAA memory",
                e,
            )
        })?
    };

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    let view = unsafe {
        context.device.create_image_view(&view_info, None).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanImageCreation,
                "Failed to create MSAA image view",
                e,
            )
        })?
    };

    Ok((image, view, memory))
}
