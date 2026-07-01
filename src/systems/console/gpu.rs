pub fn gpu_name_short(context: &crate::core::context::VulkanContext) -> String {
    let instance = context.instance.get();
    let properties = unsafe { instance.get_physical_device_properties(context.physical_device) };

    let name_bytes: Vec<u8> = properties
        .device_name
        .iter()
        .map(|&c| c as u8)
        .take_while(|&b| b != 0)
        .collect();

    String::from_utf8_lossy(&name_bytes).into_owned()
}
