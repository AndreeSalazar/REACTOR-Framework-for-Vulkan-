use ash::{Entry, Instance, Device};
use ash::vk;
use std::ffi::CStr;
use raw_window_handle::HasWindowHandle;
use std::error::Error;

use crate::utils::gpu_detector::GPUDetector;

pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub compute_queue: Option<vk::Queue>,
    pub transfer_queue: Option<vk::Queue>,
    pub surface_loader: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
    pub queue_family_index: u32,
    pub compute_queue_family_index: Option<u32>,
    pub transfer_queue_family_index: Option<u32>,
}

impl VulkanContext {
    pub fn new(window: &impl HasWindowHandle) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { Entry::load()? };
        
        // Layers
        let layer_names = [CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")?];
        let layers_ptr: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();

        // Extensions
        let extension_names = vec![
            ash::khr::surface::NAME.as_ptr(),
            ash::khr::win32_surface::NAME.as_ptr(),
        ];
        
        let app_info = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers_ptr)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        
        // Surface
        let surface = unsafe {
            use raw_window_handle::RawWindowHandle;
            match window.window_handle()?.as_raw() {
                RawWindowHandle::Win32(handle) => {
                    let win32_create_info = vk::Win32SurfaceCreateInfoKHR::default()
                        .hinstance(handle.hinstance.unwrap().get() as isize)
                        .hwnd(handle.hwnd.get() as isize);
                    let win32_surface_loader = ash::khr::win32_surface::Instance::new(&entry, &instance);
                    win32_surface_loader.create_win32_surface(&win32_create_info, None)?
                }
                _ => return Err("Unsupported window handle".into()),
            }
        };
        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        // Physical Device (Smart Selection)
        let gpu_info = GPUDetector::detect(&instance, &surface_loader, surface)?;
        let pdevice = gpu_info.device;
        let queue_family_index = gpu_info.queue_family_index;

        // Device Extensions
        let device_extension_names = [
            ash::khr::swapchain::NAME.as_ptr(),
            ash::khr::ray_tracing_pipeline::NAME.as_ptr(),
            ash::khr::acceleration_structure::NAME.as_ptr(),
            ash::khr::deferred_host_operations::NAME.as_ptr(),
            CStr::from_bytes_with_nul(b"VK_KHR_spirv_1_4\0")?.as_ptr(),
            CStr::from_bytes_with_nul(b"VK_KHR_shader_float_controls\0")?.as_ptr(),
            CStr::from_bytes_with_nul(b"VK_KHR_buffer_device_address\0")?.as_ptr(),
        ];

        // Queue Creation
        let queue_priorities = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities);

        // Enable Features
        let mut buffer_device_address_features = vk::PhysicalDeviceBufferDeviceAddressFeatures::default()
            .buffer_device_address(true);

        let mut ray_tracing_pipeline_features = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default()
            .ray_tracing_pipeline(true);
            
        let mut acceleration_structure_features = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default()
            .acceleration_structure(true);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names)
            .push_next(&mut buffer_device_address_features)
            .push_next(&mut ray_tracing_pipeline_features)
            .push_next(&mut acceleration_structure_features);

        let device = unsafe { instance.create_device(pdevice, &device_create_info, None)? };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok(Self {
            entry,
            instance,
            device,
            physical_device: pdevice,
            graphics_queue,
            compute_queue: None,
            transfer_queue: None,
            surface_loader,
            surface,
            queue_family_index,
            compute_queue_family_index: None,
            transfer_queue_family_index: None,
        })
    }

    pub fn wait_idle(&self) -> Result<(), vk::Result> {
        unsafe { self.device.device_wait_idle() }
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
