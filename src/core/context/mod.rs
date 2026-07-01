pub mod api;
pub mod builder;

pub use api::*;

use crate::core::arc_handle::{ArcDevice, ArcInstance, ArcSurface};
use crate::core::debug_utils::DebugNamer;
use crate::core::vrs::{VrsCapabilities, VrsContext};
use ash::vk;

#[derive(Clone)]
pub struct VulkanContext {
    pub device: ArcDevice,
    pub surface: ArcSurface,
    pub instance: ArcInstance,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub compute_queue: Option<vk::Queue>,
    pub transfer_queue: Option<vk::Queue>,
    pub queue_family_index: u32,
    pub compute_queue_family_index: Option<u32>,
    pub transfer_queue_family_index: Option<u32>,
    pub debug_namer: DebugNamer,
    pub has_memory_budget: bool,
    pub fragment_shading_rate: Option<VrsContext>,
    pub vrs_capabilities: VrsCapabilities,
}
