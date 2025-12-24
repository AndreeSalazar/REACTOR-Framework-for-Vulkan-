use ash::vk;
use crate::core::context::VulkanContext;
use std::error::Error;

pub struct RenderPass {
    pub handle: vk::RenderPass,
    device: ash::Device,
}

pub struct RenderPassConfig {
    pub color_format: vk::Format,
    pub depth_format: Option<vk::Format>,
    pub samples: vk::SampleCountFlags,
    pub clear_color: bool,
    pub clear_depth: bool,
}

impl Default for RenderPassConfig {
    fn default() -> Self {
        Self {
            color_format: vk::Format::B8G8R8A8_SRGB,
            depth_format: Some(vk::Format::D32_SFLOAT),
            samples: vk::SampleCountFlags::TYPE_1,
            clear_color: true,
            clear_depth: true,
        }
    }
}

impl RenderPass {
    pub fn new(ctx: &VulkanContext, config: &RenderPassConfig) -> Result<Self, Box<dyn Error>> {
        let mut attachments = Vec::new();
        let mut attachment_refs = Vec::new();

        // Color attachment
        let color_load_op = if config.clear_color { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD };
        
        let color_attachment = vk::AttachmentDescription::default()
            .format(config.color_format)
            .samples(config.samples)
            .load_op(color_load_op)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        attachments.push(color_attachment);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        attachment_refs.push(color_attachment_ref);

        // Depth attachment (optional)
        let depth_attachment_ref = if let Some(depth_format) = config.depth_format {
            let depth_load_op = if config.clear_depth { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD };
            
            let depth_attachment = vk::AttachmentDescription::default()
                .format(depth_format)
                .samples(config.samples)
                .load_op(depth_load_op)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

            attachments.push(depth_attachment);

            Some(vk::AttachmentReference::default()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL))
        } else {
            None
        };

        let color_attachments = [color_attachment_ref];
        let mut subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments);

        if let Some(ref depth_ref) = depth_attachment_ref {
            subpass = subpass.depth_stencil_attachment(depth_ref);
        }

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

        let subpasses = [subpass];
        let dependencies = [dependency];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let handle = unsafe { ctx.device.create_render_pass(&render_pass_info, None)? };

        Ok(Self {
            handle,
            device: ctx.device.clone(),
        })
    }

    pub fn simple(ctx: &VulkanContext, color_format: vk::Format) -> Result<Self, Box<dyn Error>> {
        Self::new(ctx, &RenderPassConfig {
            color_format,
            depth_format: None,
            ..Default::default()
        })
    }

    pub fn with_depth(ctx: &VulkanContext, color_format: vk::Format, depth_format: vk::Format) -> Result<Self, Box<dyn Error>> {
        Self::new(ctx, &RenderPassConfig {
            color_format,
            depth_format: Some(depth_format),
            ..Default::default()
        })
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_render_pass(self.handle, None);
        }
    }
}
