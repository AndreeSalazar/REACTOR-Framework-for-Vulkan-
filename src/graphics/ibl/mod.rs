pub mod baker;
pub(crate) mod compute_pass;
pub(crate) mod create;
pub(crate) mod helpers;
pub mod image;
pub(crate) mod sky;
pub mod textures;
pub(crate) mod upload;

pub use baker::IblBaker;
pub use image::IblImage;
pub use textures::IblTextures;

use crate::core::error::{ErrorCode, ReactorError};
use ash::vk;

// ── Config ───────────────────────────────────────────────────────────────────

pub const IBL_RADIANCE_SIZE: u32 = 1024;
pub const IBL_IRRADIANCE_SIZE: u32 = 32;
pub const IBL_PREFILTER_SIZE: u32 = 128;
pub const IBL_PREFILTER_MIPS: u32 = 5;
pub const IBL_BRDF_LUT_SIZE: u32 = 512;

pub(crate) const RGBA16F: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
pub(crate) const RG16F: vk::Format = vk::Format::R16G16_SFLOAT;

// ── Bytecode SPIR-V ──────────────────────────────────────────────────────────

pub(crate) const SPV_EQUIRECT_TO_CUBE: &[u8] = include_bytes!("../../../shaders/ibl/equirect_to_cube.spv");
pub(crate) const SPV_IRRADIANCE: &[u8] = include_bytes!("../../../shaders/ibl/irradiance.spv");
pub(crate) const SPV_PREFILTER: &[u8] = include_bytes!("../../../shaders/ibl/prefilter.spv");
pub(crate) const SPV_BRDF_LUT: &[u8] = include_bytes!("../../../shaders/ibl/brdf_lut.spv");

// ── Error helper ─────────────────────────────────────────────────────────────

pub(crate) fn verr<E: std::fmt::Display>(e: E) -> ReactorError {
    ReactorError::new(
        ErrorCode::VulkanImageCreation,
        format!("IBL Vulkan error: {}", e),
    )
}
