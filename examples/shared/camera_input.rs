//! First-person / orbit camera input handler.

use reactor_vulkan::app::ReactorContext;
use winit::keyboard::KeyCode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CameraMode {
    Free,
    Orbit,
}

#[derive(Clone, Copy, Debug)]
pub struct CameraInputSettings {
    pub mode: CameraMode,
    pub move_speed: f32,
    #[allow(dead_code)]
    pub look_speed: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub exit_on_escape: bool,
}

impl Default for CameraInputSettings {
    fn default() -> Self {
        Self {
            mode: CameraMode::Free,
            move_speed: 3.0,
            look_speed: 1.5,
            orbit_radius: 4.0,
            orbit_speed: 0.6,
            exit_on_escape: true,
        }
    }
}

pub struct CameraInput {
    pub settings: CameraInputSettings,
    orbit_angle: f32,
    orbit_pitch: f32,
}

impl Default for CameraInput {
    fn default() -> Self {
        Self::new(CameraInputSettings::default())
    }
}

impl CameraInput {
    pub fn new(settings: CameraInputSettings) -> Self {
        Self {
            settings,
            orbit_angle: 0.0,
            orbit_pitch: 0.3,
        }
    }

    pub fn update(&mut self, ctx: &mut ReactorContext) {
        if self.settings.exit_on_escape && ctx.input().is_key_down(KeyCode::Escape) {
            ctx.reactor.exit_requested = true;
        }

        let dt = ctx.time.delta();

        match self.settings.mode {
            CameraMode::Free => self.update_free(ctx, dt),
            CameraMode::Orbit => self.update_orbit(ctx, dt),
        }
    }

    fn update_free(&mut self, ctx: &mut ReactorContext, dt: f32) {
        let speed = self.settings.move_speed * dt;
        if ctx.input().is_key_down(KeyCode::KeyW) {
            ctx.camera.position.z -= speed;
        }
        if ctx.input().is_key_down(KeyCode::KeyS) {
            ctx.camera.position.z += speed;
        }
        if ctx.input().is_key_down(KeyCode::KeyA) {
            ctx.camera.position.x -= speed;
        }
        if ctx.input().is_key_down(KeyCode::KeyD) {
            ctx.camera.position.x += speed;
        }
        if ctx.input().is_key_down(KeyCode::Space) {
            ctx.camera.position.y += speed;
        }
        if ctx.input().is_key_down(KeyCode::ShiftLeft) {
            ctx.camera.position.y -= speed;
        }
    }

    fn update_orbit(&mut self, ctx: &mut ReactorContext, dt: f32) {
        self.orbit_angle += dt * self.settings.orbit_speed;
        let r = self.settings.orbit_radius;
        ctx.camera.position = glam::Vec3::new(
            r * self.orbit_angle.cos() * self.orbit_pitch.cos(),
            r * self.orbit_pitch.sin(),
            r * self.orbit_angle.sin() * self.orbit_pitch.cos(),
        );
    }
}
