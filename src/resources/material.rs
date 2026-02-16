use ash::vk;
use std::sync::Arc;
use crate::graphics::pipeline::{Pipeline, PipelineConfig};
use crate::vulkan_context::VulkanContext;
use crate::resources::texture::Texture;
use std::error::Error;

pub struct Material {
    pub pipeline: Arc<Pipeline>,
    pub descriptor_set: Option<vk::DescriptorSet>,
    pub descriptor_pool: Option<vk::DescriptorPool>,
    pub descriptor_layout: Option<vk::DescriptorSetLayout>,
    device: Option<ash::Device>,
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
            descriptor_pool: None,
            descriptor_layout: None,
            device: None,
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
            descriptor_pool: None,
            descriptor_layout: None,
            device: None,
        })
    }

    /// Create a textured material with a diffuse texture
    pub fn with_texture(
        ctx: &VulkanContext,
        render_pass: vk::RenderPass,
        vert_code: &[u32],
        frag_code: &[u32],
        width: u32,
        height: u32,
        texture: &Texture,
        msaa_samples: vk::SampleCountFlags,
    ) -> Result<Self, Box<dyn Error>> {
        // Create descriptor set layout for texture sampler
        let sampler_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let bindings = [sampler_binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings);

        let descriptor_layout = unsafe {
            ctx.device.create_descriptor_set_layout(&layout_info, None)?
        };

        // Create descriptor pool
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1);

        let pool_sizes = [pool_size];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1);

        let descriptor_pool = unsafe {
            ctx.device.create_descriptor_pool(&pool_info, None)?
        };

        // Allocate descriptor set
        let layouts = [descriptor_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe {
            ctx.device.allocate_descriptor_sets(&alloc_info)?
        };
        let descriptor_set = descriptor_sets[0];

        // Update descriptor set with texture
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture.view())
            .sampler(texture.sampler_handle());

        let image_infos = [image_info];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_infos);

        unsafe {
            ctx.device.update_descriptor_sets(&[write], &[]);
        }

        // Create pipeline with descriptor layout
        let config = PipelineConfig {
            samples: msaa_samples,
            ..PipelineConfig::default()
        };

        let pipeline = Pipeline::with_config(
            &ctx.device,
            render_pass,
            vert_code,
            frag_code,
            width,
            height,
            &config,
            &[descriptor_layout],
        )?;

        Ok(Self {
            pipeline: Arc::new(pipeline),
            descriptor_set: Some(descriptor_set),
            descriptor_pool: Some(descriptor_pool),
            descriptor_layout: Some(descriptor_layout),
            device: Some(ctx.device.clone()),
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

impl Drop for Material {
    fn drop(&mut self) {
        if let Some(device) = &self.device {
            unsafe {
                if let Some(pool) = self.descriptor_pool {
                    device.destroy_descriptor_pool(pool, None);
                }
                if let Some(layout) = self.descriptor_layout {
                    device.destroy_descriptor_set_layout(layout, None);
                }
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
