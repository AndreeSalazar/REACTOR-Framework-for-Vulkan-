//! One-call camera setup helpers.

use crate::app::ReactorContext;
use glam::Vec3;

pub struct CameraSetup<'a> {
    ctx: &'a mut ReactorContext,
}

impl<'a> CameraSetup<'a> {
    pub fn new(ctx: &'a mut ReactorContext) -> Self {
        Self { ctx }
    }

    pub fn ctx(&mut self) -> &mut ReactorContext {
        self.ctx
    }

    /// Position the camera at `eye`, looking at `target`, with `fov` degrees
    /// vertical FOV. This is the most common 3D visualization setup.
    pub fn look_at(&mut self, eye: Vec3, target: Vec3, fov: f32) -> &mut Self {
        self.ctx.camera.position = eye;
        let dir = (target - eye).normalize_or_zero();
        let pitch = dir.y.asin();
        let yaw = dir.x.atan2(dir.z);
        self.ctx.camera.set_rotation(yaw, pitch);
        self.ctx.camera.fov = fov;
        self
    }

    /// Position + yaw/pitch directly (no look-at math).
    pub fn at(&mut self, position: Vec3, yaw: f32, pitch: f32, fov: f32) -> &mut Self {
        self.ctx.camera.position = position;
        self.ctx.camera.set_rotation(yaw, pitch);
        self.ctx.camera.fov = fov;
        self
    }
}
