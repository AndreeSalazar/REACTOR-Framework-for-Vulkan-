//! SSGI Hi-Z — screen-space global illumination with hierarchical Z traversal
//!
//! Wraps `shaders/post/ssgi_hiz.comp` into the post-process pipeline.
//! Replaces the basic inline SSGI in `post_process.frag` when enabled.
//!
//! Bindings: 0=color, 1=depth, 2=normal, 3=Hi-Z mip 0, 4=history, 5=output (storage)

use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use crate::graphics::Image;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use std::sync::{Arc, Mutex};

pub struct SsgiHiZ {
    pub pipeline: Option<crate::compute::ComputePipeline>,
    pub descriptor_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub output_images: Vec<Image>,
    pub history_images: Vec<Image>,
    pub frame_index: u32,
    device: ash::Device,
}

impl SsgiHiZ {
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
    ) -> ReactorResult<Self> {
        let device = ctx.ash_device().clone();

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(4)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(5)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanDescriptorSet,
                    "SSGI Hi-Z: create descriptor layout",
                    e,
                )
            })?;

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/ssgi_hiz.spv"
        )))
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanShaderCompilation,
                "SSGI Hi-Z: load ssgi_hiz.spv",
                e,
            )
        })?;
        let pipeline = crate::compute::ComputePipeline::new(
            ctx,
            &spv,
            &[descriptor_layout],
            Some(128),
        )?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count * 5),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None) }
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanDescriptorSet,
                    "SSGI Hi-Z: create descriptor pool",
                    e,
                )
            })?;

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info) }.map_err(
            |e| {
                ReactorError::with_source(
                    ErrorCode::VulkanDescriptorSet,
                    "SSGI Hi-Z: allocate descriptor sets",
                    e,
                )
            },
        )?;

        let mut output_images = Vec::with_capacity(image_count as usize);
        let mut history_images = Vec::with_capacity(image_count as usize);
        for _ in 0..image_count {
            let img = Image::new(
                ctx,
                allocator.clone(),
                width,
                height,
                vk::Format::R16G16B16A16_SFLOAT,
                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
                vk::ImageAspectFlags::COLOR,
                1,
            )?;
            let hist = Image::new(
                ctx,
                allocator.clone(),
                width,
                height,
                vk::Format::R16G16B16A16_SFLOAT,
                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
                vk::ImageAspectFlags::COLOR,
                1,
            )?;
            output_images.push(img);
            history_images.push(hist);
        }

        Ok(Self {
            pipeline: Some(pipeline),
            descriptor_layout,
            descriptor_pool,
            descriptor_sets,
            output_images,
            history_images,
            frame_index: 0,
            device,
        })
    }

    pub fn advance_frame(&mut self) {
        self.frame_index = self.frame_index.wrapping_add(1);
    }

    pub fn dispatch(
        &mut self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        width: u32,
        height: u32,
        view_proj: glam::Mat4,
        inv_view_proj: glam::Mat4,
        camera_pos: glam::Vec3,
        color_view: vk::ImageView,
        depth_view: vk::ImageView,
        normal_view: vk::ImageView,
        hiz_mip0_view: vk::ImageView,
        sampler: vk::Sampler,
        intensity: f32,
    ) {
        let Some(pipeline) = self.pipeline.as_ref() else {
            return;
        };
        let Some(set) = self.descriptor_sets.get(image_index) else {
            return;
        };
        if image_index >= self.output_images.len() {
            return;
        }

        let history_view = self.history_images[image_index].view;

        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(self.output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_general],
            );
        }

        let infos = [
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(color_view)
                .sampler(sampler),
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(depth_view)
                .sampler(sampler),
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(normal_view)
                .sampler(sampler),
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(hiz_mip0_view)
                .sampler(sampler),
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(history_view)
                .sampler(sampler),
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(self.output_images[image_index].view),
        ];

        let writes: [vk::WriteDescriptorSet; 6] = std::array::from_fn(|i| {
            vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(i as u32)
                .descriptor_type(if i == 5 {
                    vk::DescriptorType::STORAGE_IMAGE
                } else {
                    vk::DescriptorType::COMBINED_IMAGE_SAMPLER
                })
                .image_info(std::slice::from_ref(&infos[i]))
        });
        unsafe {
            device.update_descriptor_sets(&writes, &[]);
        }

        let mut push_bytes = [0u8; 128];
        let mut o = 0usize;
        for col in view_proj.to_cols_array() {
            push_bytes[o..o + 4].copy_from_slice(&col.to_ne_bytes());
            o += 4;
        }
        for col in inv_view_proj.to_cols_array() {
            push_bytes[o..o + 4].copy_from_slice(&col.to_ne_bytes());
            o += 4;
        }
        push_bytes[o..o + 4].copy_from_slice(&camera_pos.x.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&camera_pos.y.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&camera_pos.z.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&(width as f32).to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&(height as f32).to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&intensity.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0.5f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0.1f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0.9f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&32u32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&1u32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0.1f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&1000f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&self.frame_index.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0f32.to_ne_bytes());

        pipeline.bind(command_buffer, device);
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout,
                0,
                &[*set],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                &push_bytes,
            );
            let gx = (width + 7) / 8;
            let gy = (height + 7) / 8;
            device.cmd_dispatch(command_buffer, gx, gy, 1);
        }

        let to_read = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(self.output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_read],
            );
        }
    }
}

impl Drop for SsgiHiZ {
    fn drop(&mut self) {
        unsafe {
            self.output_images.clear();
            self.history_images.clear();
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
    }
}
