#![allow(unused_imports)]

use crate::Xenofall;
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
use reactor_vulkan::graphics::post_process::PostProcessEffect;
use reactor_vulkan::prelude::*;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

impl Xenofall {
    pub(crate) fn detect_fire_mode(&mut self) -> FireMode {
        let time_since_last = self.t - self.last_fire_time;

        if time_since_last < TAP_WINDOW {
            self.tap_streak += 1;
            if self.tap_streak >= 2 {
                return FireMode::Tap;
            }
        } else {
            self.tap_streak = 1;
        }

        if self.mouse_hold_time >= HOLD_THRESHOLD {
            return FireMode::Hold;
        }

        FireMode::Normal
    }

    pub(crate) fn damage_for_mode(&self, base_damage: i32) -> i32 {
        let mult = match self.fire_mode {
            FireMode::Tap => self.build.tap_damage_mult,
            FireMode::Hold => HOLD_DAMAGE_MULT,
            FireMode::Normal => 1.0,
        };
        (base_damage as f32 * mult).ceil() as i32
    }

    // =========================================================================
    // COMBAT
    // =========================================================================

    pub(crate) fn fire_weapon(&mut self, ctx: &mut ReactorContext, ray_origin: Vec3, ray_dir: Vec3) {
        let mag_size = self.build.effective_mag_size();
        if self.ammo == 0 || self.reloading || self.fire_cooldown > 0.0 {
            return;
        }

        // Detect fire mode
        self.fire_mode = self.detect_fire_mode();

        self.ammo -= 1;
        self.fire_cooldown = FIRE_COOLDOWN;
        self.muzzle_flash_timer = MUZZLE_FLASH_DURATION;
        self.shots_fired += 1;
        self.last_fire_time = self.t;

        // Play gunshot sound
        if let Some(clip) = self.audio.gunshot {
            ctx.audio.play_sfx(clip, Some(ctx.camera.position), 0.8);
        }

        // Spawn tracer
        if let Some(pool_idx) = self.find_free_tracer() {
            let tracer_pos = ray_origin + ray_dir * 0.5;
            ctx.set_transform(
                self.vfx.tracer_pool[pool_idx],
                Mat4::from_translation(tracer_pos),
            );
            self.active_tracers.push(Tracer {
                pool_index: pool_idx,
                position: tracer_pos,
                direction: ray_dir,
                lifetime: TRACER_LIFETIME,
            });
        }

        // Check hits — collect all enemies hit (for piercing)
        let max_hits = 1 + self.build.piercing;
        let mut hits: Vec<(usize, f32, bool)> = Vec::new(); // (enemy_idx, distance, is_headshot)

        // Headshots first
        for (i, enemy) in self.enemies.iter().enumerate() {
            if enemy.state != EnemyState::Alive {
                continue;
            }
            if let Some(t) = ray_headshot_intersect(ray_origin, ray_dir, enemy.position) {
                if t < 100.0 {
                    hits.push((i, t, true));
                }
            }
        }

        // Body hits
        for (i, enemy) in self.enemies.iter().enumerate() {
            if enemy.state != EnemyState::Alive {
                continue;
            }
            if hits.iter().any(|(idx, _, _)| *idx == i) {
                continue; // Already hit as headshot
            }
            if let Some(t) =
                ray_sphere_intersect(ray_origin, ray_dir, enemy.position, ENEMY_HIT_RADIUS)
            {
                if t < 100.0 {
                    hits.push((i, t, false));
                }
            }
        }

        // Sort by distance
        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Apply hits up to piercing count
        let mut hit_any = false;
        for (hit_idx, (enemy_idx, t, is_headshot)) in hits.iter().enumerate() {
            if hit_idx >= max_hits as usize {
                break;
            }

            hit_any = true;
            self.shots_hit += 1;

            let hit_point = ray_origin + ray_dir * t;
            let base_damage = if *is_headshot {
                self.enemies[*enemy_idx].max_health
            } else {
                1
            };
            let actual_damage = self.damage_for_mode(base_damage);

            self.enemies[*enemy_idx].health -= actual_damage;
            let died = self.enemies[*enemy_idx].health <= 0;

            if died {
                self.enemies[*enemy_idx].state = EnemyState::Dying;
                self.enemies[*enemy_idx].death_timer = 0.0;
            }

            self.spawn_impact(ctx, hit_point);

            // Play impact/death sound
            if died {
                if let Some(clip) = self.audio.death {
                    ctx.audio
                        .play_sfx(clip, Some(self.enemies[*enemy_idx].position), 0.7);
                }
            } else if let Some(clip) = self.audio.impact {
                ctx.audio.play_sfx(clip, Some(hit_point), 0.6);
            }

            // Explosive shot — damage nearby enemies
            if self.build.explosive_radius > 0.0 && died {
                let center = self.enemies[*enemy_idx].position;
                let radius = self.build.explosive_radius;
                for j in 0..self.enemies.len() {
                    if j == *enemy_idx || self.enemies[j].state != EnemyState::Alive {
                        continue;
                    }
                    let dist = (self.enemies[j].position - center).length();
                    if dist < radius {
                        self.enemies[j].health -= 1;
                        if self.enemies[j].health <= 0 {
                            self.enemies[j].state = EnemyState::Dying;
                            self.enemies[j].death_timer = 0.0;
                            self.kills += 1;
                            self.score = (self.score + SCORE_PER_KILL).min(SCORE_CAP);
                        }
                    }
                }
            }

            if died {
                self.kills += 1;

                // Score calculation with combo and build multipliers
                let combo_mult = 1 + self.combo.min(MAX_COMBO);
                let headshot_bonus = if *is_headshot { HEADSHOT_MULTIPLIER } else { 1 };
                let raw_score = SCORE_PER_KILL * combo_mult * headshot_bonus;
                let final_score = (raw_score as f32 * self.build.score_mult) as u32;
                self.score = (self.score + final_score).min(SCORE_CAP);

                if *is_headshot {
                    self.headshots += 1;
                }

                self.combo = (self.combo + 1).min(MAX_COMBO);
                self.combo_timer = self.build.effective_combo_timeout();

                // Combo sound at milestones
                if self.combo >= 3 {
                    if let Some(clip) = self.audio.combo {
                        ctx.audio.play_sfx(clip, None, 0.5);
                    }
                }
            }
        }

        if !hit_any {
            // Miss — impact on environment
            let far_point = ray_origin + ray_dir * 50.0;
            self.spawn_impact(ctx, far_point);
        }

        // Auto-reload when empty
        if self.ammo == 0 {
            self.reloading = true;
            self.reload_timer = self.build.effective_reload_time();
            if let Some(clip) = self.audio.reload {
                ctx.audio.play_sfx(clip, None, 0.7);
            }
        }

        // Suppress unused variable warning
        let _ = mag_size;
    }

