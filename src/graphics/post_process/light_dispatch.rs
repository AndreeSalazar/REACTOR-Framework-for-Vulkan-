use ash::vk;
use gpu_allocator::MemoryLocation;
use super::PostProcessPipeline;

impl PostProcessPipeline {
    pub fn init_light_cull(&mut self, ctx: &crate::core::VulkanContext, allocator: std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>, width: u32, height: u32, image_count: u32, max_lights: u32) -> crate::core::error::ReactorResult<()> {
        use crate::graphics::Buffer;
        let device = ctx.ash_device();
        let tile_count_x = (width + 15) / 16;
        let tile_count_y = (height + 15) / 16;
        let tile_count = (tile_count_x * tile_count_y) as usize;

        let bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(2).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(3).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(4).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings).flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };
        self.light_cull_descriptor_layout = Some(descriptor_layout);

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/compute/light_cull.spv"))).unwrap();
        let pipeline = crate::compute::ComputePipeline::new(ctx, &spv, &[descriptor_layout], Some(224))?;
        self.light_cull_pipeline = Some(pipeline);

        let light_buffer = Buffer::new(ctx, allocator.clone(), (max_lights as usize * 32) as u64, vk::BufferUsageFlags::STORAGE_BUFFER, MemoryLocation::CpuToGpu)?;
        self.light_cull_light_buffer = Some(light_buffer);
        let tile_buffer = Buffer::new(ctx, allocator.clone(), (tile_count * 8) as u64, vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, MemoryLocation::GpuOnly)?;
        self.light_cull_tile_buffer = Some(tile_buffer);
        let index_buffer = Buffer::new(ctx, allocator.clone(), (tile_count * 256 * 4) as u64, vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, MemoryLocation::GpuOnly)?;
        self.light_cull_index_buffer = Some(index_buffer);
        let atomic_buffer = Buffer::new(ctx, allocator.clone(), 4, vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, MemoryLocation::GpuOnly)?;
        self.light_cull_atomic_buffer = Some(atomic_buffer);

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(image_count * 4),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(image_count).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };
        self.light_cull_descriptor_pool = Some(descriptor_pool);

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);
        self.light_cull_descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        let light_buf = self.light_cull_light_buffer.as_ref().unwrap().handle;
        let tile_buf = self.light_cull_tile_buffer.as_ref().unwrap().handle;
        let index_buf = self.light_cull_index_buffer.as_ref().unwrap().handle;
        let atomic_buf = self.light_cull_atomic_buffer.as_ref().unwrap().handle;

        let offscreen_view = self.offscreen_images.first().map(|i| i.view).unwrap_or(vk::ImageView::null());
        let lc_sampler = self.sampler.unwrap_or(vk::Sampler::null());
        for i in 0..image_count as usize {
            let light_buffer_info = vk::DescriptorBufferInfo::default().buffer(light_buf).offset(0).range(vk::WHOLE_SIZE);
            let tile_buffer_info = vk::DescriptorBufferInfo::default().buffer(tile_buf).offset(0).range(vk::WHOLE_SIZE);
            let index_buffer_info = vk::DescriptorBufferInfo::default().buffer(index_buf).offset(0).range(vk::WHOLE_SIZE);
            let atomic_buffer_info = vk::DescriptorBufferInfo::default().buffer(atomic_buf).offset(0).range(4);
            let depth_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(offscreen_view).sampler(lc_sampler);
            let writes = [
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(std::slice::from_ref(&light_buffer_info)),
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(1).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(std::slice::from_ref(&tile_buffer_info)),
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(2).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(std::slice::from_ref(&index_buffer_info)),
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(3).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(std::slice::from_ref(&atomic_buffer_info)),
                vk::WriteDescriptorSet::default().dst_set(self.light_cull_descriptor_sets[i]).dst_binding(4).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&depth_image_info)),
            ];
            unsafe { device.update_descriptor_sets(&writes, &[]); }
        }
        Ok(())
    }

    pub(super) fn destroy_light_cull_resources(&mut self, device: &ash::Device) {
        self.light_cull_descriptor_sets.clear();
        self.light_cull_pipeline = None;
        unsafe {
            if let Some(mut buf) = self.light_cull_tile_buffer.take() { buf.destroy(); }
            if let Some(mut buf) = self.light_cull_index_buffer.take() { buf.destroy(); }
            if let Some(mut buf) = self.light_cull_atomic_buffer.take() { buf.destroy(); }
            if let Some(mut buf) = self.light_cull_light_buffer.take() { buf.destroy(); }
            if let Some(pool) = self.light_cull_descriptor_pool.take() { device.destroy_descriptor_pool(pool, None); }
            if let Some(layout) = self.light_cull_descriptor_layout.take() { device.destroy_descriptor_set_layout(layout, None); }
        }
    }

    pub fn dispatch_light_cull(&self, device: &ash::Device, command_buffer: vk::CommandBuffer, image_index: usize, width: u32, height: u32, view: glam::Mat4, projection: glam::Mat4, inv_projection: glam::Mat4, light_count: u32, depth_view: vk::ImageView) {
        let light_count = light_count.min(1024);
        self.dispatch_light_cull_inner(device, command_buffer, image_index, width, height, view, projection, inv_projection, light_count, depth_view);
    }

    pub fn update_lights(&mut self, lights: &[crate::graphics::post_process::PointLightGpu]) {
        if let Some(buf) = self.light_cull_light_buffer.as_mut() {
            if lights.is_empty() { return; }
            let max_count = (buf.size as usize) / std::mem::size_of::<crate::graphics::post_process::PointLightGpu>();
            let to_write = lights.len().min(max_count);
            buf.write(&lights[..to_write]);
        }
    }

    fn dispatch_light_cull_inner(&self, device: &ash::Device, command_buffer: vk::CommandBuffer, image_index: usize, width: u32, height: u32, view: glam::Mat4, projection: glam::Mat4, inv_projection: glam::Mat4, light_count: u32, depth_view: vk::ImageView) {
        let Some(pipeline) = self.light_cull_pipeline.as_ref() else { return; };
        let Some(descriptor_set) = self.light_cull_descriptor_sets.get(image_index) else { return; };
        let sampler = match self.sampler { Some(s) => s, None => return };
        let tile_count_x = (width + 15) / 16;
        let tile_count_y = (height + 15) / 16;

        unsafe {
            pipeline.bind(command_buffer, device);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.layout, 0, &[*descriptor_set], &[]);

            let mut pb = [0u8; 224];
            let mut o = 0usize;
            for &v in &view.to_cols_array() { pb[o..o+4].copy_from_slice(&v.to_ne_bytes()); o += 4; }
            for &v in &projection.to_cols_array() { pb[o..o+4].copy_from_slice(&v.to_ne_bytes()); o += 4; }
            for &v in &inv_projection.to_cols_array() { pb[o..o+4].copy_from_slice(&v.to_ne_bytes()); o += 4; }
            pb[o..o+4].copy_from_slice(&light_count.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&tile_count_x.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&tile_count_y.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&width.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&height.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&0.1f32.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&1000f32.to_ne_bytes()); o += 4;
            pb[o..o+4].copy_from_slice(&256u32.to_ne_bytes());

            device.cmd_push_constants(command_buffer, pipeline.layout, vk::ShaderStageFlags::COMPUTE, 0, &pb);
            device.cmd_dispatch(command_buffer, tile_count_x, tile_count_y, 1);
        }

        let barriers = [
            vk::BufferMemoryBarrier::default().src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .buffer(self.light_cull_tile_buffer.as_ref().unwrap().handle).offset(0).size(vk::WHOLE_SIZE),
            vk::BufferMemoryBarrier::default().src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .buffer(self.light_cull_index_buffer.as_ref().unwrap().handle).offset(0).size(vk::WHOLE_SIZE),
        ];
        unsafe {
            device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &barriers, &[]);
        }
    }
}
