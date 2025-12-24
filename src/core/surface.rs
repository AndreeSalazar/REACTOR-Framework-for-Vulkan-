use ash::vk;

#[derive(Debug, Clone)]
pub struct SurfaceInfo {
    pub format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub capabilities: vk::SurfaceCapabilitiesKHR,
}

impl SurfaceInfo {
    pub fn query(
        surface_loader: &ash::khr::surface::Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<Self, vk::Result> {
        let capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
        };
        
        let formats = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)?
        };
        
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
        };

        // Prefer SRGB format
        let format = formats
            .iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .unwrap_or(&formats[0])
            .clone();

        // Prefer Mailbox (triple buffering), fallback to FIFO
        let present_mode = present_modes
            .iter()
            .find(|&&p| p == vk::PresentModeKHR::MAILBOX)
            .copied()
            .unwrap_or(vk::PresentModeKHR::FIFO);

        Ok(Self {
            format,
            present_mode,
            capabilities,
        })
    }
}
