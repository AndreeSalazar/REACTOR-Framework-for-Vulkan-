use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator};
use std::sync::{Arc, Mutex};

pub struct IblImage {
    pub image: vk::Image,
    pub allocation: Option<Allocation>,
    pub view: vk::ImageView,
    pub mip_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub mip_levels: u32,
    pub layer_count: u32,
    pub(crate) device: ash::Device,
    pub(crate) allocator: Arc<Mutex<Allocator>>,
}

impl Drop for IblImage {
    fn drop(&mut self) {
        unsafe {
            for v in self.mip_views.drain(..) {
                self.device.destroy_image_view(v, None);
            }
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.image, None);
        }
        if let Some(alloc) = self.allocation.take() {
            let _ = self.allocator.lock().unwrap().free(alloc);
        }
    }
}
