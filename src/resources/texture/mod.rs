use crate::core::arc_handle::ArcDevice;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use crate::graphics::buffer::Buffer;
use crate::graphics::image::Image;
use crate::graphics::sampler::Sampler;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use gpu_allocator::MemoryLocation;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod upload;

pub struct Texture {
    pub image: Image,
    pub sampler: Sampler,
    pub width: u32,
    pub height: u32,
    #[allow(dead_code)]
    device: ArcDevice,
}

impl Texture {
    pub fn from_file<P: AsRef<Path>>(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        path: P,
        generate_mipmaps: bool,
    ) -> ReactorResult<Self> {
        let path_ref = path.as_ref();
        let img = image::open(path_ref).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::TextureLoadFailed,
                format!("Failed to open texture: {}", path_ref.display()),
                e,
            )
        })?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        Self::from_rgba(ctx, allocator, &data, width, height, generate_mipmaps)
    }

    pub fn from_file_linear<P: AsRef<Path>>(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        path: P,
        generate_mipmaps: bool,
    ) -> ReactorResult<Self> {
        let path_ref = path.as_ref();
        let img = image::open(path_ref).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::TextureLoadFailed,
                format!("Failed to open linear texture: {}", path_ref.display()),
                e,
            )
        })?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        Self::from_rgba_with_format(
            ctx,
            allocator,
            &data,
            width,
            height,
            generate_mipmaps,
            vk::Format::R8G8B8A8_UNORM,
        )
    }

    pub fn from_bytes(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        bytes: &[u8],
        generate_mipmaps: bool,
    ) -> ReactorResult<Self> {
        let img = image::load_from_memory(bytes).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::TextureLoadFailed,
                "Failed to load texture from bytes",
                e,
            )
        })?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        Self::from_rgba(ctx, allocator, &data, width, height, generate_mipmaps)
    }

    pub fn from_ktx2(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        bytes: &[u8],
    ) -> ReactorResult<Self> {
        let reader = ktx2::Reader::new(bytes)
            .map_err(|e| ReactorError::internal(format!("Failed to parse KTX2: {:?}", e)))?;

        let header = reader.header();
        let width = header.pixel_width;
        let height = header.pixel_height;

        let data = reader
            .levels()
            .next()
            .ok_or_else(|| ReactorError::internal("KTX2 has no mipmap levels"))?;

        Self::from_rgba(ctx, allocator, data, width, height, false)
    }

    pub fn solid_color(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> ReactorResult<Self> {
        let data = [r, g, b, a];
        Self::from_rgba(ctx, allocator, &data, 1, 1, false)
    }

    pub fn white(ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>) -> ReactorResult<Self> {
        Self::solid_color(ctx, allocator, 255, 255, 255, 255)
    }

    pub fn black(ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>) -> ReactorResult<Self> {
        Self::solid_color(ctx, allocator, 0, 0, 0, 255)
    }

    pub fn default_normal(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
    ) -> ReactorResult<Self> {
        Self::solid_color(ctx, allocator, 128, 128, 255, 255)
    }

    pub fn neutral_lut(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
    ) -> ReactorResult<Self> {
        let width = 256;
        let height = 16;
        let mut data = Vec::with_capacity((width * height * 4) as usize);
        for row in 0..16 {
            for col in 0..256 {
                let x = (col % 16) as f32 / 15.0;
                let y = row as f32 / 15.0;
                let z = (col / 16) as f32 / 15.0;

                let r = (x * 255.0).round() as u8;
                let g = (y * 255.0).round() as u8;
                let b = (z * 255.0).round() as u8;
                let a = 255;

                data.push(r);
                data.push(g);
                data.push(b);
                data.push(a);
            }
        }

        Self::from_rgba_with_format(
            ctx,
            allocator,
            &data,
            width,
            height,
            false,
            vk::Format::R8G8B8A8_UNORM,
        )
    }

    pub fn from_rgba(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        data: &[u8],
        width: u32,
        height: u32,
        generate_mipmaps: bool,
    ) -> ReactorResult<Self> {
        Self::from_rgba_with_format(
            ctx,
            allocator,
            data,
            width,
            height,
            generate_mipmaps,
            vk::Format::R8G8B8A8_SRGB,
        )
    }

    pub fn from_rgba_with_format(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        data: &[u8],
        width: u32,
        height: u32,
        generate_mipmaps: bool,
        format: vk::Format,
    ) -> ReactorResult<Self> {
        let mip_levels = if generate_mipmaps {
            ((width.max(height) as f32).log2().floor() as u32) + 1
        } else {
            1
        };

        let image = Image::new_texture_with_format(
            ctx,
            allocator.clone(),
            width,
            height,
            format,
            mip_levels,
        )?;

        let buffer_size = (width * height * 4) as u64;
        let staging = Buffer::new(
            ctx,
            allocator.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;

        staging.write(data);

        upload::copy_buffer_to_image(ctx, staging.handle, image.handle, width, height)?;

        if generate_mipmaps && mip_levels > 1 {
            upload::generate_mipmaps(ctx, image.handle, width, height, mip_levels)?;
        } else {
            upload::transition_to_shader_read(ctx, image.handle, mip_levels)?;
        }

        let sampler = Sampler::linear(ctx)?;

        Ok(Self {
            image,
            sampler,
            width,
            height,
            device: ctx.device.clone(),
        })
    }

    pub fn view(&self) -> vk::ImageView {
        self.image.view
    }

    pub fn sampler_handle(&self) -> vk::Sampler {
        self.sampler.handle
    }
}
