use crate::reactor::Reactor;
use crate::core::error::ReactorResult;
use ash::vk;

impl Reactor {
    pub fn draw_screen_space_decals(
        &self,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        view_proj: &glam::Mat4,
    ) -> ReactorResult<()> {
        let Some(ref pipeline) = self.decal_pipeline else {
            return Ok(());
        };
        let Some(ref cube_mesh) = self.decal_cube_mesh else {
            return Ok(());
        };
        if self.decals.is_empty() {
            return Ok(());
        }

        let depth_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(self.depth_image.unwrap())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        unsafe {
            self.context.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[depth_barrier],
            );
        }

        let Some(ref gbuffer) = self.gbuffer else {
            return Ok(());
        };

        let albedo_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(gbuffer.albedo_ao.view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::LOAD)
            .store_op(vk::AttachmentStoreOp::STORE);

        let normal_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(gbuffer.normal_material.view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::LOAD)
            .store_op(vk::AttachmentStoreOp::STORE);

        let color_attachments = [albedo_attachment, normal_attachment];

        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments);

        unsafe {
            self.context.device.cmd_begin_rendering(command_buffer, &rendering_info);

            self.context.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain.extent.width as f32,
                height: self.swapchain.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            };
            self.context.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            self.context.device.cmd_set_scissor(command_buffer, 0, &[scissor]);

            let vertex_buffers = [cube_mesh.vertex_buffer.handle];
            let offsets = [0];
            self.context.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            self.context.device.cmd_bind_index_buffer(command_buffer, cube_mesh.index_buffer.handle, 0, vk::IndexType::UINT32);

            let view_proj_inv = view_proj.inverse();

            for decal in &self.decals {
                decal.update_depth_descriptor(
                    self.depth_image_view.unwrap(),
                    self.post_process.sampler.unwrap(),
                );

                self.context.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.layout,
                    0,
                    &[decal.descriptor_set],
                    &[],
                );

                #[repr(C)]
                struct DecalPushConstants {
                    mvp: glam::Mat4,
                    view_proj_inv: glam::Mat4,
                    decal_world_inv: glam::Mat4,
                    decal_color: glam::Vec4,
                    decal_params: glam::Vec4,
                }

                let mvp = *view_proj * decal.model;
                let push = DecalPushConstants {
                    mvp,
                    view_proj_inv,
                    decal_world_inv: decal.model.inverse(),
                    decal_color: decal.color,
                    decal_params: glam::Vec4::new(
                        decal.normal_strength,
                        decal.metallic,
                        decal.roughness,
                        0.0,
                    ),
                };

                let constants_array = std::slice::from_raw_parts(
                    &push as *const DecalPushConstants as *const u8,
                    std::mem::size_of::<DecalPushConstants>(),
                );

                self.context.device.cmd_push_constants(
                    command_buffer,
                    pipeline.layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    constants_array,
                );

                self.context.device.cmd_draw_indexed(
                    command_buffer,
                    cube_mesh.index_count,
                    1,
                    0,
                    0,
                    0,
                );
            }

            self.context.device.cmd_end_rendering(command_buffer);
        }

        Ok(())
    }
}
