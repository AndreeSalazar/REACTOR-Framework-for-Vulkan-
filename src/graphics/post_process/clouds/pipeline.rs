use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use crate::graphics::Image;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use std::sync::{Arc, Mutex};

use super::noise_image::{create_3d_noise_image, Noise3D};

pub struct VolumetricClouds {
    pub pipeline: Option<crate::compute::ComputePipeline>,
    pub descriptor_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub output_images: Vec<Image>,
    pub time: f32,
    shape: Noise3D,
    detail: Noise3D,
    noise_sampler: vk::Sampler,
    device: ash::Device,
}

impl VolumetricClouds {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        width: u32,
        height: u32,
        image_count: u32,
        linear_sampler: vk::Sampler,
    ) -> ReactorResult<Self> {
        let device = ctx.ash_device().clone();

        let shape = create_3d_noise_image(ctx, &device, allocator.clone(), command_pool, queue, 64, 0xDEAD_BEEF)?;
        let detail = create_3d_noise_image(ctx, &device, allocator.clone(), command_pool, queue, 16, 0xC0FF_EE42)?;

        let noise_sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS);
        let noise_sampler = unsafe { device.create_sampler(&noise_sampler_info, None) }.map_err(|e| {
            ReactorError::with_source(ErrorCode::VulkanImageCreation, "Clouds: create sampler", e)
        })?;

        let bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(2).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(3).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }.map_err(|e| {
            ReactorError::with_source(ErrorCode::VulkanDescriptorSet, "Clouds: create descriptor layout", e)
        })?;

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../../shaders/post/volumetric_clouds.spv"
        )))
        .map_err(|e| ReactorError::with_source(ErrorCode::VulkanShaderCompilation, "Clouds: load volumetric_clouds.spv", e))?;
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(144))?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(image_count),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count * 3),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes).max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None) }.map_err(|e| {
            ReactorError::with_source(ErrorCode::VulkanDescriptorSet, "Clouds: create descriptor pool", e)
        })?;

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool).set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info) }.map_err(|e| {
            ReactorError::with_source(ErrorCode::VulkanDescriptorSet, "Clouds: allocate descriptor sets", e)
        })?;

        let mut output_images = Vec::with_capacity(image_count as usize);
        for _ in 0..image_count {
            let img = Image::new(ctx, allocator.clone(), width, height, vk::Format::R16G16B16A16_SFLOAT,
                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED, vk::ImageAspectFlags::COLOR, 1)?;
            output_images.push(img);
        }

        let shape_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(shape.view).sampler(linear_sampler);
        let detail_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(detail.view).sampler(linear_sampler);

        for i in 0..image_count as usize {
            let set = descriptor_sets[i];
            let output_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL).image_view(output_images[i].view);
            let writes = [
                vk::WriteDescriptorSet::default().dst_set(set).dst_binding(0).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&output_info)),
                vk::WriteDescriptorSet::default().dst_set(set).dst_binding(2).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&shape_info)),
                vk::WriteDescriptorSet::default().dst_set(set).dst_binding(3).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&detail_info)),
            ];
            unsafe { device.update_descriptor_sets(&writes, &[]); }
        }

        Ok(Self { pipeline: Some(pipeline), descriptor_layout, descriptor_pool, descriptor_sets, output_images, time: 0.0, shape, detail, noise_sampler, device })
    }

    pub fn advance_time(&mut self, dt: f32) {
        self.time += dt;
    }

    pub fn dispatch(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        width: u32,
        height: u32,
        inv_view_proj: glam::Mat4,
        camera_pos: glam::Vec3,
        sun_direction: glam::Vec3,
        sun_color: glam::Vec3,
        depth_view: vk::ImageView,
        sampler: vk::Sampler,
    ) {
        let Some(pipeline) = self.pipeline.as_ref() else { return };
        let Some(set) = self.descriptor_sets.get(image_index) else { return };
        if image_index >= self.output_images.len() { return }

        let depth_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(depth_view).sampler(sampler);
        let depth_write = vk::WriteDescriptorSet::default()
            .dst_set(*set).dst_binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&depth_info));
        unsafe { device.update_descriptor_sets(&[depth_write], &[]); }

        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(self.output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
        unsafe {
            device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_general]);
        }

        let mut push_bytes = [0u8; 144];
        let mut o = 0usize;
        for col in inv_view_proj.to_cols_array() {
            push_bytes[o..o + 4].copy_from_slice(&col.to_ne_bytes()); o += 4;
        }
        push_bytes[o..o + 4].copy_from_slice(&camera_pos.x.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&camera_pos.y.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&camera_pos.z.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0f32.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_direction.x.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_direction.y.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_direction.z.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0f32.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_color.x.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_color.y.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_color.z.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&1.0f32.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&2000.0f32.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&6000.0f32.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0.5f32.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0.6f32.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&self.time.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&15.0f32.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&64u32.to_ne_bytes()); o += 4;
        push_bytes[o..o + 4].copy_from_slice(&6u32.to_ne_bytes());

        pipeline.bind(command_buffer, device);
        unsafe {
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.layout, 0, &[*set], &[]);
            device.cmd_push_constants(command_buffer, pipeline.layout, vk::ShaderStageFlags::COMPUTE, 0, &push_bytes);
            let gx = (width + 7) / 8;
            let gy = (height + 7) / 8;
            device.cmd_dispatch(command_buffer, gx, gy, 1);
        }

        let to_read = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(self.output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
        unsafe {
            device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_read]);
        }
    }
}

impl Drop for VolumetricClouds {
    fn drop(&mut self) {
        unsafe {
            self.output_images.clear();
            self.device.destroy_image_view(self.shape.view, None);
            self.device.destroy_image(self.shape.image, None);
            self.device.destroy_image_view(self.detail.view, None);
            self.device.destroy_image(self.detail.image, None);
            self.device.destroy_sampler(self.noise_sampler, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
    }
}
