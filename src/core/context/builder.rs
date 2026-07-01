use crate::core::arc_handle::{ArcDevice, ArcInstance, ArcSurface};
use crate::core::debug_utils::DebugNamer;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::memory_budget;
use crate::core::vrs::{self, VrsCapabilities, VrsContext};
use crate::utils::gpu_detector::GPUDetector;
use ash::vk;
use ash::Entry;
use raw_window_handle::HasWindowHandle;
use std::ffi::{c_void, CStr};

use super::VulkanContext;

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

struct QueueFamilyInfo {
    graphics_index: u32,
    compute_index: Option<u32>,
    transfer_index: Option<u32>,
}

fn discover_queue_families(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    graphics_family: u32,
) -> QueueFamilyInfo {
    let families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let compute_index = families
        .iter()
        .enumerate()
        .find(|(i, props)| {
            let idx = *i as u32;
            idx != graphics_family
                && props.queue_flags.contains(vk::QueueFlags::COMPUTE)
                && !props.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        })
        .map(|(i, _)| i as u32);

    let transfer_index = families
        .iter()
        .enumerate()
        .find(|(i, props)| {
            let idx = *i as u32;
            idx != graphics_family
                && Some(idx) != compute_index
                && props.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && !props.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && !props.queue_flags.contains(vk::QueueFlags::COMPUTE)
        })
        .map(|(i, _)| i as u32)
        .or_else(|| {
            families
                .iter()
                .enumerate()
                .find(|(i, props)| {
                    let idx = *i as u32;
                    idx != graphics_family
                        && Some(idx) != compute_index
                        && props.queue_flags.contains(vk::QueueFlags::TRANSFER)
                })
                .map(|(i, _)| i as u32)
        });

    QueueFamilyInfo {
        graphics_index: graphics_family,
        compute_index,
        transfer_index,
    }
}

fn device_extension_supported(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    ext_name: &CStr,
) -> bool {
    let exts = unsafe { instance.enumerate_device_extension_properties(physical_device) };
    exts.map(|exts| {
        exts.iter().any(|ext| {
            let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            name == ext_name
        })
    })
    .unwrap_or(false)
}

