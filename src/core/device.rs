use ash::vk;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub device_type: vk::PhysicalDeviceType,
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub vram_size: u64,
    pub supports_ray_tracing: bool,
    pub supports_mesh_shaders: bool,
    pub max_compute_work_group_count: [u32; 3],
    pub max_compute_work_group_size: [u32; 3],
}

impl DeviceInfo {
    pub fn from_physical_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Self {
        let props = unsafe { instance.get_physical_device_properties(physical_device) };
        let memory_props = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        
        let name = unsafe {
            std::ffi::CStr::from_ptr(props.device_name.as_ptr())
                .to_string_lossy()
                .into_owned()
        };

        let mut vram_size = 0u64;
        for i in 0..memory_props.memory_heap_count {
            let heap = memory_props.memory_heaps[i as usize];
            if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
                vram_size += heap.size;
            }
        }

        Self {
            name,
            device_type: props.device_type,
            api_version: props.api_version,
            driver_version: props.driver_version,
            vendor_id: props.vendor_id,
            vram_size,
            supports_ray_tracing: false, // Will be checked separately
            supports_mesh_shaders: false,
            max_compute_work_group_count: props.limits.max_compute_work_group_count,
            max_compute_work_group_size: props.limits.max_compute_work_group_size,
        }
    }

    pub fn vram_gb(&self) -> f32 {
        self.vram_size as f32 / (1024.0 * 1024.0 * 1024.0)
    }
}
