// =============================================================================
// REACTOR Debug Utils — Vulkan Resource Labeling
// =============================================================================
// Provides named labels for Vulkan resources (buffers, images, pipelines, etc.)
// so they show up with descriptive names in profiling tools like RenderDoc,
// NVIDIA Nsight, and the Vulkan validation layers.
//
// Uses VK_EXT_debug_utils (vkSetDebugUtilsObjectNameEXT) under the hood.
// When debug utils are not available, all calls become no-ops.
// =============================================================================

use ash::vk;
use std::ffi::CString;

/// Wrapper around `ash::ext::debug_utils::Device` for naming Vulkan objects.
///
/// When the extension is available, every `set_debug_name` call labels the
/// resource in the driver so profiling tools display a human-readable name.
/// When unavailable, all operations silently no-op.
#[derive(Clone)]
pub struct DebugNamer {
    /// The device-level debug utils loader (None in release builds without the extension).
    loader: Option<ash::ext::debug_utils::Device>,
}

impl DebugNamer {
    /// Create a new `DebugNamer` from the instance and device.
    ///
    /// This is only functional when `VK_EXT_debug_utils` was enabled in the
    /// instance. Pass `None` for `instance_debug_utils` to create a no-op namer.
    pub fn new(
        instance_debug_utils: Option<&ash::ext::debug_utils::Instance>,
        instance: &ash::Instance,
        device: &ash::Device,
    ) -> Self {
        let loader =
            instance_debug_utils.map(|_| ash::ext::debug_utils::Device::new(instance, device));
        Self { loader }
    }

    /// Create a no-op namer (for when debug utils are not available).
    pub fn noop() -> Self {
        Self { loader: None }
    }

    /// Returns `true` if debug naming is functional.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.loader.is_some()
    }

    /// Assign a debug name to a Vulkan object.
    ///
    /// # Type Safety
    ///
    /// This method accepts any Vulkan handle type that implements `vk::Handle`.
    /// The `object_type` must match the handle type — see `vk::ObjectType` docs.
    ///
    /// # Example
    /// ```ignore
    /// namer.set_name(my_buffer, vk::ObjectType::BUFFER, "Vertex Buffer: Player");
    /// namer.set_name(my_image, vk::ObjectType::IMAGE, "Texture: ZombieAlbedo");
    /// ```
    pub fn set_name<H: vk::Handle>(&self, handle: H, object_type: vk::ObjectType, name: &str) {
        if let Some(ref loader) = self.loader {
            let c_name = CString::new(name).unwrap_or_else(|_| CString::new("?").unwrap());
            let mut name_info = vk::DebugUtilsObjectNameInfoEXT::default()
                .object_handle(handle)
                .object_name(&c_name);
            name_info.object_type = object_type;
            unsafe {
                let _ = loader.set_debug_utils_object_name(&name_info);
            }
        }
    }

    // ─── Convenience helpers for common resource types ───────────────────

    /// Label a `VkBuffer`.
    #[inline]
    pub fn name_buffer(&self, buffer: vk::Buffer, name: &str) {
        self.set_name(buffer, vk::ObjectType::BUFFER, name);
    }

    /// Label a `VkImage`.
    #[inline]
    pub fn name_image(&self, image: vk::Image, name: &str) {
        self.set_name(image, vk::ObjectType::IMAGE, name);
    }

    /// Label a `VkImageView`.
    #[inline]
    pub fn name_image_view(&self, view: vk::ImageView, name: &str) {
        self.set_name(view, vk::ObjectType::IMAGE_VIEW, name);
    }

    /// Label a `VkPipeline`.
    #[inline]
    pub fn name_pipeline(&self, pipeline: vk::Pipeline, name: &str) {
        self.set_name(pipeline, vk::ObjectType::PIPELINE, name);
    }

    /// Label a `VkPipelineLayout`.
    #[inline]
    pub fn name_pipeline_layout(&self, layout: vk::PipelineLayout, name: &str) {
        self.set_name(layout, vk::ObjectType::PIPELINE_LAYOUT, name);
    }

    /// Label a `VkCommandPool`.
    #[inline]
    pub fn name_command_pool(&self, pool: vk::CommandPool, name: &str) {
        self.set_name(pool, vk::ObjectType::COMMAND_POOL, name);
    }

    /// Label a `VkCommandBuffer`.
    #[inline]
    pub fn name_command_buffer(&self, cmd: vk::CommandBuffer, name: &str) {
        self.set_name(cmd, vk::ObjectType::COMMAND_BUFFER, name);
    }

    /// Label a `VkSemaphore`.
    #[inline]
    pub fn name_semaphore(&self, semaphore: vk::Semaphore, name: &str) {
        self.set_name(semaphore, vk::ObjectType::SEMAPHORE, name);
    }

    /// Label a `VkFence`.
    #[inline]
    pub fn name_fence(&self, fence: vk::Fence, name: &str) {
        self.set_name(fence, vk::ObjectType::FENCE, name);
    }

    /// Label a `VkSwapchainKHR`.
    #[inline]
    pub fn name_swapchain(&self, swapchain: vk::SwapchainKHR, name: &str) {
        self.set_name(swapchain, vk::ObjectType::SWAPCHAIN_KHR, name);
    }

    /// Label a `VkDescriptorSet`.
    #[inline]
    pub fn name_descriptor_set(&self, set: vk::DescriptorSet, name: &str) {
        self.set_name(set, vk::ObjectType::DESCRIPTOR_SET, name);
    }

    /// Label a `VkDescriptorSetLayout`.
    #[inline]
    pub fn name_descriptor_set_layout(&self, layout: vk::DescriptorSetLayout, name: &str) {
        self.set_name(layout, vk::ObjectType::DESCRIPTOR_SET_LAYOUT, name);
    }

    /// Label a `VkFramebuffer`.
    #[inline]
    pub fn name_framebuffer(&self, fb: vk::Framebuffer, name: &str) {
        self.set_name(fb, vk::ObjectType::FRAMEBUFFER, name);
    }

    /// Label a `VkRenderPass`.
    #[inline]
    pub fn name_render_pass(&self, rp: vk::RenderPass, name: &str) {
        self.set_name(rp, vk::ObjectType::RENDER_PASS, name);
    }

    /// Label a `VkSampler`.
    #[inline]
    pub fn name_sampler(&self, sampler: vk::Sampler, name: &str) {
        self.set_name(sampler, vk::ObjectType::SAMPLER, name);
    }

    /// Label a `VkShaderModule`.
    #[inline]
    pub fn name_shader_module(&self, module: vk::ShaderModule, name: &str) {
        self.set_name(module, vk::ObjectType::SHADER_MODULE, name);
    }

    /// Label a `VkDeviceMemory`.
    #[inline]
    pub fn name_device_memory(&self, memory: vk::DeviceMemory, name: &str) {
        self.set_name(memory, vk::ObjectType::DEVICE_MEMORY, name);
    }

    /// Label a `VkQueue`.
    #[inline]
    pub fn name_queue(&self, queue: vk::Queue, name: &str) {
        self.set_name(queue, vk::ObjectType::QUEUE, name);
    }
}
