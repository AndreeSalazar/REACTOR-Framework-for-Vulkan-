use ash::vk;
use super::PostProcessPipeline;

impl PostProcessPipeline {
    pub(super) fn init_fog(&mut self, ctx: &crate::core::VulkanContext, allocator: std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>, width: u32, height: u32, image_count: u32) -> crate::core::error::ReactorResult<()> {
        use crate::graphics::Image;
        let device = ctx.ash_device();
        let bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(2).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings).flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post/volumetric_fog.spv"))).unwrap();
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(156))?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count * 2),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(image_count).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        self.fog_output_images.clear();
        for _ in 0..image_count as usize {
            let img = Image::new(ctx, allocator.clone(), width, height, vk::Format::R16G16B16A16_SFLOAT, vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED, vk::ImageAspectFlags::COLOR, 1)?;
            self.fog_output_images.push(img);
        }

        self.fog_pipeline = Some(pipeline);
        self.fog_descriptor_layout = Some(descriptor_layout);
        self.fog_descriptor_pool = Some(descriptor_pool);
        self.fog_descriptor_sets = descriptor_sets;
        Ok(())
    }

    pub(super) fn destroy_fog_resources(&mut self, device: &ash::Device) {
        self.fog_descriptor_sets.clear();
        self.fog_pipeline = None;
        self.fog_output_images.clear();
        unsafe {
            if let Some(pool) = self.fog_descriptor_pool.take() { device.destroy_descriptor_pool(pool, None); }
            if let Some(layout) = self.fog_descriptor_layout.take() { device.destroy_descriptor_set_layout(layout, None); }
        }
    }

    pub fn dispatch_volumetric_fog(&mut self, device: &ash::Device, command_buffer: vk::CommandBuffer, image_index: usize, camera_view: glam::Mat4, camera_proj: glam::Mat4, camera_pos: glam::Vec3, sun_direction: glam::Vec3, sun_color: glam::Vec3, near: f32, far: f32, time: f32) {
        let Some(pipeline) = self.fog_pipeline.as_ref() else { return; };
        let Some(descriptor_set) = self.fog_descriptor_sets.get(image_index) else { return; };
        let sampler = match self.sampler { Some(s) => s, None => return };
        let width = self.fog_output_images.first().map(|i| i.extent.width).unwrap_or(1);
        let height = self.fog_output_images.first().map(|i| i.extent.height).unwrap_or(1);

        let to_general = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::SHADER_WRITE).image(self.fog_output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));
        unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_general]); }

        let depth_view = self.depth_resolved_images.first().map(|i| i.view).unwrap_or(self.offscreen_images.first().map(|i| i.view).unwrap_or(vk::ImageView::null()));
        let depth_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(depth_view).sampler(sampler);
        let scene_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(self.offscreen_images[image_index].view).sampler(sampler);
        let fog_output_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(self.fog_output_images[image_index].view);
        let writes = [
            vk::WriteDescriptorSet::default().dst_set(*descriptor_set).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&depth_image_info)),
            vk::WriteDescriptorSet::default().dst_set(*descriptor_set).dst_binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&scene_image_info)),
            vk::WriteDescriptorSet::default().dst_set(*descriptor_set).dst_binding(2).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&fog_output_image_info)),
        ];
        unsafe { device.update_descriptor_sets(&writes, &[]); }

        pipeline.bind(command_buffer, device);
        unsafe { device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.layout, 0, &[*descriptor_set], &[]); }

        let inv_view_proj = (camera_proj * camera_view).inverse();
        let light_dir_view = camera_view.transform_vector3(-sun_direction.normalize()).normalize();

        let mut push_bytes = [0u8; 156];
        let mut offset = 0;
        let wf = |push: &mut [u8; 156], o: &mut usize, v: f32| { push[*o..*o+4].copy_from_slice(&v.to_ne_bytes()); *o += 4; };
        let wv4 = |push: &mut [u8; 156], o: &mut usize, v: glam::Vec4| { wf(push, o, v.x); wf(push, o, v.y); wf(push, o, v.z); wf(push, o, v.w); };
        let wm4 = |push: &mut [u8; 156], o: &mut usize, m: glam::Mat4| { for c in m.to_cols_array() { wf(push, o, c); } };

        wm4(&mut push_bytes, &mut offset, inv_view_proj);
        wv4(&mut push_bytes, &mut offset, glam::Vec4::new(camera_pos.x, camera_pos.y, camera_pos.z, 0.0));
        wv4(&mut push_bytes, &mut offset, glam::Vec4::new(light_dir_view.x, light_dir_view.y, light_dir_view.z, 1.0));
        wv4(&mut push_bytes, &mut offset, glam::Vec4::new(sun_color.x, sun_color.y, sun_color.z, 1.0));
        wf(&mut push_bytes, &mut offset, width as f32); wf(&mut push_bytes, &mut offset, height as f32);
        wf(&mut push_bytes, &mut offset, self.settings.fog_density); wf(&mut push_bytes, &mut offset, self.settings.fog_scatter);
        wf(&mut push_bytes, &mut offset, time); wf(&mut push_bytes, &mut offset, near); wf(&mut push_bytes, &mut offset, far);
        wf(&mut push_bytes, &mut offset, 0.35); wf(&mut push_bytes, &mut offset, -1.0);
        push_bytes[offset..offset+4].copy_from_slice(&48u32.to_ne_bytes()); offset += 4;
        push_bytes[offset..offset+4].copy_from_slice(&0u32.to_ne_bytes());

        unsafe {
            device.cmd_push_constants(command_buffer, pipeline.layout, vk::ShaderStageFlags::COMPUTE, 0, &push_bytes);
            device.cmd_dispatch(command_buffer, (width + 7) / 8, (height + 7) / 8, 1);
        }

        let to_read = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ).image(self.fog_output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));
        unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_read]); }
    }
}
