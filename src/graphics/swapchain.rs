use ash::{vk, Device};
use ash::khr::swapchain;
use crate::core::context::VulkanContext;
use std::error::Error;

pub struct Swapchain {
    pub loader: swapchain::Device,
    pub handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_count: u32,
}

impl Swapchain {
    pub fn new(ctx: &VulkanContext, width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let surface_capabilities = unsafe {
            ctx.surface_loader
                .get_physical_device_surface_capabilities(ctx.physical_device, ctx.surface)?
        };
        
        let surface_formats = unsafe {
            ctx.surface_loader
                .get_physical_device_surface_formats(ctx.physical_device, ctx.surface)?
        };
        
        let present_modes = unsafe {
            ctx.surface_loader
                .get_physical_device_surface_present_modes(ctx.physical_device, ctx.surface)?
        };

        let format = surface_formats
            .iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .unwrap_or(&surface_formats[0]);

        let present_mode = present_modes
            .iter()
            .find(|&p| *p == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(&vk::PresentModeKHR::FIFO);

        let extent = if surface_capabilities.current_extent.width != u32::MAX {
            surface_capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    surface_capabilities.min_image_extent.width,
                    surface_capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    surface_capabilities.min_image_extent.height,
                    surface_capabilities.max_image_extent.height,
                ),
            }
        };

        let image_count = if surface_capabilities.max_image_count > 0 {
            (surface_capabilities.min_image_count + 1).min(surface_capabilities.max_image_count)
        } else {
            surface_capabilities.min_image_count + 1
        };

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(ctx.surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(*present_mode)
            .clipped(true);

        let loader = swapchain::Device::new(&ctx.instance, &ctx.device);
        let handle = unsafe { loader.create_swapchain(&create_info, None)? };
        let images = unsafe { loader.get_swapchain_images(handle)? };
        
        let image_views = images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format.format)
                    .components(vk::ComponentMapping::default())
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    );
                unsafe { ctx.device.create_image_view(&create_info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            loader,
            handle,
            images,
            image_views,
            format: format.format,
            extent,
            image_count,
        })
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            for &view in &self.image_views {
                device.destroy_image_view(view, None);
            }
            self.loader.destroy_swapchain(self.handle, None);
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.extent.width as f32 / self.extent.height as f32
    }
}
