use ash::vk;
use crate::core::context::VulkanContext;
use std::error::Error;

pub struct Sampler {
    pub handle: vk::Sampler,
    device: ash::Device,
}

#[derive(Clone, Copy)]
pub enum FilterMode {
    Nearest,
    Linear,
    Cubic,
}

#[derive(Clone, Copy)]
pub enum WrapMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

pub struct SamplerConfig {
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_mode: FilterMode,
    pub address_mode: WrapMode,
    pub anisotropy: Option<f32>,
    pub max_lod: f32,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_mode: FilterMode::Linear,
            address_mode: WrapMode::Repeat,
            anisotropy: Some(16.0),
            max_lod: 12.0,
        }
    }
}

impl Sampler {
    pub fn new(ctx: &VulkanContext, config: &SamplerConfig) -> Result<Self, Box<dyn Error>> {
        let mag_filter = match config.mag_filter {
            FilterMode::Nearest => vk::Filter::NEAREST,
            FilterMode::Linear | FilterMode::Cubic => vk::Filter::LINEAR,
        };

        let min_filter = match config.min_filter {
            FilterMode::Nearest => vk::Filter::NEAREST,
            FilterMode::Linear | FilterMode::Cubic => vk::Filter::LINEAR,
        };

        let mipmap_mode = match config.mipmap_mode {
            FilterMode::Nearest => vk::SamplerMipmapMode::NEAREST,
            FilterMode::Linear | FilterMode::Cubic => vk::SamplerMipmapMode::LINEAR,
        };

        let address_mode = match config.address_mode {
            WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
            WrapMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
            WrapMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            WrapMode::ClampToBorder => vk::SamplerAddressMode::CLAMP_TO_BORDER,
        };

        let (anisotropy_enable, max_anisotropy) = match config.anisotropy {
            Some(level) => (true, level),
            None => (false, 1.0),
        };

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(mag_filter)
            .min_filter(min_filter)
            .mipmap_mode(mipmap_mode)
            .address_mode_u(address_mode)
            .address_mode_v(address_mode)
            .address_mode_w(address_mode)
            .anisotropy_enable(anisotropy_enable)
            .max_anisotropy(max_anisotropy)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .min_lod(0.0)
            .max_lod(config.max_lod)
            .mip_lod_bias(0.0);

        let handle = unsafe { ctx.device.create_sampler(&sampler_info, None)? };

        Ok(Self {
            handle,
            device: ctx.device.clone(),
        })
    }

    pub fn linear(ctx: &VulkanContext) -> Result<Self, Box<dyn Error>> {
        Self::new(ctx, &SamplerConfig::default())
    }

    pub fn nearest(ctx: &VulkanContext) -> Result<Self, Box<dyn Error>> {
        Self::new(ctx, &SamplerConfig {
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_mode: FilterMode::Nearest,
            anisotropy: None,
            ..Default::default()
        })
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_sampler(self.handle, None);
        }
    }
}
