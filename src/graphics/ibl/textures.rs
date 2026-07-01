use crate::graphics::ibl::image::IblImage;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator};
use std::sync::{Arc, Mutex};

pub struct IblTextures {
    pub irradiance: IblImage,
    pub prefiltered: IblImage,
    pub brdf_lut: IblImage,
    pub sampler_cube: vk::Sampler,
    pub sampler_2d: vk::Sampler,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
    pub params_buffer: vk::Buffer,
    pub params_allocation: Option<Allocation>,
    pub max_mip_level: f32,
    pub(crate) device: ash::Device,
    pub(crate) allocator: Arc<Mutex<Allocator>>,
}

impl Drop for IblTextures {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device.destroy_sampler(self.sampler_cube, None);
            self.device.destroy_sampler(self.sampler_2d, None);
            self.device.destroy_buffer(self.params_buffer, None);
        }
        if let Some(a) = self.params_allocation.take() {
            let _ = self.allocator.lock().unwrap().free(a);
        }
    }
}
