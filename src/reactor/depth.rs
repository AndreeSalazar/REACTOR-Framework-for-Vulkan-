//! Depth buffer — selección de formato y creación del recurso GPU.

use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use ash::vk;

/// Busca un formato de depth soportado por la GPU.
///
/// Preferencia: D32_SFLOAT > D32_SFLOAT_S8_UINT > D24_UNORM_S8_UINT.
pub(super) fn find_depth_format(context: &VulkanContext) -> ReactorResult<vk::Format> {
    let candidates = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    for &format in &candidates {
        let props = unsafe {
            context
                .instance
                .get_physical_device_format_properties(context.physical_device, format)
        };
        if props
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
        {
            return Ok(format);
        }
    }

    Err(ReactorError::new(
        ErrorCode::VulkanRenderPass,
        "Failed to find supported depth format",
    ))
}

/// Crea la imagen de depth + view + memoria con MSAA opcional.
pub(super) fn create_depth_resources(
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
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples);

    let image = unsafe {
        context.device.create_image(&image_info, None).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanImageCreation,
                "Failed to create depth image",
                e,
            )
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
                "Failed to find suitable memory type for depth buffer",
            )
        })?;

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);

    let memory = unsafe {
        context.device.allocate_memory(&alloc_info, None).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanMemoryAllocation,
                "Failed to allocate depth memory",
                e,
            )
        })?
    };
    unsafe {
        context.device.bind_image_memory(image, memory, 0).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanImageCreation,
                "Failed to bind depth memory",
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
                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    let view = unsafe {
        context.device.create_image_view(&view_info, None).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanImageCreation,
                "Failed to create depth image view",
                e,
            )
        })?
    };

    Ok((image, view, memory))
}
