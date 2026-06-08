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
    pub(crate) fn spawn_enemy(&mut self, ctx: &mut ReactorContext, pos: Vec3, hp: i32, speed: f32) {
        let id = self.next_enemy_id;
        self.next_enemy_id += 1;

        // Place at ground level — center of model at ZOMBIE_GROUND_Y (~0.9m)
        let ground_pos = Vec3::new(pos.x, ZOMBIE_GROUND_Y, pos.z);

        // 🌑 Blob shadow bajo los pies del zombie (radius 0.45m ≈ silueta humana).
        let blob = ctx.spawn_blob_shadow(ground_pos, 0.45).ok();

        // 🧊 Spawn inteligente: REACTOR mide el modelo, lo escala a 1.8 m y lo
        // orienta automáticamente hacia +Z (la cámara mira desde +Z hacia -Z).
        // Antes esto requería medir el modelo a mano en Blender. Ahora es 1 línea.
        let spawn = GltfSpawn::at(Vec3::new(ground_pos.x, 0.0, ground_pos.z))
            .with_height(1.8)
            .facing(Vec3::new(0.0, 0.0, -1.0));

        match ctx.spawn_gltf_smart("assets/models/zombie_basic.glb", spawn) {
            Ok(info) => {
                Log::game(&format!(
                    "Zombie #{} spawn (glTF, escala {:.2}x, altura {:.2}m) en ({:.1}, {:.1}, {:.1})",
                    id, info.applied_scale, info.world_height,
                    ground_pos.x, ground_pos.y, ground_pos.z
                ));

                // Compute local node transforms relative to the initial parent base transform
                let p_base_0 = Mat4::from_rotation_translation(
                    Quat::from_rotation_y(ZOMBIE_MODEL_YAW_OFFSET),
                    Vec3::new(ground_pos.x, 0.0, ground_pos.z),
                );
                let mut initial_transforms = Vec::new();
                for &idx in &info.indices {
                    if let Some(m0) = ctx.get_transform(idx) {
                        let local_xf = p_base_0.inverse() * m0;
                        initial_transforms.push((idx, local_xf));
                    }
                }

                self.enemies.push(Enemy {
                    _scene_indices: info.indices,
                    position: ground_pos,
                    health: hp,
                    max_health: hp,
                    state: EnemyState::Alive,
                    death_timer: 0.0,
                    attack_timer: 0.0,
                    speed,
                    is_gltf: true,
                    _id: id,
                    blob_shadow: blob,
                    _gltf_scale: info.applied_scale,
                    initial_transforms,
                });
                self.total_enemies_alive += 1;
            }
            Err(_) => {
                // Fallback to cube — human-proportioned silhouette (0.5m × 1.8m × 0.35m)
                if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
                    let cube_xf = Mat4::from_scale_rotation_translation(
                        ZOMBIE_CUBE_SCALE,
                        Quat::IDENTITY,
                        ground_pos,
                    );
                    ctx.set_transform(idx, cube_xf);
                    Log::game(&format!(
                        "Zombie #{} spawn (cube fallback, {:.1}x{:.1}x{:.1}m) en ({:.1}, {:.1}, {:.1})",
                        id, ZOMBIE_CUBE_SCALE.x, ZOMBIE_CUBE_SCALE.y, ZOMBIE_CUBE_SCALE.z,
                        ground_pos.x, ground_pos.y, ground_pos.z
                    ));

                    // Base parent for cube: position = ground_pos, rotation = IDENTITY
                    let p_base_0 = Mat4::from_rotation_translation(Quat::IDENTITY, ground_pos);
                    let local_xf = p_base_0.inverse() * cube_xf; // which is just Mat4::from_scale(ZOMBIE_CUBE_SCALE)

                    self.enemies.push(Enemy {
                        _scene_indices: vec![idx],
                        position: ground_pos,
                        health: hp,
                        max_health: hp,
                        state: EnemyState::Alive,
                        death_timer: 0.0,
                        attack_timer: 0.0,
                        speed,
                        is_gltf: false,
                        _id: id,
                        blob_shadow: blob,
                        _gltf_scale: 1.0,
                        initial_transforms: vec![(idx, local_xf)],
                    });
                    self.total_enemies_alive += 1;
                } else if let Some(b) = blob {
                    // Si tampoco se pudo spawn-ear el cubo, recogemos el blob huérfano.
                    ctx.hide_blob_shadow(b);
                }
            }
        }
    }

    // =========================================================================
    // FIRE MODE DETECTION
    // =========================================================================

    pub(crate) fn update_enemies(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();
        let cam_pos = ctx.camera.position;

        // Occasional zombie groan
        if (self.t * 0.7).fract() < dt * 0.7 && self.total_enemies_alive > 0 {
            if let Some(clip) = self.audio.zombie_groan {
                // Pick a random alive enemy for spatial position
                if let Some(enemy) = self.enemies.iter().find(|e| e.state == EnemyState::Alive) {
                    ctx.audio.play_sfx(clip, Some(enemy.position), 0.3);
                }
            }
        }

        // Track how many died this frame so we can decrement total_enemies_alive
        let mut deaths_this_frame: u32 = 0;

        for enemy in &mut self.enemies {
            // 🌑 Mover blob shadow para que siga al zombie cada frame.
            if enemy.state == EnemyState::Alive {
                if let Some(b) = enemy.blob_shadow {
                    ctx.move_blob_shadow(b, enemy.position, 0.45);
                }
            }

            match enemy.state {
                EnemyState::Alive => {
                    // Move toward the player along the XZ plane only (keep Y at ground level)
                    let target = Vec3::new(cam_pos.x, enemy.position.y, cam_pos.z);
                    let to_player = target - enemy.position;
                    let dist = to_player.length();

                    // Procedural zombie waddle (bobbing up/down and rolling side-to-side)
                    // Makes T-posing models feel like funny/creepy spirits with deliberate waddles!
                    let bob = (self.t * 6.0 + enemy._id as f32 * 1.5).sin() * 0.05;
                    let sway = (self.t * 5.0 + enemy._id as f32 * 1.5).cos() * 0.07;

                    if dist > ENEMY_ATTACK_DIST {
                        let move_dir = to_player.normalize();
                        enemy.position += move_dir * enemy.speed * dt;

                        // Face toward the player. zombie_basic.glb looks at +Z in bind pose.
                        let facing = Quat::from_rotation_y(
                            move_dir.x.atan2(move_dir.z) + ZOMBIE_MODEL_YAW_OFFSET,
                        ) * Quat::from_rotation_z(sway); // Z-roll waddle
                        let new_parent_pos = if enemy.is_gltf {
                            Vec3::new(enemy.position.x, bob, enemy.position.z)
                        } else {
                            enemy.position + Vec3::new(0.0, bob, 0.0)
                        };
                        let p_base = Mat4::from_rotation_translation(facing, new_parent_pos);
                        for &(idx, local_xf) in &enemy.initial_transforms {
                            ctx.set_transform(idx, p_base * local_xf);
                        }
                    } else {
                        // Within attack range — deal damage periodically
                        enemy.attack_timer -= dt;
                        if enemy.attack_timer <= 0.0 {
                            enemy.attack_timer = ENEMY_ATTACK_COOLDOWN;
                            self.damage_pending += DAMAGE_PER_HIT;
                        }

                        // Still face the player while attacking, with aggressive tremble
                        if dist > 0.1 {
                            let face_dir = to_player.normalize();
                            let tremble = (self.t * 22.0).sin() * 0.02; // fast jitter
                            let facing = Quat::from_rotation_y(
                                face_dir.x.atan2(face_dir.z) + ZOMBIE_MODEL_YAW_OFFSET,
                            ) * Quat::from_rotation_x(tremble);
                            let new_parent_pos = if enemy.is_gltf {
                                Vec3::new(enemy.position.x, tremble * 0.5, enemy.position.z)
                            } else {
                                enemy.position + Vec3::new(0.0, tremble * 0.5, 0.0)
                            };
                            let p_base = Mat4::from_rotation_translation(facing, new_parent_pos);
                            for &(idx, local_xf) in &enemy.initial_transforms {
                                ctx.set_transform(idx, p_base * local_xf);
                            }
                        }
                    }
                }

                EnemyState::Dying => {
                    enemy.death_timer += dt;
                    let t = enemy.death_timer / ENEMY_DEATH_DURATION;

                    let fall_angle = t * std::f32::consts::FRAC_PI_2;
                    let sink = t * 0.8;

                    let to_player =
                        Vec3::new(cam_pos.x, enemy.position.y, cam_pos.z) - enemy.position;
                    let face_dir = if to_player.length_squared() > 1e-6 {
                        to_player.normalize()
                    } else {
                        Vec3::new(0.0, 0.0, 1.0)
                    };

                    // Keep correct facing yaw direction when falling.
                    let facing_yaw = Quat::from_rotation_y(
                        face_dir.x.atan2(face_dir.z) + ZOMBIE_MODEL_YAW_OFFSET,
                    );

                    let new_parent_pos = if enemy.is_gltf {
                        Vec3::new(enemy.position.x, 0.0, enemy.position.z)
                    } else {
                        enemy.position
                    };

                    let p_base =
                        Mat4::from_translation(new_parent_pos + Vec3::new(0.0, -sink, 0.0))
                            * Mat4::from_quat(facing_yaw)
                            * Mat4::from_rotation_x(fall_angle);

                    for &(idx, local_xf) in &enemy.initial_transforms {
                        ctx.set_transform(idx, p_base * local_xf);
                    }

                    // 🌑 Encoge el blob shadow conforme el zombie cae (0.45 → 0).
                    if let Some(b) = enemy.blob_shadow {
                        let shrink = (1.0 - t).max(0.0) * 0.45;
                        ctx.move_blob_shadow(b, enemy.position, shrink);
                    }

                    if enemy.death_timer >= ENEMY_DEATH_DURATION {
                        enemy.state = EnemyState::Dead;
                        deaths_this_frame += 1;
                        for &(idx, _) in &enemy.initial_transforms {
                            ctx.set_transform(
                                idx,
                                Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
                            );
                        }
                        // 🌑 Ocultar definitivamente el blob al morir.
                        if let Some(b) = enemy.blob_shadow {
                            ctx.hide_blob_shadow(b);
                        }
                    }
                }

                EnemyState::Dead => {}
            }
        }

        // Properly decrement alive counter when Dying → Dead transition completes
        self.total_enemies_alive = self.total_enemies_alive.saturating_sub(deaths_this_frame);

        self.enemies.retain(|e| e.state != EnemyState::Dead);
    }

    pub(crate) fn update_waves(&mut self, ctx: &mut ReactorContext) {
        if self.state != GameState::Playing {
            return;
        }
        if self.wave_index >= self.waves.len() {
            return;
        }

        let wave = self.waves[self.wave_index];

        if self.rail_progress >= wave.trigger_z {
            let prev_wave = self.current_wave;
            self.current_wave = self.wave_index as u32 + 1;

            // Play wave start sound
            if let Some(clip) = self.audio.wave_start {
                ctx.audio.play_sfx(clip, None, 0.7);
            }

            println!(
                "  ⚠️ OLEADA {} — {} infectados!",
                self.current_wave, wave.count
            );

            // Spawn enemies in a visible range ahead of the camera
            // They appear INSIDE the corridor, spread laterally and in depth
            for i in 0..wave.count {
                let seed = self.next_enemy_id + i;
                // Lateral spread within corridor walls (±CORRIDOR_HALF_WIDTH with padding)
                let max_lateral = (CORRIDOR_HALF_WIDTH - 0.8).min(wave.spread * 0.5);
                let x_offset = hash_rand_signed(seed * 7 + 1) * max_lateral;
                // Depth spread: spawn between SPAWN_DIST_MIN and SPAWN_DIST_MAX ahead
                let z_offset = ENEMY_SPAWN_DIST_MIN
                    + hash_rand(seed * 13 + 3)
                        * (ENEMY_SPAWN_DIST_MAX - ENEMY_SPAWN_DIST_MIN + wave.depth * 0.3);

                // Position in world: camera looks toward -Z, so spawn further in -Z
                let pos = Vec3::new(x_offset, 0.8, -(self.rail_progress + z_offset));
                let speed =
                    ENEMY_BASE_SPEED * wave.speed_mult * (0.8 + hash_rand(seed * 23 + 7) * 0.4);

                self.spawn_enemy(ctx, pos, wave.enemy_hp, speed);
            }

            self.wave_index += 1;

            // Apply regeneration from build
            if self.build.regen_per_wave > 0 {
                self.hp = (self.hp + self.build.regen_per_wave).min(self.build.max_hp);
            }

            // Card selection every 2 waves (after wave 2, 4, 6)
            if prev_wave > 0 && prev_wave % 2 == 0 && self.wave_index < self.waves.len() {
                self.enter_card_select();
            }
        }
    }
}
