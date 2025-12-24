use winit::window::Window;
use ash::vk;
use crate::vulkan_context::VulkanContext;
use crate::swapchain::Swapchain;
use crate::mesh::Mesh;
use crate::material::Material;
use gpu_allocator::vulkan::*;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct Reactor {
    pub context: VulkanContext,
    pub swapchain: Swapchain,
    pub allocator: Arc<Mutex<Allocator>>,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

impl Reactor {
    pub fn init(window: &Window) -> Result<Self, Box<dyn Error>> {
        let context = VulkanContext::new(window)?;
        
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: context.instance.clone(),
            device: context.device.clone(),
            physical_device: context.physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false,
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

        // Command Buffer
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { context.device.allocate_command_buffers(&alloc_info)?[0] };

        // Sync Objects
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let image_available_semaphore = unsafe { context.device.create_semaphore(&semaphore_info, None)? };
        let render_finished_semaphore = unsafe { context.device.create_semaphore(&semaphore_info, None)? };
        let in_flight_fence = unsafe { context.device.create_fence(&fence_info, None)? };

        Ok(Self { 
            context, 
            swapchain,
            allocator,
            render_pass,
            framebuffers,
            command_pool,
            command_buffer,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        })
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

    pub fn draw_frame(&mut self, mesh: &Mesh, material: &Material) -> Result<(), Box<dyn Error>> {
        unsafe {
            self.context.device.wait_for_fences(&[self.in_flight_fence], true, u64::MAX)?;
            self.context.device.reset_fences(&[self.in_flight_fence])?;
        }

        let (image_index, _) = unsafe {
            match self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            ) {
                Ok(result) => result,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    // Recreate swapchain (TODO)
                    return Ok(());
                }
                Err(e) => return Err(Box::new(e)),
            }
        };

        // Record command buffer
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
            self.context.device.begin_command_buffer(self.command_buffer, &begin_info)?;
            
            self.context.device.cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            
            self.context.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                material.pipeline.pipeline,
            );

            let vertex_buffers = [mesh.vertex_buffer.handle];
            let offsets = [0];
            self.context.device.cmd_bind_vertex_buffers(self.command_buffer, 0, &vertex_buffers, &offsets);
            
            self.context.device.cmd_bind_index_buffer(
                self.command_buffer, 
                mesh.index_buffer.handle, 
                0, 
                vk::IndexType::UINT32
            );

            self.context.device.cmd_draw_indexed(self.command_buffer, mesh.index_count, 1, 0, 0, 0);
            
            self.context.device.cmd_end_render_pass(self.command_buffer);
            
            self.context.device.end_command_buffer(self.command_buffer)?;
        }

        // Submit
        let wait_semaphores = [self.image_available_semaphore];
        let signal_semaphores = [self.render_finished_semaphore];
        let command_buffers = [self.command_buffer];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.context.device.queue_submit(
                self.context.graphics_queue,
                &[submit_info],
                self.in_flight_fence,
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

        match result {
            Ok(_) => Ok(()),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                // Recreate swapchain (TODO)
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

            self.context.device.destroy_semaphore(self.image_available_semaphore, None);
            self.context.device.destroy_semaphore(self.render_finished_semaphore, None);
            self.context.device.destroy_fence(self.in_flight_fence, None);
            
            self.context.device.destroy_command_pool(self.command_pool, None);
        }
        self.swapchain.destroy(&self.context.device);
        // Allocator is dropped automatically by Arc/Mutex
    }
}
