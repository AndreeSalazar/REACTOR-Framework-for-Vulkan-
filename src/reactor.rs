use winit::window::Window;
use ash::vk;
use crate::vulkan_context::VulkanContext;
use crate::swapchain::Swapchain;
use crate::mesh::Mesh;
use crate::material::Material;
use gpu_allocator::vulkan::*;
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::input::Input;
use crate::ecs::World;
use crate::ray_tracing::RayTracingContext;
use winit::event::WindowEvent;

pub struct Reactor {
    pub swapchain: Swapchain,
    pub allocator: Arc<Mutex<Allocator>>,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub current_frame: usize,
    
    // New Systems
    pub input: Input,
    pub world: World,
    pub ray_tracing: Option<RayTracingContext>,
    pub resized: bool,
    pub context: VulkanContext,
}

const MAX_FRAMES_IN_FLIGHT: usize = 3;

impl Reactor {
    pub fn init(window: &Window) -> Result<Self, Box<dyn Error>> {
        let context = VulkanContext::new(window)?;
        
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: context.instance.clone(),
            device: context.device.clone(),
            physical_device: context.physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true, // Enabled for RT
            allocation_sizes: Default::default(),
        })?;
        let allocator = Arc::new(Mutex::new(allocator));

        let inner_size = window.inner_size();
        let swapchain = Swapchain::new(&context, inner_size.width, inner_size.height)?;
        
        let render_pass = Self::create_render_pass(&context, swapchain.format)?;
        let framebuffers = Self::create_framebuffers(&context, &swapchain, render_pass)?;

        // Command Pool
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(context.queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe { context.device.create_command_pool(&pool_create_info, None)? };

        // Command Buffers
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
        let command_buffers = unsafe { context.device.allocate_command_buffers(&alloc_info)? };

        // Sync Objects
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                image_available_semaphores.push(context.device.create_semaphore(&semaphore_info, None)?);
                render_finished_semaphores.push(context.device.create_semaphore(&semaphore_info, None)?);
                in_flight_fences.push(context.device.create_fence(&fence_info, None)?);
            }
        }

        // Initialize Ray Tracing (Try)
        let ray_tracing = match RayTracingContext::new(&context) {
            Ok(rt) => {
                println!("Ray Tracing initialized successfully!");
                Some(rt)
            },
            Err(e) => {
                println!("Ray Tracing not supported or failed to init: {}", e);
                None
            }
        };

        Ok(Self { 
            context, 
            swapchain,
            allocator,
            render_pass,
            framebuffers,
            command_pool,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
            input: Input::new(),
            world: World::new(),
            ray_tracing,
            resized: false,
        })
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.input.process_event(event);
        if let WindowEvent::Resized(_) = event {
            self.resized = true;
        }
    }

    pub fn recreate_swapchain(&mut self) -> Result<(), Box<dyn Error>> {
        unsafe { self.context.device.device_wait_idle()?; }

        let capabilities = unsafe {
            self.context.surface_loader
                .get_physical_device_surface_capabilities(self.context.physical_device, self.context.surface)?
        };

        if capabilities.current_extent.width == 0 || capabilities.current_extent.height == 0 {
            return Ok(());
        }

        unsafe {
            for &framebuffer in &self.framebuffers {
                self.context.device.destroy_framebuffer(framebuffer, None);
            }
        }
        self.swapchain.destroy(&self.context.device);

        self.swapchain = Swapchain::new(&self.context, capabilities.current_extent.width, capabilities.current_extent.height)?;
        self.framebuffers = Self::create_framebuffers(&self.context, &self.swapchain, self.render_pass)?;
        
        Ok(())
    }

    pub fn create_mesh(&self, vertices: &[crate::vertex::Vertex], indices: &[u32]) -> Result<Mesh, Box<dyn Error>> {
        Mesh::new(&self.context, &self.allocator, vertices, indices)
    }

    pub fn create_material(&self, vert_code: &[u32], frag_code: &[u32]) -> Result<Material, Box<dyn Error>> {
        Material::new(
            &self.context, 
            self.render_pass, 
            vert_code, 
            frag_code, 
            self.swapchain.extent.width, 
            self.swapchain.extent.height
        )
    }

    fn create_render_pass(context: &VulkanContext, format: vk::Format) -> Result<vk::RenderPass, Box<dyn Error>> {
        let color_attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass = unsafe { context.device.create_render_pass(&render_pass_info, None)? };
        Ok(render_pass)
    }

    fn create_framebuffers(
        context: &VulkanContext, 
        swapchain: &Swapchain, 
        render_pass: vk::RenderPass
    ) -> Result<Vec<vk::Framebuffer>, Box<dyn Error>> {
        swapchain.image_views
            .iter()
            .map(|&view| {
                let attachments = [view];
                let framebuffer_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(swapchain.extent.width)
                    .height(swapchain.extent.height)
                    .layers(1);
                unsafe { context.device.create_framebuffer(&framebuffer_info, None).map_err(|e| e.into()) }
            })
            .collect()
    }

    pub fn draw_scene(&mut self, scene: &crate::scene::Scene, view_projection: &glam::Mat4) -> Result<(), Box<dyn Error>> {
        if self.resized {
            self.recreate_swapchain()?;
            self.resized = false;
        }

        unsafe {
            self.context.device.wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, u64::MAX)?;
        }

        let (image_index, _) = unsafe {
            match self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            ) {
                Ok(result) => {
                    self.context.device.reset_fences(&[self.in_flight_fences[self.current_frame]])?;
                    result
                },
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain()?;
                    return Ok(());
                }
                Err(e) => return Err(Box::new(e)),
            }
        };

        // Record command buffer
        let command_buffer = self.command_buffers[self.current_frame];
        unsafe {
            self.context.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;
        }

        let begin_info = vk::CommandBufferBeginInfo::default();
        
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.context.device.begin_command_buffer(command_buffer, &begin_info)?;
            
            self.context.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            
            // Dynamic State (Viewport/Scissor) - Set once per frame as it covers the whole screen
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
                self.context.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    object.material.pipeline.pipeline,
                );

                // Calculate final transform (MVP)
                let mvp = *view_projection * object.transform;

                // Push Constants
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

                let vertex_buffers = [object.mesh.vertex_buffer.handle];
                let offsets = [0];
                self.context.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                
                self.context.device.cmd_bind_index_buffer(
                    command_buffer, 
                    object.mesh.index_buffer.handle, 
                    0, 
                    vk::IndexType::UINT32
                );

                self.context.device.cmd_draw_indexed(command_buffer, object.mesh.index_count, 1, 0, 0, 0);
            }
            
            self.context.device.cmd_end_render_pass(command_buffer);
            
            self.context.device.end_command_buffer(command_buffer)?;
        }

        // Submit
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
            self.context.device.queue_submit(
                self.context.graphics_queue, 
                &[submit_info], 
                self.in_flight_fences[self.current_frame]
            )?;
        }

        let swapchains = [self.swapchain.handle];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let result = unsafe {
            self.swapchain.loader.queue_present(self.context.graphics_queue, &present_info)
        };

        match result {
            Ok(_) => {},
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                self.resized = true;
            },
            Err(e) => return Err(Box::new(e)),
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    pub fn draw_frame(&mut self, mesh: &Mesh, material: &Material, transform: &glam::Mat4) -> Result<(), Box<dyn Error>> {
        if self.resized {
            self.recreate_swapchain()?;
            self.resized = false;
        }

        unsafe {
            self.context.device.wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, u64::MAX)?;
        }

        let (image_index, _) = unsafe {
            match self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            ) {
                Ok(result) => {
                    self.context.device.reset_fences(&[self.in_flight_fences[self.current_frame]])?;
                    result
                },
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain()?;
                    return Ok(());
                }
                Err(e) => return Err(Box::new(e)),
            }
        };

        // Record command buffer
        let command_buffer = self.command_buffers[self.current_frame];
        unsafe {
            self.context.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;
        }

        let begin_info = vk::CommandBufferBeginInfo::default();
        
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.context.device.begin_command_buffer(command_buffer, &begin_info)?;
            
            self.context.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            
            self.context.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                material.pipeline.pipeline,
            );

            // Dynamic State
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

            // Push Constants
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
            self.context.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            
            self.context.device.cmd_bind_index_buffer(
                command_buffer, 
                mesh.index_buffer.handle, 
                0, 
                vk::IndexType::UINT32
            );

            self.context.device.cmd_draw_indexed(command_buffer, mesh.index_count, 1, 0, 0, 0);
            
            self.context.device.cmd_end_render_pass(command_buffer);
            
            self.context.device.end_command_buffer(command_buffer)?;
        }

        // Submit
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
            self.context.device.queue_submit(
                self.context.graphics_queue,
                &[submit_info],
                self.in_flight_fences[self.current_frame],
            )?;
        }

        // Present
        let swapchains = [self.swapchain.handle];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let result = unsafe {
            self.swapchain.loader.queue_present(self.context.graphics_queue, &present_info)
        };

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        match result {
            Ok(_) => Ok(()),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                self.recreate_swapchain()?;
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        unsafe {
            self.context.device.device_wait_idle().unwrap();
            
            for &framebuffer in &self.framebuffers {
                self.context.device.destroy_framebuffer(framebuffer, None);
            }
            self.context.device.destroy_render_pass(self.render_pass, None);

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.context.device.destroy_semaphore(self.image_available_semaphores[i], None);
                self.context.device.destroy_semaphore(self.render_finished_semaphores[i], None);
                self.context.device.destroy_fence(self.in_flight_fences[i], None);
            }
            
            self.context.device.destroy_command_pool(self.command_pool, None);
        }
        self.swapchain.destroy(&self.context.device);
        // Allocator is dropped automatically by Arc/Mutex
    }
}
