use ash::vk;
use super::PostProcessPipeline;

impl PostProcessPipeline {
    pub fn dispatch_depth_resolve(&mut self, device: &ash::Device, command_buffer: vk::CommandBuffer, image_index: usize, width: u32, height: u32, sample_count: vk::SampleCountFlags) {
        let Some(pipeline) = self.depth_resolve_pipeline.as_ref() else { return; };
        let Some(resolved) = self.depth_resolved_images.get(image_index) else { return; };
        let Some(descriptor_set) = self.depth_resolve_sets.get(image_index) else { return; };
        let initialized = self.depth_resolved_initialized.get(image_index).copied().unwrap_or(false);

        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(if initialized { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL } else { vk::ImageLayout::UNDEFINED })
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(if initialized { vk::AccessFlags::SHADER_READ } else { vk::AccessFlags::empty() })
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(resolved.handle)
            .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));

        unsafe {
            device.cmd_pipeline_barrier(command_buffer,
                if initialized { vk::PipelineStageFlags::FRAGMENT_SHADER } else { vk::PipelineStageFlags::TOP_OF_PIPE },
                vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_general]);
            pipeline.bind(command_buffer, device);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.layout, 0, &[*descriptor_set], &[]);
            let sample_count = sample_count.as_raw();
            device.cmd_push_constants(command_buffer, pipeline.layout, vk::ShaderStageFlags::COMPUTE, 0, &sample_count.to_ne_bytes());
            device.cmd_dispatch(command_buffer, (width + 15) / 16, (height + 15) / 16, 1);

            let ready = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(resolved.handle)
                .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));
            device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[ready]);
        }
        if let Some(initialized) = self.depth_resolved_initialized.get_mut(image_index) { *initialized = true; }
    }

    pub(super) fn init_depth_resolve(&mut self, ctx: &crate::core::VulkanContext, allocator: std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>, width: u32, height: u32, image_count: u32, depth_view: vk::ImageView, sampler: vk::Sampler) -> crate::core::error::ReactorResult<()> {
        use crate::graphics::Image;
        let device = ctx.ash_device();
        let bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings).flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post/depth_resolve.spv"))).unwrap();
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(4))?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(image_count).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        let mut resolved_images = Vec::with_capacity(image_count as usize);
        for i in 0..image_count as usize {
            let resolved = Image::new(ctx, allocator.clone(), width, height, vk::Format::R32_SFLOAT, vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED, vk::ImageAspectFlags::COLOR, 1)?;
            let input_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(depth_view).sampler(sampler);
            let output_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(resolved.view);
            let writes = [
                vk::WriteDescriptorSet::default().dst_set(descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&input_info)),
                vk::WriteDescriptorSet::default().dst_set(descriptor_sets[i]).dst_binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&output_info)),
            ];
            unsafe { device.update_descriptor_sets(&writes, &[]); }
            resolved_images.push(resolved);
        }

        self.depth_resolve_pipeline = Some(pipeline);
        self.depth_resolve_descriptor_layout = Some(descriptor_layout);
        self.depth_resolve_descriptor_pool = Some(descriptor_pool);
        self.depth_resolve_sets = descriptor_sets;
        self.depth_resolved_images = resolved_images;
        self.depth_resolved_initialized = vec![false; image_count as usize];
        Ok(())
    }

    pub(super) fn destroy_depth_resolve_resources(&mut self, device: &ash::Device) {
        self.depth_resolve_sets.clear();
        self.depth_resolve_pipeline = None;
        unsafe {
            if let Some(pool) = self.depth_resolve_descriptor_pool.take() { device.destroy_descriptor_pool(pool, None); }
            if let Some(layout) = self.depth_resolve_descriptor_layout.take() { device.destroy_descriptor_set_layout(layout, None); }
        }
        self.depth_resolved_images.clear();
        self.depth_resolved_initialized.clear();
    }
}
