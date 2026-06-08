#![allow(unused_imports)]

use crate::xenofall::{
    audio::GameAudio,
    cards::{Build, CardType},
    constants::*,
    helpers::{
        hash_rand, hash_rand_signed, pick_random_cards, ray_headshot_intersect,
        ray_sphere_intersect,
    },
    types::{Enemy, EnemyState, FireMode, GameState, Impact, Tracer, WaveDef},
    vfx::VfxPools,
    world::WorldGeometry,
};
use crate::Xenofall;
use reactor_vulkan::graphics::post_process::PostProcessEffect;
use reactor_vulkan::prelude::*;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

impl Xenofall {
    pub(crate) fn update_rail(&mut self, dt: f32) {
        if self.state != GameState::Playing {
            return;
        }

        // Stop camera movement while there are active enemies in the wave!
        if self.total_enemies_alive > 0 {
            self.wave_clear_timer = WAVE_CLEAR_DELAY;
            return;
        }

        // Brief pause after clearing a wave before camera resumes
        if self.wave_clear_timer > 0.0 {
            self.wave_clear_timer -= dt;
            return;
        }

        if self.rail_progress < RAIL_LENGTH {
            self.rail_progress += RAIL_SPEED * dt;
        }
    }

    pub(crate) fn update_camera(&mut self, ctx: &mut ReactorContext) {
        // Camera at human eye level (1.7m) — UE5 standard
        let cam_pos = Vec3::new(0.0, CAMERA_Y, -self.rail_progress);
        ctx.camera.position = cam_pos;

        // Subtle walking bob
        let bob = (self.t * 3.0).sin() * 0.02;
        ctx.camera.position.y += bob;
        ctx.camera.set_rotation(0.0, 0.0);

        // Update audio listener to match camera
        ctx.audio
            .update_listener(AudioListener::from_camera(&ctx.camera));
    }

    pub(crate) fn get_aim_ray(&self, ctx: &ReactorContext) -> (Vec3, Vec3) {
        let mouse_pos = ctx.input().mouse_position();
        let (w, h) = ctx.window_size();
        let (w, h) = (w as f32, h as f32);

        let ndc_x = (mouse_pos.x / w) * 2.0 - 1.0;
        let ndc_y = 1.0 - (mouse_pos.y / h) * 2.0;

        let fov_h = ctx.camera.fov * AIM_FOV_SCALE * ctx.camera.aspect_ratio;
        let fov_v = ctx.camera.fov * AIM_FOV_SCALE;

        let aim_yaw = ndc_x * fov_h * 0.5;
        let aim_pitch = ndc_y * fov_v * 0.5;

        let forward = Vec3::new(0.0, 0.0, -1.0);
        let right = Vec3::X;
        let up = Vec3::Y;

        let dir = (forward + right * aim_yaw.tan() + up * aim_pitch.tan()).normalize();
        let origin = ctx.camera.position;

        (origin, dir)
    }

    pub(crate) fn update_player(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();

        if self.state != GameState::Playing {
            return;
        }

        if self.damage_pending > 0 {
            self.hp -= self.damage_pending;
            self.damage_flash = 0.3;
            self.damage_pending = 0;

            // Play damage sound
            if let Some(clip) = self.audio.damage {
                ctx.audio.play_sfx(clip, None, 0.8);
            }
        }

        if self.damage_flash > 0.0 {
            self.damage_flash -= dt;
        }

        // Combo timeout
        if self.combo_timer > 0.0 {
            self.combo_timer -= dt;
            if self.combo_timer <= 0.0 {
                self.combo = 0;
            }
        }

        // Game over check
        if self.hp <= 0 {
            self.hp = 0;
            self.state = GameState::GameOver;
        }

        // Victory check
        if self.rail_progress >= RAIL_LENGTH
            && self.total_enemies_alive == 0
            && self.wave_index >= self.waves.len()
        {
            self.state = GameState::Victory;
            if let Some(clip) = self.audio.victory {
                ctx.audio.play_sfx(clip, None, 0.9);
            }
        }
    }

    pub(crate) fn update_audio(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();
        ctx.audio.update(dt);
    }
}
