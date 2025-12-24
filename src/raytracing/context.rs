use ash::vk;
use crate::core::context::VulkanContext;
use std::error::Error;

pub struct RayTracingContext {
    pub pipeline_fn: ash::khr::ray_tracing_pipeline::Device,
    pub accel_fn: ash::khr::acceleration_structure::Device,
    pub shader_group_handle_size: u32,
    pub shader_group_base_alignment: u32,
    pub shader_group_handle_alignment: u32,
    pub max_recursion_depth: u32,
    pub max_ray_dispatch_invocation_count: u32,
}

impl RayTracingContext {
    pub fn new(ctx: &VulkanContext) -> Result<Self, Box<dyn Error>> {
        let pipeline_fn = ash::khr::ray_tracing_pipeline::Device::new(&ctx.instance, &ctx.device);
        let accel_fn = ash::khr::acceleration_structure::Device::new(&ctx.instance, &ctx.device);

        // Get properties
        let mut pipeline_properties = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
        let mut properties = vk::PhysicalDeviceProperties2::default()
            .push_next(&mut pipeline_properties);
            
        unsafe {
            ctx.instance.get_physical_device_properties2(ctx.physical_device, &mut properties);
        }

        println!("Ray Tracing Properties:");
        println!("  Max Recursion Depth: {}", pipeline_properties.max_ray_recursion_depth);
        println!("  Shader Group Handle Size: {}", pipeline_properties.shader_group_handle_size);

        Ok(Self {
            pipeline_fn,
            accel_fn,
            shader_group_handle_size: pipeline_properties.shader_group_handle_size,
            shader_group_base_alignment: pipeline_properties.shader_group_base_alignment,
            shader_group_handle_alignment: pipeline_properties.shader_group_handle_alignment,
            max_recursion_depth: pipeline_properties.max_ray_recursion_depth,
            max_ray_dispatch_invocation_count: pipeline_properties.max_ray_dispatch_invocation_count,
        })
    }

    pub fn is_supported(ctx: &VulkanContext) -> bool {
        // Check if ray tracing extensions are available
        let extensions = unsafe {
            ctx.instance.enumerate_device_extension_properties(ctx.physical_device)
        };

        if let Ok(extensions) = extensions {
            let has_rt_pipeline = extensions.iter().any(|ext| {
                let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
                name.to_str().map(|s| s == "VK_KHR_ray_tracing_pipeline").unwrap_or(false)
            });

            let has_accel = extensions.iter().any(|ext| {
                let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
                name.to_str().map(|s| s == "VK_KHR_acceleration_structure").unwrap_or(false)
            });

            has_rt_pipeline && has_accel
        } else {
            false
        }
    }
}
