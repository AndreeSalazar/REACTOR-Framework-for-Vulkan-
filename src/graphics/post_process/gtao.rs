use ash::vk;
use super::PostProcessPipeline;

impl PostProcessPipeline {
    pub fn init_gtao(&mut self, ctx: &crate::core::VulkanContext, allocator: std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>, width: u32, height: u32, image_count: u32, depth_view: vk::ImageView, gbuffer_normal_view: vk::ImageView) -> crate::core::error::ReactorResult<()> {
        use crate::graphics::Image;
        let device = ctx.ash_device();
        let bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(2).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings).flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };
        self.gtao_descriptor_layout = Some(descriptor_layout);

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post/gtao.spv"))).unwrap();
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(32))?;
        self.gtao_pipeline = Some(pipeline);

        let sampler = self.sampler.unwrap();
        self.gtao_ao_images.clear();
        self.gtao_initialized.clear();
        for _ in 0..image_count as usize {
            let img = Image::new(ctx, allocator.clone(), width, height, vk::Format::R16_SFLOAT, vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE, vk::ImageAspectFlags::COLOR, 1)?;
            self.gtao_ao_images.push(img);
            self.gtao_initialized.push(false);
        }

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count * 2),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(image_count).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };
        self.gtao_descriptor_pool = Some(descriptor_pool);

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);
        self.gtao_descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        for i in 0..image_count as usize {
            let depth_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(depth_view).sampler(sampler);
            let normal_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(gbuffer_normal_view).sampler(sampler);
            let ao_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(self.gtao_ao_images[i].view);
            let writes = [
                vk::WriteDescriptorSet::default().dst_set(self.gtao_descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&depth_image_info)),
                vk::WriteDescriptorSet::default().dst_set(self.gtao_descriptor_sets[i]).dst_binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&normal_image_info)),
                vk::WriteDescriptorSet::default().dst_set(self.gtao_descriptor_sets[i]).dst_binding(2).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&ao_image_info)),
            ];
            unsafe { device.update_descriptor_sets(&writes, &[]); }
        }
        Ok(())
    }

    pub(super) fn destroy_gtao_resources(&mut self, device: &ash::Device) {
        self.gtao_descriptor_sets.clear();
        self.gtao_pipeline = None;
        self.gtao_ao_images.clear();
        self.gtao_initialized.clear();
        unsafe {
            if let Some(pool) = self.gtao_descriptor_pool.take() { device.destroy_descriptor_pool(pool, None); }
            if let Some(layout) = self.gtao_descriptor_layout.take() { device.destroy_descriptor_set_layout(layout, None); }
        }
    }

    pub fn dispatch_gtao(&self, device: &ash::Device, command_buffer: vk::CommandBuffer, image_index: usize, width: u32, height: u32, proj_x: f32, proj_y: f32, near: f32, far: f32, frame_index: f32) {
        let Some(pipeline) = self.gtao_pipeline.as_ref() else { return; };
        let Some(descriptor_set) = self.gtao_descriptor_sets.get(image_index) else { return; };
        let Some(ao_image) = self.gtao_ao_images.get(image_index) else { return; };
        let initialized = self.gtao_initialized.get(image_index).copied().unwrap_or(false);

        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(if initialized { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL } else { vk::ImageLayout::UNDEFINED })
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(if initialized { vk::AccessFlags::SHADER_READ } else { vk::AccessFlags::empty() })
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE).image(ao_image.handle)
            .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));

        unsafe {
            device.cmd_pipeline_barrier(command_buffer, if initialized { vk::PipelineStageFlags::FRAGMENT_SHADER } else { vk::PipelineStageFlags::TOP_OF_PIPE }, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_general]);
            pipeline.bind(command_buffer, device);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.layout, 0, &[*descriptor_set], &[]);

            let push_bytes = [proj_x.to_ne_bytes(), proj_y.to_ne_bytes(), near.to_ne_bytes(), far.to_ne_bytes(), (width as f32 * 0.05).to_ne_bytes(), 0.5f32.to_ne_bytes(), 1.5f32.to_ne_bytes(), frame_index.to_ne_bytes()].concat();
            device.cmd_push_constants(command_buffer, pipeline.layout, vk::ShaderStageFlags::COMPUTE, 0, &push_bytes);
            device.cmd_dispatch(command_buffer, (width + 7) / 8, (height + 7) / 8, 1);

            let ready = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ).image(ao_image.handle).subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));
            device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[ready]);
        }
    }
}
