//! Math helpers and utilities
//!
//! Re-exports glam types and provides additional math functions.

pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

/// Convert degrees to radians
#[inline]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg.to_radians()
}

/// Convert radians to degrees
#[inline]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad.to_degrees()
}

/// Clamp a value between min and max
#[inline]
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Linear interpolation between two values
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Smoothstep interpolation
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
