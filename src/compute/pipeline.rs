use ash::vk;
use crate::core::context::VulkanContext;
use std::error::Error;
use std::ffi::CStr;

pub struct ComputePipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    device: ash::Device,
}

impl ComputePipeline {
    pub fn new(
        ctx: &VulkanContext,
        shader_code: &[u32],
        descriptor_layouts: &[vk::DescriptorSetLayout],
        push_constant_size: Option<u32>,
    ) -> Result<Self, Box<dyn Error>> {
        let shader_module = unsafe {
            let create_info = vk::ShaderModuleCreateInfo::default().code(shader_code);
            ctx.device.create_shader_module(&create_info, None)?
        };

        let stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap());

        // Pipeline layout
        let push_constant_ranges = if let Some(size) = push_constant_size {
            vec![vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                offset: 0,
                size,
            }]
        } else {
            vec![]
        };

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let layout = unsafe { ctx.device.create_pipeline_layout(&layout_info, None)? };

        let pipeline_info = vk::ComputePipelineCreateInfo::default()
            .stage(stage_info)
            .layout(layout);

        let pipelines = unsafe {
            ctx.device.create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, e)| e)?
        };

        unsafe { ctx.device.destroy_shader_module(shader_module, None); }

        Ok(Self {
            pipeline: pipelines[0],
            layout,
            device: ctx.device.clone(),
        })
    }

    pub fn bind(&self, command_buffer: vk::CommandBuffer, device: &ash::Device) {
        unsafe {
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, self.pipeline);
        }
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
