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
//
// Vulkan Coverage:
// - Debug Utils (VK_EXT_debug_utils) for resource labeling in RenderDoc/NSight.
// - Memory Budget (VK_EXT_memory_budget) for VRAM monitoring.
// - Async Queues: dedicated compute and transfer queues when available.
// =============================================================================

use ash::vk;
use ash::Entry;
use raw_window_handle::HasWindowHandle;
use std::ffi::{c_void, CStr};

use crate::core::arc_handle::{ArcDevice, ArcInstance, ArcSurface};
use crate::core::debug_utils::DebugNamer;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::memory_budget::{self, GpuMemoryBudget};
use crate::core::vrs::{self, VrsCapabilities, VrsContext};
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
    /// Arc-wrapped VkDevice (RAII, destroyed FIRST - depends on instance).
    pub device: ArcDevice,

    /// Arc-wrapped VkSurfaceKHR (RAII, destroyed SECOND - depends on instance).
    pub surface: ArcSurface,

    /// Arc-wrapped VkInstance (RAII, destroyed LAST).
    pub instance: ArcInstance,

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

    /// Debug namer for labeling Vulkan resources in profiling tools.
    pub debug_namer: DebugNamer,

    /// Whether VK_EXT_memory_budget is available on this device.
    pub has_memory_budget: bool,

    /// Vulkan Variable Rate Shading context (`VK_KHR_fragment_shading_rate`).
    pub fragment_shading_rate: Option<VrsContext>,

    /// Queried VRS support, retained even when the device cannot enable it.
    pub vrs_capabilities: VrsCapabilities,
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
// Queue family discovery helpers
// =============================================================================

/// Info about discovered queue families on the physical device.
struct QueueFamilyInfo {
    /// Graphics + present queue family index (mandatory).
    graphics_index: u32,
    /// Dedicated compute queue family index (None if not found).
    compute_index: Option<u32>,
    /// Dedicated transfer queue family index (None if not found).
    transfer_index: Option<u32>,
}

