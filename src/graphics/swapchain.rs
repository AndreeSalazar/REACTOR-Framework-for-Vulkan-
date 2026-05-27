use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use ash::khr::swapchain;
use ash::{vk, Device};

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
    pub fn new(ctx: &VulkanContext, width: u32, height: u32, vsync: bool) -> ReactorResult<Self> {
        let surface_loader = ctx.surface_loader();
        let surface = ctx.surface_khr();
        let device = ctx.ash_device();

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(ctx.physical_device, surface)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSwapchainCreation,
                        "get_surface_capabilities failed",
                        e,
                    )
                })?
        };

        let surface_formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(ctx.physical_device, surface)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSwapchainCreation,
                        "get_surface_formats failed",
                        e,
                    )
                })?
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(ctx.physical_device, surface)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSwapchainCreation,
                        "get_present_modes failed",
                        e,
                    )
                })?
        };

        let format = surface_formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&surface_formats[0]);

        let present_mode = if vsync {
            // VSync enabled: prefer FIFO (standard, locks to refresh rate), then MAILBOX
            present_modes
                .iter()
                .find(|&&p| p == vk::PresentModeKHR::FIFO)
                .or_else(|| present_modes.iter().find(|&&p| p == vk::PresentModeKHR::MAILBOX))
                .unwrap_or(&vk::PresentModeKHR::FIFO)
        } else {
            // VSync disabled: prefer IMMEDIATE (absolute lowest latency, unlocked FPS), then MAILBOX (unlocked FPS, no tearing), then FIFO
            present_modes
                .iter()
                .find(|&&p| p == vk::PresentModeKHR::IMMEDIATE)
                .or_else(|| present_modes.iter().find(|&&p| p == vk::PresentModeKHR::MAILBOX))
                .unwrap_or(&vk::PresentModeKHR::FIFO)
        };

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
            .surface(surface)
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

        let loader = swapchain::Device::new(ctx.ash_instance(), device);
        let handle = unsafe {
            loader.create_swapchain(&create_info, None).map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanSwapchainCreation,
                    "create_swapchain failed",
                    e,
                )
            })?
        };
        let images = unsafe {
            loader.get_swapchain_images(handle).map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanSwapchainCreation,
                    "get_swapchain_images failed",
                    e,
                )
            })?
        };

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
                unsafe {
                    device.create_image_view(&create_info, None).map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanSwapchainCreation,
                            "create_image_view failed",
                            e,
                        )
                    })
                }
            })
            .collect::<ReactorResult<Vec<_>>>()?;

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
