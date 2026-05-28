use crate::core::arc_handle::ArcDevice;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};

pub struct Image {
    pub handle: vk::Image,
    pub view: vk::ImageView,
    pub allocation: Option<Allocation>,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub mip_levels: u32,
    device: ArcDevice,
    allocator: Arc<Mutex<Allocator>>,
}

impl Image {
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        aspect: vk::ImageAspectFlags,
        mip_levels: u32,
    ) -> ReactorResult<Self> {
        let device = ctx.ash_device();
        let extent = vk::Extent3D { width, height, depth: 1 };

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(extent)
            .mip_levels(mip_levels)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let handle = unsafe {
            device.create_image(&image_info, None).map_err(|e| {
                ReactorError::with_source(ErrorCode::VulkanImageCreation, "create_image failed", e)
            })?
        };
        let requirements = unsafe { device.get_image_memory_requirements(handle) };

        let allocation = allocator
            .lock()
            .unwrap()
            .allocate(&AllocationCreateDesc {
                name: "image",
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: false,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanMemoryAllocation,
                    "image allocation failed",
                    e,
                )
            })?;

        unsafe {
            device
                .bind_image_memory(handle, allocation.memory(), allocation.offset())
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanImageCreation,
                        "bind_image_memory failed",
                        e,
                    )
                })?;
        }

        let view_info = vk::ImageViewCreateInfo::default()
            .image(handle)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect)
                    .base_mip_level(0)
                    .level_count(mip_levels)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        let view = unsafe {
            device.create_image_view(&view_info, None).map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanImageCreation,
                    "create_image_view failed",
                    e,
                )
            })?
        };

        Ok(Self {
            handle,
            view,
            allocation: Some(allocation),
            format,
            extent,
            mip_levels,
            device: ctx.device.clone(),
            allocator,
        })
    }

    pub fn new_texture(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> ReactorResult<Self> {
        Self::new(
            ctx,
            allocator,
            width,
            height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::ImageAspectFlags::COLOR,
            mip_levels,
        )
    }

    pub fn new_depth(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
    ) -> ReactorResult<Self> {
        Self::new(
            ctx,
            allocator,
            width,
            height,
            vk::Format::D32_SFLOAT,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::ImageAspectFlags::DEPTH,
            1,
        )
    }

    pub fn transition_layout(
        &self,
        ctx: &VulkanContext,
        command_buffer: vk::CommandBuffer,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let (src_access, dst_access, src_stage, dst_stage) = match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            ),
            (vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            (vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
                vk::AccessFlags::SHADER_READ,
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            ),
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::empty(),
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            ),
        };

        let aspect = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            || old_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        {
            vk::ImageAspectFlags::DEPTH
        } else {
            vk::ImageAspectFlags::COLOR
        };

        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect)
                    .base_mip_level(0)
                    .level_count(self.mip_levels)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .src_access_mask(src_access)
            .dst_access_mask(dst_access);

        unsafe {
            ctx.device.cmd_pipeline_barrier(
                command_buffer,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.handle, None);
        }
        if let Some(allocation) = self.allocation.take() {
            if let Err(e) = self.allocator.lock().unwrap().free(allocation) {
                eprintln!("Failed to free image memory: {:?}", e);
            }
        }
    }
}
