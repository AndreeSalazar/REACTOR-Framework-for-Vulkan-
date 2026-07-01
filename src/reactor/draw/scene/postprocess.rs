use crate::graphics::post_process::PostProcessEffect;
use crate::reactor::Reactor;
use crate::systems::scene::Scene;
use ash::vk;

impl Reactor {
    pub(super) fn update_post_descriptors(&mut self, image_index: u32) -> (bool, bool) {
        let use_post_process = self.post_process.enabled && !self.post_process.offscreen_images.is_empty();
        let taa_enabled = self.post_process.enabled
            && self.post_process.settings.is_effect_enabled(PostProcessEffect::TAA)
            && self.temporal_history.is_some()
            && self.gbuffer.is_some();

        if use_post_process {
            let sampler = self.post_process.sampler.unwrap();
            let color_view = if taa_enabled {
                self.temporal_history.as_ref().unwrap().current_color().view
            } else {
                self.post_process.offscreen_images[image_index as usize].view
            };
            let motion_view = self.gbuffer.as_ref().map(|gb| gb.motion_depth_flags.view).unwrap_or(color_view);

            let image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(color_view).sampler(sampler);
            let motion_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(motion_view).sampler(sampler);

            let writes = [
                vk::WriteDescriptorSet::default().dst_set(self.post_process.descriptor_sets[image_index as usize])
                    .dst_binding(0).dst_array_element(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&image_info)),
                vk::WriteDescriptorSet::default().dst_set(self.post_process.descriptor_sets[image_index as usize])
                    .dst_binding(5).dst_array_element(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&motion_info)),
            ];
            unsafe { self.context.device.update_descriptor_sets(&writes, &[]); }
        }

        (use_post_process, taa_enabled)
    }

    pub(super) fn render_post_process(
        &mut self,
        scene: &Scene,
        command_buffer: vk::CommandBuffer,
        image_index: u32,
        use_post_process: bool,
        taa_enabled: bool,
        local_vp: &glam::Mat4,
        swapchain_view: vk::ImageView,
        swapchain_image: vk::Image,
    ) {
        if !use_post_process { return; }

        let offscreen_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(self.post_process.offscreen_images[image_index as usize].handle)
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });

        let depth_old_layout = if self.decals.is_empty() { vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL } else { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL };
        let depth_read_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(depth_old_layout).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(self.depth_image.unwrap())
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });

        unsafe {
            self.context.device.cmd_pipeline_barrier(command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[], &[], &[offscreen_barrier, depth_read_barrier]);
        }

        if self.msaa_samples != vk::SampleCountFlags::TYPE_1 {
            self.post_process.dispatch_depth_resolve(self.context.ash_device(), command_buffer, image_index as usize,
                self.swapchain.extent.width, self.swapchain.extent.height, self.msaa_samples);
        }

        if let Some(hiz) = self.hiz_pyramid.as_ref() {
            let src_depth_view = if self.msaa_samples == vk::SampleCountFlags::TYPE_1 { self.depth_image_view.unwrap() } else { self.post_process.depth_resolved_images[image_index as usize].view };
            if let Some(sampler) = self.post_process.sampler {
                hiz.build(self.context.ash_device(), command_buffer, image_index as usize, src_depth_view, sampler);
            }
        }

        if let (Some(ssgi), Some(sampler), Some(hiz)) = (self.ssgi_hiz.as_mut(), self.post_process.sampler, self.hiz_pyramid.as_ref()) {
            if let Some(gb) = self.gbuffer.as_ref() {
                let color_view = self.post_process.offscreen_images[image_index as usize].view;
                let depth_view = if self.msaa_samples == vk::SampleCountFlags::TYPE_1 { self.depth_image_view.unwrap() } else { self.post_process.depth_resolved_images[image_index as usize].view };
                let vp = self.camera_proj * self.camera_view;
                ssgi.dispatch(self.context.ash_device(), command_buffer, image_index as usize,
                    self.swapchain.extent.width, self.swapchain.extent.height, vp, vp.inverse(), self.camera_pos,
                    color_view, depth_view, gb.normal_material.view, hiz.mip_view(image_index as usize, 0), sampler,
                    self.post_process.settings.ssgi_intensity);
            }
        }

        if self.post_process.gtao_pipeline.is_some() {
            if self.post_process.gtao_descriptor_sets.is_empty() {
                if let (Some(depth_view), Some(gb)) = (self.depth_image_view, self.gbuffer.as_ref()) {
                    let _ = self.post_process.init_gtao(&self.context, self.allocator.clone(),
                        self.swapchain.extent.width, self.swapchain.extent.height, self.swapchain.images.len() as u32,
                        depth_view, gb.normal_material.view);
                }
            }
            let depth_view = if self.msaa_samples == vk::SampleCountFlags::TYPE_1 { self.depth_image_view.unwrap() } else { self.post_process.depth_resolved_images[image_index as usize].view };
            self.post_process.dispatch_gtao(self.context.ash_device(), command_buffer, image_index as usize,
                self.swapchain.extent.width, self.swapchain.extent.height,
                self.camera_proj.x_axis.x, self.camera_proj.y_axis.y, self.camera_near, self.camera_far, self.post_process.last_time);
        }

        if self.post_process.light_cull_pipeline.is_some() {
            let depth_view = if self.msaa_samples == vk::SampleCountFlags::TYPE_1 { self.depth_image_view.unwrap() } else { self.post_process.depth_resolved_images[image_index as usize].view };
            let mut gpu_lights: Vec<crate::graphics::post_process::PointLightGpu> = Vec::with_capacity(scene.lights.len());
            crate::graphics::post_process::lights_to_gpu_buffer(&scene.lights, &mut gpu_lights);
            self.post_process.update_lights(&gpu_lights);
            self.post_process.dispatch_light_cull(self.context.ash_device(), command_buffer, image_index as usize,
                self.swapchain.extent.width, self.swapchain.extent.height, self.camera_view, self.camera_proj,
                self.camera_proj.inverse(), gpu_lights.len() as u32, depth_view);
        }

        if self.post_process.bloom_downsample_pipeline.is_some() {
            self.post_process.dispatch_bloom(self.context.ash_device(), command_buffer, image_index as usize,
                self.swapchain.extent.width, self.swapchain.extent.height);
        }

        if self.post_process.auto_exposure_pipeline.is_some() && self.post_process.settings.is_effect_enabled(PostProcessEffect::AutoExposure) {
            self.post_process.dispatch_auto_exposure(self.context.ash_device(), command_buffer, image_index as usize, self.post_process.delta_time);
        }

        if taa_enabled {
            let history = self.temporal_history.as_ref().unwrap();
            let gbuffer = self.gbuffer.as_ref().unwrap();
            let current_depth_view = if self.msaa_samples == vk::SampleCountFlags::TYPE_1 { self.depth_image_view.unwrap() } else { self.post_process.depth_resolved_images[image_index as usize].view };
            self.post_process.dispatch_taa(self.context.ash_device(), command_buffer, image_index as usize, history, gbuffer, current_depth_view, false);
        }

        if self.post_process.fog_pipeline.is_some() && self.post_process.settings.is_effect_enabled(PostProcessEffect::VolumetricFog) {
            self.post_process.dispatch_volumetric_fog(self.context.ash_device(), command_buffer, image_index as usize,
                self.camera_view, self.camera_proj, self.camera_pos, scene.sun_direction,
                glam::Vec3::new(1.0, 0.95, 0.85), self.camera_near, self.camera_far,
                self.post_process.last_time + self.post_process.delta_time);
        }

        if let (Some(clouds), Some(sampler)) = (self.volumetric_clouds.as_mut(), self.post_process.sampler) {
            let inv_vp = (self.camera_proj * self.camera_view).inverse();
            let depth_view = if self.msaa_samples == vk::SampleCountFlags::TYPE_1 { self.depth_image_view.unwrap() } else { self.post_process.depth_resolved_images[image_index as usize].view };
            clouds.dispatch(self.context.ash_device(), command_buffer, image_index as usize,
                self.swapchain.extent.width, self.swapchain.extent.height, inv_vp, self.camera_pos,
                scene.sun_direction, glam::Vec3::new(1.0, 0.95, 0.85), depth_view, sampler);
            clouds.advance_time(self.post_process.delta_time);
        }

        if self.post_process.lens_flare_pipeline.is_some() && self.post_process.settings.is_effect_enabled(PostProcessEffect::AnamorphicFlares) {
            self.post_process.dispatch_lens_flare(self.context.ash_device(), command_buffer, image_index as usize,
                self.swapchain.extent.width, self.swapchain.extent.height,
                self.post_process.last_time + self.post_process.delta_time);
        }

        let swapchain_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .image(swapchain_image)
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });

        unsafe {
            self.context.device.cmd_pipeline_barrier(command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(), &[], &[], &[swapchain_barrier]);

            let post_color_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(swapchain_view).image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::DONT_CARE).store_op(vk::AttachmentStoreOp::STORE);

            let post_rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain.extent })
                .layer_count(1).color_attachments(std::slice::from_ref(&post_color_attachment));

            self.context.device.cmd_begin_rendering(command_buffer, &post_rendering_info);
            self.context.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.post_process.pipeline.unwrap());
            self.context.device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS,
                self.post_process.layout.unwrap(), 0, &[self.post_process.descriptor_sets[image_index as usize]], &[]);

            let mut post_settings = self.post_process.settings;
            post_settings.depth_near = self.camera_near.max(0.001);
            post_settings.depth_far = self.camera_far.max(post_settings.depth_near + 0.001);
            post_settings.camera_proj_x = self.camera_proj.x_axis.x;
            post_settings.camera_proj_y = self.camera_proj.y_axis.y;

            let sun_dir_world = -scene.sun_direction;
            let sun_dir_view = self.camera_view.transform_vector3(sun_dir_world).normalize();
            post_settings.light_dir_x = sun_dir_view.x;
            post_settings.light_dir_y = sun_dir_view.y;
            post_settings.light_dir_z = sun_dir_view.z;

            let settings_bytes = bytemuck::bytes_of(&post_settings);
            self.context.device.cmd_push_constants(command_buffer, self.post_process.layout.unwrap(),
                vk::ShaderStageFlags::FRAGMENT, 0, settings_bytes);

            let viewport = vk::Viewport { x: 0.0, y: 0.0, width: self.swapchain.extent.width as f32, height: self.swapchain.extent.height as f32, min_depth: 0.0, max_depth: 1.0 };
            let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain.extent };
            self.context.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            self.context.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            self.context.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            self.context.device.cmd_end_rendering(command_buffer);

            if let Some(ref mut history) = self.temporal_history {
                history.advance();
            }
        }
    }
}
