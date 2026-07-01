use ash::vk;
use super::PostProcessPipeline;

impl PostProcessPipeline {
    pub(super) fn init_taa(&mut self, ctx: &crate::core::VulkanContext, image_count: u32) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();
        let bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(2).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(3).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(4).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(5).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(6).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings).flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post/taa_resolve.spv"))).unwrap();
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(16))?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count * 5),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(image_count * 2),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(image_count).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        self.taa_pipeline = Some(pipeline);
        self.taa_descriptor_layout = Some(descriptor_layout);
        self.taa_descriptor_pool = Some(descriptor_pool);
        self.taa_descriptor_sets = descriptor_sets;
        Ok(())
    }

    pub(super) fn destroy_taa_resources(&mut self, device: &ash::Device) {
        self.taa_descriptor_sets.clear();
        self.taa_pipeline = None;
        unsafe {
            if let Some(pool) = self.taa_descriptor_pool.take() { device.destroy_descriptor_pool(pool, None); }
            if let Some(layout) = self.taa_descriptor_layout.take() { device.destroy_descriptor_set_layout(layout, None); }
        }
    }

    pub fn dispatch_taa(&self, device: &ash::Device, command_buffer: vk::CommandBuffer, image_index: usize, history: &crate::graphics::TemporalHistory, gbuffer: &crate::graphics::GBuffer, depth_view: vk::ImageView, reset_history: bool) {
        let Some(pipeline) = self.taa_pipeline.as_ref() else { return; };

        let prev_color_old_layout = if history.frame_index == 0 { vk::ImageLayout::UNDEFINED } else { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL };
        let prev_depth_old_layout = if history.frame_index == 0 { vk::ImageLayout::UNDEFINED } else { vk::ImageLayout::GENERAL };
        let curr_old_layout = if history.frame_index == 0 { vk::ImageLayout::UNDEFINED } else { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL };
        let prev_color_src_access = if history.frame_index == 0 { vk::AccessFlags::empty() } else { vk::AccessFlags::SHADER_READ };
        let prev_depth_src_access = if history.frame_index == 0 { vk::AccessFlags::empty() } else { vk::AccessFlags::SHADER_WRITE };
        let curr_src_access = if history.frame_index == 0 { vk::AccessFlags::empty() } else { vk::AccessFlags::SHADER_READ };
        let src_stage = if history.frame_index == 0 { vk::PipelineStageFlags::TOP_OF_PIPE } else { vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER };

        let barriers = [
            vk::ImageMemoryBarrier::default().old_layout(prev_color_old_layout).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).src_access_mask(prev_color_src_access).dst_access_mask(vk::AccessFlags::SHADER_READ).image(history.previous_color().handle).subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1)),
            vk::ImageMemoryBarrier::default().old_layout(prev_depth_old_layout).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).src_access_mask(prev_depth_src_access).dst_access_mask(vk::AccessFlags::SHADER_READ).image(history.previous_depth().handle).subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1)),
            vk::ImageMemoryBarrier::default().old_layout(curr_old_layout).new_layout(vk::ImageLayout::GENERAL).src_access_mask(curr_src_access).dst_access_mask(vk::AccessFlags::SHADER_WRITE).image(history.current_color().handle).subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1)),
            vk::ImageMemoryBarrier::default().old_layout(curr_old_layout).new_layout(vk::ImageLayout::GENERAL).src_access_mask(curr_src_access).dst_access_mask(vk::AccessFlags::SHADER_WRITE).image(history.current_depth().handle).subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1)),
        ];
        unsafe { device.cmd_pipeline_barrier(command_buffer, src_stage, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &barriers); }

        let sampler = self.sampler.unwrap();
        let offscreen_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(self.offscreen_images[image_index].view).sampler(sampler);
        let history_color_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(history.previous_color().view).sampler(sampler);
        let motion_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(gbuffer.motion_depth_flags.view).sampler(sampler);
        let depth_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(depth_view).sampler(sampler);
        let history_depth_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(history.previous_depth().view).sampler(sampler);
        let output_color_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(history.current_color().view);
        let output_depth_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(history.current_depth().view);
        let writes = [
            vk::WriteDescriptorSet::default().dst_set(self.taa_descriptor_sets[image_index]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&offscreen_image_info)),
            vk::WriteDescriptorSet::default().dst_set(self.taa_descriptor_sets[image_index]).dst_binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&history_color_image_info)),
            vk::WriteDescriptorSet::default().dst_set(self.taa_descriptor_sets[image_index]).dst_binding(2).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&motion_image_info)),
            vk::WriteDescriptorSet::default().dst_set(self.taa_descriptor_sets[image_index]).dst_binding(3).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&depth_image_info)),
            vk::WriteDescriptorSet::default().dst_set(self.taa_descriptor_sets[image_index]).dst_binding(4).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&history_depth_image_info)),
            vk::WriteDescriptorSet::default().dst_set(self.taa_descriptor_sets[image_index]).dst_binding(5).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&output_color_image_info)),
            vk::WriteDescriptorSet::default().dst_set(self.taa_descriptor_sets[image_index]).dst_binding(6).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&output_depth_image_info)),
        ];
        unsafe { device.update_descriptor_sets(&writes, &[]); }

        pipeline.bind(command_buffer, device);
        unsafe {
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.layout, 0, &[self.taa_descriptor_sets[image_index]], &[]);
            let pc = [0.90f32, 0.003f32, 1.0f32, if reset_history { 1.0f32 } else { 0.0f32 }];
            device.cmd_push_constants(command_buffer, pipeline.layout, vk::ShaderStageFlags::COMPUTE, 0, bytemuck::cast_slice(&pc));
            device.cmd_dispatch(command_buffer, (history.width + 15) / 16, (history.height + 15) / 16, 1);
        }

        let post_resolve_barrier = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ).image(history.current_color().handle)
            .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));
        unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[post_resolve_barrier]); }
    }
}
