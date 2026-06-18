use crate::core::arc_handle::ArcDevice;
use crate::core::error::ReactorResult;
use crate::core::VulkanContext;
use crate::resources::texture::Texture;
use ash::vk;
use glam::{Mat4, Vec4};
use std::sync::Arc;

pub struct Decal {
    pub model: Mat4,
    pub albedo: Arc<Texture>,
    pub normal: Arc<Texture>,
    pub material: Arc<Texture>,
    pub color: Vec4,
    pub normal_strength: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub descriptor_set: vk::DescriptorSet,
    pub descriptor_pool: vk::DescriptorPool,
    device: ArcDevice,
}

impl Decal {
    pub fn new(
        ctx: &VulkanContext,
        descriptor_layout: vk::DescriptorSetLayout,
        model: Mat4,
        albedo: Arc<Texture>,
        normal: Arc<Texture>,
        material: Arc<Texture>,
        color: Vec4,
        normal_strength: f32,
        metallic: f32,
        roughness: f32,
    ) -> ReactorResult<Self> {
        let device = ctx.ash_device();

        // 1. Create descriptor pool for this decal
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(4),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        // 2. Allocate descriptor set
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&descriptor_layout));
        let descriptor_set = unsafe { device.allocate_descriptor_sets(&alloc_info)?[0] };

        // 3. Write static descriptors (bindings 1, 2, 3)
        // Note: binding 0 (depth texture) is written dynamically per frame inside draw_scene.
        let albedo_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(albedo.view())
            .sampler(albedo.sampler_handle());

        let normal_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(normal.view())
            .sampler(normal.sampler_handle());

        let material_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(material.view())
            .sampler(material.sampler_handle());

        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&albedo_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&normal_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(3)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&material_info)),
        ];

        unsafe {
            device.update_descriptor_sets(&writes, &[]);
        }

        Ok(Self {
            model,
            albedo,
            normal,
            material,
            color,
            normal_strength,
            metallic,
            roughness,
            descriptor_set,
            descriptor_pool,
            device: ctx.device.clone(),
        })
    }

    /// Dynamic update of the depth texture descriptor (binding 0) for a given frame
    pub fn update_depth_descriptor(&self, depth_view: vk::ImageView, sampler: vk::Sampler) {
        let depth_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(depth_view)
            .sampler(sampler);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&depth_info));

        unsafe {
            self.device.update_descriptor_sets(&[write], &[]);
        }
    }
}

impl Drop for Decal {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
