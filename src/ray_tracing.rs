use ash::vk;
use crate::vulkan_context::VulkanContext;
use std::error::Error;

pub struct RayTracingContext {
    pub ray_tracing_pipeline_fn: ash::khr::ray_tracing_pipeline::Device,
    pub acceleration_structure_fn: ash::khr::acceleration_structure::Device,
    pub shader_group_handle_size: u32,
    pub shader_group_base_alignment: u32,
    pub max_recursion_depth: u32,
}

impl RayTracingContext {
    pub fn new(ctx: &VulkanContext) -> Result<Self, Box<dyn Error>> {
        let ray_tracing_pipeline_fn = ash::khr::ray_tracing_pipeline::Device::new(&ctx.instance, &ctx.device);
        let acceleration_structure_fn = ash::khr::acceleration_structure::Device::new(&ctx.instance, &ctx.device);

        // Get properties
        let mut pipeline_properties = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
        let mut properties = vk::PhysicalDeviceProperties2::default()
            .push_next(&mut pipeline_properties);
            
        unsafe {
            ctx.instance.get_physical_device_properties2(ctx.physical_device, &mut properties);
        }

        Ok(Self {
            ray_tracing_pipeline_fn,
            acceleration_structure_fn,
            shader_group_handle_size: pipeline_properties.shader_group_handle_size,
            shader_group_base_alignment: pipeline_properties.shader_group_base_alignment,
            max_recursion_depth: pipeline_properties.max_ray_recursion_depth,
        })
    }
}

// Placeholder for Acceleration Structure
pub struct AccelerationStructure {
    pub handle: vk::AccelerationStructureKHR,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

// Placeholder for Ray Tracing Pipeline
pub struct RayTracingPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub sbt_buffer: vk::Buffer,
}
