use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use crate::core::context::VulkanContext;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct DepthBuffer {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub format: vk::Format,
    allocation: Option<Allocation>,
    device: ash::Device,
    allocator: Arc<Mutex<Allocator>>,
}

impl DepthBuffer {
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let format = Self::find_depth_format(ctx)?;

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
            .samples(vk::SampleCountFlags::TYPE_1);

        let image = unsafe { ctx.device.create_image(&image_info, None)? };
        let requirements = unsafe { ctx.device.get_image_memory_requirements(image) };

        let allocation = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
            name: "depth_buffer",
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            ctx.device.bind_image_memory(image, allocation.memory(), allocation.offset())?;
        }

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

        let view = unsafe { ctx.device.create_image_view(&view_info, None)? };

        Ok(Self {
            image,
            view,
            format,
            allocation: Some(allocation),
            device: ctx.device.clone(),
            allocator,
        })
    }

    fn find_depth_format(ctx: &VulkanContext) -> Result<vk::Format, Box<dyn Error>> {
        let candidates = [
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];

        for format in candidates {
            let props = unsafe {
                ctx.instance.get_physical_device_format_properties(ctx.physical_device, format)
            };

            if props.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
                return Ok(format);
            }
        }

        Err("Failed to find supported depth format".into())
    }

    pub fn has_stencil(&self) -> bool {
        matches!(self.format, vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT)
    }
}

impl Drop for DepthBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.image, None);
        }
        if let Some(allocation) = self.allocation.take() {
            if let Err(e) = self.allocator.lock().unwrap().free(allocation) {
                eprintln!("Failed to free depth buffer memory: {:?}", e);
            }
        }
    }
}
