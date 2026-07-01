use crate::reactor::Reactor;
use crate::systems::scene::Scene;
use ash::vk;
use ash::vk::Handle;

impl Reactor {
    pub(super) fn render_geometry(
        &mut self,
        scene: &Scene,
        command_buffer: vk::CommandBuffer,
        image_index: u32,
        view_projection: &glam::Mat4,
        use_post_process: bool,
        taa_enabled: bool,
    ) -> glam::Mat4 {
        let mut local_vp = *view_projection;
        if taa_enabled {
            if let Some(ref history) = self.temporal_history {
                let f_idx = history.frame_index;

                fn halton(index: u32, base: u32) -> f32 {
                    let mut result = 0.0;
                    let mut f = 1.0 / base as f32;
                    let mut i = index;
                    while i > 0 {
                        result += f * (i % base) as f32;
                        f /= base as f32;
                        i /= base;
                    }
                    result
                }

                let halton_x = halton((f_idx % 8) as u32 + 1, 2);
                let halton_y = halton((f_idx % 8) as u32 + 1, 3);
                let width = self.swapchain.extent.width as f32;
                let height = self.swapchain.extent.height as f32;
                let jitter_x = (halton_x - 0.5) * 2.0 / width;
                let jitter_y = (halton_y - 0.5) * 2.0 / height;
                self.camera_proj.z_axis.x += jitter_x;
                self.camera_proj.z_axis.y += jitter_y;
                local_vp = self.camera_proj * self.camera_view;
            }
        }

        let target_view = if use_post_process {
            self.post_process.offscreen_images[image_index as usize].view
        } else {
            self.swapchain.image_views[image_index as usize]
        };
        let target_image = if use_post_process {
            self.post_process.offscreen_images[image_index as usize].handle
        } else {
            self.swapchain.images[image_index as usize]
        };

        let msaa_enabled = self.msaa_samples != vk::SampleCountFlags::TYPE_1 && self.msaa_image_view.is_some();

        let color_attachment = if msaa_enabled {
            vk::RenderingAttachmentInfo::default()
                .image_view(self.msaa_image_view.unwrap())
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .resolve_mode(vk::ResolveModeFlags::AVERAGE)
                .resolve_image_view(target_view)
                .resolve_image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue { color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] } })
        } else {
            vk::RenderingAttachmentInfo::default()
                .image_view(target_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue { color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] } })
        };

        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.depth_image_view.unwrap())
            .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } });

        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain.extent })
            .layer_count(1)
            .color_attachments(std::slice::from_ref(&color_attachment))
            .depth_attachment(&depth_attachment);

        let depth_img = self.depth_image.unwrap();

        let color_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .image(target_image)
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });

        let depth_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .image(depth_img)
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });

        let mut start_barriers: Vec<vk::ImageMemoryBarrier> = vec![color_barrier, depth_barrier];

        if msaa_enabled {
            start_barriers.push(vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .image(self.msaa_image.unwrap())
                .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 }));
        }

        if let Some(ref gbuffer) = self.gbuffer {
            for image_handle in gbuffer.images() {
                start_barriers.push(vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::SHADER_READ)
                    .image(image_handle)
                    .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 }));
            }
        }

        unsafe {
            self.context.device.cmd_pipeline_barrier(command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[], &[], &start_barriers);

            self.context.device.cmd_begin_rendering(command_buffer, &rendering_info);

            let viewport = vk::Viewport { x: 0.0, y: 0.0, width: self.swapchain.extent.width as f32, height: self.swapchain.extent.height as f32, min_depth: 0.0, max_depth: 1.0 };
            let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain.extent };
            self.context.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            self.context.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
        }

        let visible_objects = scene.objects.iter().filter(|object| object.visible).count();
        self.apply_pixel_intelligent_vrs(command_buffer, visible_objects);

        let frustum = crate::systems::frustum::Frustum::from_view_projection(local_vp);
        let mut active_pipeline = vk::Pipeline::null();
        let mut active_descriptor_set = vk::DescriptorSet::null();

        unsafe {
            for object in &scene.objects {
                if !object.visible { continue; }

                let center = glam::Vec3::new(object.transform.w_axis.x, object.transform.w_axis.y, object.transform.w_axis.z);
                let name = object.name.as_deref().unwrap_or("");

                let radius = if name.contains("Floor") || name.contains("Wall") || name.contains("Techo") { 12.0 }
                    else if name.contains("Pillar") { 4.0 }
                    else if name.contains("zombie") || name.contains("Zombie") { 2.2 }
                    else if name.contains("Shadow") || name.contains("shadow") { 1.8 }
                    else if name.contains("Crosshair") || name.contains("GoScreen") || name.contains("VicScreen") { 100.0 }
                    else { 1.5 };

                let sphere = crate::systems::physics::Sphere::new(center, radius);
                if !frustum.intersects_sphere(&sphere) { continue; }

                let pipeline_handle = object.material.pipeline.pipeline;
                let descriptor_set_handle = object.material.descriptor_set.unwrap_or(vk::DescriptorSet::null());

                if pipeline_handle != active_pipeline {
                    self.context.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_handle);
                    active_pipeline = pipeline_handle;
                    active_descriptor_set = vk::DescriptorSet::null();

                    if object.material.uses_ibl {
                        self.bind_reactor_system_descriptors(command_buffer, object.material.pipeline.layout, true, object.material.has_shadow_set);
                    } else if object.material.has_shadow_set {
                        self.bind_reactor_system_descriptors(command_buffer, object.material.pipeline.layout, false, object.material.has_shadow_set);
                    }
                }

                if descriptor_set_handle != active_descriptor_set && !descriptor_set_handle.is_null() {
                    self.context.device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS,
                        object.material.pipeline.layout, 0, &[descriptor_set_handle], &[]);
                    active_descriptor_set = descriptor_set_handle;
                }

                let mvp = local_vp * object.transform;
                let prev_mvp = self.prev_view_projection * object.transform;

                #[repr(C)]
                struct PushConstants {
                    mvp: glam::Mat4,
                    model: glam::Mat4,
                    prev_mvp: glam::Mat4,
                    camera_pos: glam::Vec4,
                    light_pos: glam::Vec4,
                    color: glam::Vec4,
                    emission: glam::Vec4,
                }
                let push = PushConstants {
                    mvp,
                    model: object.transform,
                    prev_mvp,
                    camera_pos: glam::Vec4::new(self.camera_pos.x, self.camera_pos.y, self.camera_pos.z, object.metallic),
                    light_pos: glam::Vec4::new(self.light_pos.x, self.light_pos.y, self.light_pos.z, object.roughness),
                    color: glam::Vec4::new(object.color.x, object.color.y, object.color.z, object.anisotropy),
                    emission: object.emission,
                };
                let constants_array = std::slice::from_raw_parts(&push as *const PushConstants as *const u8, std::mem::size_of::<PushConstants>());
                self.context.device.cmd_push_constants(command_buffer, object.material.pipeline.layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT, 0, constants_array);

                let vertex_buffers = [object.mesh.vertex_buffer.handle];
                self.context.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &[0]);
                self.context.device.cmd_bind_index_buffer(command_buffer, object.mesh.index_buffer.handle, 0, vk::IndexType::UINT32);
                self.context.device.cmd_draw_indexed(command_buffer, object.mesh.index_count, 1, 0, 0, 0);
            }

            self.context.device.cmd_end_rendering(command_buffer);
        }

        self.prev_view_projection = self.camera_proj * self.camera_view;

        if use_post_process && !self.decals.is_empty() {
            let _ = self.draw_screen_space_decals(command_buffer, image_index as usize, &local_vp);
        }

        local_vp
    }
}
