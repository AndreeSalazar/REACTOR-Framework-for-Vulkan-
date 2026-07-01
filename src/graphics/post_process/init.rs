use crate::core::VulkanContext;
use crate::graphics::{Buffer, Image};
use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::Allocator;
use std::sync::{Arc, Mutex};

use super::types::PostProcessSettings;
use super::PostProcessPipeline;

impl PostProcessPipeline {
    pub fn init(
        &mut self,
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        image_count: u32,
        swapchain_format: vk::Format,
        depth_view: vk::ImageView,
        sample_depth: bool,
    ) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();
        self.device = Some(ctx.device.clone());

        let lut_texture = crate::resources::texture::Texture::neutral_lut(ctx, allocator.clone())?;
        self.lut_texture = Some(lut_texture);

        let pp_bindings = [
            vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default().binding(2).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default().binding(3).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default().binding(4).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default().binding(5).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default().binding(6).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default().binding(7).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default().binding(8).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&pp_bindings).flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let push_range = vk::PushConstantRange { stage_flags: vk::ShaderStageFlags::FRAGMENT, offset: 0, size: std::mem::size_of::<PostProcessSettings>() as u32 };
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(std::slice::from_ref(&descriptor_layout)).push_constant_ranges(std::slice::from_ref(&push_range));
        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        let vert_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post_process_vert.spv"))).unwrap();
        let frag_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post_process_frag.spv"))).unwrap();
        let vert_module = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&vert_spv), None)? };
        let frag_module = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&frag_spv), None)? };
        let entry_point = std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap();
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_module).name(entry_point),
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_module).name(entry_point),
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::Viewport { x: 0.0, y: 0.0, width: width as f32, height: height as f32, min_depth: 0.0, max_depth: 1.0 };
        let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width, height } };
        let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewports(std::slice::from_ref(&viewport)).scissors(std::slice::from_ref(&scissor));
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default().cull_mode(vk::CullModeFlags::NONE).front_face(vk::FrontFace::COUNTER_CLOCKWISE).polygon_mode(vk::PolygonMode::FILL).line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default().depth_test_enable(false).depth_write_enable(false);
        let blend_attachment = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::RGBA).blend_enable(false);
        let color_blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&blend_attachment));
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let mut rendering_info = vk::PipelineRenderingCreateInfo::default().color_attachment_formats(std::slice::from_ref(&swapchain_format));
        let pipeline_info = vk::GraphicsPipelineCreateInfo::default().stages(&shader_stages).vertex_input_state(&vertex_input).input_assembly_state(&input_assembly).viewport_state(&viewport_state).rasterization_state(&rasterization).multisample_state(&multisample).depth_stencil_state(&depth_stencil).color_blend_state(&color_blend).dynamic_state(&dynamic_state_info).layout(pipeline_layout).push_next(&mut rendering_info);
        let pipelines = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None).map_err(|(_, e)| e)? };
        let pipeline = pipelines[0];
        unsafe { device.destroy_shader_module(vert_module, None); device.destroy_shader_module(frag_module, None); }

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count * 7),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(image_count),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(image_count).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        self.pipeline = Some(pipeline);
        self.layout = Some(pipeline_layout);
        self.descriptor_layout = Some(descriptor_layout);
        self.descriptor_pool = Some(descriptor_pool);
        self.descriptor_sets = descriptor_sets;

        self.recreate_offscreen_images(ctx, allocator.clone(), width, height, image_count, swapchain_format, depth_view, sample_depth)?;
        Ok(())
    }

    pub fn recreate_offscreen_images(&mut self, ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>, width: u32, height: u32, image_count: u32, format: vk::Format, depth_view: vk::ImageView, sample_depth: bool) -> crate::core::error::ReactorResult<()> {
        let device = ctx.ash_device();

        self.destroy_bloom_resources(device);
        self.destroy_depth_resolve_resources(device);
        self.destroy_gtao_resources(device);
        self.destroy_light_cull_resources(device);
        self.exposure_buffers.clear();
        self.auto_exposure_pipeline = None;
        self.offscreen_images.clear();
        if let Some(sampler) = self.sampler.take() { unsafe { device.destroy_sampler(sampler, None); } }

        let sampler_info = vk::SamplerCreateInfo::default().mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE).address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE).anisotropy_enable(false).max_anisotropy(1.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK).unnormalized_coordinates(false).compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS).mipmap_mode(vk::SamplerMipmapMode::LINEAR);
        let sampler = unsafe { device.create_sampler(&sampler_info, None)? };
        self.sampler = Some(sampler);

        if !sample_depth {
            self.init_depth_resolve(ctx, allocator.clone(), width, height, image_count, depth_view, sampler)?;
        }

        if self.descriptor_sets.len() != image_count as usize {
            if let Some(pool) = self.descriptor_pool.take() { unsafe { device.destroy_descriptor_pool(pool, None); } }
            let descriptor_layout = self.descriptor_layout.ok_or_else(|| crate::core::error::ReactorError::new(crate::core::error::ErrorCode::VulkanDescriptorSet, "post-process descriptor layout is not initialized"))?;
            let pool_sizes = [
                vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(image_count * 9),
                vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(image_count),
            ];
            let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(image_count).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
            let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };
            let layouts = vec![descriptor_layout; image_count as usize];
            let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&layouts);
            self.descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };
            self.descriptor_pool = Some(descriptor_pool);
        }

        self.exposure_buffers.clear();
        for _ in 0..image_count as usize {
            let buf = Buffer::new(ctx, allocator.clone(), 4, vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, MemoryLocation::CpuToGpu)?;
            buf.write(&[1.0f32]);
            self.exposure_buffers.push(buf);
        }

        let ae_spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../../../shaders/post/auto_exposure.spv"))).unwrap();
        let ae_pipeline = crate::compute::ComputePipeline::new(ctx, &ae_spv, &[self.descriptor_layout.unwrap()], Some(20))?;
        self.auto_exposure_pipeline = Some(ae_pipeline);

        self.destroy_taa_resources(device);
        self.init_taa(ctx, image_count)?;

        for i in 0..image_count as usize {
            let img = Image::new(ctx, allocator.clone(), width, height, format, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED, vk::ImageAspectFlags::COLOR, 1)?;
            let image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(img.view).sampler(sampler);
            let depth_or_fallback_view = if sample_depth { depth_view } else { self.depth_resolved_images[i].view };
            let depth_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(depth_or_fallback_view).sampler(sampler);
            let buffer_info = vk::DescriptorBufferInfo::default().buffer(self.exposure_buffers[i].handle).offset(0).range(4);
            let lut_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(self.lut_texture.as_ref().unwrap().view()).sampler(self.lut_texture.as_ref().unwrap().sampler_handle());

            let writes = [
                vk::WriteDescriptorSet::default().dst_set(self.descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&image_info)),
                vk::WriteDescriptorSet::default().dst_set(self.descriptor_sets[i]).dst_binding(2).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&depth_info)),
                vk::WriteDescriptorSet::default().dst_set(self.descriptor_sets[i]).dst_binding(3).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(std::slice::from_ref(&buffer_info)),
                vk::WriteDescriptorSet::default().dst_set(self.descriptor_sets[i]).dst_binding(4).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&lut_info)),
            ];
            unsafe { device.update_descriptor_sets(&writes, &[]); }
            self.offscreen_images.push(img);
        }

        self.init_bloom(ctx, allocator.clone(), width, height, image_count)?;
        self.destroy_fog_resources(device);
        self.init_fog(ctx, allocator.clone(), width, height, image_count)?;
        self.destroy_lens_flare_resources(device);
        self.init_lens_flare(ctx, allocator.clone(), width, height, image_count)?;
        self.init_light_cull(ctx, allocator.clone(), width, height, image_count, 1024)?;

        if !self.fog_output_images.is_empty() {
            let fog_sampler = self.sampler.unwrap();
            for i in 0..image_count as usize {
                let fog_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(self.fog_output_images[i].view).sampler(fog_sampler);
                let fog_write = vk::WriteDescriptorSet::default().dst_set(self.descriptor_sets[i]).dst_binding(6).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&fog_info));
                unsafe { device.update_descriptor_sets(&[fog_write], &[]); }
            }
        }
        {
            let ao_sampler = self.sampler.unwrap();
            for i in 0..image_count as usize {
                let ao_view = if !self.gtao_ao_images.is_empty() { self.gtao_ao_images[i].view } else { self.offscreen_images[i].view };
                let ao_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(ao_view).sampler(ao_sampler);
                let ao_write = vk::WriteDescriptorSet::default().dst_set(self.descriptor_sets[i]).dst_binding(7).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&ao_info));
                unsafe { device.update_descriptor_sets(&[ao_write], &[]); }
            }
        }
        {
            let flare_sampler = self.sampler.unwrap();
            for i in 0..image_count as usize {
                let flare_view = if !self.lens_flare_output_images.is_empty() { self.lens_flare_output_images[i].view } else { self.offscreen_images[i].view };
                let flare_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(flare_view).sampler(flare_sampler);
                let flare_write = vk::WriteDescriptorSet::default().dst_set(self.descriptor_sets[i]).dst_binding(8).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&flare_info));
                unsafe { device.update_descriptor_sets(&[flare_write], &[]); }
            }
        }
        Ok(())
    }
}
