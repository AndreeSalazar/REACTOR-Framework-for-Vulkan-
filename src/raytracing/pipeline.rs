use ash::vk;
use crate::core::context::VulkanContext;
use crate::raytracing::context::RayTracingContext;
use std::error::Error;
use std::ffi::CStr;

pub struct RayTracingPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub shader_group_count: u32,
    device: ash::Device,
}

pub struct ShaderGroup {
    pub ty: vk::RayTracingShaderGroupTypeKHR,
    pub general_shader: u32,
    pub closest_hit_shader: u32,
    pub any_hit_shader: u32,
    pub intersection_shader: u32,
}

impl ShaderGroup {
    pub fn raygen(shader_index: u32) -> Self {
        Self {
            ty: vk::RayTracingShaderGroupTypeKHR::GENERAL,
            general_shader: shader_index,
            closest_hit_shader: vk::SHADER_UNUSED_KHR,
            any_hit_shader: vk::SHADER_UNUSED_KHR,
            intersection_shader: vk::SHADER_UNUSED_KHR,
        }
    }

    pub fn miss(shader_index: u32) -> Self {
        Self {
            ty: vk::RayTracingShaderGroupTypeKHR::GENERAL,
            general_shader: shader_index,
            closest_hit_shader: vk::SHADER_UNUSED_KHR,
            any_hit_shader: vk::SHADER_UNUSED_KHR,
            intersection_shader: vk::SHADER_UNUSED_KHR,
        }
    }

    pub fn hit(closest_hit: u32) -> Self {
        Self {
            ty: vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP,
            general_shader: vk::SHADER_UNUSED_KHR,
            closest_hit_shader: closest_hit,
            any_hit_shader: vk::SHADER_UNUSED_KHR,
            intersection_shader: vk::SHADER_UNUSED_KHR,
        }
    }
}

pub struct ShaderStage {
    pub code: Vec<u32>,
    pub stage: vk::ShaderStageFlags,
}

impl RayTracingPipeline {
    pub fn new(
        ctx: &VulkanContext,
        rt_ctx: &RayTracingContext,
        stages: &[ShaderStage],
        groups: &[ShaderGroup],
        descriptor_layouts: &[vk::DescriptorSetLayout],
        max_recursion: u32,
    ) -> Result<Self, Box<dyn Error>> {
        // Create shader modules
        let mut shader_modules = Vec::new();
        let mut stage_infos = Vec::new();

        for stage in stages {
            let create_info = vk::ShaderModuleCreateInfo::default().code(&stage.code);
            let module = unsafe { ctx.device.create_shader_module(&create_info, None)? };
            shader_modules.push(module);

            let stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(stage.stage)
                .module(module)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap());
            stage_infos.push(stage_info);
        }

        // Create shader groups
        let vk_groups: Vec<vk::RayTracingShaderGroupCreateInfoKHR> = groups
            .iter()
            .map(|g| {
                vk::RayTracingShaderGroupCreateInfoKHR::default()
                    .ty(g.ty)
                    .general_shader(g.general_shader)
                    .closest_hit_shader(g.closest_hit_shader)
                    .any_hit_shader(g.any_hit_shader)
                    .intersection_shader(g.intersection_shader)
            })
            .collect();

        // Pipeline layout
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_layouts);
        let layout = unsafe { ctx.device.create_pipeline_layout(&layout_info, None)? };

        // Create pipeline
        let pipeline_info = vk::RayTracingPipelineCreateInfoKHR::default()
            .stages(&stage_infos)
            .groups(&vk_groups)
            .max_pipeline_ray_recursion_depth(max_recursion.min(rt_ctx.max_recursion_depth))
            .layout(layout);

        let pipelines = unsafe {
            rt_ctx.pipeline_fn.create_ray_tracing_pipelines(
                vk::DeferredOperationKHR::null(),
                vk::PipelineCache::null(),
                &[pipeline_info],
                None,
            ).map_err(|(_, e)| e)?
        };

        // Cleanup shader modules
        for module in shader_modules {
            unsafe { ctx.device.destroy_shader_module(module, None); }
        }

        Ok(Self {
            pipeline: pipelines[0],
            layout,
            shader_group_count: groups.len() as u32,
            device: ctx.device.clone(),
        })
    }
}

impl Drop for RayTracingPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