    pub(crate) fn spawn_impact(&mut self, ctx: &mut ReactorContext, position: Vec3) {
        let used_pools: Vec<usize> = self.active_impacts.iter().map(|i| i.pool_index).collect();
        for (pool_idx, &scene_idx) in self.vfx.impact_pool.iter().enumerate() {
            if !used_pools.contains(&pool_idx) {
                ctx.set_transform(scene_idx, Mat4::from_translation(position));
                self.active_impacts.push(Impact {
                    pool_index: pool_idx,
                    position,
                    lifetime: IMPACT_LIFETIME,
                    max_lifetime: IMPACT_LIFETIME,
                });
                return;
            }
        }
    }

    pub(crate) fn find_free_tracer(&self) -> Option<usize> {
        let used: Vec<usize> = self.active_tracers.iter().map(|t| t.pool_index).collect();
        (0..self.vfx.tracer_pool.len()).find(|pool_idx| !used.contains(pool_idx))
    }

    // =========================================================================
    // CARD SELECTION
    // =========================================================================


    pub(crate) fn update_shooting(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();

        // Timers
        if self.fire_cooldown > 0.0 {
            self.fire_cooldown -= dt;
        }
        if self.muzzle_flash_timer > 0.0 {
            self.muzzle_flash_timer -= dt;
        }
        if self.reloading {
            self.reload_timer -= dt;
            if self.reload_timer <= 0.0 {
                self.reloading = false;
                self.ammo = self.build.effective_mag_size();
            }
        }

        // Manual reload
        if ctx.input().is_key_just_pressed(KeyCode::KeyR)
            && !self.reloading
            && self.ammo < self.build.effective_mag_size()
        {
            self.reloading = true;
            self.reload_timer = self.build.effective_reload_time();
            if let Some(clip) = self.audio.reload {
                ctx.audio.play_sfx(clip, None, 0.7);
            }
        }

        // Pause/config toggle (P/Esc).
        // El toggle real ocurre en `on_event` (única fuente de verdad), aquí
        // sólo detectamos si la pausa se acaba de abrir/cerrar este frame para
        // que `pause_config.update()` no consuma la MISMA pulsación como
        // "resume" y cierre la pausa en el mismo frame.
        let mut pause_opened_this_frame = false;
        if self.pause_event_consumed {
            self.pause_event_consumed = false;
            // El on_event toggleó este frame: si el resultado es Paused,
            // significa que se acaba de abrir → bloquear update() del overlay
            // este frame para no ver la tecla como "resume".
            if self.state == GameState::Paused {
                pause_opened_this_frame = true;
            }
        } else if ctx.input().is_key_just_pressed(KeyCode::KeyP)
            || ctx.input().is_key_just_pressed(KeyCode::Escape)
        {
            // Fallback: el evento no llegó por on_event (p.ej. focus perdido).
            let was_playing = self.state == GameState::Playing;
            self.toggle_pause_config(ctx);
            pause_opened_this_frame = was_playing && self.state == GameState::Paused;
        }

        // VSync dynamic toggle (unlocked FPS showcase)
        if self.state != GameState::Paused && ctx.input().is_key_just_pressed(KeyCode::KeyV) {
            ctx.reactor.vsync = !ctx.reactor.vsync;
            if let Err(e) = ctx.reactor.recreate_swapchain() {
                Log::error(&format!("Failed to toggle VSync: {}", e));
            } else {
                Log::success(&format!(
                    "VSync dynamically toggled to: {}",
                    if ctx.reactor.vsync {
                        "ON (Locked FPS)"
                    } else {
                        "OFF (Unlocked FPS!)"
                    }
                ));
            }
        }

        // Fullscreen dynamic toggle (F11 or F)
        if self.state != GameState::Paused
            && (ctx.input().is_key_just_pressed(KeyCode::F11)
                || ctx.input().is_key_just_pressed(KeyCode::KeyF))
        {
            let is_fullscreen = ctx.window.fullscreen().is_some();
            if is_fullscreen {
                ctx.window.set_fullscreen(None);
                Log::success("Windowed mode activated");
            } else {
                ctx.window
                    .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                Log::success("Fullscreen mode activated");
            }
        }

        // Restart on game over
        if (self.state == GameState::GameOver || self.state == GameState::Victory)
            && ctx.input().is_key_just_pressed(KeyCode::Space)
        {
            ctx.reactor.exit_requested = true;
        }

        // Q to quit
        if ctx.input().is_key_just_pressed(KeyCode::KeyQ) {
            ctx.reactor.exit_requested = true;
        }

        // Card selection
        if self.state == GameState::CardSelect {
            if ctx.input().is_key_just_pressed(KeyCode::Digit1) {
                self.select_card(0, ctx);
            } else if ctx.input().is_key_just_pressed(KeyCode::Digit2) {
                self.select_card(1, ctx);
            } else if ctx.input().is_key_just_pressed(KeyCode::Digit3) {
                self.select_card(2, ctx);
            }
            return;
        }

        if self.state == GameState::Paused && !pause_opened_this_frame {
            let pause_result = self.pause_config.update(ctx);
            if pause_result.requested_resume {
                println!(
                    "\n  \x1b[38;2;180;0;0m▓▓▓ RESUMING — BLOOD PROTOCOL DEACTIVATED ▓▓▓\x1b[0m\n"
                );
                self.state = GameState::Playing;
            }
            if pause_result.requested_quit {
                ctx.reactor.exit_requested = true;
            }
        }

        if self.state != GameState::Playing {
            return;
        }

        // Track mouse hold time
        if ctx.input().is_mouse_button_down(MouseButton::Left) {
            self.mouse_hold_time += dt;
        } else {
            self.mouse_hold_time = 0.0;
        }

        // Fire on click
        let (ray_origin, ray_dir) = self.get_aim_ray(ctx);
        if ctx.input().is_mouse_button_down(MouseButton::Left) {
            self.fire_weapon(ctx, ray_origin, ray_dir);
        }

        // Muzzle flash visual
        if let Some(flash_idx) = self.vfx.muzzle_flash_index {
            if self.muzzle_flash_timer > 0.0 {
                let flash_pos = ctx.camera.position
                    + ctx.camera.forward() * 0.8
                    + ctx.camera.right() * 0.3
                    + ctx.camera.up() * (-0.2);
                ctx.set_transform(flash_idx, Mat4::from_translation(flash_pos));
            } else {
                ctx.set_transform(
                    flash_idx,
                    Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
                );
            }
        }
    }

