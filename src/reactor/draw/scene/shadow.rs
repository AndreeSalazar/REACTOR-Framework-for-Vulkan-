use crate::reactor::Reactor;
use crate::systems::scene::Scene;
use ash::vk;

impl Reactor {
    pub(super) fn render_shadow_cascades(&mut self, scene: &Scene, command_buffer: vk::CommandBuffer) {
        if self.shadow_map.is_none() || self.shadow_pipeline.is_none() {
            return;
        }

        let cascade_count;
        let shadow_resolution;
        let shadow_img;
        let shadow_pipe;
        let sun_dir;

        {
            let shadow_map = self.shadow_map.as_mut().unwrap();
            sun_dir = scene.sun_direction;
            shadow_map.set_light_direction(sun_dir);
            shadow_map.update(self.camera_view, self.camera_proj, self.camera_near, self.camera_far);
            cascade_count = shadow_map.config.cascade_count;
            shadow_resolution = shadow_map.config.resolution as f32;

            let shadow_uniform = crate::graphics::shadows::ShadowUniformData::from_shadow_map(shadow_map);
            self.shadow_uniform_buffers[self.current_frame].write(&[shadow_uniform]);
        }

        shadow_img = self.shadow_image.unwrap();
        shadow_pipe = self.shadow_pipeline.as_ref().unwrap();

        let shadow_start_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .image(shadow_img)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: cascade_count,
            });

        unsafe {
            self.context.device.cmd_pipeline_barrier(command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                vk::DependencyFlags::empty(), &[], &[], &[shadow_start_barrier]);
        }

        for layer in 0..cascade_count {
            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.shadow_image_views[layer as usize])
                .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } });

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: shadow_resolution as u32, height: shadow_resolution as u32 } })
                .layer_count(1)
                .depth_attachment(&depth_attachment);

            let viewport = vk::Viewport { x: 0.0, y: 0.0, width: shadow_resolution, height: shadow_resolution, min_depth: 0.0, max_depth: 1.0 };
            let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: shadow_resolution as u32, height: shadow_resolution as u32 } };

            unsafe {
                self.context.device.cmd_begin_rendering(command_buffer, &rendering_info);
                self.context.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
                self.context.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
                self.context.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, shadow_pipe.pipeline);
            }

            let cascade = &self.shadow_map.as_ref().unwrap().cascades[layer as usize];
            for object in &scene.objects {
                if !object.visible { continue; }
                let name = object.name.as_deref().unwrap_or("");
                if name.contains("Crosshair") || name.contains("GoScreen") || name.contains("VicScreen") { continue; }

                let light_mvp = cascade.view_proj * object.transform;
                let push_bytes = unsafe {
                    std::slice::from_raw_parts(&light_mvp as *const glam::Mat4 as *const u8, std::mem::size_of::<glam::Mat4>())
                };

                unsafe {
                    self.context.device.cmd_push_constants(command_buffer, shadow_pipe.layout,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT, 0, push_bytes);
                    let vertex_buffers = [object.mesh.vertex_buffer.handle];
                    self.context.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &[0]);
                    self.context.device.cmd_bind_index_buffer(command_buffer, object.mesh.index_buffer.handle, 0, vk::IndexType::UINT32);
                    self.context.device.cmd_draw_indexed(command_buffer, object.mesh.index_count, 1, 0, 0, 0);
                }
            }

            unsafe { self.context.device.cmd_end_rendering(command_buffer); }
        }

        let shadow_end_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(shadow_img)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: cascade_count,
            });

        unsafe {
            self.context.device.cmd_pipeline_barrier(command_buffer,
                vk::PipelineStageFlags::LATE_FRAGMENT_TESTS, vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[], &[], &[shadow_end_barrier]);
        }
    }
}
