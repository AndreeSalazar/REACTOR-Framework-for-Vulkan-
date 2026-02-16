use ash::{Entry, Instance, Device};
use ash::vk;
use std::ffi::{CStr, c_void};
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
    debug_utils: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

/// Vulkan debug callback - logs validation messages
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    let type_str = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "GENERAL",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "VALIDATION",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "PERFORMANCE",
        _ => "UNKNOWN",
    };

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("[Vulkan {}] {}", type_str, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("[Vulkan {}] {}", type_str, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("[Vulkan {}] {}", type_str, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::trace!("[Vulkan {}] {}", type_str, message);
        }
        _ => {
            log::debug!("[Vulkan {}] {}", type_str, message);
        }
    }

    vk::FALSE
}

impl VulkanContext {
    pub fn new(window: &impl HasWindowHandle) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { Entry::load()? };
        
        // Layers
        let layer_names = [CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")?];
        let layers_ptr: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();

        // Extensions - add debug utils in debug builds
        let mut extension_names = vec![
            ash::khr::surface::NAME.as_ptr(),
            ash::khr::win32_surface::NAME.as_ptr(),
        ];
        
        #[cfg(debug_assertions)]
        {
            extension_names.push(ash::ext::debug_utils::NAME.as_ptr());
        }
        
        let app_info = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers_ptr)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        
        // Setup debug messenger in debug builds
        #[cfg(debug_assertions)]
        let (debug_utils, debug_messenger) = {
            let debug_utils = ash::ext::debug_utils::Instance::new(&entry, &instance);
            
            let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                )
                .pfn_user_callback(Some(vulkan_debug_callback));
            
            let messenger = unsafe {
                debug_utils.create_debug_utils_messenger(&debug_create_info, None)
                    .expect("Failed to create debug messenger")
            };
            
            log::info!("🔍 Vulkan validation layers enabled");
            (Some(debug_utils), Some(messenger))
        };
        
        #[cfg(not(debug_assertions))]
        let (debug_utils, debug_messenger): (Option<ash::ext::debug_utils::Instance>, Option<vk::DebugUtilsMessengerEXT>) = (None, None);
        
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
            debug_utils,
            debug_messenger,
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
            
            // Destroy debug messenger before instance
            if let (Some(debug_utils), Some(messenger)) = (&self.debug_utils, self.debug_messenger) {
                debug_utils.destroy_debug_utils_messenger(messenger, None);
            }
            
            self.instance.destroy_instance(None);
        }
    }
}
