//! Comandos de dibujo: `draw_scene` (escena completa) y `draw_frame`
//! (single-mesh, modo legado).
//!
//! Ambos métodos siguen el mismo esquema:
//! 1. Wait fence del frame en vuelo.
//! 2. Adquirir imagen del swapchain (recreate si OUT_OF_DATE).
//! 3. Reset + begin del command buffer.
//! 4. Barreras → `cmd_begin_rendering` (Dynamic Rendering 1.3).
//! 5. Bind material/mesh + `cmd_draw_indexed`.
//! 6. `cmd_end_rendering` → barrera al layout PRESENT_SRC_KHR.
//! 7. Submit + present.

use super::{Reactor, MAX_FRAMES_IN_FLIGHT};
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::resources::material::Material;
use crate::resources::mesh::Mesh;
use crate::systems::scene::Scene;
use ash::vk;

impl Reactor {
    /// Dibuja una escena completa (todos los `SceneObject`) con MVP precomputado.
    pub fn draw_scene(
        &mut self,
        scene: &Scene,
        view_projection: &glam::Mat4,
    ) -> ReactorResult<()> {
        if self.device_lost {
            return Ok(());
        }

        if self.resized {
            self.recreate_swapchain()?;
            self.resized = false;
        }

        // ── 1. Wait fence ──
        unsafe {
            match self.context.device.wait_for_fences(
                &[self.in_flight_fences[self.current_frame]],
                true,
                u64::MAX,
            ) {
                Ok(_) => {}
                Err(vk::Result::ERROR_DEVICE_LOST) => {
                    eprintln!(
                        "REACTOR FATAL: Dispositivo Vulkan perdido (wait_for_fences). El driver puede haber crasheado."
                    );
                    self.device_lost = true;
                    return Err(ReactorError::new(
                        ErrorCode::VulkanSynchronization,
                        "Device lost",
                    ));
                }
                Err(e) => {
                    return Err(ReactorError::with_source(
                        ErrorCode::VulkanSynchronization,
                        "wait_for_fences failed",
                        e,
                    ))
                }
            }
        }

        // ── 2. Acquire ──
        let (image_index, _) = unsafe {
            match self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            ) {
                Ok(result) => {
                    self.context
                        .device
                        .reset_fences(&[self.in_flight_fences[self.current_frame]])
                        .map_err(|e| {
                            ReactorError::with_source(
                                ErrorCode::VulkanSynchronization,
                                "reset_fences failed",
                                e,
                            )
                        })?;
                    result
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain()?;
                    return Ok(());
                }
                Err(e) => {
                    return Err(ReactorError::with_source(
                        ErrorCode::VulkanSwapchain,
                        "acquire_next_image failed",
                        e,
                    ))
                }
            }
        };

