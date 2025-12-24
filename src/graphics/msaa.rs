use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use crate::core::context::VulkanContext;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct MsaaTarget {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub samples: vk::SampleCountFlags,
    allocation: Option<Allocation>,
    device: ash::Device,
    allocator: Arc<Mutex<Allocator>>,
}

impl MsaaTarget {
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        format: vk::Format,
        samples: vk::SampleCountFlags,
    ) -> Result<Self, Box<dyn Error>> {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(samples);

        let image = unsafe { ctx.device.create_image(&image_info, None)? };
        let requirements = unsafe { ctx.device.get_image_memory_requirements(image) };

        let allocation = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
            name: "msaa_target",
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
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        let view = unsafe { ctx.device.create_image_view(&view_info, None)? };

        Ok(Self {
            image,
            view,
            samples,
            allocation: Some(allocation),
            device: ctx.device.clone(),
            allocator,
        })
    }

    pub fn get_max_samples(ctx: &VulkanContext) -> vk::SampleCountFlags {
        let props = unsafe { ctx.instance.get_physical_device_properties(ctx.physical_device) };
        let counts = props.limits.framebuffer_color_sample_counts
            & props.limits.framebuffer_depth_sample_counts;

        if counts.contains(vk::SampleCountFlags::TYPE_64) { vk::SampleCountFlags::TYPE_64 }
        else if counts.contains(vk::SampleCountFlags::TYPE_32) { vk::SampleCountFlags::TYPE_32 }
        else if counts.contains(vk::SampleCountFlags::TYPE_16) { vk::SampleCountFlags::TYPE_16 }
        else if counts.contains(vk::SampleCountFlags::TYPE_8) { vk::SampleCountFlags::TYPE_8 }
        else if counts.contains(vk::SampleCountFlags::TYPE_4) { vk::SampleCountFlags::TYPE_4 }
        else if counts.contains(vk::SampleCountFlags::TYPE_2) { vk::SampleCountFlags::TYPE_2 }
        else { vk::SampleCountFlags::TYPE_1 }
    }
}

impl Drop for MsaaTarget {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.image, None);
        }
        if let Some(allocation) = self.allocation.take() {
            if let Err(e) = self.allocator.lock().unwrap().free(allocation) {
                eprintln!("Failed to free MSAA target memory: {:?}", e);
            }
        }
    }
}
