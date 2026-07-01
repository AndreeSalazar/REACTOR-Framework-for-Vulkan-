use std::ffi::CStr;
use ash::vk;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};

pub fn check_mesh_shader_support(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> ReactorResult<bool> {
    let props = unsafe { instance.enumerate_device_extension_properties(physical_device)? };
    let ext_name = CStr::from_bytes_with_nul(b"VK_EXT_mesh_shader\0").unwrap();
    Ok(props.iter().any(|p| {
        let name = unsafe { CStr::from_ptr(p.extension_name.as_ptr()) };
        name == ext_name
    }))
}

pub fn mesh_shader_feature_chain() -> vk::PhysicalDeviceMeshShaderFeaturesEXT<'static> {
    vk::PhysicalDeviceMeshShaderFeaturesEXT::default()
        .task_shader(true)
        .mesh_shader(true)
}

#[derive(Debug, Clone, Copy)]
pub struct MeshShaderProperties {
    pub max_task_work_group_total_count: u32,
    pub max_task_work_group_count: [u32; 3],
    pub max_task_work_group_invocations: u32,
    pub max_task_work_group_size: [u32; 3],
    pub max_task_payload_size: u32,
    pub max_mesh_work_group_total_count: u32,
    pub max_mesh_work_group_count: [u32; 3],
    pub max_mesh_work_group_invocations: u32,
    pub max_mesh_work_group_size: [u32; 3],
    pub max_mesh_output_vertices: u32,
    pub max_mesh_output_primitives: u32,
    pub max_mesh_multiview_view_count: u32,
}

pub fn query_mesh_shader_properties(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> MeshShaderProperties {
    let mut props = vk::PhysicalDeviceMeshShaderPropertiesEXT::default();
    let mut props2 = vk::PhysicalDeviceProperties2::default().push_next(&mut props);
    unsafe {
        instance.get_physical_device_properties2(physical_device, &mut props2);
    }

    MeshShaderProperties {
        max_task_work_group_total_count: props.max_task_work_group_total_count,
        max_task_work_group_count: [
            props.max_task_work_group_count[0],
            props.max_task_work_group_count[1],
            props.max_task_work_group_count[2],
        ],
        max_task_work_group_invocations: props.max_task_work_group_invocations,
        max_task_work_group_size: [
            props.max_task_work_group_size[0],
            props.max_task_work_group_size[1],
            props.max_task_work_group_size[2],
        ],
        max_task_payload_size: props.max_task_payload_size,
        max_mesh_work_group_total_count: props.max_mesh_work_group_total_count,
        max_mesh_work_group_count: [
            props.max_mesh_work_group_count[0],
            props.max_mesh_work_group_count[1],
            props.max_mesh_work_group_count[2],
        ],
        max_mesh_work_group_invocations: props.max_mesh_work_group_invocations,
        max_mesh_work_group_size: [
            props.max_mesh_work_group_size[0],
            props.max_mesh_work_group_size[1],
            props.max_mesh_work_group_size[2],
        ],
        max_mesh_output_vertices: props.max_mesh_output_vertices,
        max_mesh_output_primitives: props.max_mesh_output_primitives,
        max_mesh_multiview_view_count: props.max_mesh_multiview_view_count,
    }
}
