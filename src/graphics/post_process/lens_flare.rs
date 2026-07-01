use ash::vk;
use super::PostProcessPipeline;

impl PostProcessPipeline {
    pub(super) fn init_lens_flare(&mut self, ctx: &crate::core::VulkanContext, allocator: std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>, width: u32, height: u32, image_count: u32) -> crate::core::error::ReactorResult<()> {
        use crate::graphics::Image;
        let device = ctx.ash_device();
        let bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings).flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post/lens_flare.spv"))).unwrap();
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(48))?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(image_count).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        self.lens_flare_output_images.clear();
        for _ in 0..image_count as usize {
            let img = Image::new(ctx, allocator.clone(), width, height, vk::Format::R16G16B16A16_SFLOAT, vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED, vk::ImageAspectFlags::COLOR, 1)?;
            self.lens_flare_output_images.push(img);
        }

        self.lens_flare_pipeline = Some(pipeline);
        self.lens_flare_descriptor_layout = Some(descriptor_layout);
        self.lens_flare_descriptor_pool = Some(descriptor_pool);
        self.lens_flare_descriptor_sets = descriptor_sets;
        Ok(())
    }

    pub(super) fn destroy_lens_flare_resources(&mut self, device: &ash::Device) {
        self.lens_flare_descriptor_sets.clear();
        self.lens_flare_pipeline = None;
        self.lens_flare_output_images.clear();
        unsafe {
            if let Some(pool) = self.lens_flare_descriptor_pool.take() { device.destroy_descriptor_pool(pool, None); }
            if let Some(layout) = self.lens_flare_descriptor_layout.take() { device.destroy_descriptor_set_layout(layout, None); }
        }
    }

    pub fn dispatch_lens_flare(&mut self, device: &ash::Device, command_buffer: vk::CommandBuffer, image_index: usize, width: u32, height: u32, time: f32) {
        let Some(pipeline) = self.lens_flare_pipeline.as_ref() else { return; };
        let Some(descriptor_set) = self.lens_flare_descriptor_sets.get(image_index) else { return; };
        let sampler = match self.sampler { Some(s) => s, None => return };

        let bloom_mip = self.bloom_mip_views_sampled.get(image_index).and_then(|v| v.get(2)).copied()
            .unwrap_or_else(|| self.offscreen_images.get(image_index).map(|i| i.view).unwrap_or(vk::ImageView::null()));

        let to_general = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::SHADER_WRITE).image(self.lens_flare_output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));
        unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_general]); }

        let bloom_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(bloom_mip).sampler(sampler);
        let flare_output_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(self.lens_flare_output_images[image_index].view);
        let writes = [
            vk::WriteDescriptorSet::default().dst_set(*descriptor_set).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&bloom_image_info)),
            vk::WriteDescriptorSet::default().dst_set(*descriptor_set).dst_binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&flare_output_image_info)),
        ];
        unsafe { device.update_descriptor_sets(&writes, &[]); }

        pipeline.bind(command_buffer, device);
        unsafe { device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.layout, 0, &[*descriptor_set], &[]); }

        let mut push_bytes = [0u8; 48];
        push_bytes[0..4].copy_from_slice(&(width as f32).to_ne_bytes());
        push_bytes[4..8].copy_from_slice(&(height as f32).to_ne_bytes());
        push_bytes[8..12].copy_from_slice(&0.37f32.to_ne_bytes());
        push_bytes[12..16].copy_from_slice(&6.0f32.to_ne_bytes());
        push_bytes[16..20].copy_from_slice(&0.85f32.to_ne_bytes());
        push_bytes[20..24].copy_from_slice(&0.45f32.to_ne_bytes());
        push_bytes[24..28].copy_from_slice(&0.06f32.to_ne_bytes());
        push_bytes[28..32].copy_from_slice(&0.02f32.to_ne_bytes());
        push_bytes[32..36].copy_from_slice(&0.5f32.to_ne_bytes());
        push_bytes[36..40].copy_from_slice(&self.settings.flare_intensity.to_ne_bytes());
        push_bytes[40..44].copy_from_slice(&time.to_ne_bytes());
        push_bytes[44..48].copy_from_slice(&0.0f32.to_ne_bytes());

        unsafe {
            device.cmd_push_constants(command_buffer, pipeline.layout, vk::ShaderStageFlags::COMPUTE, 0, &push_bytes);
            device.cmd_dispatch(command_buffer, (width + 15) / 16, (height + 15) / 16, 1);
        }

        let to_read = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ).image(self.lens_flare_output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));
        unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_read]); }
    }
}
