use ash::vk;
use std::sync::Arc;
use crate::pipeline::Pipeline;
use crate::vulkan_context::VulkanContext;
use std::error::Error;

pub struct Material {
    pub pipeline: Arc<Pipeline>,
}

impl Material {
    pub fn new(
        ctx: &VulkanContext,
        render_pass: vk::RenderPass,
        vert_code: &[u32],
        frag_code: &[u32],
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let pipeline = Pipeline::new(
            &ctx.device,
            render_pass,
            vert_code,
            frag_code,
            width,
            height,
        )?;

        Ok(Self {
            pipeline: Arc::new(pipeline),
        })
    }
}

// We need to manage pipeline lifetime. For now, Reactor or Material system should handle it.
// Since we used Arc<Pipeline>, we can share it. But Pipeline needs explicit destroy.
// For "Very Easy", we might need a centralized resource manager.
// For now, we assume user keeps Material alive or drops it properly.
// But wait, Pipeline struct doesn't implement Drop, it has a destroy method.
// We should implement Drop for Pipeline or wrap it.
