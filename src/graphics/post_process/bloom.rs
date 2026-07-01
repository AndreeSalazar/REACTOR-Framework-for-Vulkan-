use ash::vk;
use super::PostProcessPipeline;

impl PostProcessPipeline {
    pub fn init_bloom(&mut self, ctx: &crate::core::VulkanContext, allocator: std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>, width: u32, height: u32, image_count: u32) -> crate::core::error::ReactorResult<()> {
        use crate::graphics::Image;
        let device = ctx.ash_device();
        let sampler = match self.sampler { Some(s) => s, None => return Ok(()) };

        let bloom_w = (width / 2).max(1);
        let bloom_h = (height / 2).max(1);
        let mip_count = ((bloom_w.min(bloom_h) as f32).log2().floor() as u32).max(1).min(6);

        let bloom_bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let bloom_layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bloom_bindings).flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let bloom_desc_layout = unsafe { device.create_descriptor_set_layout(&bloom_layout_info, None)? };

        let down_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post/bloom_downsample.spv"))).unwrap();
        let up_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post/bloom_upsample.spv"))).unwrap();
        let down_pipeline = crate::compute::ComputePipeline::new(ctx, &down_spv, &[bloom_desc_layout], Some(16))?;
        let up_pipeline = crate::compute::ComputePipeline::new(ctx, &up_spv, &[bloom_desc_layout], Some(12))?;

        let mut bloom_images = Vec::with_capacity(image_count as usize);
        let mut mip_views_sampled: Vec<Vec<vk::ImageView>> = Vec::with_capacity(image_count as usize);
        let mut mip_views_storage: Vec<Vec<vk::ImageView>> = Vec::with_capacity(image_count as usize);

