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
use crate::core::VrsRate;
use crate::resources::material::Material;
use crate::resources::mesh::Mesh;
use crate::systems::scene::Scene;
use ash::vk;
use ash::vk::Handle;

impl Reactor {
    fn apply_pixel_intelligent_vrs(
        &mut self,
        command_buffer: vk::CommandBuffer,
        visible_objects: usize,
    ) {
        let desired = self
            .pixel_intelligent
            .desired_rate(self.swapchain.extent, visible_objects);

        let Some(vrs) = self.context.fragment_shading_rate.as_ref() else {
            self.pixel_intelligent.current_rate = VrsRate::NATIVE;
            return;
        };

        let rate = vrs
            .capabilities
            .best_supported_rate(desired, self.msaa_samples);
        self.pixel_intelligent.current_rate = rate;

        unsafe {
            vrs.cmd_set_rate(command_buffer, rate);
        }
    }

    /// Dibuja una escena completa (todos los `SceneObject`) con MVP precomputado.
    pub fn draw_scene(&mut self, scene: &Scene, view_projection: &glam::Mat4) -> ReactorResult<()> {
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

            // ── 3.1. CSM Shadow Pass ──
            if self.shadow_map.is_some() && self.shadow_pipeline.is_some() {
                let sun_dir = scene.sun_direction;
                if let Some(ref mut sm) = self.shadow_map {
                    sm.set_light_direction(sun_dir);
                    sm.update(
                        self.camera_view,
                        self.camera_proj,
                        self.camera_near,
                        self.camera_far,
                    );
                }

                let shadow_uniform = crate::graphics::shadows::ShadowUniformData::from_shadow_map(
                    self.shadow_map.as_ref().unwrap(),
                );
                self.shadow_uniform_buffers[self.current_frame].write(&[shadow_uniform]);

                let shadow_img = self.shadow_image.unwrap();
                let shadow_pipe = self.shadow_pipeline.as_ref().unwrap();

                let shadow_start_barrier = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .src_access_mask(vk::AccessFlags::empty())
                    .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                    .image(shadow_img)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::DEPTH,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 4,
                    });

