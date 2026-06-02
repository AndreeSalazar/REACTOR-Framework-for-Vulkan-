use crate::core::{ReactorResult, VulkanContext};
use crate::graphics::Image;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GBufferAttachment {
    AlbedoAo,
    NormalMaterial,
    EmissiveMaterial,
    MotionDepthFlags,
}

impl GBufferAttachment {
    pub const ALL: [Self; 4] = [
        Self::AlbedoAo,
        Self::NormalMaterial,
        Self::EmissiveMaterial,
        Self::MotionDepthFlags,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::AlbedoAo => "GBuffer0_Albedo_AO",
            Self::NormalMaterial => "GBuffer1_Normal_Material",
            Self::EmissiveMaterial => "GBuffer2_Emissive_Material",
            Self::MotionDepthFlags => "GBuffer3_Motion_Depth_Flags",
        }
    }

    pub fn format(self) -> vk::Format {
        match self {
            Self::AlbedoAo => vk::Format::R8G8B8A8_UNORM,
            Self::NormalMaterial | Self::EmissiveMaterial | Self::MotionDepthFlags => {
                vk::Format::R16G16B16A16_SFLOAT
            }
        }
    }

    pub fn bytes_per_pixel(self) -> u64 {
        match self {
            Self::AlbedoAo => 4,
            Self::NormalMaterial | Self::EmissiveMaterial | Self::MotionDepthFlags => 8,
        }
    }
}

pub struct GBuffer {
    pub width: u32,
    pub height: u32,
    pub albedo_ao: Image,
    pub normal_material: Image,
    pub emissive_material: Image,
    pub motion_depth_flags: Image,
    pub storage_writes_supported: bool,
}

impl GBuffer {
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        width: u32,
        height: u32,
    ) -> ReactorResult<Self> {
        let storage_writes_supported = GBufferAttachment::ALL
            .iter()
            .all(|attachment| format_supports_storage(ctx, attachment.format()));

        let albedo_ao = create_attachment(
            ctx,
            allocator.clone(),
            width,
            height,
            GBufferAttachment::AlbedoAo,
            storage_writes_supported,
        )?;
        let normal_material = create_attachment(
            ctx,
            allocator.clone(),
            width,
            height,
            GBufferAttachment::NormalMaterial,
            storage_writes_supported,
        )?;
        let emissive_material = create_attachment(
            ctx,
            allocator.clone(),
            width,
            height,
            GBufferAttachment::EmissiveMaterial,
            storage_writes_supported,
        )?;
        let motion_depth_flags = create_attachment(
            ctx,
            allocator,
            width,
            height,
            GBufferAttachment::MotionDepthFlags,
            storage_writes_supported,
        )?;

        Ok(Self {
            width,
            height,
            albedo_ao,
            normal_material,
            emissive_material,
            motion_depth_flags,
            storage_writes_supported,
        })
    }

    pub fn views(&self) -> [vk::ImageView; 4] {
        [
            self.albedo_ao.view,
            self.normal_material.view,
            self.emissive_material.view,
            self.motion_depth_flags.view,
        ]
    }

    pub fn images(&self) -> [vk::Image; 4] {
        [
            self.albedo_ao.handle,
            self.normal_material.handle,
            self.emissive_material.handle,
            self.motion_depth_flags.handle,
        ]
    }

    pub fn formats(&self) -> [vk::Format; 4] {
        GBufferAttachment::ALL.map(|attachment| attachment.format())
    }

    pub fn sampled_descriptor_infos(&self, sampler: vk::Sampler) -> [vk::DescriptorImageInfo; 4] {
        self.views().map(|view| {
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(view)
                .sampler(sampler)
        })
    }

    pub fn color_attachment_infos(
        &self,
        clear_values: [vk::ClearColorValue; 4],
    ) -> [vk::RenderingAttachmentInfo<'_>; 4] {
        let views = self.views();
        std::array::from_fn(|index| {
            vk::RenderingAttachmentInfo::default()
                .image_view(views[index])
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue { color: clear_values[index] })
        })
    }

    pub fn estimated_bytes(&self) -> u64 {
        Self::estimated_bytes_for_extent(self.width, self.height)
    }

    pub fn estimated_bytes_for_extent(width: u32, height: u32) -> u64 {
        let bytes_per_pixel: u64 = GBufferAttachment::ALL
            .iter()
            .map(|attachment| attachment.bytes_per_pixel())
            .sum();
        width as u64 * height as u64 * bytes_per_pixel
    }

    pub fn estimated_mib_for_extent(width: u32, height: u32) -> f32 {
        Self::estimated_bytes_for_extent(width, height) as f32 / (1024.0 * 1024.0)
    }
}

fn create_attachment(
    ctx: &VulkanContext,
    allocator: Arc<Mutex<Allocator>>,
    width: u32,
    height: u32,
    attachment: GBufferAttachment,
    storage_writes_supported: bool,
) -> ReactorResult<Image> {
    let mut usage = vk::ImageUsageFlags::COLOR_ATTACHMENT
        | vk::ImageUsageFlags::SAMPLED
        | vk::ImageUsageFlags::TRANSFER_SRC;

    if storage_writes_supported {
        usage |= vk::ImageUsageFlags::STORAGE;
    }

    let image = Image::new(
        ctx,
        allocator,
        width,
        height,
        attachment.format(),
        usage,
        vk::ImageAspectFlags::COLOR,
        1,
    )?;

    ctx.debug_namer()
        .name_image(image.handle, &format!("Image: {}", attachment.name()));
    ctx.debug_namer()
        .name_image_view(image.view, &format!("ImageView: {}", attachment.name()));

    Ok(image)
}

fn format_supports_storage(ctx: &VulkanContext, format: vk::Format) -> bool {
    let props = unsafe {
        ctx.instance
            .get_physical_device_format_properties(ctx.physical_device, format)
    };
    props
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::STORAGE_IMAGE)
}