impl VulkanContext {
    pub fn new(window: &impl HasWindowHandle, enable_ray_tracing: bool) -> ReactorResult<Self> {
        let entry = unsafe {
            Entry::load().map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanInstanceCreation,
                    "Failed to load Vulkan entry — is the Vulkan runtime installed?",
                    e,
                )
            })?
        };

        let (instance, debug_utils, debug_messenger) =
            Self::create_instance(&entry).map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanInstanceCreation,
                    "Failed to create VkInstance",
                    e,
                )
            })?;

        let arc_instance = ArcInstance::new(entry, instance, debug_utils, debug_messenger);

        let (surface, surface_loader) = Self::create_surface(&arc_instance, window)?;
        let arc_surface = ArcSurface::new(surface, surface_loader);

        let gpu_info = GPUDetector::detect(
            arc_instance.get(),
            arc_surface.loader(),
            arc_surface.handle(),
        )
        .map_err(|e| {
            ReactorError::with_source(ErrorCode::VulkanDeviceCreation, "GPU detection failed", e)
        })?;
        let pdevice = gpu_info.device;
        let queue_family_index = gpu_info.queue_family_index;

        log::info!(
            "🎮 Selected GPU: {} (queue family {})",
            gpu_info.name,
            queue_family_index
        );

        let queue_info = discover_queue_families(arc_instance.get(), pdevice, queue_family_index);

        if queue_info.compute_index.is_some() {
            log::info!(
                "⚡ Async Compute queue family: {}",
                queue_info.compute_index.unwrap()
            );
        }
        if queue_info.transfer_index.is_some() {
            log::info!(
                "📦 Async Transfer queue family: {}",
                queue_info.transfer_index.unwrap()
            );
        }

        let memory_budget_ext_name = CStr::from_bytes_with_nul(b"VK_EXT_memory_budget\0").unwrap();
        let has_memory_budget =
            device_extension_supported(arc_instance.get(), pdevice, memory_budget_ext_name);

        let vrs_capabilities =
            vrs::query_capabilities(arc_instance.entry(), arc_instance.get(), pdevice);
        let enable_fragment_shading_rate = vrs_capabilities.is_pipeline_ready();

        if enable_fragment_shading_rate {
            log::info!(
                "Pixel Inteligente VRS ready ({} supported rates)",
                vrs_capabilities.rates.len()
            );
        } else {
            log::info!("Pixel Inteligente VRS unavailable; using native 1x1 shading");
        }

        let (device, graphics_queue, compute_queue, transfer_queue) = Self::create_device(
            &arc_instance,
            pdevice,
            &queue_info,
            enable_ray_tracing,
            has_memory_budget,
            enable_fragment_shading_rate,
        )?;
        let arc_device = ArcDevice::new(device);

        let fragment_shading_rate = enable_fragment_shading_rate.then(|| {
            VrsContext::new(
                arc_instance.get(),
                arc_device.get(),
                vrs_capabilities.clone(),
            )
        });

        let debug_namer = DebugNamer::new(
            arc_instance.debug_utils(),
            arc_instance.get(),
            arc_device.get(),
        );

        debug_namer.name_queue(graphics_queue, "Queue: Graphics (Main)");
        if let Some(cq) = compute_queue {
            debug_namer.name_queue(cq, "Queue: Async Compute");
        }
        if let Some(tq) = transfer_queue {
            debug_namer.name_queue(tq, "Queue: Async Transfer");
        }

        if debug_namer.is_active() {
            log::info!("🏷️  Debug resource labeling active (VK_EXT_debug_utils)");
        }
        if has_memory_budget {
            log::info!("📊 VRAM budget monitoring active (VK_EXT_memory_budget)");
        }

        Ok(Self {
            instance: arc_instance,
            device: arc_device,
            surface: arc_surface,
            physical_device: pdevice,
            graphics_queue,
            compute_queue,
            transfer_queue,
            queue_family_index,
            compute_queue_family_index: queue_info.compute_index,
            transfer_queue_family_index: queue_info.transfer_index,
            debug_namer,
            has_memory_budget,
            fragment_shading_rate,
            vrs_capabilities,
        })
    }

    fn create_instance(
        entry: &Entry,
    ) -> Result<
        (
            ash::Instance,
            Option<ash::ext::debug_utils::Instance>,
            Option<vk::DebugUtilsMessengerEXT>,
        ),
        vk::Result,
    > {
        let layer_names = [CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap()];
        let layers_ptr: Vec<*const i8> = layer_names.iter().map(|r| r.as_ptr()).collect();

        let extension_names = vec![
            ash::khr::surface::NAME.as_ptr(),
            ash::khr::win32_surface::NAME.as_ptr(),
            ash::ext::debug_utils::NAME.as_ptr(),
        ];

        let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers_ptr)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        #[cfg(debug_assertions)]
        let (debug_utils, debug_messenger) = {
            let debug_utils = ash::ext::debug_utils::Instance::new(entry, &instance);

            let messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));

            let messenger = unsafe {
                debug_utils
                    .create_debug_utils_messenger(&messenger_info, None)
                    .expect("Failed to create debug messenger")
            };

            log::info!("🔍 Vulkan validation layers enabled");
            (Some(debug_utils), Some(messenger))
        };

        #[cfg(not(debug_assertions))]
        let (debug_utils, debug_messenger) = {
            let debug_utils = ash::ext::debug_utils::Instance::new(entry, &instance);
            (Some(debug_utils), None)
        };

        Ok((instance, debug_utils, debug_messenger))
    }

    fn create_surface(
        instance: &ArcInstance,
        window: &impl HasWindowHandle,
    ) -> ReactorResult<(vk::SurfaceKHR, ash::khr::surface::Instance)> {
        let surface_loader = ash::khr::surface::Instance::new(instance.entry(), instance.get());

        let surface = unsafe {
            use raw_window_handle::RawWindowHandle;
            match window
                .window_handle()
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSurfaceCreation,
                        "Failed to get window handle",
                        e,
                    )
                })?
                .as_raw()
            {
                RawWindowHandle::Win32(handle) => {
                    let create_info = vk::Win32SurfaceCreateInfoKHR::default()
                        .hinstance(handle.hinstance.unwrap().get())
                        .hwnd(handle.hwnd.get());
                    let win32_loader =
                        ash::khr::win32_surface::Instance::new(instance.entry(), instance.get());
                    win32_loader
                        .create_win32_surface(&create_info, None)
                        .map_err(|e| {
                            ReactorError::with_source(
                                ErrorCode::VulkanSurfaceCreation,
                                "Failed to create Win32 surface",
                                e,
                            )
                        })?
                }
                _ => {
                    return Err(ReactorError::new(
                        ErrorCode::VulkanSurfaceCreation,
                        "Unsupported window platform (only Win32 supported)",
                    ));
                }
            }
        };

        Ok((surface, surface_loader))
    }

    fn create_device(
        instance: &ArcInstance,
        physical_device: vk::PhysicalDevice,
        queue_info: &QueueFamilyInfo,
        enable_ray_tracing: bool,
        has_memory_budget: bool,
        enable_fragment_shading_rate: bool,
    ) -> ReactorResult<(ash::Device, vk::Queue, Option<vk::Queue>, Option<vk::Queue>)> {
        let mut device_extension_names: Vec<*const i8> = vec![
            ash::khr::swapchain::NAME.as_ptr(),
            ash::khr::dynamic_rendering::NAME.as_ptr(),
        ];

        if has_memory_budget {
            device_extension_names.push(
                CStr::from_bytes_with_nul(b"VK_EXT_memory_budget\0")
                    .unwrap()
                    .as_ptr(),
            );
        }

        if enable_ray_tracing {
            device_extension_names.push(ash::khr::ray_tracing_pipeline::NAME.as_ptr());
            device_extension_names.push(ash::khr::acceleration_structure::NAME.as_ptr());
            device_extension_names.push(ash::khr::deferred_host_operations::NAME.as_ptr());
            device_extension_names.push(
                CStr::from_bytes_with_nul(b"VK_KHR_spirv_1_4\0")
                    .unwrap()
                    .as_ptr(),
            );
            device_extension_names.push(
                CStr::from_bytes_with_nul(b"VK_KHR_shader_float_controls\0")
                    .unwrap()
                    .as_ptr(),
            );
            device_extension_names.push(
                CStr::from_bytes_with_nul(b"VK_KHR_buffer_device_address\0")
                    .unwrap()
                    .as_ptr(),
            );
        }

        if enable_fragment_shading_rate {
            device_extension_names.push(ash::khr::fragment_shading_rate::NAME.as_ptr());
        }

        let queue_priorities = [1.0f32];
        let mut queue_create_infos = vec![vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_info.graphics_index)
            .queue_priorities(&queue_priorities)];

        if let Some(compute_idx) = queue_info.compute_index {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(compute_idx)
                    .queue_priorities(&queue_priorities),
            );
        }

        if let Some(transfer_idx) = queue_info.transfer_index {
            if Some(transfer_idx) != queue_info.compute_index {
                queue_create_infos.push(
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(transfer_idx)
                        .queue_priorities(&queue_priorities),
                );
            }
        }

        let mut dynamic_rendering_features =
            vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);
        let mut fragment_shading_rate_features =
            vk::PhysicalDeviceFragmentShadingRateFeaturesKHR::default()
                .pipeline_fragment_shading_rate(true);

        let mut device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extension_names)
            .push_next(&mut dynamic_rendering_features);

        let mut buffer_device_address_features =
            vk::PhysicalDeviceBufferDeviceAddressFeatures::default().buffer_device_address(true);
        let mut ray_tracing_pipeline_features =
            vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default().ray_tracing_pipeline(true);
        let mut acceleration_structure_features =
            vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default()
                .acceleration_structure(true);

        if enable_ray_tracing {
            device_create_info = device_create_info
                .push_next(&mut buffer_device_address_features)
                .push_next(&mut ray_tracing_pipeline_features)
                .push_next(&mut acceleration_structure_features);
        }

        if enable_fragment_shading_rate {
            device_create_info = device_create_info.push_next(&mut fragment_shading_rate_features);
        }

        let device = unsafe {
            instance
                .get()
                .create_device(physical_device, &device_create_info, None)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanDeviceCreation,
                        "Failed to create logical device",
                        e,
                    )
                })?
        };

        let graphics_queue = unsafe { device.get_device_queue(queue_info.graphics_index, 0) };

        let compute_queue = queue_info
            .compute_index
            .map(|idx| unsafe { device.get_device_queue(idx, 0) });

        let transfer_queue = queue_info
            .transfer_index
            .map(|idx| unsafe { device.get_device_queue(idx, 0) });

        Ok((device, graphics_queue, compute_queue, transfer_queue))
    }
}