        for _ in 0..image_count {
            let bloom_img = Image::new(ctx, allocator.clone(), bloom_w, bloom_h, vk::Format::R16G16B16A16_SFLOAT, vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE, vk::ImageAspectFlags::COLOR, mip_count)?;
            let mut sampled_views = Vec::with_capacity(mip_count as usize);
            let mut storage_views = Vec::with_capacity(mip_count as usize);
            for mip in 0..mip_count {
                let view_info = vk::ImageViewCreateInfo::default().image(bloom_img.handle).view_type(vk::ImageViewType::TYPE_2D).format(vk::Format::R16G16B16A16_SFLOAT)
                    .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(mip).level_count(1).base_array_layer(0).layer_count(1));
                sampled_views.push(unsafe { device.create_image_view(&view_info, None)? });
                storage_views.push(unsafe { device.create_image_view(&view_info, None)? });
            }
            mip_views_sampled.push(sampled_views);
            mip_views_storage.push(storage_views);
            bloom_images.push(bloom_img);
        }

        let total_sets = (2 * mip_count - 1) * image_count;
        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(total_sets),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(total_sets),
        ];
        let bloom_pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(total_sets).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let bloom_desc_pool = unsafe { device.create_descriptor_pool(&bloom_pool_info, None)? };

        let mut downsample_sets: Vec<Vec<vk::DescriptorSet>> = Vec::with_capacity(image_count as usize);
        let mut upsample_sets: Vec<Vec<vk::DescriptorSet>> = Vec::with_capacity(image_count as usize);

        for img_idx in 0..image_count as usize {
            let down_layouts = vec![bloom_desc_layout; mip_count as usize];
            let down_alloc = vk::DescriptorSetAllocateInfo::default().descriptor_pool(bloom_desc_pool).set_layouts(&down_layouts);
            let down_ds = unsafe { device.allocate_descriptor_sets(&down_alloc)? };
            for mip in 0..mip_count as usize {
                let input_info = if mip == 0 {
                    vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(self.offscreen_images[img_idx].view).sampler(sampler)
                } else {
                    vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(mip_views_sampled[img_idx][mip - 1]).sampler(sampler)
                };
                let output_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(mip_views_storage[img_idx][mip]);
                let writes = [
                    vk::WriteDescriptorSet::default().dst_set(down_ds[mip]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&input_info)),
                    vk::WriteDescriptorSet::default().dst_set(down_ds[mip]).dst_binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&output_info)),
                ];
                unsafe { device.update_descriptor_sets(&writes, &[]); }
            }
            downsample_sets.push(down_ds);

            if mip_count > 1 {
                let up_count = (mip_count - 1) as usize;
                let up_layouts = vec![bloom_desc_layout; up_count];
                let up_alloc = vk::DescriptorSetAllocateInfo::default().descriptor_pool(bloom_desc_pool).set_layouts(&up_layouts);
                let up_ds = unsafe { device.allocate_descriptor_sets(&up_alloc)? };
                for pass in 0..up_count {
                    let src_mip = mip_count as usize - 1 - pass;
                    let dst_mip = src_mip - 1;
                    let input_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(mip_views_sampled[img_idx][src_mip]).sampler(sampler);
                    let output_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(mip_views_storage[img_idx][dst_mip]);
                    let writes = [
                        vk::WriteDescriptorSet::default().dst_set(up_ds[pass]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&input_info)),
                        vk::WriteDescriptorSet::default().dst_set(up_ds[pass]).dst_binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&output_info)),
                    ];
                    unsafe { device.update_descriptor_sets(&writes, &[]); }
                }
                upsample_sets.push(up_ds);
            } else {
                upsample_sets.push(Vec::new());
            }
        }

        for img_idx in 0..image_count as usize {
            let bloom_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(mip_views_sampled[img_idx][0]).sampler(sampler);
            let write = vk::WriteDescriptorSet::default().dst_set(self.descriptor_sets[img_idx]).dst_binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&bloom_info));
            unsafe { device.update_descriptor_sets(&[write], &[]); }
        }

        self.bloom_downsample_pipeline = Some(down_pipeline);
        self.bloom_upsample_pipeline = Some(up_pipeline);
        self.bloom_descriptor_layout = Some(bloom_desc_layout);
        self.bloom_descriptor_pool = Some(bloom_desc_pool);
        self.bloom_images = bloom_images;
        self.bloom_mip_views_sampled = mip_views_sampled;
        self.bloom_mip_views_storage = mip_views_storage;
        self.bloom_downsample_sets = downsample_sets;
        self.bloom_upsample_sets = upsample_sets;
        Ok(())
    }

    pub(super) fn destroy_bloom_resources(&mut self, device: &ash::Device) {
        self.bloom_downsample_sets.clear();
        self.bloom_upsample_sets.clear();
        self.bloom_downsample_pipeline = None;
        self.bloom_upsample_pipeline = None;
        unsafe {
            for views in self.bloom_mip_views_sampled.drain(..) { for view in views { device.destroy_image_view(view, None); } }
            for views in self.bloom_mip_views_storage.drain(..) { for view in views { device.destroy_image_view(view, None); } }
            if let Some(pool) = self.bloom_descriptor_pool.take() { device.destroy_descriptor_pool(pool, None); }
            if let Some(layout) = self.bloom_descriptor_layout.take() { device.destroy_descriptor_set_layout(layout, None); }
        }
        self.bloom_images.clear();
    }

    pub fn dispatch_bloom(&self, device: &ash::Device, command_buffer: vk::CommandBuffer, image_index: usize, scene_width: u32, scene_height: u32) {
        let mip_count = match self.bloom_mip_views_sampled.get(image_index) { Some(v) if !v.is_empty() => v.len() as u32, _ => return };
        let bloom_img = &self.bloom_images[image_index];

        let to_general = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::SHADER_WRITE).image(bloom_img.handle)
            .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(mip_count).base_array_layer(0).layer_count(1));
        unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_general]); }

        let down = self.bloom_downsample_pipeline.as_ref().unwrap();
        down.bind(command_buffer, device);
        let mut out_w = (scene_width / 2).max(1);
        let mut out_h = (scene_height / 2).max(1);

        for mip in 0..mip_count as usize {
            unsafe { device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, down.layout, 0, &[self.bloom_downsample_sets[image_index][mip]], &[]); }
            let input_w = if mip == 0 { scene_width } else { out_w * 2 };
            let input_h = if mip == 0 { scene_height } else { out_h * 2 };
            let texel_x = 1.0f32 / input_w.max(1) as f32;
            let texel_y = 1.0f32 / input_h.max(1) as f32;
            let threshold = self.settings.bloom_threshold;
            let mut push = [0u8; 16];
            push[0..4].copy_from_slice(&texel_x.to_ne_bytes());
            push[4..8].copy_from_slice(&texel_y.to_ne_bytes());
            push[8..12].copy_from_slice(&(mip as i32).to_ne_bytes());
            push[12..16].copy_from_slice(&threshold.to_ne_bytes());
            unsafe {
                device.cmd_push_constants(command_buffer, down.layout, vk::ShaderStageFlags::COMPUTE, 0, &push);
                device.cmd_dispatch(command_buffer, (out_w + 15) / 16, (out_h + 15) / 16, 1);
            }
            let barrier = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::GENERAL)
                .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ).image(bloom_img.handle)
                .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(mip as u32).level_count(1).base_array_layer(0).layer_count(1));
            unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[barrier]); }
            out_w = (out_w / 2).max(1);
            out_h = (out_h / 2).max(1);
        }

        if mip_count > 1 {
            let up = self.bloom_upsample_pipeline.as_ref().unwrap();
            up.bind(command_buffer, device);
            let upsample_count = (mip_count - 1) as usize;
            for pass in 0..upsample_count {
                let dst_mip = mip_count as usize - 2 - pass;
                unsafe { device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, up.layout, 0, &[self.bloom_upsample_sets[image_index][pass]], &[]); }
                let src_mip = dst_mip + 1;
                let src_w = ((scene_width / 2) >> src_mip).max(1);
                let src_h = ((scene_height / 2) >> src_mip).max(1);
                let texel_x = 1.0f32 / src_w as f32;
                let texel_y = 1.0f32 / src_h as f32;
                let mut push = [0u8; 12];
                push[0..4].copy_from_slice(&texel_x.to_ne_bytes());
                push[4..8].copy_from_slice(&texel_y.to_ne_bytes());
                push[8..12].copy_from_slice(&1.0f32.to_ne_bytes());
                let dst_w = ((scene_width / 2) >> dst_mip).max(1);
                let dst_h = ((scene_height / 2) >> dst_mip).max(1);
                unsafe {
                    device.cmd_push_constants(command_buffer, up.layout, vk::ShaderStageFlags::COMPUTE, 0, &push);
                    device.cmd_dispatch(command_buffer, (dst_w + 15) / 16, (dst_h + 15) / 16, 1);
                }
                let barrier = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::GENERAL)
                    .src_access_mask(vk::AccessFlags::SHADER_WRITE | vk::AccessFlags::SHADER_READ).dst_access_mask(vk::AccessFlags::SHADER_READ).image(bloom_img.handle)
                    .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(dst_mip as u32).level_count(1).base_array_layer(0).layer_count(1));
                unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[barrier]); }
            }
        }

        let final_barrier = vk::ImageMemoryBarrier::default().old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ).image(bloom_img.handle)
            .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1));
        unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[final_barrier]); }
    }
}