/// Discover queue families on the physical device.
/// Prefers dedicated queues (not sharing flags with the graphics family).
fn discover_queue_families(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    graphics_family: u32,
) -> QueueFamilyInfo {
    let families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    // Look for a dedicated compute queue (has COMPUTE but NOT GRAPHICS)
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

    // Look for a dedicated transfer queue (has TRANSFER but NOT GRAPHICS and NOT COMPUTE)
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
            // Fallback: any transfer-capable family that isn't the graphics family
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

// =============================================================================
// Check if a device extension is available
// =============================================================================

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
    /// 6. Discover async queue families (compute, transfer)
    /// 7. Create VkDevice with requested features + extensions
    /// 8. Retrieve queues
    /// 9. Initialize debug namer for resource labeling
    /// 10. Query memory budget capabilities
    ///
    /// # Errors
    ///
    /// Returns `ReactorError` with appropriate `ErrorCode` if any step fails.
    /// Never panics.
    pub fn new(window: &impl HasWindowHandle, enable_ray_tracing: bool) -> ReactorResult<Self> {
        // 1. Load Vulkan entry
        let entry = unsafe {
            Entry::load().map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanInstanceCreation,
                    "Failed to load Vulkan entry — is the Vulkan runtime installed?",
                    e,
                )
            })?
        };

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
            ReactorError::with_source(ErrorCode::VulkanDeviceCreation, "GPU detection failed", e)
        })?;
        let pdevice = gpu_info.device;
        let queue_family_index = gpu_info.queue_family_index;

        log::info!(
            "🎮 Selected GPU: {} (queue family {})",
            gpu_info.name,
            queue_family_index
        );

        // 5. Discover async queue families
        let queue_info =
            discover_queue_families(arc_instance.get(), pdevice, queue_family_index);

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

        // 6. Check for VK_EXT_memory_budget support
        let memory_budget_ext_name =
            CStr::from_bytes_with_nul(b"VK_EXT_memory_budget\0").unwrap();
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

        // 7. Create logical device with all queues + extensions
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
            VrsContext::new(arc_instance.get(), arc_device.get(), vrs_capabilities.clone())
        });

        // 8. Initialize debug namer
        let debug_namer = DebugNamer::new(
            arc_instance.debug_utils(),
            arc_instance.get(),
            arc_device.get(),
        );

        // Label the graphics queue
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
        let layer_names = [CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap()];
        let layers_ptr: Vec<*const i8> = layer_names.iter().map(|r| r.as_ptr()).collect();

        // Extensions — always include debug_utils for resource labeling
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
        let (debug_utils, debug_messenger) = {
            // In release: still create the Instance loader for debug naming,
            // but don't create a validation messenger.
            let debug_utils = ash::ext::debug_utils::Instance::new(entry, &instance);
            (Some(debug_utils), None)
        };

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

    // -------------------------------------------------------------------------
    // Internal: device creation (with async queues + extensions)
    // -------------------------------------------------------------------------
    fn create_device(
        instance: &ArcInstance,
        physical_device: vk::PhysicalDevice,
        queue_info: &QueueFamilyInfo,
        enable_ray_tracing: bool,
        has_memory_budget: bool,
        enable_fragment_shading_rate: bool,
    ) -> ReactorResult<(ash::Device, vk::Queue, Option<vk::Queue>, Option<vk::Queue>)> {
        // Device extensions
        let mut device_extension_names: Vec<*const i8> = vec![
            ash::khr::swapchain::NAME.as_ptr(),
            ash::khr::dynamic_rendering::NAME.as_ptr(),
        ];

        // VK_EXT_memory_budget (device extension)
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
            device_extension_names.push(CStr::from_bytes_with_nul(b"VK_KHR_spirv_1_4\0")
                .unwrap()
                .as_ptr());
            device_extension_names.push(CStr::from_bytes_with_nul(b"VK_KHR_shader_float_controls\0")
                .unwrap()
                .as_ptr());
            device_extension_names.push(CStr::from_bytes_with_nul(b"VK_KHR_buffer_device_address\0")
                .unwrap()
                .as_ptr());
        }

        if enable_fragment_shading_rate {
            device_extension_names.push(ash::khr::fragment_shading_rate::NAME.as_ptr());
        }

        // ── Build queue create infos for all discovered families ──
        let queue_priorities = [1.0f32];
        let mut queue_create_infos = vec![
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_info.graphics_index)
                .queue_priorities(&queue_priorities),
        ];

        // Add compute queue if on a different family
        if let Some(compute_idx) = queue_info.compute_index {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(compute_idx)
                    .queue_priorities(&queue_priorities),
            );
        }

        // Add transfer queue if on a different family
        if let Some(transfer_idx) = queue_info.transfer_index {
            // Avoid duplicate if transfer == compute (already added)
            if Some(transfer_idx) != queue_info.compute_index {
                queue_create_infos.push(
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(transfer_idx)
                        .queue_priorities(&queue_priorities),
                );
            }
        }

        // Features
        let mut dynamic_rendering_features =
            vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);
        let mut fragment_shading_rate_features =
            vk::PhysicalDeviceFragmentShadingRateFeaturesKHR::default()
                .pipeline_fragment_shading_rate(true);

        let mut device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extension_names)
            .push_next(&mut dynamic_rendering_features);

        // Ray tracing specific features (keep structures in scope during device creation)
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
            device_create_info =
                device_create_info.push_next(&mut fragment_shading_rate_features);
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

        // Retrieve queues
        let graphics_queue =
            unsafe { device.get_device_queue(queue_info.graphics_index, 0) };

        let compute_queue = queue_info
            .compute_index
            .map(|idx| unsafe { device.get_device_queue(idx, 0) });

        let transfer_queue = queue_info
            .transfer_index
            .map(|idx| unsafe { device.get_device_queue(idx, 0) });

        Ok((device, graphics_queue, compute_queue, transfer_queue))
    }

    // =========================================================================
    // Public API
    // =========================================================================

    /// Block until the device is idle (all submitted work completed).
    pub fn wait_idle(&self) -> ReactorResult<()> {
        unsafe {
            self.device.get().device_wait_idle().map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanSynchronization,
                    "device_wait_idle failed",
                    e,
                )
            })
        }
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

    // ─── Debug Utils API ────────────────────────────────────────────────

    /// Access the debug namer for labeling Vulkan resources.
    ///
    /// Usage in engine subsystems:
    /// ```ignore
    /// ctx.debug_namer().name_buffer(my_buffer, "Buffer: PlayerVertices");
    /// ctx.debug_namer().name_image(my_image, "Image: ZombieAlbedoTexture");
    /// ```
    #[inline]
    pub fn debug_namer(&self) -> &DebugNamer {
        &self.debug_namer
    }

    // ─── Memory Budget API ──────────────────────────────────────────────

    /// Query the current GPU memory budget (VRAM usage + available).
    ///
    /// If `VK_EXT_memory_budget` is available, returns real-time data from the
    /// driver. Otherwise, returns static heap sizes with usage = 0.
    ///
    /// # Example
    /// ```ignore
    /// let budget = ctx.get_vram_budget();
    /// println!("VRAM: {}/{} MB used", budget.total_vram_usage_mb(), budget.total_vram_budget_mb());
    /// if budget.is_vram_pressure_high() {
    ///     // Reduce texture quality or unload distant models
    /// }
    /// ```
    pub fn get_vram_budget(&self) -> GpuMemoryBudget {
        memory_budget::query_memory_budget(
            self.ash_instance(),
            self.physical_device,
            self.has_memory_budget,
        )
    }

    // ─── Pixel Inteligente / VRS API ────────────────────────────────────

    /// Returns `true` when `VK_KHR_fragment_shading_rate` is enabled in
    /// pipeline mode and REACTOR can issue VRS commands.
    #[inline]
    pub fn supports_fragment_shading_rate(&self) -> bool {
        self.fragment_shading_rate.is_some()
    }

    /// Returns the negotiated VRS capabilities for diagnostics and tooling.
    #[inline]
    pub fn vrs_capabilities(&self) -> &VrsCapabilities {
        &self.vrs_capabilities
    }

    // ─── Async Queue API ────────────────────────────────────────────────

    /// Returns `true` if a dedicated compute queue is available.
    #[inline]
    pub fn has_async_compute(&self) -> bool {
        self.compute_queue.is_some()
    }

    /// Returns `true` if a dedicated transfer queue is available.
    #[inline]
    pub fn has_async_transfer(&self) -> bool {
        self.transfer_queue.is_some()
    }

    /// Submit a command buffer to the compute queue.
    ///
    /// Falls back to the graphics queue if no dedicated compute queue exists.
    pub fn submit_compute(
        &self,
        submit_info: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> ReactorResult<()> {
        let queue = self.compute_queue.unwrap_or(self.graphics_queue);
        unsafe {
            self.device
                .get()
                .queue_submit(queue, submit_info, fence)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSynchronization,
                        "Compute queue submit failed",
                        e,
                    )
                })
        }
    }

    /// Submit a command buffer to the transfer queue.
    ///
    /// Falls back to the graphics queue if no dedicated transfer queue exists.
    pub fn submit_transfer(
        &self,
        submit_info: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> ReactorResult<()> {
        let queue = self.transfer_queue.unwrap_or(self.graphics_queue);
        unsafe {
            self.device
                .get()
                .queue_submit(queue, submit_info, fence)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSynchronization,
                        "Transfer queue submit failed",
                        e,
                    )
                })
        }
    }

    /// Get the queue family index for compute operations.
    /// Falls back to the graphics queue family if no dedicated compute queue.
    #[inline]
    pub fn compute_family(&self) -> u32 {
        self.compute_queue_family_index
            .unwrap_or(self.queue_family_index)
    }

    /// Get the queue family index for transfer operations.
    /// Falls back to the graphics queue family if no dedicated transfer queue.
    #[inline]
    pub fn transfer_family(&self) -> u32 {
        self.transfer_queue_family_index
            .unwrap_or(self.queue_family_index)
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
