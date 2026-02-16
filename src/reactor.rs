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
    
    // MSAA Anti-Aliasing
    pub msaa_samples: vk::SampleCountFlags,
    pub msaa_image: Option<vk::Image>,
    pub msaa_image_view: Option<vk::ImageView>,
    pub msaa_memory: Option<vk::DeviceMemory>,
    
    // Depth Buffer
    pub depth_image: Option<vk::Image>,
    pub depth_image_view: Option<vk::ImageView>,
    pub depth_memory: Option<vk::DeviceMemory>,
    pub depth_format: vk::Format,
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
        
        // Get MSAA samples (4x for good quality/performance balance)
        let msaa_samples = Self::get_max_msaa_samples_static(&context);
        println!("🔷 MSAA: {:?} samples enabled for anti-aliasing", msaa_samples);
        
        // Create MSAA resources if supported
        let (msaa_image, msaa_image_view, msaa_memory) = if msaa_samples != vk::SampleCountFlags::TYPE_1 {
            let (img, view, mem) = Self::create_msaa_resources(
                &context, 
                swapchain.extent.width, 
                swapchain.extent.height, 
                swapchain.format,
                msaa_samples
            )?;
            (Some(img), Some(view), Some(mem))
        } else {
            (None, None, None)
        };
        
        // Create Depth Buffer
        let depth_format = Self::find_depth_format(&context)?;
        let (depth_image, depth_image_view, depth_memory) = Self::create_depth_resources(
            &context,
            swapchain.extent.width,
            swapchain.extent.height,
            depth_format,
            msaa_samples,
        )?;
        println!("🔷 Depth buffer created: {:?}", depth_format);
        
        let render_pass = Self::create_render_pass_with_depth(&context, swapchain.format, depth_format, msaa_samples)?;
        let framebuffers = Self::create_framebuffers_with_depth(&context, &swapchain, render_pass, msaa_image_view, depth_image_view, msaa_samples)?;

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
            // MSAA Anti-Aliasing
            msaa_samples,
            msaa_image,
            msaa_image_view,
            msaa_memory,
            // Depth Buffer
            depth_image: Some(depth_image),
            depth_image_view: Some(depth_image_view),
            depth_memory: Some(depth_memory),
            depth_format,
        })
    }

    /// Get maximum supported MSAA sample count
    pub fn get_max_msaa_samples(&self) -> vk::SampleCountFlags {
        let props = unsafe { 
            self.context.instance.get_physical_device_properties(self.context.physical_device) 
        };
        let counts = props.limits.framebuffer_color_sample_counts
            & props.limits.framebuffer_depth_sample_counts;

        if counts.contains(vk::SampleCountFlags::TYPE_8) { vk::SampleCountFlags::TYPE_8 }
        else if counts.contains(vk::SampleCountFlags::TYPE_4) { vk::SampleCountFlags::TYPE_4 }
        else if counts.contains(vk::SampleCountFlags::TYPE_2) { vk::SampleCountFlags::TYPE_2 }
        else { vk::SampleCountFlags::TYPE_1 }
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

        // Destroy old framebuffers
        unsafe {
            for &framebuffer in &self.framebuffers {
                self.context.device.destroy_framebuffer(framebuffer, None);
            }
        }
        
        // Destroy old depth resources
        if let Some(view) = self.depth_image_view.take() {
            unsafe { self.context.device.destroy_image_view(view, None); }
        }
        if let Some(image) = self.depth_image.take() {
            unsafe { self.context.device.destroy_image(image, None); }
        }
        if let Some(memory) = self.depth_memory.take() {
            unsafe { self.context.device.free_memory(memory, None); }
        }
        
        // Destroy old MSAA resources if they exist
        if let Some(view) = self.msaa_image_view.take() {
            unsafe { self.context.device.destroy_image_view(view, None); }
        }
        if let Some(image) = self.msaa_image.take() {
            unsafe { self.context.device.destroy_image(image, None); }
        }
        if let Some(memory) = self.msaa_memory.take() {
            unsafe { self.context.device.free_memory(memory, None); }
        }
        
        self.swapchain.destroy(&self.context.device);

        self.swapchain = Swapchain::new(&self.context, capabilities.current_extent.width, capabilities.current_extent.height)?;
        
        // Recreate MSAA resources if MSAA is enabled
        if self.msaa_samples != vk::SampleCountFlags::TYPE_1 {
            let (img, view, mem) = Self::create_msaa_resources(
                &self.context,
                self.swapchain.extent.width,
                self.swapchain.extent.height,
                self.swapchain.format,
                self.msaa_samples
            )?;
            self.msaa_image = Some(img);
            self.msaa_image_view = Some(view);
            self.msaa_memory = Some(mem);
        }
        
        // Recreate depth buffer
        let (depth_img, depth_view, depth_mem) = Self::create_depth_resources(
            &self.context,
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            self.depth_format,
            self.msaa_samples,
        )?;
        self.depth_image = Some(depth_img);
        self.depth_image_view = Some(depth_view);
        self.depth_memory = Some(depth_mem);
        
        // Recreate framebuffers with depth support
        self.framebuffers = Self::create_framebuffers_with_depth(
            &self.context, 
            &self.swapchain, 
            self.render_pass,
            self.msaa_image_view,
            depth_view,
            self.msaa_samples
        )?;
        
        Ok(())
    }

    pub fn create_mesh(&self, vertices: &[crate::vertex::Vertex], indices: &[u32]) -> Result<Mesh, Box<dyn Error>> {
        Mesh::new(&self.context, &self.allocator, vertices, indices)
    }

    /// Load texture from file (PNG, JPG, BMP, etc.)
    pub fn load_texture(&self, path: &str) -> Result<crate::resources::texture::Texture, Box<dyn Error>> {
        crate::resources::texture::Texture::from_file(&self.context, self.allocator.clone(), path, true)
    }

    /// Load texture from embedded bytes
    pub fn load_texture_bytes(&self, bytes: &[u8]) -> Result<crate::resources::texture::Texture, Box<dyn Error>> {
        crate::resources::texture::Texture::from_bytes(&self.context, self.allocator.clone(), bytes, true)
    }

    /// Create a solid color texture
    pub fn create_solid_texture(&self, r: u8, g: u8, b: u8, a: u8) -> Result<crate::resources::texture::Texture, Box<dyn Error>> {
        crate::resources::texture::Texture::solid_color(&self.context, self.allocator.clone(), r, g, b, a)
    }

    pub fn create_material(&self, vert_code: &[u32], frag_code: &[u32]) -> Result<Material, Box<dyn Error>> {
        Material::new_with_msaa(
            &self.context, 
            self.render_pass, 
            vert_code, 
            frag_code, 
            self.swapchain.extent.width, 
            self.swapchain.extent.height,
            self.msaa_samples
        )
    }

    fn create_render_pass(context: &VulkanContext, format: vk::Format) -> Result<vk::RenderPass, Box<dyn Error>> {
        // TODO: MSAA requires creating MSAA image buffers and modifying framebuffers
        // For now, use simple render pass. MSAA can be enabled by:
        // 1. Creating MSAA color buffer with MsaaTarget
        // 2. Using create_render_pass_msaa
        // 3. Modifying framebuffers to include MSAA attachment
        // 4. Modifying pipeline to use MSAA samples
        Self::create_render_pass_simple(context, format)
    }

    fn get_max_msaa_samples_static(context: &VulkanContext) -> vk::SampleCountFlags {
        let props = unsafe { 
            context.instance.get_physical_device_properties(context.physical_device) 
        };
        let counts = props.limits.framebuffer_color_sample_counts
            & props.limits.framebuffer_depth_sample_counts;

        // Use 4x MSAA as default (good balance of quality/performance)
        if counts.contains(vk::SampleCountFlags::TYPE_4) { vk::SampleCountFlags::TYPE_4 }
        else if counts.contains(vk::SampleCountFlags::TYPE_2) { vk::SampleCountFlags::TYPE_2 }
        else { vk::SampleCountFlags::TYPE_1 }
    }

    fn create_render_pass_simple(context: &VulkanContext, format: vk::Format) -> Result<vk::RenderPass, Box<dyn Error>> {
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

    fn create_render_pass_msaa(context: &VulkanContext, format: vk::Format, samples: vk::SampleCountFlags) -> Result<vk::RenderPass, Box<dyn Error>> {
        println!("🔷 Enabling MSAA {:?} for anti-aliasing", samples);
        
        // MSAA color attachment (multisampled)
        let msaa_attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE) // Don't need to store MSAA
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        // Resolve attachment (single sample, for presentation)
        let resolve_attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let resolve_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments = [color_attachment_ref];
        let resolve_attachments = [resolve_attachment_ref];

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .resolve_attachments(&resolve_attachments);

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let attachments = [msaa_attachment, resolve_attachment];
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

    /// Create MSAA color buffer resources
    fn create_msaa_resources(
        context: &VulkanContext,
        width: u32,
        height: u32,
        format: vk::Format,
        samples: vk::SampleCountFlags,
    ) -> Result<(vk::Image, vk::ImageView, vk::DeviceMemory), Box<dyn Error>> {
        // Create MSAA image
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(samples);

        let image = unsafe { context.device.create_image(&image_info, None)? };
        let requirements = unsafe { context.device.get_image_memory_requirements(image) };

        // Find memory type
        let memory_props = unsafe { context.instance.get_physical_device_memory_properties(context.physical_device) };
        let memory_type_index = (0..memory_props.memory_type_count)
            .find(|&i| {
                let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
                let memory_type = memory_props.memory_types[i as usize];
                suitable && memory_type.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })
            .ok_or("Failed to find suitable memory type for MSAA")?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe { context.device.allocate_memory(&alloc_info, None)? };
        unsafe { context.device.bind_image_memory(image, memory, 0)? };

        // Create image view
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        let view = unsafe { context.device.create_image_view(&view_info, None)? };

        Ok((image, view, memory))
    }

    /// Create render pass with MSAA support
    fn create_render_pass_with_msaa(
        context: &VulkanContext, 
        format: vk::Format,
        samples: vk::SampleCountFlags,
    ) -> Result<vk::RenderPass, Box<dyn Error>> {
        if samples == vk::SampleCountFlags::TYPE_1 {
            // No MSAA - use simple render pass
            return Self::create_render_pass_simple(context, format);
        }

        // MSAA color attachment (multisampled)
        let msaa_attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        // Resolve attachment (single sample, for presentation)
        let resolve_attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let resolve_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments = [color_attachment_ref];
        let resolve_attachments = [resolve_attachment_ref];

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .resolve_attachments(&resolve_attachments);

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let attachments = [msaa_attachment, resolve_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass = unsafe { context.device.create_render_pass(&render_pass_info, None)? };
        Ok(render_pass)
    }

    /// Create framebuffers with MSAA support
    fn create_framebuffers_msaa(
        context: &VulkanContext, 
        swapchain: &Swapchain, 
        render_pass: vk::RenderPass,
        msaa_view: Option<vk::ImageView>,
        samples: vk::SampleCountFlags,
    ) -> Result<Vec<vk::Framebuffer>, Box<dyn Error>> {
        println!("🔷 Creating framebuffers: samples={:?}, msaa_view={:?}", samples, msaa_view.is_some());
        
        if samples == vk::SampleCountFlags::TYPE_1 {
            // No MSAA - use simple framebuffers
            println!("🔷 Using simple framebuffers (no MSAA)");
            return Self::create_framebuffers(context, swapchain, render_pass);
        }

        let msaa_view = match msaa_view {
            Some(view) => view,
            None => {
                println!("⚠️ MSAA requested but no MSAA view available, falling back to simple");
                return Self::create_framebuffers(context, swapchain, render_pass);
            }
        };
        
        println!("🔷 Creating MSAA framebuffers with 2 attachments");

        swapchain.image_views
            .iter()
            .map(|&resolve_view| {
                // MSAA framebuffer has 2 attachments: MSAA color + resolve
                let attachments = [msaa_view, resolve_view];
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
        
        // Clear values: [MSAA color, resolve color, depth]
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            },
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
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

    /// Find a supported depth format
    fn find_depth_format(context: &VulkanContext) -> Result<vk::Format, Box<dyn Error>> {
        let candidates = [
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];

        for &format in &candidates {
            let props = unsafe {
                context.instance.get_physical_device_format_properties(context.physical_device, format)
            };
            if props.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
                return Ok(format);
            }
        }

        Err("Failed to find supported depth format".into())
    }

    /// Create depth buffer resources
    fn create_depth_resources(
        context: &VulkanContext,
        width: u32,
        height: u32,
        format: vk::Format,
        samples: vk::SampleCountFlags,
    ) -> Result<(vk::Image, vk::ImageView, vk::DeviceMemory), Box<dyn Error>> {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(samples);

        let image = unsafe { context.device.create_image(&image_info, None)? };
        let requirements = unsafe { context.device.get_image_memory_requirements(image) };

        let memory_props = unsafe { context.instance.get_physical_device_memory_properties(context.physical_device) };
        let memory_type_index = (0..memory_props.memory_type_count)
            .find(|&i| {
                let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
                let memory_type = memory_props.memory_types[i as usize];
                suitable && memory_type.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })
            .ok_or("Failed to find suitable memory type for depth buffer")?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe { context.device.allocate_memory(&alloc_info, None)? };
        unsafe { context.device.bind_image_memory(image, memory, 0)? };

        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        let view = unsafe { context.device.create_image_view(&view_info, None)? };

        Ok((image, view, memory))
    }

    /// Create render pass with MSAA and depth support
    fn create_render_pass_with_depth(
        context: &VulkanContext,
        color_format: vk::Format,
        depth_format: vk::Format,
        samples: vk::SampleCountFlags,
    ) -> Result<vk::RenderPass, Box<dyn Error>> {
        if samples == vk::SampleCountFlags::TYPE_1 {
            // No MSAA - simple render pass with depth
            let color_attachment = vk::AttachmentDescription::default()
                .format(color_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

            let depth_attachment = vk::AttachmentDescription::default()
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

            let color_ref = vk::AttachmentReference::default()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

            let depth_ref = vk::AttachmentReference::default()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

            let color_attachments = [color_ref];
            let subpass = vk::SubpassDescription::default()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachments)
                .depth_stencil_attachment(&depth_ref);

            let dependency = vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

            let attachments = [color_attachment, depth_attachment];
            let subpasses = [subpass];
            let dependencies = [dependency];

            let render_pass_info = vk::RenderPassCreateInfo::default()
                .attachments(&attachments)
                .subpasses(&subpasses)
                .dependencies(&dependencies);

            return unsafe { Ok(context.device.create_render_pass(&render_pass_info, None)?) };
        }

        // MSAA with depth
        let msaa_color = vk::AttachmentDescription::default()
            .format(color_format)
            .samples(samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let resolve_color = vk::AttachmentDescription::default()
            .format(color_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let depth_attachment = vk::AttachmentDescription::default()
            .format(depth_format)
            .samples(samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let resolve_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_ref = vk::AttachmentReference::default()
            .attachment(2)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_attachments = [color_ref];
        let resolve_attachments = [resolve_ref];
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .resolve_attachments(&resolve_attachments)
            .depth_stencil_attachment(&depth_ref);

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

        let attachments = [msaa_color, resolve_color, depth_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        unsafe { Ok(context.device.create_render_pass(&render_pass_info, None)?) }
    }

    /// Create framebuffers with depth support
    fn create_framebuffers_with_depth(
        context: &VulkanContext,
        swapchain: &Swapchain,
        render_pass: vk::RenderPass,
        msaa_view: Option<vk::ImageView>,
        depth_view: vk::ImageView,
        samples: vk::SampleCountFlags,
    ) -> Result<Vec<vk::Framebuffer>, Box<dyn Error>> {
        if samples == vk::SampleCountFlags::TYPE_1 {
            // No MSAA: [color, depth]
            return swapchain.image_views
                .iter()
                .map(|&color_view| {
                    let attachments = [color_view, depth_view];
                    let framebuffer_info = vk::FramebufferCreateInfo::default()
                        .render_pass(render_pass)
                        .attachments(&attachments)
                        .width(swapchain.extent.width)
                        .height(swapchain.extent.height)
                        .layers(1);
                    unsafe { context.device.create_framebuffer(&framebuffer_info, None).map_err(|e| e.into()) }
                })
                .collect();
        }

        // MSAA: [msaa_color, resolve_color, depth]
        let msaa_view = msaa_view.ok_or("MSAA view required for MSAA framebuffers")?;
        
        swapchain.image_views
            .iter()
            .map(|&resolve_view| {
                let attachments = [msaa_view, resolve_view, depth_view];
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
}

impl Drop for Reactor {
    fn drop(&mut self) {
        unsafe {
            // Wait for all GPU operations to complete
            let _ = self.context.device.device_wait_idle();
            
            // Destroy depth resources
            if let Some(depth_view) = self.depth_image_view.take() {
                self.context.device.destroy_image_view(depth_view, None);
            }
            if let Some(depth_image) = self.depth_image.take() {
                self.context.device.destroy_image(depth_image, None);
            }
            if let Some(depth_memory) = self.depth_memory.take() {
                self.context.device.free_memory(depth_memory, None);
            }
            
            // Destroy MSAA resources
            if let Some(msaa_view) = self.msaa_image_view.take() {
                self.context.device.destroy_image_view(msaa_view, None);
            }
            if let Some(msaa_image) = self.msaa_image.take() {
                self.context.device.destroy_image(msaa_image, None);
            }
            if let Some(msaa_memory) = self.msaa_memory.take() {
                self.context.device.free_memory(msaa_memory, None);
            }
            
            // Destroy framebuffers
            for &framebuffer in &self.framebuffers {
                self.context.device.destroy_framebuffer(framebuffer, None);
            }
            
            // Destroy render pass
            self.context.device.destroy_render_pass(self.render_pass, None);

            // Destroy sync objects
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.context.device.destroy_semaphore(self.image_available_semaphores[i], None);
                self.context.device.destroy_semaphore(self.render_finished_semaphores[i], None);
                self.context.device.destroy_fence(self.in_flight_fences[i], None);
            }
            
            // Destroy command pool
            self.context.device.destroy_command_pool(self.command_pool, None);
        }
        
        // Destroy swapchain
        self.swapchain.destroy(&self.context.device);
        
        // Allocator must be dropped before device
        // Force drop of allocator by taking ownership
        if let Ok(allocator) = self.allocator.lock() {
            // Allocator will be dropped when lock is released
            drop(allocator);
        }
    }
}
