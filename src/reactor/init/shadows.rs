//! `Reactor::init_shadows` — Cascaded Shadow Maps initialization
//!
//! Responsible for creating the shadow map image, views, sampler, descriptor
//! set, and depth-only render pipeline for the CSM system.

use super::super::{Reactor, MAX_FRAMES_IN_FLIGHT};
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use ash::vk;

impl Reactor {
    /// Inicializa toda la infraestructura para Cascaded Shadow Maps (CSM)
    pub fn init_shadows(&mut self) -> ReactorResult<()> {

        let shadow_map = crate::graphics::shadows::ShadowMap::new(
            crate::graphics::shadows::ShadowConfig::default(),
        );

        let width = shadow_map.config.resolution;
        let height = shadow_map.config.resolution;
        let cascade_count = shadow_map.config.cascade_count;
        let format = vk::Format::D32_SFLOAT;
        let device = self.context.ash_device();

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(cascade_count)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let shadow_image = unsafe { device.create_image(&image_info, None)? };
        let requirements = unsafe { device.get_image_memory_requirements(shadow_image) };

        let memory_props = unsafe {
            self.context
                .instance
                .get_physical_device_memory_properties(self.context.physical_device)
        };
        let memory_type_index = (0..memory_props.memory_type_count)
            .find(|&i| {
                let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
                let memory_type = memory_props.memory_types[i as usize];
                suitable
                    && memory_type
                        .property_flags
                        .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })
            .ok_or_else(|| {
                ReactorError::new(
                    ErrorCode::VulkanMemoryAllocation,
                    "Failed to find memory type for shadow map",
                )
            })?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);

        let shadow_memory = unsafe { device.allocate_memory(&alloc_info, None)? };
        unsafe { device.bind_image_memory(shadow_image, shadow_memory, 0)? };

        let array_view_info = vk::ImageViewCreateInfo::default()
            .image(shadow_image)
            .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: cascade_count,
            });
        let shadow_array_view = unsafe { device.create_image_view(&array_view_info, None)? };

        let mut shadow_image_views = Vec::with_capacity(cascade_count as usize);
        for layer in 0..cascade_count {
            let view_info = vk::ImageViewCreateInfo::default()
                .image(shadow_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: layer,
                    layer_count: 1,
                });
            let view = unsafe { device.create_image_view(&view_info, None)? };
            shadow_image_views.push(view);
        }

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .compare_enable(false)
            .compare_op(vk::CompareOp::LESS_OR_EQUAL);
        let shadow_sampler = unsafe { device.create_sampler(&sampler_info, None)? };

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let shadow_descriptor_layout =
            unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32)
            .flags(
                vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET
                    | vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND,
            );
        let shadow_descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let layouts = vec![shadow_descriptor_layout; MAX_FRAMES_IN_FLIGHT];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(shadow_descriptor_pool)
            .set_layouts(&layouts);
        let shadow_descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        let mut shadow_uniform_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let size = std::mem::size_of::<crate::graphics::shadows::ShadowUniformData>() as u64;
            let buffer = crate::graphics::buffer::Buffer::new_uniform(
                &self.context,
                self.allocator.clone(),
                size,
            )?;

            let image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(shadow_array_view)
                .sampler(shadow_sampler);

            let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(buffer.handle)
                .offset(0)
                .range(size);

            let write_image = vk::WriteDescriptorSet::default()
                .dst_set(shadow_descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&image_info));

            let write_buffer = vk::WriteDescriptorSet::default()
                .dst_set(shadow_descriptor_sets[i])
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&buffer_info));

            unsafe {
                device.update_descriptor_sets(&[write_image, write_buffer], &[]);
            }
            shadow_uniform_buffers.push(buffer);
        }

        let shadow_vert_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/shadow_vert.spv"
        )))
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanShaderCompilation,
                "Failed to load shadow_vert spv",
                e,
            )
        })?;

        let shadow_frag_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/shadow_frag.spv"
        )))
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanShaderCompilation,
                "Failed to load shadow_frag spv",
                e,
            )
        })?;

        let config = crate::graphics::pipeline::PipelineConfig {
            cull_mode: vk::CullModeFlags::BACK,
            depth_write: true,
            depth_test: true,
            ..Default::default()
        };

        let shadow_pipeline = crate::graphics::pipeline::Pipeline::with_config(
            &self.context.device,
            None,
            &shadow_vert_spv,
            &shadow_frag_spv,
            width,
            height,
            &config,
            &[shadow_descriptor_layout],
            vk::Format::UNDEFINED,
            Some(vk::Format::D32_SFLOAT),
        )?;

        self.shadow_map = Some(shadow_map);
        self.shadow_image = Some(shadow_image);
        self.shadow_image_views = shadow_image_views;
        self.shadow_array_view = Some(shadow_array_view);
        self.shadow_sampler = Some(shadow_sampler);
        self.shadow_memory = Some(shadow_memory);
        self.shadow_pipeline = Some(shadow_pipeline);
        self.shadow_descriptor_layout = Some(shadow_descriptor_layout);
        self.shadow_descriptor_pool = Some(shadow_descriptor_pool);
        self.shadow_descriptor_sets = shadow_descriptor_sets;
        self.shadow_uniform_buffers = shadow_uniform_buffers;

        println!(
            "✅ CSM Shadow Maps initialized: {} cascades @ {}x{}",
            cascade_count, width, height
        );

        Ok(())
    }
}
