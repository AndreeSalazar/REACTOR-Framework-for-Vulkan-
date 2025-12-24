use ash::vk;
use ash::Instance;
use std::ffi::CStr;

#[derive(Debug, Clone)]
pub struct GPUInfo {
    pub device: vk::PhysicalDevice,
    pub name: String,
    pub device_type: vk::PhysicalDeviceType,
    pub score: u32,
    pub queue_family_index: u32,
}

pub struct GPUDetector;

impl GPUDetector {
    pub fn detect(
        instance: &Instance,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> Result<GPUInfo, Box<dyn std::error::Error>> {
        let pdevices = unsafe { instance.enumerate_physical_devices()? };
        
        let mut candidates = Vec::new();

        println!("Detecting GPUs...");

        for pdevice in pdevices {
            let props = unsafe { instance.get_physical_device_properties(pdevice) };
            let queue_families = unsafe { instance.get_physical_device_queue_family_properties(pdevice) };
            
            // Check queue support (Graphics + Present)
            let queue_index = queue_families.iter().enumerate().position(|(i, info)| {
                let supports_graphic = info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                let supports_surface = unsafe { 
                    surface_loader.get_physical_device_surface_support(pdevice, i as u32, surface).unwrap_or(false) 
                };
                supports_graphic && supports_surface
            });

            if let Some(index) = queue_index {
                let mut score = 0;
                
                // Prefer Discrete GPU
                match props.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => score += 10000,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => score += 1000,
                    vk::PhysicalDeviceType::VIRTUAL_GPU => score += 500,
                    vk::PhysicalDeviceType::CPU => score += 100,
                    _ => {},
                }

                // Add score for memory
                let memory_props = unsafe { instance.get_physical_device_memory_properties(pdevice) };
                let mut vram = 0;
                for i in 0..memory_props.memory_heap_count {
                    let heap = memory_props.memory_heaps[i as usize];
                    if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
                        vram += heap.size;
                    }
                }
                score += (vram / (1024 * 1024 * 1024)) as u32 * 100; // 100 points per GB

                let name = unsafe { 
                    CStr::from_ptr(props.device_name.as_ptr())
                        .to_string_lossy()
                        .into_owned() 
                };

                println!("Found GPU: {} (Score: {}, Type: {:?})", name, score, props.device_type);

                candidates.push(GPUInfo {
                    device: pdevice,
                    name,
                    device_type: props.device_type,
                    score,
                    queue_family_index: index as u32,
                });
            }
        }

        // Sort by score descending
        candidates.sort_by(|a, b| b.score.cmp(&a.score));

        if let Some(best) = candidates.first() {
            println!("Selected GPU: {}", best.name);
            Ok(best.clone())
        } else {
            Err("No suitable GPU found (Must support Graphics and Presentation)".into())
        }
    }
}
