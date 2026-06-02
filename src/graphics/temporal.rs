use crate::core::{ReactorResult, VulkanContext};
use crate::graphics::Image;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use std::sync::{Arc, Mutex};

pub struct TemporalHistory {
    pub width: u32,
    pub height: u32,
    pub color_a: Image,
    pub color_b: Image,
    pub depth_a: Image,
    pub depth_b: Image,
    pub storage_writes_supported: bool,
    frame_index: u64,
}

impl TemporalHistory {
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
    ) -> ReactorResult<Self> {
        let storage_writes_supported =
            format_supports_storage(ctx, vk::Format::R16G16B16A16_SFLOAT)
                && format_supports_storage(ctx, vk::Format::R32_SFLOAT);

        let color_a = create_history_image(
            ctx,
            allocator.clone(),
            width,
            height,
            vk::Format::R16G16B16A16_SFLOAT,
            storage_writes_supported,
            "TemporalHistory_Color_A",
        )?;
        let color_b = create_history_image(
            ctx,
            allocator.clone(),
            width,
            height,
            vk::Format::R16G16B16A16_SFLOAT,
            storage_writes_supported,
            "TemporalHistory_Color_B",
        )?;
        let depth_a = create_history_image(
            ctx,
            allocator.clone(),
            width,
            height,
            vk::Format::R32_SFLOAT,
            storage_writes_supported,
            "TemporalHistory_Depth_A",
        )?;
        let depth_b = create_history_image(
            ctx,
            allocator,
            width,
            height,
            vk::Format::R32_SFLOAT,
            storage_writes_supported,
            "TemporalHistory_Depth_B",
        )?;

        Ok(Self {
            width,
            height,
            color_a,
            color_b,
            depth_a,
            depth_b,
            storage_writes_supported,
            frame_index: 0,
        })
    }

    pub fn current_color(&self) -> &Image {
        if self.frame_index % 2 == 0 {
            &self.color_a
        } else {
            &self.color_b
        }
    }

    pub fn previous_color(&self) -> &Image {
        if self.frame_index % 2 == 0 {
            &self.color_b
        } else {
            &self.color_a
        }
    }

    pub fn current_depth(&self) -> &Image {
        if self.frame_index % 2 == 0 {
            &self.depth_a
        } else {
            &self.depth_b
        }
    }

    pub fn previous_depth(&self) -> &Image {
        if self.frame_index % 2 == 0 {
            &self.depth_b
        } else {
            &self.depth_a
        }
    }

    pub fn advance(&mut self) {
        self.frame_index = self.frame_index.wrapping_add(1);
    }

    pub fn reset(&mut self) {
        self.frame_index = 0;
    }

    pub fn estimated_bytes(&self) -> u64 {
        Self::estimated_bytes_for_extent(self.width, self.height)
    }

    pub fn estimated_bytes_for_extent(width: u32, height: u32) -> u64 {
        let bytes_per_pixel = (8 + 4) * 2;
        width as u64 * height as u64 * bytes_per_pixel
    }

    pub fn estimated_mib_for_extent(width: u32, height: u32) -> f32 {
        Self::estimated_bytes_for_extent(width, height) as f32 / (1024.0 * 1024.0)
    }
}

fn create_history_image(
    ctx: &VulkanContext,
    allocator: Arc<Mutex<Allocator>>,
    width: u32,
    height: u32,
    format: vk::Format,
    storage_writes_supported: bool,
    name: &str,
) -> ReactorResult<Image> {
    let mut usage = vk::ImageUsageFlags::SAMPLED
        | vk::ImageUsageFlags::TRANSFER_SRC
        | vk::ImageUsageFlags::TRANSFER_DST;

    if storage_writes_supported {
        usage |= vk::ImageUsageFlags::STORAGE;
    }

    let image = Image::new(
        ctx,
        allocator,
        width,
        height,
        format,
        usage,
        vk::ImageAspectFlags::COLOR,
        1,
    )?;

    ctx.debug_namer()
        .name_image(image.handle, &format!("Image: {}", name));
    ctx.debug_namer()
        .name_image_view(image.view, &format!("ImageView: {}", name));

    Ok(image)
}

fn format_supports_storage(ctx: &VulkanContext, format: vk::Format) -> bool {
    let props = unsafe {
        ctx.instance
            .get_physical_device_format_properties(ctx.physical_device, format)
    };
    props
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::STORAGE_IMAGE)
}