        // ── 3. Reset + begin command buffer ──
        let command_buffer = self.command_buffers[self.current_frame];
        unsafe {
            self.context
                .device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanCommandPool,
                        "reset_command_buffer failed",
                        e,
                    )
                })?;
        }

        let begin_info = vk::CommandBufferBeginInfo::default();

        unsafe {
            self.context
                .device
                .begin_command_buffer(command_buffer, &begin_info)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanCommandPool,
                        "begin_command_buffer failed",
                        e,
                    )
                })?;

            // ── Dynamic Rendering: color + depth (MSAA opcional) ──
            let swapchain_view = self.swapchain.image_views[image_index as usize];
            let msaa_enabled = self.msaa_samples != vk::SampleCountFlags::TYPE_1
                && self.msaa_image_view.is_some();

            let color_attachment = if msaa_enabled {
                vk::RenderingAttachmentInfo::default()
                    .image_view(self.msaa_image_view.unwrap())
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .resolve_mode(vk::ResolveModeFlags::AVERAGE)
                    .resolve_image_view(swapchain_view)
                    .resolve_image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .clear_value(vk::ClearValue {
                        color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] },
                    })
            } else {
                vk::RenderingAttachmentInfo::default()
                    .image_view(swapchain_view)
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .clear_value(vk::ClearValue {
                        color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] },
                    })
            };

            let mut depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.depth_image_view.unwrap())
                .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
                });

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain.extent,
                })
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment))
                .depth_attachment(&mut depth_attachment);

            // ── Barreras de inicio ──
            let swapchain_image = self.swapchain.images[image_index as usize];
            let depth_img = self.depth_image.unwrap();

            let color_barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .image(swapchain_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let depth_barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .image(depth_img)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let mut start_barriers: Vec<vk::ImageMemoryBarrier> =
                vec![color_barrier, depth_barrier];

            if msaa_enabled {
                start_barriers.push(
                    vk::ImageMemoryBarrier::default()
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .src_access_mask(vk::AccessFlags::empty())
                        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                        .image(self.msaa_image.unwrap())
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        }),
                );
            }

            self.context.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &start_barriers,
            );

            self.context
                .device
                .cmd_begin_rendering(command_buffer, &rendering_info);

            // Dynamic State (Viewport/Scissor)
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

            for object in &scene.objects {
                // Bind pipeline + descriptor sets (texturas si las hay).
                object.material.bind(&self.context.device, command_buffer);

                let mvp = *view_projection * object.transform;

                if object.material.descriptor_set.is_some() {
                    #[repr(C)]
                    struct PushConstants {
                        mvp: glam::Mat4,
                        model: glam::Mat4,
                    }
                    let push = PushConstants { mvp, model: object.transform };
                    let constants_array = std::slice::from_raw_parts(
                        &push as *const PushConstants as *const u8,
                        std::mem::size_of::<PushConstants>(),
                    );
                    self.context.device.cmd_push_constants(
                        command_buffer,
                        object.material.pipeline.layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        constants_array,
                    );
                } else {
                    let constants_array = std::slice::from_raw_parts(
                        &mvp as *const glam::Mat4 as *const u8,
                        std::mem::size_of::<glam::Mat4>(),
                    );
                    self.context.device.cmd_push_constants(
                        command_buffer,
                        object.material.pipeline.layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        constants_array,
                    );
                }

                let vertex_buffers = [object.mesh.vertex_buffer.handle];
                let offsets = [0];
                self.context.device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &vertex_buffers,
                    &offsets,
                );
                self.context.device.cmd_bind_index_buffer(
                    command_buffer,
                    object.mesh.index_buffer.handle,
                    0,
                    vk::IndexType::UINT32,
                );
                self.context.device.cmd_draw_indexed(
                    command_buffer,
                    object.mesh.index_count,
                    1,
                    0,
                    0,
                    0,
                );
            }

            self.context.device.cmd_end_rendering(command_buffer);

            // ── Barrera 2: Color → Present ──
            let image_barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .image(swapchain_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            self.context.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            );

            self.context
                .device
                .end_command_buffer(command_buffer)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanCommandPool,
                        "end_command_buffer failed",
                        e,
                    )
                })?;
        }

        // ── Submit ──
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let command_buffers_submit = [command_buffer];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers_submit)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.context
                .device
                .queue_submit(
                    self.context.graphics_queue,
                    &[submit_info],
                    self.in_flight_fences[self.current_frame],
                )
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSynchronization,
                        "queue_submit failed",
                        e,
                    )
                })?;
        }

        // ── Present ──
        let swapchains = [self.swapchain.handle];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let result = unsafe {
            self.swapchain
                .loader
                .queue_present(self.context.graphics_queue, &present_info)
        };

        match result {
            Ok(_) => {}
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                self.resized = true;
            }
            Err(e) => {
                return Err(ReactorError::with_source(
                    ErrorCode::VulkanSwapchain,
                    "queue_present failed",
                    e,
                ))
            }
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    /// Dibuja un único `mesh` con un material y un transform dados.
    ///
    /// Útil para demos minimalistas o tests. Para escenas reales usar
    /// [`Reactor::draw_scene`].
    pub fn draw_frame(
        &mut self,
        mesh: &Mesh,
        material: &Material,
        transform: &glam::Mat4,
    ) -> ReactorResult<()> {
        if self.device_lost {
            return Ok(());
        }

        if self.resized {
            self.recreate_swapchain()?;
            self.resized = false;
        }

        unsafe {
            match self.context.device.wait_for_fences(
                &[self.in_flight_fences[self.current_frame]],
                true,
                u64::MAX,
            ) {
                Ok(_) => {}
                Err(vk::Result::ERROR_DEVICE_LOST) => {
                    eprintln!(
                        "REACTOR FATAL: Dispositivo Vulkan perdido (wait_for_fences). El driver puede haber crasheado."
                    );
                    self.device_lost = true;
                    return Err(ReactorError::new(
                        ErrorCode::VulkanSynchronization,
                        "Device lost",
                    ));
                }
                Err(e) => {
                    return Err(ReactorError::with_source(
                        ErrorCode::VulkanSynchronization,
                        "wait_for_fences failed",
                        e,
                    ))
                }
            }
        }

        let (image_index, _) = unsafe {
            match self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            ) {
                Ok(result) => {
                    self.context
                        .device
                        .reset_fences(&[self.in_flight_fences[self.current_frame]])
                        .map_err(|e| {
                            ReactorError::with_source(
                                ErrorCode::VulkanSynchronization,
                                "reset_fences failed",
                                e,
                            )
                        })?;
                    result
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain()?;
                    return Ok(());
                }
                Err(e) => {
                    return Err(ReactorError::with_source(
                        ErrorCode::VulkanSwapchain,
                        "acquire_next_image failed",
                        e,
                    ))
                }
            }
        };

        let command_buffer = self.command_buffers[self.current_frame];
        unsafe {
            self.context
                .device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanCommandPool,
                        "reset_command_buffer failed",
                        e,
                    )
                })?;
        }

        let begin_info = vk::CommandBufferBeginInfo::default();

        unsafe {
            self.context
                .device
                .begin_command_buffer(command_buffer, &begin_info)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanCommandPool,
                        "begin_command_buffer failed",
                        e,
                    )
                })?;

            let swapchain_view = self.swapchain.image_views[image_index as usize];
            let msaa_enabled = self.msaa_samples != vk::SampleCountFlags::TYPE_1
                && self.msaa_image_view.is_some();

            let color_attachment = if msaa_enabled {
                vk::RenderingAttachmentInfo::default()
                    .image_view(self.msaa_image_view.unwrap())
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .resolve_mode(vk::ResolveModeFlags::AVERAGE)
                    .resolve_image_view(swapchain_view)
                    .resolve_image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .clear_value(vk::ClearValue {
                        color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] },
                    })
            } else {
                vk::RenderingAttachmentInfo::default()
                    .image_view(swapchain_view)
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .clear_value(vk::ClearValue {
                        color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] },
                    })
            };

            let mut depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.depth_image_view.unwrap())
                .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
                });

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain.extent,
                })
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment))
                .depth_attachment(&mut depth_attachment);

            let swapchain_image = self.swapchain.images[image_index as usize];
            let depth_img = self.depth_image.unwrap();

            let color_barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .image(swapchain_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let depth_barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .image(depth_img)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let mut start_barriers: Vec<vk::ImageMemoryBarrier> =
                vec![color_barrier, depth_barrier];

            if msaa_enabled {
                start_barriers.push(
                    vk::ImageMemoryBarrier::default()
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .src_access_mask(vk::AccessFlags::empty())
                        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                        .image(self.msaa_image.unwrap())
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        }),
                );
            }

            self.context.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &start_barriers,
            );

            self.context
                .device
                .cmd_begin_rendering(command_buffer, &rendering_info);

            self.context.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                material.pipeline.pipeline,
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

            let constants_array = std::slice::from_raw_parts(
                transform as *const glam::Mat4 as *const u8,
                std::mem::size_of::<glam::Mat4>(),
            );
            self.context.device.cmd_push_constants(
                command_buffer,
                material.pipeline.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                constants_array,
            );

            let vertex_buffers = [mesh.vertex_buffer.handle];
            let offsets = [0];
            self.context
                .device
                .cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            self.context.device.cmd_bind_index_buffer(
                command_buffer,
                mesh.index_buffer.handle,
                0,
                vk::IndexType::UINT32,
            );
            self.context
                .device
                .cmd_draw_indexed(command_buffer, mesh.index_count, 1, 0, 0, 0);

            self.context.device.cmd_end_rendering(command_buffer);

            let image_barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .image(swapchain_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            self.context.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            );

            self.context
                .device
                .end_command_buffer(command_buffer)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanCommandPool,
                        "end_command_buffer failed",
                        e,
                    )
                })?;
        }

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let command_buffers_submit = [command_buffer];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers_submit)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.context
                .device
                .queue_submit(
                    self.context.graphics_queue,
                    &[submit_info],
                    self.in_flight_fences[self.current_frame],
                )
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanSynchronization,
                        "queue_submit failed",
                        e,
                    )
                })?;
        }

        let swapchains = [self.swapchain.handle];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let result = unsafe {
            self.swapchain
                .loader
                .queue_present(self.context.graphics_queue, &present_info)
        };

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        match result {
            Ok(_) => Ok(()),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                self.recreate_swapchain()?;
                Ok(())
            }
            Err(e) => Err(ReactorError::with_source(
                ErrorCode::VulkanSwapchain,
                "queue_present failed",
                e,
            )),
        }
    }
}
