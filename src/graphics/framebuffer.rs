use ash::vk;
use crate::vulkan_context::VulkanContext;
use std::error::Error;

pub struct Framebuffer {
    pub handle: vk::Framebuffer,
    pub width: u32,
    pub height: u32,
    device: ash::Device,
}

impl Framebuffer {
    pub fn new(
        ctx: &VulkanContext,
        render_pass: vk::RenderPass,
        attachments: &[vk::ImageView],
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(width)
            .height(height)
            .layers(1);

        let handle = unsafe { ctx.device.create_framebuffer(&framebuffer_info, None)? };

        Ok(Self {
            handle,
            width,
            height,
            device: ctx.device.clone(),
        })
    }

    pub fn from_swapchain(
        ctx: &VulkanContext,
        render_pass: vk::RenderPass,
        swapchain_view: vk::ImageView,
        depth_view: Option<vk::ImageView>,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let mut attachments = vec![swapchain_view];
        if let Some(depth) = depth_view {
            attachments.push(depth);
        }
        Self::new(ctx, render_pass, &attachments, width, height)
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_framebuffer(self.handle, None);
        }
    }
}

pub struct FramebufferSet {
    pub framebuffers: Vec<Framebuffer>,
}

impl FramebufferSet {
    pub fn from_swapchain(
        ctx: &VulkanContext,
        render_pass: vk::RenderPass,
        swapchain_views: &[vk::ImageView],
        depth_view: Option<vk::ImageView>,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let framebuffers = swapchain_views
            .iter()
            .map(|&view| Framebuffer::from_swapchain(ctx, render_pass, view, depth_view, width, height))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { framebuffers })
    }

    pub fn get(&self, index: usize) -> &Framebuffer {
        &self.framebuffers[index]
    }

    pub fn len(&self) -> usize {
        self.framebuffers.len()
    }
}
