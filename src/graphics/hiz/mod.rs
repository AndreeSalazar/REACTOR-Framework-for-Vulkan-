//! Hi-Z (Hierarchical Z) depth pyramid — pre-filtered mip chain of min depth
//!
//! Provides a compute pipeline that reduces the resolved single-sample depth
//! image into a mip chain of min-depth values, used by:
//! - `cull.comp` (GPU frustum + occlusion culling)
//! - `ssgi_hiz.comp` (Hi-Z ray-march SSGI)
//!
//! Public API:
//! - `HiZPyramid::new(ctx, allocator, width, height, image_count)` — allocate
//!   the depth image with mip levels and descriptor resources.
//! - `HiZPyramid::build(cmd, device, image_index, frame_time)` — record a
//!   mip-down chain dispatch starting from the resolved depth.
//! - `HiZPyramid::mip_view(index, level)` — sampler view for a specific mip
//!

use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use crate::graphics::Image;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use std::sync::{Arc, Mutex};

pub struct HiZPyramid {
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    pub images: Vec<Image>,
    pub mip_views: Vec<Vec<vk::ImageView>>,
    pub pipeline: Option<crate::compute::ComputePipeline>,
    pub descriptor_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    device: ash::Device,
}

impl HiZPyramid {
    pub fn new(
        ctx: &VulkanContext,
        _allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
    ) -> ReactorResult<Self> {
        let device = ctx.ash_device().clone();

        let mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;
        let mip_levels = mip_levels.min(13);

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
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
                    "Hi-Z: create descriptor set layout",
                    e,
                )
            })?;

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/hiz_build.spv"
        )))
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanShaderCompilation,
                "Hi-Z: load hiz_build.spv",
                e,
            )
        })?;
        let pipeline = crate::compute::ComputePipeline::new(
            ctx,
            &spv,
            &[descriptor_layout],
            Some(16),
        )?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count * mip_levels),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(image_count * mip_levels),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(image_count * mip_levels)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None) }
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanDescriptorSet,
                    "Hi-Z: create descriptor pool",
                    e,
                )
            })?;

        let mut images = Vec::with_capacity(image_count as usize);
        let mut mip_views = Vec::with_capacity(image_count as usize);

        for _ in 0..image_count {
            let image = Image::new(
                ctx,
                _allocator.clone(),
                width,
                height,
                vk::Format::R32_SFLOAT,
                vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE,
                vk::ImageAspectFlags::COLOR,
                mip_levels,
            )?;
            let mut per_image_views = Vec::with_capacity(mip_levels as usize);
            for level in 0..mip_levels {
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(image.handle)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(vk::Format::R32_SFLOAT)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: level,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                let view = unsafe { device.create_image_view(&view_info, None) }.map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanImageCreation,
                        "Hi-Z: create mip view",
                        e,
                    )
                })?;
                per_image_views.push(view);
            }
            mip_views.push(per_image_views);
            images.push(image);
        }

        let layouts = vec![descriptor_layout; (image_count * mip_levels) as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info) }.map_err(
            |e| {
                ReactorError::with_source(
                    ErrorCode::VulkanDescriptorSet,
                    "Hi-Z: allocate descriptor sets",
                    e,
                )
            },
        )?;

        Ok(Self {
            width,
            height,
            mip_levels,
            images,
            mip_views,
            pipeline: Some(pipeline),
            descriptor_layout,
            descriptor_pool,
            descriptor_sets,
            device,
        })
    }

    pub fn mip_view(&self, image_index: usize, mip_level: u32) -> vk::ImageView {
        self.mip_views
            .get(image_index)
            .and_then(|v| v.get(mip_level as usize))
            .copied()
            .unwrap_or(vk::ImageView::null())
    }

    pub fn build(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        src_depth_view: vk::ImageView,
        sampler: vk::Sampler,
    ) {
        let Some(pipeline) = self.pipeline.as_ref() else {
            return;
        };
        if image_index >= self.images.len() {
            return;
        }
        let dst_image = &self.images[image_index];

        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(dst_image.handle)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: self.mip_levels,
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

        let mut src_view = src_depth_view;
        for level in 0..self.mip_levels {
            let dst_w = (self.width >> level).max(1);
            let dst_h = (self.height >> level).max(1);

            let set_index = (image_index * self.mip_levels as usize) + level as usize;
            let set = match self.descriptor_sets.get(set_index) {
                Some(s) => *s,
                None => return,
            };

            let src_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(src_view)
                .sampler(sampler);
            let dst_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(self.mip_view(image_index, level));

            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&src_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .image_info(std::slice::from_ref(&dst_info)),
            ];
            unsafe {
                device.update_descriptor_sets(&writes, &[]);
            }

            let mut push_bytes = [0u8; 16];
            push_bytes[0..4].copy_from_slice(&0i32.to_ne_bytes());
            push_bytes[4..8].copy_from_slice(&0i32.to_ne_bytes());
            push_bytes[8..12].copy_from_slice(&0i32.to_ne_bytes());
            push_bytes[12..16].copy_from_slice(&0i32.to_ne_bytes());

            pipeline.bind(command_buffer, device);
            unsafe {
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    pipeline.layout,
                    0,
                    &[set],
                    &[],
                );
                device.cmd_push_constants(
                    command_buffer,
                    pipeline.layout,
                    vk::ShaderStageFlags::COMPUTE,
                    0,
                    &push_bytes,
                );
                let gx = (dst_w + 7) / 8;
                let gy = (dst_h + 7) / 8;
                device.cmd_dispatch(command_buffer, gx, gy, 1);
            }

            src_view = self.mip_view(image_index, level);
        }

        let to_read = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(dst_image.handle)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: self.mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            });
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_read],
            );
        }
    }
}

impl Drop for HiZPyramid {
    fn drop(&mut self) {
        unsafe {
            for per_image_views in self.mip_views.drain(..) {
                for view in per_image_views {
                    self.device.destroy_image_view(view, None);
                }
            }
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
    }
}

