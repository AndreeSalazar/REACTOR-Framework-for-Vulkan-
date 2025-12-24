use ash::vk;
use std::sync::Arc;
use crate::graphics::pipeline::{Pipeline, PipelineConfig};
use crate::core::context::VulkanContext;
use std::error::Error;

pub struct Material {
    pub pipeline: Arc<Pipeline>,
    pub descriptor_set: Option<vk::DescriptorSet>,
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
            descriptor_set: None,
        })
    }

    pub fn with_config(
        ctx: &VulkanContext,
        render_pass: vk::RenderPass,
        vert_code: &[u32],
        frag_code: &[u32],
        width: u32,
        height: u32,
        config: &PipelineConfig,
        descriptor_layouts: &[vk::DescriptorSetLayout],
    ) -> Result<Self, Box<dyn Error>> {
        let pipeline = Pipeline::with_config(
            &ctx.device,
            render_pass,
            vert_code,
            frag_code,
            width,
            height,
            config,
            descriptor_layouts,
        )?;

        Ok(Self {
            pipeline: Arc::new(pipeline),
            descriptor_set: None,
        })
    }

    pub fn with_descriptor_set(mut self, descriptor_set: vk::DescriptorSet) -> Self {
        self.descriptor_set = Some(descriptor_set);
        self
    }

    pub fn bind(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipeline);
            
            if let Some(descriptor_set) = self.descriptor_set {
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline.layout,
                    0,
                    &[descriptor_set],
                    &[],
                );
            }
        }
    }
}

// Common material types
pub struct MaterialBuilder {
    pub vert_code: Vec<u32>,
    pub frag_code: Vec<u32>,
    pub config: PipelineConfig,
    pub descriptor_layouts: Vec<vk::DescriptorSetLayout>,
}

impl MaterialBuilder {
    pub fn new(vert_code: Vec<u32>, frag_code: Vec<u32>) -> Self {
        Self {
            vert_code,
            frag_code,
            config: PipelineConfig::default(),
            descriptor_layouts: Vec::new(),
        }
    }

    pub fn cull_mode(mut self, mode: vk::CullModeFlags) -> Self {
        self.config.cull_mode = mode;
        self
    }

    pub fn no_cull(mut self) -> Self {
        self.config.cull_mode = vk::CullModeFlags::NONE;
        self
    }

    pub fn wireframe(mut self) -> Self {
        self.config.polygon_mode = vk::PolygonMode::LINE;
        self
    }

    pub fn no_depth(mut self) -> Self {
        self.config.depth_test = false;
        self.config.depth_write = false;
        self
    }

    pub fn blend(mut self) -> Self {
        self.config.blend_enable = true;
        self
    }

    pub fn msaa(mut self, samples: vk::SampleCountFlags) -> Self {
        self.config.samples = samples;
        self
    }

    pub fn descriptor_layout(mut self, layout: vk::DescriptorSetLayout) -> Self {
        self.descriptor_layouts.push(layout);
        self
    }

    pub fn build(
        self,
        ctx: &VulkanContext,
        render_pass: vk::RenderPass,
        width: u32,
        height: u32,
    ) -> Result<Material, Box<dyn Error>> {
        Material::with_config(
            ctx,
            render_pass,
            &self.vert_code,
            &self.frag_code,
            width,
            height,
            &self.config,
            &self.descriptor_layouts,
        )
    }
}
