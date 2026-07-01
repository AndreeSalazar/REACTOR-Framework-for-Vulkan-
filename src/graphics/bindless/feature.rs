use std::ffi::CStr;

use ash::vk;

use crate::core::error::ReactorResult;

pub fn check_bindless_support(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> ReactorResult<bool> {
    let props = unsafe { instance.enumerate_device_extension_properties(physical_device)? };
    let ext_name = CStr::from_bytes_with_nul(b"VK_EXT_descriptor_indexing\0").unwrap();
    Ok(props.iter().any(|p| {
        let name = unsafe { CStr::from_ptr(p.extension_name.as_ptr()) };
        name == ext_name
    }))
}

pub fn bindless_feature_chain() -> vk::PhysicalDeviceDescriptorIndexingFeatures<'static> {
    vk::PhysicalDeviceDescriptorIndexingFeatures::default()
        .shader_sampled_image_array_non_uniform_indexing(true)
        .shader_storage_buffer_array_non_uniform_indexing(true)
        .shader_storage_image_array_non_uniform_indexing(true)
        .descriptor_binding_variable_descriptor_count(true)
        .runtime_descriptor_array(true)
        .descriptor_binding_partially_bound(true)
        .descriptor_binding_update_unused_while_pending(true)
}
