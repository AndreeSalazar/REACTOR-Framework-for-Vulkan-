// =============================================================================
// REACTOR VulkanContext — The engine's central Vulkan state
// =============================================================================
// Thread-safe, Arc-based ownership model.
// Cloning is cheap (refcount++) and safe (RAII ensures correct destruction).
//
// Design (UE5-inspired):
// - All Vulkan handles are wrapped in Arc<Handle> for safe sharing.
// - Drop order is enforced: device → surface → instance (reverse creation).
// - All APIs return ReactorResult<T>, never panic or Box<dyn std::error::Error + Send + Sync>.
// - Validation layers are enabled by default in debug builds.
// =============================================================================

use ash::vk;
use ash::Entry;
use raw_window_handle::HasWindowHandle;
use std::ffi::{c_void, CStr};

use crate::core::arc_handle::{ArcDevice, ArcInstance, ArcSurface};
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::utils::gpu_detector::GPUDetector;

// =============================================================================
// VulkanContext
// =============================================================================

/// Central Vulkan state for the REACTOR engine.
///
/// `VulkanContext` owns (via `Arc`) the Vulkan instance, device, and surface.
/// It is cheap to clone (`Arc::clone`) and safe to share across threads.
///
/// # Lifetime & Ownership
///
/// The underlying Vulkan handles are destroyed in the correct order when the
/// last clone of this context is dropped:
///
/// 1. Device + queues destroyed first (they depend on the instance).
/// 2. Surface destroyed next (it depends on the instance).
/// 3. Debug messenger destroyed (depends on instance).
/// 4. Instance destroyed last.
///
/// # Example
///
/// ```ignore
/// let ctx = VulkanContext::new(&window)?;
/// let shared = ctx.clone(); // cheap, no deep copy
/// // Use ctx and shared across threads...
/// drop(ctx);     // instance/device still alive via `shared`
/// drop(shared);  // now Vulkan handles are destroyed
/// ```
#[derive(Clone)]
pub struct VulkanContext {
    /// Arc-wrapped VkInstance (RAII, destroyed last).
    pub instance: ArcInstance,

    /// Arc-wrapped VkDevice (RAII, destroyed before instance).
    pub device: ArcDevice,

    /// Arc-wrapped VkSurfaceKHR (RAII, destroyed before instance).
    pub surface: ArcSurface,

    /// Selected physical device (does not need RAII — owned by Instance).
    pub physical_device: vk::PhysicalDevice,

    /// Graphics queue (owned by Device).
    pub graphics_queue: vk::Queue,

    /// Optional dedicated compute queue.
    pub compute_queue: Option<vk::Queue>,

    /// Optional dedicated transfer queue.
    pub transfer_queue: Option<vk::Queue>,

    /// Graphics queue family index.
    pub queue_family_index: u32,

    /// Compute queue family index (if separate from graphics).
    pub compute_queue_family_index: Option<u32>,

    /// Transfer queue family index (if separate from graphics/compute).
    pub transfer_queue_family_index: Option<u32>,
}

// =============================================================================
// Vulkan debug callback
// =============================================================================

/// Vulkan debug callback — logs validation messages via the `log` crate.
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

// =============================================================================
// Construction
// =============================================================================

