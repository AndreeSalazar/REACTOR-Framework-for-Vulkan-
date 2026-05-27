// Shader Binding Table for Ray Tracing
// NOTE: This is a placeholder module. Full implementation requires specific ash version compatibility.

use ash::vk;

/// Placeholder for Shader Binding Table
#[derive(Default)]
pub struct ShaderBindingTable {
    pub raygen_region: vk::StridedDeviceAddressRegionKHR,
    pub miss_region: vk::StridedDeviceAddressRegionKHR,
    pub hit_region: vk::StridedDeviceAddressRegionKHR,
    pub callable_region: vk::StridedDeviceAddressRegionKHR,
}
