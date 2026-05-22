//! Light types and management
//! 
//! Defines light types used in the scene graph.

use glam::Vec3;

/// Type of light source
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    /// Directional light (sun/moon)
    Directional,
    /// Point light (omnidirectional)
    Point,
    /// Spot light (conical)
    Spot,
}

/// Light representation in the scene
#[derive(Debug, Clone)]
pub struct Light {
    /// Type of this light
    pub light_type: LightType,
    /// Position in world space (ignored for directional lights)
    pub position: Vec3,
    /// Direction in world space (for directional and spot lights)
    pub direction: Vec3,
    /// Color of the light
    pub color: Vec3,
    /// Intensity/brightness of the light
    pub intensity: f32,
    /// Range/attenuation (for point and spot lights)
    pub range: f32,
    /// Spot angle in degrees (for spot lights)
    pub spot_angle: f32,
}

impl Light {
    /// Create a new directional light (like the sun)
    pub fn directional(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional,
            position: Vec3::ZERO,
            direction: direction.normalize(),
            color,
            intensity,
            range: 0.0,
            spot_angle: 0.0,
        }
    }

    /// Create a new point light
    pub fn point(position: Vec3, color: Vec3, intensity: f32, range: f32) -> Self {
        Self {
            light_type: LightType::Point,
            position,
            direction: Vec3::ZERO,
            color,
            intensity,
            range,
            spot_angle: 0.0,
        }
    }

    /// Create a new spot light
    pub fn spot(
        position: Vec3,
        direction: Vec3,
        color: Vec3,
        intensity: f32,
        range: f32,
        angle_degrees: f32,
    ) -> Self {
        Self {
            light_type: LightType::Spot,
            position,
            direction: direction.normalize(),
            color,
            intensity,
            range,
            spot_angle: angle_degrees,
        }
    }
}