impl VulkanContext {
    /// Create a new Vulkan context for the given window.
    ///
    /// This performs the full Vulkan initialization sequence:
    /// 1. Load Vulkan entry point
    /// 2. Create VkInstance (with validation layers in debug)
    /// 3. Set up debug messenger (debug builds only)
    /// 4. Create VkSurfaceKHR for the window
    /// 5. Select best physical device (via GPUDetector)
    /// 6. Create VkDevice with requested features
    /// 7. Retrieve queues
    ///
    /// # Errors
    ///
    /// Returns `ReactorError` with appropriate `ErrorCode` if any step fails.
    /// Never panics.
    pub fn new(window: &impl HasWindowHandle) -> ReactorResult<Self> {
        // 1. Load Vulkan entry
        let entry = unsafe { Entry::load().map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanInstanceCreation,
                "Failed to load Vulkan entry — is the Vulkan runtime installed?",
                e,
            )
        })? };

        // 2. Instance creation (with validation layers in debug)
        let (instance, debug_utils, debug_messenger) =
            Self::create_instance(&entry).map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanInstanceCreation,
                    "Failed to create VkInstance",
                    e,
                )
            })?;

        let arc_instance = ArcInstance::new(entry, instance, debug_utils, debug_messenger);

        // 3. Create surface for the window
        let (surface, surface_loader) = Self::create_surface(&arc_instance, window)?;
        let arc_surface = ArcSurface::new(surface, surface_loader);

        // 4. Select best physical device
        let gpu_info = GPUDetector::detect(
            arc_instance.get(),
            arc_surface.loader(),
            arc_surface.handle(),
        )
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanDeviceCreation,
                "GPU detection failed",
                e,
            )
        })?;
        let pdevice = gpu_info.device;
        let queue_family_index = gpu_info.queue_family_index;

        log::info!(
            "🎮 Selected GPU: {} (queue family {})",
            gpu_info.name,
            queue_family_index
        );

        // 5. Create logical device
        let (device, graphics_queue) =
            Self::create_device(&arc_instance, pdevice, queue_family_index)?;
        let arc_device = ArcDevice::new(device);

        Ok(Self {
            instance: arc_instance,
            device: arc_device,
            surface: arc_surface,
            physical_device: pdevice,
            graphics_queue,
            compute_queue: None,
            transfer_queue: None,
            queue_family_index,
            compute_queue_family_index: None,
            transfer_queue_family_index: None,
        })
    }

    // -------------------------------------------------------------------------
    // Internal: instance creation
    // -------------------------------------------------------------------------
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
        // Validation layers (debug only)
        let layer_names =
            [CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap()];
        let layers_ptr: Vec<*const i8> = layer_names.iter().map(|r| r.as_ptr()).collect();

        // Extensions
        let mut extension_names = vec![
            ash::khr::surface::NAME.as_ptr(),
            ash::khr::win32_surface::NAME.as_ptr(),
        ];

        #[cfg(debug_assertions)]
        {
            extension_names.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers_ptr)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        // Debug messenger (debug builds only)
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
        let (debug_utils, debug_messenger): (
            Option<ash::ext::debug_utils::Instance>,
            Option<vk::DebugUtilsMessengerEXT>,
        ) = (None, None);

        Ok((instance, debug_utils, debug_messenger))
    }

    // -------------------------------------------------------------------------
    // Internal: surface creation
    // -------------------------------------------------------------------------
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
                        .hinstance(handle.hinstance.unwrap().get() as isize)
                        .hwnd(handle.hwnd.get() as isize);
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

    // -------------------------------------------------------------------------
    // Internal: device creation
    // -------------------------------------------------------------------------
    fn create_device(
        instance: &ArcInstance,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> ReactorResult<(ash::Device, vk::Queue)> {
        // Device extensions
        let device_extension_names = [
            ash::khr::swapchain::NAME.as_ptr(),
            ash::khr::dynamic_rendering::NAME.as_ptr(),
            ash::khr::ray_tracing_pipeline::NAME.as_ptr(),
            ash::khr::acceleration_structure::NAME.as_ptr(),
            ash::khr::deferred_host_operations::NAME.as_ptr(),
            CStr::from_bytes_with_nul(b"VK_KHR_spirv_1_4\0")
                .unwrap()
                .as_ptr(),
            CStr::from_bytes_with_nul(b"VK_KHR_shader_float_controls\0")
                .unwrap()
                .as_ptr(),
            CStr::from_bytes_with_nul(b"VK_KHR_buffer_device_address\0")
                .unwrap()
                .as_ptr(),
        ];

        // Queue
        let queue_priorities = [1.0f32];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities);

        // Features
        let mut buffer_device_address_features =
            vk::PhysicalDeviceBufferDeviceAddressFeatures::default().buffer_device_address(true);
        let mut ray_tracing_pipeline_features =
            vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default().ray_tracing_pipeline(true);
        let mut acceleration_structure_features =
            vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default()
                .acceleration_structure(true);
        let mut dynamic_rendering_features =
            vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names)
            .push_next(&mut buffer_device_address_features)
            .push_next(&mut ray_tracing_pipeline_features)
            .push_next(&mut acceleration_structure_features)
            .push_next(&mut dynamic_rendering_features);

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

        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok((device, graphics_queue))
    }

    // =========================================================================
    // Public API
    // =========================================================================

    /// Block until the device is idle (all submitted work completed).
    pub fn wait_idle(&self) -> ReactorResult<()> {
        unsafe { self.device.get().device_wait_idle().map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanSynchronization,
                "device_wait_idle failed",
                e,
            )
        }) }
    }

    /// Borrow the underlying `ash::Instance`.
    #[inline]
    pub fn ash_instance(&self) -> &ash::Instance {
        self.instance.get()
    }

    /// Borrow the underlying `ash::Device`.
    #[inline]
    pub fn ash_device(&self) -> &ash::Device {
        self.device.get()
    }

    /// Raw surface handle.
    #[inline]
    pub fn surface_khr(&self) -> vk::SurfaceKHR {
        self.surface.handle()
    }

    /// Surface loader for queries (capabilities, formats, present modes).
    /// Backward-compatible alias — delegates to `self.surface.loader()`.
    #[inline]
    pub fn surface_loader(&self) -> &ash::khr::surface::Instance {
        self.surface.loader()
    }

    /// Raw surface handle (backward-compatible alias for `surface_khr()`).
    #[inline]
    pub fn surface_handle(&self) -> vk::SurfaceKHR {
        self.surface.handle()
    }

    /// Diagnostic: how many Arc clones exist for each handle.
    pub fn ref_counts(&self) -> (usize, usize, usize) {
        (
            self.instance.ref_count(),
            self.device.ref_count(),
            self.surface.ref_count(),
        )
    }
}

// =============================================================================
// NOTE: No Drop impl here.
// =============================================================================
// Destruction is delegated to ArcInstance, ArcDevice, and ArcSurface.
// When the last clone of VulkanContext is dropped, the Arcs trigger
// their own Drop impls in the correct reverse-creation order:
//
//   1. ArcDevice drops → VkDevice destroyed
//   2. ArcSurface drops → VkSurfaceKHR destroyed
//   3. ArcInstance drops → Debug messenger + VkInstance destroyed
//
// This is the UE5-style "handle-based" ownership model: resources
// hold Arc clones of what they depend on, and everything cleans
// itself up naturally.
