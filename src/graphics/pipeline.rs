use crate::resources::vertex::Vertex;
use crate::core::arc_handle::ArcDevice;
use crate::core::error::{ReactorResult, ReactorError, ErrorCode};
use ash::vk;
use std::ffi::CStr;

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    device: ArcDevice,
}

pub struct PipelineConfig {
    pub cull_mode: vk::CullModeFlags,
    pub front_face: vk::FrontFace,
    pub polygon_mode: vk::PolygonMode,
    pub depth_test: bool,
    pub depth_write: bool,
    pub blend_enable: bool,
    pub samples: vk::SampleCountFlags,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            polygon_mode: vk::PolygonMode::FILL,
            depth_test: true,
            depth_write: true,
            blend_enable: false,
            samples: vk::SampleCountFlags::TYPE_1,
        }
    }
}

impl Pipeline {
    pub fn new(
        device: &ArcDevice,
        render_pass: Option<vk::RenderPass>,
        vert_spv: &[u32],
        frag_spv: &[u32],
        width: u32,
        height: u32,
        color_format: vk::Format,
        depth_format: Option<vk::Format>,
    ) -> ReactorResult<Self> {
        Self::with_config(
            device,
            render_pass,
            vert_spv,
            frag_spv,
            width,
            height,
            &PipelineConfig::default(),
            &[],
            color_format,
            depth_format,
        )
    }

    pub fn with_config(
        device: &ArcDevice,
        render_pass: Option<vk::RenderPass>,
        vert_spv: &[u32],
        frag_spv: &[u32],
        width: u32,
        height: u32,
        config: &PipelineConfig,
        descriptor_layouts: &[vk::DescriptorSetLayout],
        color_format: vk::Format,
        depth_format: Option<vk::Format>,
    ) -> ReactorResult<Self> {
        let vert_shader_module = unsafe {
            let create_info = vk::ShaderModuleCreateInfo::default().code(vert_spv);
            device.create_shader_module(&create_info, None)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanShaderCompilation, "create_shader_module (vert) failed", e))?
        };

        let frag_shader_module = unsafe {
            let create_info = vk::ShaderModuleCreateInfo::default().code(frag_spv);
            device.create_shader_module(&create_info, None)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanShaderCompilation, "create_shader_module (frag) failed", e))?
        };

        let vert_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap());

        let frag_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap());

        let shader_stages = [vert_stage, frag_stage];

        let binding_descriptions = [Vertex::binding_description()];
        let attribute_descriptions = Vertex::attribute_descriptions();

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width, height },
        };

        let viewports = [viewport];
        let scissors = [scissor];
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(config.polygon_mode)
            .line_width(1.0)
            .cull_mode(config.cull_mode)
            .front_face(config.front_face)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(config.samples);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(config.depth_test)
            .depth_write_enable(config.depth_write)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(config.blend_enable)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let attachments = [color_blend_attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&attachments);

        let push_constant_range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: std::mem::size_of::<glam::Mat4>() as u32,
        };

        let push_constant_ranges = [push_constant_range];
        let layout_create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_layouts)
            .push_constant_ranges(&push_constant_ranges);
        let layout = unsafe {
            device.create_pipeline_layout(&layout_create_info, None)
                .map_err(|e| ReactorError::with_source(ErrorCode::VulkanPipelineCreation, "create_pipeline_layout failed", e))?
        };

        // Dynamic Rendering support
        let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(std::slice::from_ref(&color_format))
            .depth_attachment_format(depth_format.unwrap_or(vk::Format::UNDEFINED));

        let mut create_info_builder = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(layout);

        if let Some(rp) = render_pass {
            create_info_builder = create_info_builder.render_pass(rp).subpass(0);
        } else {
            create_info_builder = create_info_builder.push_next(&mut rendering_info);
        }

        let create_info = create_info_builder;

        let pipelines = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
                .map_err(|(_, e)| ReactorError::with_source(ErrorCode::VulkanPipelineCreation, "create_graphics_pipelines failed", e))?
        };

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        Ok(Self {
            pipeline: pipelines[0],
            layout,
            device: device.clone(),
        })
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
