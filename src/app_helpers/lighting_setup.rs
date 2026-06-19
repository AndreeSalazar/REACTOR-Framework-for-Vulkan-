//! One-call lighting setup helpers.

use crate::app::ReactorContext;
use crate::systems::lighting::Light;
use glam::Vec3;

pub struct LightingSetup<'a> {
    ctx: &'a mut ReactorContext,
}

impl<'a> LightingSetup<'a> {
    pub fn new(ctx: &'a mut ReactorContext) -> Self {
        Self { ctx }
    }

    pub fn ctx(&mut self) -> &mut ReactorContext {
        self.ctx
    }

    /// Add a directional light (sun-like).
    pub fn sun(&mut self, direction: Vec3, color: Vec3, intensity: f32) -> &mut Self {
        self.ctx
            .lighting
            .add_light(Light::directional(direction, color, intensity));
        self
    }

    /// Add a point light at a position.
    pub fn point(&mut self, position: Vec3, color: Vec3, intensity: f32, range: f32) -> &mut Self {
        self.ctx
            .lighting
            .add_light(Light::point(position, color, intensity, range));
        self
    }

    /// Add a spot light.
    #[allow(clippy::too_many_arguments)]
    pub fn spot(
        &mut self,
        position: Vec3,
        direction: Vec3,
        color: Vec3,
        intensity: f32,
        range: f32,
        angle_degrees: f32,
    ) -> &mut Self {
        self.ctx.lighting.add_light(Light::spot(
            position,
            direction,
            color,
            intensity,
            range,
            angle_degrees,
        ));
        self
    }

    /// Default 3-point lighting: warm key from above-left, cool fill from
    /// right, cool back rim. Good for most showcase scenes.
    pub fn default_three_point(&mut self) -> &mut Self {
        self.sun(
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Vec3::new(1.0, 0.98, 0.95),
            1.0,
        )
        .point(
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::new(0.3, 0.3, 0.4),
            0.3,
            10.0,
        )
        .point(
            Vec3::new(-1.5, 0.5, -1.0),
            Vec3::new(0.2, 0.2, 0.5),
            0.2,
            8.0,
        )
    }
}
