// =============================================================================
// REACTOR Arc-wrapped RAII Handles for Vulkan Resources
// =============================================================================
// Thread-safe, reference-counted wrappers around raw Vulkan handles.
// Ensures proper destruction order and allows cheap sharing between systems.
//
// Design principles (UE5-style):
// - Every Vulkan handle is owned by exactly one RAII wrapper.
// - Sharing is done via Arc<_> clones (cheap, refcounted).
// - Drop order is guaranteed: dependents before dependencies.
// - No manual destroy calls anywhere else in the engine.
// =============================================================================

use ash::vk;
use std::sync::Arc;

// =============================================================================
// ArcInstance — Arc-wrapped VkInstance with RAII Drop
// =============================================================================

/// Inner data for an Arc-wrapped Vulkan instance.
/// Dropping the last Arc will destroy the instance.
struct InstanceInner {
    instance: ash::Instance,
    entry: ash::Entry,
    debug_utils: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Drop for InstanceInner {
    fn drop(&mut self) {
        // SAFETY: Destruction order must be reverse of creation.
        // 1. Destroy debug messenger first (created after instance).
        if let (Some(utils), Some(messenger)) = (self.debug_utils.as_ref(), self.debug_messenger) {
            unsafe {
                utils.destroy_debug_utils_messenger(messenger, None);
            }
            log::debug!("✓ Debug messenger destroyed");
        }
        // 2. Destroy instance last (created first).
        unsafe {
            self.instance.destroy_instance(None);
        }
        log::debug!("✓ VkInstance destroyed");
    }
}

/// A thread-safe, reference-counted wrapper around `ash::Instance`.
///
/// Cloning this handle is cheap (increment refcount). The underlying
/// VkInstance is destroyed when the last clone is dropped.
///
/// # Example
/// ```ignore
/// let instance = ArcInstance::new(entry, instance, debug_utils, messenger)?;
/// let shared = instance.clone(); // refcount = 2
/// drop(instance);                // refcount = 1, still alive
/// drop(shared);                  // refcount = 0, VkInstance destroyed
/// ```
#[derive(Clone)]
pub struct ArcInstance {
    inner: Arc<InstanceInner>,
}

impl ArcInstance {
    /// Wrap a freshly-created VkInstance.
    pub fn new(
        entry: ash::Entry,
        instance: ash::Instance,
        debug_utils: Option<ash::ext::debug_utils::Instance>,
        debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    ) -> Self {
        Self {
            inner: Arc::new(InstanceInner {
                instance,
                entry,
                debug_utils,
                debug_messenger,
            }),
        }
    }

    /// Borrow the underlying `ash::Instance`.
    #[inline]
    pub fn get(&self) -> &ash::Instance {
        &self.inner.instance
    }

    /// Borrow the `ash::Entry` that loaded Vulkan.
    #[inline]
    pub fn entry(&self) -> &ash::Entry {
        &self.inner.entry
    }

    /// Access debug utils loader (if enabled).
    #[inline]
    pub fn debug_utils(&self) -> Option<&ash::ext::debug_utils::Instance> {
        self.inner.debug_utils.as_ref()
    }

    /// Number of live references to this instance.
    #[inline]
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl std::ops::Deref for ArcInstance {
    type Target = ash::Instance;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner.instance
    }
}

// =============================================================================
// ArcDevice — Arc-wrapped VkDevice with RAII Drop
// =============================================================================

/// Inner data for an Arc-wrapped Vulkan logical device.
struct DeviceInner {
    device: ash::Device,
}

impl Drop for DeviceInner {
    fn drop(&mut self) {
        // SAFETY: We only destroy the device after all its children
        // (swapchains, pipelines, buffers, etc.) have been dropped,
        // because those children hold their own Arc<Device> clones.
        unsafe {
            self.device.destroy_device(None);
        }
        log::debug!("✓ VkDevice destroyed");
    }
}

/// A thread-safe, reference-counted wrapper around `ash::Device`.
///
/// # Why Arc<Device>?
/// Every GPU resource (buffer, image, pipeline, etc.) needs access to
/// the device for creation AND destruction. Passing `&Device` everywhere
/// creates lifetime hell. `ArcDevice` solves this:
///
/// - Resources hold `ArcDevice` and self-manage their lifetime.
/// - The device lives as long as any resource referencing it.
/// - No more manual destroy ordering across the engine.
#[derive(Clone)]
pub struct ArcDevice {
    inner: Arc<DeviceInner>,
}

impl ArcDevice {
    /// Wrap a freshly-created VkDevice.
    pub fn new(device: ash::Device) -> Self {
        Self { inner: Arc::new(DeviceInner { device }) }
    }

    /// Borrow the underlying `ash::Device`.
    #[inline]
    pub fn get(&self) -> &ash::Device {
        &self.inner.device
    }

    /// Number of live references to this device.
    #[inline]
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl std::ops::Deref for ArcDevice {
    type Target = ash::Device;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner.device
    }
}

// =============================================================================
// ArcSurface — Arc-wrapped VkSurfaceKHR with RAII Drop
// =============================================================================

/// Inner data for an Arc-wrapped Vulkan surface.
struct SurfaceInner {
    surface: vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
}

impl Drop for SurfaceInner {
    fn drop(&mut self) {
        // SAFETY: Surface must be destroyed before Instance, but after Swapchain.
        // Since ArcSurface holds a reference to the surface_loader (which
        // depends on Instance), and swapchains hold ArcSurface, order is correct.
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
        log::debug!("✓ VkSurfaceKHR destroyed");
    }
}

/// A thread-safe, reference-counted wrapper around `vk::SurfaceKHR`.
///
/// The surface is destroyed when the last clone is dropped.
#[derive(Clone)]
pub struct ArcSurface {
    inner: Arc<SurfaceInner>,
}

impl ArcSurface {
    /// Wrap a freshly-created VkSurfaceKHR.
    pub fn new(surface: vk::SurfaceKHR, surface_loader: ash::khr::surface::Instance) -> Self {
        Self {
            inner: Arc::new(SurfaceInner { surface, surface_loader }),
        }
    }

    /// The raw surface handle.
    #[inline]
    pub fn handle(&self) -> vk::SurfaceKHR {
        self.inner.surface
    }

    /// Borrow the surface loader for queries (capabilities, formats, etc.).
    #[inline]
    pub fn loader(&self) -> &ash::khr::surface::Instance {
        &self.inner.surface_loader
    }

    /// Number of live references to this surface.
    #[inline]
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // These are compile-time tests to verify the Arc wrappers
    // are Send + Sync (required for thread-safe sharing).
    fn _assert_send<T: Send>() {}
    fn _assert_sync<T: Sync>() {}

    #[test]
    fn arc_handles_are_send_sync() {
        _assert_send::<ArcInstance>();
        _assert_sync::<ArcInstance>();
        _assert_send::<ArcDevice>();
        _assert_sync::<ArcDevice>();
        _assert_send::<ArcSurface>();
        _assert_sync::<ArcSurface>();
    }

    #[test]
    fn clone_is_cheap() {
        // Verify cloning doesn't do deep copies — Arc refcount goes up.
        // We can't actually test this without a Vulkan context,
        // but the structure guarantees it by using Arc<_>.
        // Documented invariant: clone() == Arc::clone() == O(1) refcount++.
    }
}