                self.context.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[shadow_start_barrier],
                );

                for layer in 0..4 {
                    let depth_attachment = vk::RenderingAttachmentInfo::default()
                        .image_view(self.shadow_image_views[layer])
                        .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        .load_op(vk::AttachmentLoadOp::CLEAR)
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .clear_value(vk::ClearValue {
                            depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
                        });

                    let rendering_info = vk::RenderingInfo::default()
                        .render_area(vk::Rect2D {
                            offset: vk::Offset2D { x: 0, y: 0 },
                            extent: vk::Extent2D { width: 2048, height: 2048 },
                        })
                        .layer_count(1)
                        .depth_attachment(&depth_attachment);

                    self.context
                        .device
                        .cmd_begin_rendering(command_buffer, &rendering_info);

                    let viewport = vk::Viewport {
                        x: 0.0,
                        y: 0.0,
                        width: 2048.0,
                        height: 2048.0,
                        min_depth: 0.0,
                        max_depth: 1.0,
                    };
                    let scissor = vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: vk::Extent2D { width: 2048, height: 2048 },
                    };
                    self.context
                        .device
                        .cmd_set_viewport(command_buffer, 0, &[viewport]);
                    self.context
                        .device
                        .cmd_set_scissor(command_buffer, 0, &[scissor]);

                    self.context.device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        shadow_pipe.pipeline,
                    );

                    let cascade = &self.shadow_map.as_ref().unwrap().cascades[layer];
                    for object in &scene.objects {
                        if !object.visible {
                            continue;
                        }
                        let name = object.name.as_deref().unwrap_or("");
                        if name.contains("Crosshair")
                            || name.contains("GoScreen")
                            || name.contains("VicScreen")
                        {
                            continue;
                        }

                        let light_mvp = cascade.view_proj * object.transform;

                        let push_bytes = std::slice::from_raw_parts(
                            &light_mvp as *const glam::Mat4 as *const u8,
                            std::mem::size_of::<glam::Mat4>(),
                        );

                        self.context.device.cmd_push_constants(
                            command_buffer,
                            shadow_pipe.layout,
                            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                            0,
                            push_bytes,
                        );

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
                }

                let shadow_end_barrier = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ)
                    .image(shadow_img)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::DEPTH,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 4,
                    });

                self.context.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[shadow_end_barrier],
                );
            }

            let taa_enabled = self.post_process.enabled
                && self.post_process.settings.is_effect_enabled(crate::graphics::post_process::PostProcessEffect::TAA)
                && self.temporal_history.is_some()
                && self.gbuffer.is_some();

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

            // Determine if we should render to offscreen target for post-processing
            let use_post_process =
                self.post_process.enabled && !self.post_process.offscreen_images.is_empty();

            let swapchain_view = self.swapchain.image_views[image_index as usize];
            let swapchain_image = self.swapchain.images[image_index as usize];

            let target_view = if use_post_process {
                self.post_process.offscreen_images[image_index as usize].view
            } else {
                swapchain_view
            };
            let target_image = if use_post_process {
                self.post_process.offscreen_images[image_index as usize].handle
            } else {
                swapchain_image
            };

            let msaa_enabled =
                self.msaa_samples != vk::SampleCountFlags::TYPE_1 && self.msaa_image_view.is_some();

            let color_attachment = if msaa_enabled {
                vk::RenderingAttachmentInfo::default()
                    .image_view(self.msaa_image_view.unwrap())
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .resolve_mode(vk::ResolveModeFlags::AVERAGE)
                    .resolve_image_view(target_view)
                    .resolve_image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .clear_value(vk::ClearValue {
                        color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] },
                    })
            } else {
                vk::RenderingAttachmentInfo::default()
                    .image_view(target_view)
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .clear_value(vk::ClearValue {
                        color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] },
                    })
            };

            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.depth_image_view.unwrap())
                .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
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
                .depth_attachment(&depth_attachment);

            // ── Barreras de inicio ──
            let depth_img = self.depth_image.unwrap();

            let color_barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .image(target_image)
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
            self.context
                .device
                .cmd_set_viewport(command_buffer, 0, &[viewport]);
            self.context
                .device
                .cmd_set_scissor(command_buffer, 0, &[scissor]);

            let visible_objects = scene.objects.iter().filter(|object| object.visible).count();
            self.apply_pixel_intelligent_vrs(command_buffer, visible_objects);

            let frustum = crate::systems::frustum::Frustum::from_view_projection(local_vp);
            let mut active_pipeline = vk::Pipeline::null();
            let mut active_descriptor_set = vk::DescriptorSet::null();

            for object in &scene.objects {
                if !object.visible {
                    continue;
                }

                // ── Frustum Culling ──
                let center = glam::Vec3::new(
                    object.transform.w_axis.x,
                    object.transform.w_axis.y,
                    object.transform.w_axis.z,
                );
                let name = object.name.as_deref().unwrap_or("");

                // Conservatively estimate bounding radius based on object type/name
                let radius =
                    if name.contains("Floor") || name.contains("Wall") || name.contains("Techo") {
                        12.0
                    } else if name.contains("Pillar") {
                        4.0
                    } else if name.contains("zombie") || name.contains("Zombie") {
                        2.2
                    } else if name.contains("Shadow") || name.contains("shadow") {
                        1.8
                    } else if name.contains("Crosshair")
                        || name.contains("GoScreen")
                        || name.contains("VicScreen")
                    {
                        // Interface/Overlays must always render if active
                        100.0
                    } else {
                        1.5 // Tracers, impacts, muzzle flash, etc.
                    };

                let sphere = crate::systems::physics::Sphere::new(center, radius);
                if !frustum.intersects_sphere(&sphere) {
                    continue;
                }

                // ── Bind Pipeline & Descriptor Sets (State Caching Optimization) ──
                let pipeline_handle = object.material.pipeline.pipeline;
                let descriptor_set_handle = object
                    .material
                    .descriptor_set
                    .unwrap_or(vk::DescriptorSet::null());

                if pipeline_handle != active_pipeline {
                    self.context.device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline_handle,
                    );
                    active_pipeline = pipeline_handle;
                    active_descriptor_set = vk::DescriptorSet::null(); // Reset active descriptor set on pipeline change

                    if object.material.uses_ibl {
                        if let Some(ref ibl) = self.ibl_textures {
                            self.context.device.cmd_bind_descriptor_sets(
                                command_buffer,
                                vk::PipelineBindPoint::GRAPHICS,
                                object.material.pipeline.layout,
                                1, // Set 1 for IBL Textures
                                &[ibl.descriptor_set],
                                &[],
                            );
                        }
                    }

                    if !self.shadow_descriptor_sets.is_empty() {
                        self.context.device.cmd_bind_descriptor_sets(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            object.material.pipeline.layout,
                            2, // Set 2 for Shadows
                            &[self.shadow_descriptor_sets[self.current_frame]],
                            &[],
                        );
                    }
                }

                if descriptor_set_handle != active_descriptor_set
                    && !descriptor_set_handle.is_null()
                {
                    self.context.device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        object.material.pipeline.layout,
                        0,
                        &[descriptor_set_handle],
                        &[],
                    );
                    active_descriptor_set = descriptor_set_handle;
                }

                let mvp = local_vp * object.transform;

                #[repr(C)]
                struct PushConstants {
                    mvp: glam::Mat4,
                    model: glam::Mat4,
                    camera_pos: glam::Vec4,
                    light_pos: glam::Vec4,
                    color: glam::Vec4,
                    emission: glam::Vec4,
                }
                let push = PushConstants {
                    mvp,
                    model: object.transform,
                    camera_pos: glam::Vec4::new(
                        self.camera_pos.x,
                        self.camera_pos.y,
                        self.camera_pos.z,
                        object.metallic, // pack metallic in camera_pos.w
                    ),
                    light_pos: glam::Vec4::new(
                        self.light_pos.x,
                        self.light_pos.y,
                        self.light_pos.z,
                        object.roughness, // pack roughness in light_pos.w
                    ),
                    color: glam::Vec4::new(
                        object.color.x,
                        object.color.y,
                        object.color.z,
                        object.anisotropy, // pack anisotropy in color.w
                    ),
                    emission: object.emission,
                };
                let constants_array = std::slice::from_raw_parts(
                    &push as *const PushConstants as *const u8,
                    std::mem::size_of::<PushConstants>(),
                );
                self.context.device.cmd_push_constants(
                    command_buffer,
                    object.material.pipeline.layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    constants_array,
                );

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

            // ── Render Screen-Space Decals (Fase 20) ──
            if use_post_process && !self.decals.is_empty() {
                self.draw_screen_space_decals(command_buffer, image_index as usize, &local_vp)?;
            }

            if use_post_process {
                // ── Transition offscreen → SHADER_READ_ONLY with proper compute + fragment sync ──
                let offscreen_barrier = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ)
                    .image(self.post_process.offscreen_images[image_index as usize].handle)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });

                let depth_old_layout = if self.decals.is_empty() {
                    vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
                } else {
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                };

                let depth_read_barrier = vk::ImageMemoryBarrier::default()
                    .old_layout(depth_old_layout)
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

                self.context.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                        | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                    vk::PipelineStageFlags::COMPUTE_SHADER
                        | vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[offscreen_barrier, depth_read_barrier],
                );

                // ── Bloom Compute Pipeline (mip-chain downsample + upsample) ──
                if self.msaa_samples != vk::SampleCountFlags::TYPE_1 {
                    self.post_process.dispatch_depth_resolve(
                        self.context.ash_device(),
                        command_buffer,
                        image_index as usize,
                        self.swapchain.extent.width,
                        self.swapchain.extent.height,
                        self.msaa_samples,
                    );
                }

                if self.post_process.bloom_downsample_pipeline.is_some() {
                    self.post_process.dispatch_bloom(
                        self.context.ash_device(),
                        command_buffer,
                        image_index as usize,
                        self.swapchain.extent.width,
                        self.swapchain.extent.height,
                    );
                }

                // ── Auto-Exposure Compute Pipeline ──
                if self.post_process.auto_exposure_pipeline.is_some()
                    && self.post_process
                        .settings
                        .is_effect_enabled(crate::graphics::post_process::PostProcessEffect::AutoExposure)
                {
                    self.post_process.dispatch_auto_exposure(
                        self.context.ash_device(),
                        command_buffer,
                        image_index as usize,
                        self.post_process.delta_time,
                    );
                }

                // ── Temporal Anti-Aliasing (TAA) Compute Pass ──
                if taa_enabled {
                    let history = self.temporal_history.as_ref().unwrap();
                    let gbuffer = self.gbuffer.as_ref().unwrap();
                    let current_depth_view = if self.msaa_samples == vk::SampleCountFlags::TYPE_1 {
                        self.depth_image_view.unwrap()
                    } else {
                        self.post_process.depth_resolved_images[image_index as usize].view
                    };
                    
                    self.post_process.dispatch_taa(
                        self.context.ash_device(),
                        command_buffer,
                        image_index as usize,
                        history,
                        gbuffer,
                        current_depth_view,
                        false, // reset_history
                    );
                }

                // Transition swapchain image from UNDEFINED to COLOR_ATTACHMENT_OPTIMAL
                let swapchain_barrier = vk::ImageMemoryBarrier::default()
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

                self.context.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[swapchain_barrier],
                );

                // Begin post-processing rendering pass
                let post_color_attachment = vk::RenderingAttachmentInfo::default()
                    .image_view(swapchain_view)
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(vk::AttachmentLoadOp::DONT_CARE) // Overwrite whole screen
                    .store_op(vk::AttachmentStoreOp::STORE);

                let post_rendering_info = vk::RenderingInfo::default()
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: self.swapchain.extent,
                    })
                    .layer_count(1)
                    .color_attachments(std::slice::from_ref(&post_color_attachment));

                self.context
                    .device
                    .cmd_begin_rendering(command_buffer, &post_rendering_info);

                // Bind post-processing pipeline
                self.context.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.post_process.pipeline.unwrap(),
                );

                // Update binding 0 dynamically: use resolved TAA history color if TAA is enabled,
                // or fallback to the raw offscreen image if TAA is disabled.
                let sampler = self.post_process.sampler.unwrap();
                let color_view = if taa_enabled {
                    self.temporal_history.as_ref().unwrap().current_color().view
                } else {
                    self.post_process.offscreen_images[image_index as usize].view
                };
                
                let image_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(color_view)
                    .sampler(sampler);

                let motion_view = self.gbuffer.as_ref().map(|gb| gb.motion_depth_flags.view).unwrap_or(color_view);
                let motion_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(motion_view)
                    .sampler(sampler);

                let writes = [
                    vk::WriteDescriptorSet::default()
                        .dst_set(self.post_process.descriptor_sets[image_index as usize])
                        .dst_binding(0)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&image_info)),
                    vk::WriteDescriptorSet::default()
                        .dst_set(self.post_process.descriptor_sets[image_index as usize])
                        .dst_binding(5)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&motion_info)),
                ];
                    
                self.context.device.update_descriptor_sets(&writes, &[]);

                // Bind descriptor set (the offscreen texture)
                self.context.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.post_process.layout.unwrap(),
                    0,
                    &[self.post_process.descriptor_sets[image_index as usize]],
                    &[],
                );

                // Push settings
                let mut post_settings = self.post_process.settings;
                post_settings.depth_near = self.camera_near.max(0.001);
                post_settings.depth_far = self.camera_far.max(post_settings.depth_near + 0.001);
                post_settings.camera_proj_x = self.camera_proj.x_axis.x;
                post_settings.camera_proj_y = self.camera_proj.y_axis.y;

                // Transform sun direction to view space for contact shadows
                let sun_dir_world = -scene.sun_direction; // pointing towards light
                let sun_dir_view = self.camera_view.transform_vector3(sun_dir_world).normalize();
                post_settings.light_dir_x = sun_dir_view.x;
                post_settings.light_dir_y = sun_dir_view.y;
                post_settings.light_dir_z = sun_dir_view.z;

                let settings_bytes = bytemuck::bytes_of(&post_settings);
                self.context.device.cmd_push_constants(
                    command_buffer,
                    self.post_process.layout.unwrap(),
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    settings_bytes,
                );

                // Set dynamic viewport and scissor matching swapchain size
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
                self.context
                    .device
                    .cmd_set_viewport(command_buffer, 0, &[viewport]);
                self.context
                    .device
                    .cmd_set_scissor(command_buffer, 0, &[scissor]);

                // Draw fullscreen triangle
                self.context.device.cmd_draw(command_buffer, 3, 1, 0, 0);

                // End post-processing rendering pass
                self.context.device.cmd_end_rendering(command_buffer);

                if let Some(ref mut history) = self.temporal_history {
                    history.advance();
                }
            }

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
        let signal_semaphores = [self.render_finished_semaphores[image_index as usize]];
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
            let msaa_enabled =
                self.msaa_samples != vk::SampleCountFlags::TYPE_1 && self.msaa_image_view.is_some();

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

            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.depth_image_view.unwrap())
                .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
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
                .depth_attachment(&depth_attachment);

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
            self.context
                .device
                .cmd_set_viewport(command_buffer, 0, &[viewport]);
            self.context
                .device
                .cmd_set_scissor(command_buffer, 0, &[scissor]);

            self.apply_pixel_intelligent_vrs(command_buffer, 1);

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
            self.context.device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &vertex_buffers,
                &offsets,
            );
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
        let signal_semaphores = [self.render_finished_semaphores[image_index as usize]];
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

    /// Renderiza los decals en espacio de pantalla directamente sobre el target offscreen de color.
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

        // 1. Transición del depth buffer de DEPTH_STENCIL_ATTACHMENT_OPTIMAL -> SHADER_READ_ONLY_OPTIMAL
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

        // 2. Comenzar render pass de decals en el color target offscreen (cargando el color actual)
        let target_view = self.post_process.offscreen_images[image_index].view;
        let color_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(target_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::LOAD)
            .store_op(vk::AttachmentStoreOp::STORE);

        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .layer_count(1)
            .color_attachments(std::slice::from_ref(&color_attachment));

        unsafe {
            self.context.device.cmd_begin_rendering(command_buffer, &rendering_info);

            self.context.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );

            // viewport y scissor
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

            // bind mesh
            let vertex_buffers = [cube_mesh.vertex_buffer.handle];
            let offsets = [0];
            self.context.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            self.context.device.cmd_bind_index_buffer(command_buffer, cube_mesh.index_buffer.handle, 0, vk::IndexType::UINT32);

            let view_proj_inv = view_proj.inverse();

            for decal in &self.decals {
                // Actualizar descriptor dinámico con el depth buffer actual
                decal.update_depth_descriptor(
                    self.depth_image_view.unwrap(),
                    self.post_process.sampler.unwrap(),
                );

                // Bind descriptor set
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
