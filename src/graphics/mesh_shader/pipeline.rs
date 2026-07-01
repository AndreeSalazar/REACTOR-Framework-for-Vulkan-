use std::ffi::CStr;
use ash::vk;
use crate::core::arc_handle::ArcDevice;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};

pub struct MeshShaderPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    task_module: Option<vk::ShaderModule>,
    mesh_module: vk::ShaderModule,
    fragment_module: vk::ShaderModule,
    device: ArcDevice,
}

impl MeshShaderPipeline {
    pub fn new(
        device: ArcDevice,
        task_spirv: Option<&[u32]>,
        mesh_spirv: &[u32],
        fragment_spirv: &[u32],
        set_layouts: &[vk::DescriptorSetLayout],
        push_constant_size: u32,
        render_format: vk::Format,
        depth_format: vk::Format,
    ) -> ReactorResult<Self> {
        let task_module = if let Some(spirv) = task_spirv {
            let info = vk::ShaderModuleCreateInfo::default().code(spirv);
            Some(unsafe {
                device.create_shader_module(&info, None).map_err(|e| {
                    ReactorError::with_source(ErrorCode::VulkanShaderCompilation, "Failed to create task shader module", e)
                })?
            })
        } else {
            None
        };

        let mesh_info = vk::ShaderModuleCreateInfo::default().code(mesh_spirv);
        let mesh_module = unsafe {
            device.create_shader_module(&mesh_info, None).map_err(|e| {
                ReactorError::with_source(ErrorCode::VulkanShaderCompilation, "Failed to create mesh shader module", e)
            })?
        };

        let frag_info = vk::ShaderModuleCreateInfo::default().code(fragment_spirv);
        let fragment_module = unsafe {
            device.create_shader_module(&frag_info, None).map_err(|e| {
                ReactorError::with_source(ErrorCode::VulkanShaderCompilation, "Failed to create fragment shader module", e)
            })?
        };

        let push_constant_ranges = if push_constant_size > 0 {
            vec![vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::TASK_EXT
                    | vk::ShaderStageFlags::MESH_EXT
                    | vk::ShaderStageFlags::FRAGMENT,
                offset: 0,
                size: push_constant_size,
            }]
        } else {
            vec![]
        };

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let layout = unsafe {
            device.create_pipeline_layout(&layout_info, None).map_err(|e| {
                ReactorError::with_source(ErrorCode::VulkanPipelineCreation, "Failed to create mesh shader pipeline layout", e)
            })?
        };

        let mut stages = Vec::new();

        if let Some(task_mod) = task_module {
            stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::TASK_EXT)
                    .module(task_mod)
                    .name(CStr::from_bytes_with_nul(b"main\0").unwrap()),
            );
        }

        stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::MESH_EXT)
                .module(mesh_module)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap()),
        );

        stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap()),
        );

        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0);

        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let depth_stencil = if depth_format != vk::Format::UNDEFINED {
            vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
        } else {
            vk::PipelineDepthStencilStateCreateInfo::default()
        };

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)];
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachments);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(layout);

        let pipelines = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, e)| {
                    ReactorError::with_source(ErrorCode::VulkanPipelineCreation, "Failed to create mesh shader pipeline", e)
                })?
        };

        Ok(Self {
            pipeline: pipelines[0],
            layout,
            task_module,
            mesh_module,
            fragment_module,
            device,
        })
    }
}

impl MeshShaderPipeline {
    pub fn draw_mesh_tasks(
        &self,
        mesh_ext: &ash::ext::mesh_shader::Device,
        cmd: vk::CommandBuffer,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unsafe {
            self.device
                .cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            mesh_ext.cmd_draw_mesh_tasks(cmd, group_count_x, group_count_y, group_count_z);
        }
    }

    pub fn draw_mesh_tasks_indirect(
        &self,
        mesh_ext: &ash::ext::mesh_shader::Device,
        cmd: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            self.device
                .cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            mesh_ext.cmd_draw_mesh_tasks_indirect(cmd, buffer, offset, draw_count, stride);
        }
    }

    pub fn draw_mesh_tasks_indirect_count(
        &self,
        mesh_ext: &ash::ext::mesh_shader::Device,
        cmd: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        count_buffer: vk::Buffer,
        count_buffer_offset: vk::DeviceSize,
        max_draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            self.device
                .cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            mesh_ext.cmd_draw_mesh_tasks_indirect_count(
                cmd, buffer, offset, count_buffer, count_buffer_offset, max_draw_count, stride,
            );
        }
    }

    #[inline]
    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    #[inline]
    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Drop for MeshShaderPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            if let Some(task_mod) = self.task_module {
                self.device.destroy_shader_module(task_mod, None);
            }
            self.device.destroy_shader_module(self.mesh_module, None);
            self.device.destroy_shader_module(self.fragment_module, None);
        }
    }
}