    pub(crate) fn update_tracers(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();
        let mut to_remove = Vec::new();

        for (i, tracer) in self.active_tracers.iter_mut().enumerate() {
            tracer.position += tracer.direction * TRACER_SPEED * dt;
            tracer.lifetime -= dt;

            if tracer.lifetime <= 0.0 {
                ctx.set_transform(
                    self.vfx.tracer_pool[tracer.pool_index],
                    Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
                );
                to_remove.push(i);
            } else {
                ctx.set_transform(
                    self.vfx.tracer_pool[tracer.pool_index],
                    Mat4::from_translation(tracer.position),
                );
            }
        }

        for &i in to_remove.iter().rev() {
            self.active_tracers.remove(i);
        }
    }

    pub(crate) fn update_impacts(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();
        let mut to_remove = Vec::new();

        for (i, impact) in self.active_impacts.iter_mut().enumerate() {
            impact.lifetime -= dt;

            if impact.lifetime <= 0.0 {
                ctx.set_transform(
                    self.vfx.impact_pool[impact.pool_index],
                    Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
                );
                to_remove.push(i);
            } else {
                let scale = (impact.lifetime / impact.max_lifetime) * 0.12;
                ctx.set_transform(
                    self.vfx.impact_pool[impact.pool_index],
                    Mat4::from_scale_rotation_translation(
                        Vec3::splat(scale),
                        Quat::IDENTITY,
                        impact.position,
                    ),
                );
            }
        }

        for &i in to_remove.iter().rev() {
            self.active_impacts.remove(i);
        }
    }


}
