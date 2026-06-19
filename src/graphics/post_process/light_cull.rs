//! Light culling — clustered/tiled light culling compute pass
//!
//! Wraps `shaders/compute/light_cull.comp`.
//! The shader expects `PointLight { position_radius: vec4, color_intensity: vec4 }`
//! = 32 bytes per light, layout-compatible with `PointLightGpu` defined here.
//!
//! The Rust-side `LightData` (48 bytes) is a different layout used for the
//! forward path's uniform buffer. This module converts scene lights into the
//! shader-compatible `PointLightGpu` for the culling pass.

use crate::systems::lighting::{Light, LightType};
use ash::vk;
use bytemuck::{Pod, Zeroable};

/// Layout-compatible with `PointLight` in `light_cull.comp`. 32 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PointLightGpu {
    pub position_radius: [f32; 4],
    pub color_intensity: [f32; 4],
}

impl PointLightGpu {
    pub fn from_light(light: &Light) -> Option<Self> {
        match light.light_type {
            LightType::Point => Some(Self {
                position_radius: [
                    light.position.x,
                    light.position.y,
                    light.position.z,
                    light.range,
                ],
                color_intensity: [
                    light.color.x * light.intensity,
                    light.color.y * light.intensity,
                    light.color.z * light.intensity,
                    light.intensity,
                ],
            }),
            LightType::Spot => {
                let cos_outer = (light.spot_angle * 0.5).cos();
                Some(Self {
                    position_radius: [
                        light.position.x,
                        light.position.y,
                        light.position.z,
                        light.range,
                    ],
                    color_intensity: [
                        light.color.x * light.intensity,
                        light.color.y * light.intensity,
                        light.color.z * light.intensity,
                        light.intensity * cos_outer,
                    ],
                })
            }
            LightType::Directional => None,
        }
    }
}

pub fn lights_to_gpu_buffer(lights: &[Light], out: &mut Vec<PointLightGpu>) {
    out.clear();
    for light in lights {
        if !light.enabled {
            continue;
        }
        if let Some(p) = PointLightGpu::from_light(light) {
            out.push(p);
        }
    }
}

#[allow(dead_code)]
pub(crate) fn _suppress_unused_vk_warning() -> vk::BufferUsageFlags {
    vk::BufferUsageFlags::STORAGE_BUFFER
}
