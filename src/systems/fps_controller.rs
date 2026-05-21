// =============================================================================
// fps_controller.rs — Free-look FPS camera controller
// =============================================================================
// Heredable / componible: instancia uno en tu juego y llámalo cada frame.
//   let mut fps = FpsController::default();
//   fps.update(&mut ctx);
// Controles por defecto:
//   WASD          → mover horizontalmente
//   Space / Ctrl  → subir / bajar
//   Shift         → boost de velocidad
//   Flechas       → look (yaw / pitch)
// =============================================================================

use glam::Vec3;
use winit::keyboard::KeyCode;

use crate::app::ReactorContext;

#[derive(Debug, Clone)]
pub struct FpsController {
    pub move_speed: f32,
    pub boost_multiplier: f32,
    pub look_speed: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub yaw: f32,
    pub pitch: f32,
    /// Atajo opcional para cerrar la app (None desactiva).
    pub quit_key: Option<KeyCode>,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            move_speed: 6.0,
            boost_multiplier: 3.0,
            look_speed: 1.6,
            min_pitch: -1.4, // ~ -80°
            max_pitch:  1.4, // ~ +80°
            yaw: 0.0,
            pitch: 0.0,
            quit_key: Some(KeyCode::Escape),
        }
    }
}

impl FpsController {
    /// Construye con velocidad personalizada.
    pub fn new(move_speed: f32, look_speed: f32) -> Self {
        Self { move_speed, look_speed, ..Self::default() }
    }

    /// Aplica input → cámara. Llama una vez por frame en `update`.
    pub fn update(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();

        // ── Salir ──
        if let Some(k) = self.quit_key {
            if ctx.input().is_key_down(k) {
                std::process::exit(0);
            }
        }

        // ── Look (flechas) ──
        let input = ctx.input();
        let mut yaw_delta = 0.0_f32;
        let mut pitch_delta = 0.0_f32;
        if input.is_key_down(KeyCode::ArrowLeft)  { yaw_delta   += self.look_speed * dt; }
        if input.is_key_down(KeyCode::ArrowRight) { yaw_delta   -= self.look_speed * dt; }
        if input.is_key_down(KeyCode::ArrowUp)    { pitch_delta += self.look_speed * dt; }
        if input.is_key_down(KeyCode::ArrowDown)  { pitch_delta -= self.look_speed * dt; }
        self.yaw   += yaw_delta;
        self.pitch = (self.pitch + pitch_delta).clamp(self.min_pitch, self.max_pitch);
        ctx.camera.set_rotation(self.yaw, self.pitch);

        // ── Velocidad (Shift acelera) ──
        let boost = if ctx.input().is_key_down(KeyCode::ShiftLeft) { self.boost_multiplier } else { 1.0 };
        let speed = self.move_speed * boost * dt;

        // ── WASD ──
        let forward = ctx.camera.forward();
        let right   = ctx.camera.right();
        let mut delta = Vec3::ZERO;
        if ctx.input().is_key_down(KeyCode::KeyW) { delta += forward; }
        if ctx.input().is_key_down(KeyCode::KeyS) { delta -= forward; }
        if ctx.input().is_key_down(KeyCode::KeyD) { delta += right; }
        if ctx.input().is_key_down(KeyCode::KeyA) { delta -= right; }
        if ctx.input().is_key_down(KeyCode::Space)        { delta += Vec3::Y; }
        if ctx.input().is_key_down(KeyCode::ControlLeft)  { delta -= Vec3::Y; }

        if delta.length_squared() > 0.0 {
            ctx.camera.position += delta.normalize() * speed;
        }
    }
}
