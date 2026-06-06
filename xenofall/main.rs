// =============================================================================
// XENOFALL — Rail Shooter Roguelite sobre REACTOR
// =============================================================================
// Género: Rail Shooter Roguelite · Inspirado en House of the Dead + Slay the
// Spire + DMC combo systems.
//
// 🎮 Mecánicas implementadas:
//   ✓ Cámara sobre rieles (avanza automáticamente por el corredor)
//   ✓ Apuntado con mouse (el cursor ES la mira)
//   ✓ TAP rápido → daño x2 (click rápido sucesivo)
//   ✓ HOLD → daño x0.5 pero precisión perfecta (mantener click)
//   ✓ Combo counter hasta x10 → multiplicador de puntos
//   ✓ Sistema de recarga con R (8 balas por cargador)
//   ✓ Sistema de Cartas: modificadores de build entre oleadas
//   ✓ Score cap: 9,999,999
//   ✓ Enemigos zombie con modelo 3D glTF
//   ✓ 🔊 Audio real con rodio (disparos, recargas, impactos, groans)
//   ✓ HUD completo en el título de la ventana
//   ✓ 8 oleadas con dificultad creciente
//   ✓ Sin un solo `unsafe` en el código de gameplay
//
// 🕹️ Controles:
//   Mouse       → Apuntar (el cursor es la mira)
//   Click Izq.  → Disparar (TAP rápido = x2 dmg, HOLD = x0.5 dmg)
//   R           → Recargar
//   P           → Pausar
//   1/2/3       → Seleccionar carta (durante selección de carta)
//   Esc         → Salir
//
// 📖 LORE:
//   Nave alienígena cayó en Rumania. Laboratorio de experimentos ADN.
//   Accidentes escaparon → infección. Cuarentena encubierta día 47+.
//   Élites atrapadas adentro. The Contractor enviado solo.
//
// =============================================================================

use reactor_vulkan::graphics::post_process::PostProcessEffect;
use reactor_vulkan::prelude::*;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[path = "mod.rs"]
mod xenofall;
use xenofall::{
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

// =============================================================================
// GAME STATE
// =============================================================================

struct Xenofall {
    // Estado del juego
    state: GameState,

    // Rail / Cámara
    rail_progress: f32,

    // Combate
    ammo: u32,
    reloading: bool,
    reload_timer: f32,
    fire_cooldown: f32,
    muzzle_flash_timer: f32,

    // TAP / HOLD system
    last_fire_time: f32,
    mouse_hold_time: f32,
    fire_mode: FireMode,
    tap_streak: u32,

    // Jugador
    hp: i32,
    score: u32,
    combo: u32,
    combo_timer: f32,
    kills: u32,
    headshots: u32,
    shots_fired: u32,
    shots_hit: u32,
    damage_flash: f32,
    current_wave: u32,

    // Build / Cards
    build: Build,
    card_options: Vec<CardType>,
    card_select_seed: u32,

    // Entidades
    enemies: Vec<Enemy>,
    active_tracers: Vec<Tracer>,
    active_impacts: Vec<Impact>,

    // Escenario / VFX modularizados
    world: WorldGeometry,
    vfx: VfxPools,

    // Oleadas
    waves: Vec<WaveDef>,
    wave_index: usize,
    total_enemies_alive: u32,

    /// Timer after wave is cleared before camera resumes
    wave_clear_timer: f32,

    next_enemy_id: u32,

    // Timing
    t: f32,

    // Pending damage
    damage_pending: i32,

    // Audio
    audio: GameAudio,

    // Visuals extra
    crosshair_index: Option<usize>,
    game_over_index: Option<usize>,
    victory_index: Option<usize>,
    pause_config: PauseConfig,
    pause_event_consumed: bool,
}

impl Xenofall {
    fn new() -> Self {
        Self {
            state: GameState::Playing,
            rail_progress: 0.0,
            ammo: MAX_AMMO,
            reloading: false,
            reload_timer: 0.0,
            fire_cooldown: 0.0,
            muzzle_flash_timer: 0.0,
            last_fire_time: -10.0,
            mouse_hold_time: 0.0,
            fire_mode: FireMode::Normal,
            tap_streak: 0,
            hp: PLAYER_MAX_HP,
            score: 0,
            combo: 0,
            combo_timer: 0.0,
            kills: 0,
            headshots: 0,
            shots_fired: 0,
            shots_hit: 0,
            damage_flash: 0.0,
            current_wave: 0,
            build: Build::new(),
            card_options: Vec::new(),
            card_select_seed: 42,
            enemies: Vec::new(),
            active_tracers: Vec::new(),
            active_impacts: Vec::new(),
            world: WorldGeometry::default(),
            vfx: VfxPools::default(),
            waves: xenofall::waves::build_waves(),
            wave_index: 0,
            total_enemies_alive: 0,
            wave_clear_timer: 0.0,
            next_enemy_id: 0,
            t: 0.0,
            damage_pending: 0,
            audio: GameAudio::new(),
            crosshair_index: None,
            game_over_index: None,
            victory_index: None,
            pause_config: PauseConfig::new().with_title("XENOFALL - REACTOR TOTAL CONFIG"),
            pause_event_consumed: false,
        }
    }

    fn toggle_pause_config(&mut self, ctx: &mut ReactorContext) {
        self.state = match self.state {
            GameState::Playing => {
                self.pause_config.show(ctx);
                GameState::Paused
            }
            GameState::Paused => {
                println!(
                    "\n  \x1b[38;2;180;0;0m▓▓▓ RESUMING - BLOOD PROTOCOL DEACTIVATED ▓▓▓\x1b[0m\n"
                );
                self.pause_config.hide(ctx);
                GameState::Playing
            }
            other => other,
        };
    }

    // =========================================================================
    // SPAWN HELPERS
    // =========================================================================

    fn build_corridor(&mut self, ctx: &mut ReactorContext) {
        self.world = xenofall::world::build_corridor(ctx);
    }

    fn build_pools(&mut self, ctx: &mut ReactorContext) {
        self.vfx = xenofall::vfx::build_pools(ctx);
    }

    fn apply_render_showcase_profile(&mut self, ctx: &mut ReactorContext) {
        xenofall::render_lab::apply_professional_profile(ctx);
    }

    fn apply_render_showcase_materials(&mut self, ctx: &mut ReactorContext) {
        xenofall::world::apply_world_materials(ctx, &self.world);
        xenofall::vfx::apply_vfx_materials(ctx, &self.vfx);
    }

    fn print_render_showcase_budget(&self, ctx: &ReactorContext) {
        xenofall::render_lab::log_phase_one_budget(ctx);
    }

    fn spawn_enemy(&mut self, ctx: &mut ReactorContext, pos: Vec3, hp: i32, speed: f32) {
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

    fn detect_fire_mode(&mut self) -> FireMode {
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

    fn damage_for_mode(&self, base_damage: i32) -> i32 {
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

    fn fire_weapon(&mut self, ctx: &mut ReactorContext, ray_origin: Vec3, ray_dir: Vec3) {
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

    fn spawn_impact(&mut self, ctx: &mut ReactorContext, position: Vec3) {
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

    fn find_free_tracer(&self) -> Option<usize> {
        let used: Vec<usize> = self.active_tracers.iter().map(|t| t.pool_index).collect();
        (0..self.vfx.tracer_pool.len()).find(|pool_idx| !used.contains(pool_idx))
    }

    // =========================================================================
    // CARD SELECTION
    // =========================================================================

    fn enter_card_select(&mut self) {
        self.state = GameState::CardSelect;
        self.card_select_seed = self.card_select_seed.wrapping_add(self.kills + self.score);
        self.card_options = pick_random_cards(self.card_select_seed, 3);

        Log::header("🃏 SELECCIONA UNA CARTA 🃏");
        let mut rows = Vec::new();
        for (i, card) in self.card_options.iter().enumerate() {
            rows.push(vec![
                format!("[{}]", i + 1),
                card.name().to_string(),
                card.description().to_string(),
            ]);
        }
        Log::table(
            &["Opción", "Carta", "Efecto / Descripción"],
            &rows,
            &[6, 18, 30],
        );
        Log::info("Presiona 1, 2 o 3 en tu teclado para seleccionar");
        println!();
    }

    fn select_card(&mut self, index: usize, ctx: &mut ReactorContext) {
        if index < self.card_options.len() {
            let card = self.card_options[index];
            self.build.apply_card(card);
            Log::success(&format!(
                "Carta seleccionada: {}{} — {}{}",
                color::BOLD,
                card.name(),
                color::RESET,
                card.description()
            ));

            // Play card select sound
            if let Some(clip) = self.audio.card_select {
                ctx.audio.play_sfx(clip, None, 0.6);
            }

            // Apply regeneration if we have it
            if self.build.regen_per_wave > 0 {
                self.hp = (self.hp + self.build.regen_per_wave).min(self.build.max_hp);
            }

            // Apply armor plating
            if self.hp < self.build.max_hp
                && self.build.cards_collected.contains(&CardType::ArmorPlating)
            {
                self.hp = self.build.max_hp;
            }

            // Refill ammo
            self.ammo = self.build.effective_mag_size();
            self.reloading = false;

            self.state = GameState::Playing;
        }
    }

    // =========================================================================
    // UPDATE SUB-SYSTEMS
    // =========================================================================

    fn update_rail(&mut self, dt: f32) {
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

    fn update_camera(&mut self, ctx: &mut ReactorContext) {
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

    fn get_aim_ray(&self, ctx: &ReactorContext) -> (Vec3, Vec3) {
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

    fn update_shooting(&mut self, ctx: &mut ReactorContext) {
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

    fn update_tracers(&mut self, ctx: &mut ReactorContext) {
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

    fn update_impacts(&mut self, ctx: &mut ReactorContext) {
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

    fn update_enemies(&mut self, ctx: &mut ReactorContext) {
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

    fn update_player(&mut self, ctx: &mut ReactorContext) {
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

    fn update_waves(&mut self, ctx: &mut ReactorContext) {
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

    fn update_audio(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();
        ctx.audio.update(dt);
    }

    fn update_hud(&self, ctx: &mut ReactorContext) {
        let accuracy = if self.shots_fired > 0 {
            (self.shots_hit as f32 / self.shots_fired as f32 * 100.0) as u32
        } else {
            0
        };

        let state_icon = match self.state {
            GameState::Playing => "🎯",
            GameState::Paused => "⏸️",
            GameState::CardSelect => "🃏",
            GameState::GameOver => "💀",
            GameState::Victory => "🏆",
        };

        let ammo_str = if self.reloading {
            format!("⟳ {:.1}s", self.reload_timer)
        } else {
            format!("{}/{}", self.ammo, self.build.effective_mag_size())
        };

        let combo_str = if self.combo > 1 {
            format!(" x{}", self.combo)
        } else {
            String::new()
        };

        let mode_str = match self.fire_mode {
            FireMode::Tap => " ⚡TAP",
            FireMode::Hold => " 🔒HOLD",
            FireMode::Normal => "",
        };

        let flash = if self.damage_flash > 0.0 {
            "⚠️ "
        } else {
            ""
        };

        let cards_str = if !self.build.cards_collected.is_empty() {
            format!(" · {}🃏", self.build.cards_collected.len())
        } else {
            String::new()
        };

        let vrs_rate = ctx.reactor.pixel_intelligent_rate();
        let pixel_str = if ctx.reactor.pixel_intelligent_enabled() {
            format!(" PI {}x{}", vrs_rate.width, vrs_rate.height)
        } else {
            String::new()
        };
        let hud_status = format!("{}{}", cards_str, pixel_str);

        ctx.set_title(&format!(
            "{}{} XENOFALL · {} HP · {} balas{} · {:>7} pts{} · {} kills · {}%{} · Ola {}/{} · {:.0} FPS",
            flash,
            state_icon,
            self.hp,
            ammo_str,
            mode_str,
            self.score,
            combo_str,
            self.kills,
            accuracy,
            hud_status,
            self.current_wave,
            self.waves.len(),
            ctx.fps(),
        ));
    }

    fn update_interface(&mut self, ctx: &mut ReactorContext) {
        // ── Crosshair ──
        if self.state == GameState::Playing {
            if let Some(crosshair_idx) = self.crosshair_index {
                let (ray_origin, ray_dir) = self.get_aim_ray(ctx);
                let crosshair_pos = ray_origin + ray_dir * 1.5; // 1.5m in front of camera
                let crosshair_xf = Mat4::from_scale_rotation_translation(
                    Vec3::splat(0.012), // small red dot
                    Quat::IDENTITY,
                    crosshair_pos,
                );
                ctx.set_transform(crosshair_idx, crosshair_xf);
            }
        } else {
            // Hide crosshair when not playing (paused, game over, card select)
            if let Some(crosshair_idx) = self.crosshair_index {
                ctx.set_transform(
                    crosshair_idx,
                    Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
                );
            }
        }

        // ── Game Over Screen Overlay ──
        if self.state == GameState::GameOver {
            if let Some(go_idx) = self.game_over_index {
                let cam_pos = ctx.camera.position;
                let aspect = ctx.aspect_ratio();
                let overlay_pos = cam_pos + Vec3::new(0.0, 0.0, -0.4); // 40cm in front of camera
                let overlay_xf = Mat4::from_scale_rotation_translation(
                    Vec3::new(0.45 * aspect, 0.45, 1.0), // scaled to aspect ratio
                    Quat::IDENTITY,
                    overlay_pos,
                );
                ctx.set_transform(go_idx, overlay_xf);
            }
        } else {
            // Hide Game Over Screen when not active
            if let Some(go_idx) = self.game_over_index {
                ctx.set_transform(go_idx, Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)));
            }
        }

        // ── Victory Screen Overlay ──
        if self.state == GameState::Victory {
            if let Some(vic_idx) = self.victory_index {
                let cam_pos = ctx.camera.position;
                let aspect = ctx.aspect_ratio();
                let overlay_pos = cam_pos + Vec3::new(0.0, 0.0, -0.4); // 40cm in front of camera
                let overlay_xf = Mat4::from_scale_rotation_translation(
                    Vec3::new(0.45 * aspect, 0.45, 1.0), // scaled to aspect ratio
                    Quat::IDENTITY,
                    overlay_pos,
                );
                ctx.set_transform(vic_idx, overlay_xf);
            }
        } else {
            // Hide Victory Screen when not active
            if let Some(vic_idx) = self.victory_index {
                ctx.set_transform(
                    vic_idx,
                    Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
                );
            }
        }
    }

    #[allow(dead_code)]
    fn print_config_pause(&self, ctx: &ReactorContext) {
        let settings = &ctx.reactor.post_process.settings;
        let is_vignette = settings.is_effect_enabled(PostProcessEffect::Vignette);
        let is_bloom = settings.is_effect_enabled(PostProcessEffect::Bloom);
        let is_grain = settings.is_effect_enabled(PostProcessEffect::FilmGrain);
        let is_chromatic = settings.is_effect_enabled(PostProcessEffect::ChromaticAberration);
        let is_fxaa = settings.is_effect_enabled(PostProcessEffect::FXAA);
        let is_sharpen = settings.is_effect_enabled(PostProcessEffect::Sharpen);
        let is_tonemap = settings.is_effect_enabled(PostProcessEffect::ToneMapping);
        let is_ssgi = settings.is_effect_enabled(PostProcessEffect::SSGI);
        let is_fog = settings.is_effect_enabled(PostProcessEffect::VolumetricFog);
        let is_lut = settings.is_effect_enabled(PostProcessEffect::LutColorGrading);
        let is_ssr = settings.is_effect_enabled(PostProcessEffect::SSR);
        let is_pt = settings.is_effect_enabled(PostProcessEffect::PathTracedLighting);
        let is_flares = settings.is_effect_enabled(PostProcessEffect::AnamorphicFlares);

        let exposure = settings.exposure;
        let bloom_intensity = settings.bloom_intensity;
        let grain_intensity = settings.grain_intensity;

        let acc = if self.shots_fired > 0 {
            (self.shots_hit as f32 / self.shots_fired as f32 * 100.0) as u32
        } else {
            0
        };

        let msaa_str = format!("{:?}", ctx.reactor.msaa_samples);
        let msaa_display = if msaa_str.contains("TYPE_4") {
            "4x"
        } else if msaa_str.contains("TYPE_8") {
            "8x"
        } else if msaa_str.contains("TYPE_2") {
            "2x"
        } else if msaa_str.contains("TYPE_16") {
            "16x"
        } else if msaa_str.contains("TYPE_1") {
            "1x"
        } else {
            &msaa_str
        };

        let vsync_display = if ctx.reactor.vsync {
            "ON (Locked)"
        } else {
            "OFF (Unlocked)"
        };
        let fps_display = format!("{:.1}", ctx.fps());
        let vrs_rate = ctx.reactor.pixel_intelligent_rate();
        let pixel_display = if ctx.reactor.pixel_intelligent_enabled() {
            format!(
                "{:?} {}x{}",
                ctx.reactor.pixel_intelligent.profile, vrs_rate.width, vrs_rate.height
            )
        } else if ctx.reactor.context.supports_fragment_shading_rate() {
            "OFF".to_string()
        } else {
            "No HW".to_string()
        };

        println!();
        println!("  \x1b[38;2;180;0;0m╔══════════════════════════════════════════════════════════════════════════╗\x1b[0m");
        println!("  \x1b[38;2;220;0;0m║                                                                          ║\x1b[0m");
        println!("  \x1b[38;2;255;20;20m║   ██╗  ██╗███████╗███╗   ██╗ ██████╗ ███████╗ █████╗ ██╗     ██╗         ║\x1b[0m");
        println!("  \x1b[38;2;255;0;0m║   ╚██╗██╔╝██╔════╝████╗  ██║██╔═══██╗██╔════╝██╔══██╗██║     ██║         ║\x1b[0m");
        println!("  \x1b[38;2;220;0;0m║    ╚███╔╝ █████╗  ██╔██╗ ██║██║   ██║█████╗  ███████║██║     ██║         ║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║    ██╔██╗ ██╔══╝  ██║╚██╗██║██║   ██║██╔══╝  ██╔══██║██║     ██║         ║\x1b[0m");
        println!("  \x1b[38;2;140;0;0m║   ██╔╝ ██╗███████╗██║ ╚████║╚██████╔╝██║     ██║  ██║███████╗███████╗   ║\x1b[0m");
        println!("  \x1b[38;2;100;0;0m║   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝   ║\x1b[0m");
        println!("  \x1b[38;2;140;0;0m║                                                                          ║\x1b[0m");
        println!("  \x1b[38;2;255;0;0m║        🩸 ░▒▓ TACTICAL PAUSE CONFIGURATION — BLOOD PROTOCOL ▓▒░ 🩸       ║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m╠══════════════════════════════════════════════════════════════════════════╣\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[97m\x1b[1m  ■ COMBAT STATUS\x1b[0m                                                         \x1b[38;2;180;0;0m║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    HP: \x1b[91m{:<3}\x1b[0m/{:<3}   | Score: \x1b[93m{:<8}\x1b[0m   | Kills: \x1b[91m{:<4}\x1b[0m                       \x1b[38;2;180;0;0m║\x1b[0m", self.hp, self.build.max_hp, self.score, self.kills);
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    Headshots: \x1b[93m{:<4}\x1b[0m  | Accuracy: \x1b[92m{:>3}%\x1b[0m  | Wave: \x1b[96m{:<2}\x1b[0m/{:<2}                        \x1b[38;2;180;0;0m║\x1b[0m", self.headshots, acc, self.current_wave, self.waves.len());
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    Combo: x\x1b[95m{:<2}\x1b[0m      | Cards: \x1b[94m{:<2}\x1b[0m        | Ammo: \x1b[97m{:<2}\x1b[0m/{:<2}                      \x1b[38;2;180;0;0m║\x1b[0m", self.combo, self.build.cards_collected.len(), self.ammo, self.build.effective_mag_size());
        println!("  \x1b[38;2;180;0;0m╠══════════════════════════════════════════════════════════════════════════╣\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[97m\x1b[1m  ■ POST-PROCESSING EFFECT TOGGLES\x1b[0m                                        \x1b[38;2;180;0;0m║\x1b[0m");

        let print_toggle = |key: &str, name: &str, enabled: bool| -> String {
            let status = if enabled {
                "\x1b[92m██ ON \x1b[0m"
            } else {
                "\x1b[90m░░ OFF\x1b[0m"
            };
            format!("[{}] {:.<18} {}", key, name, status)
        };

        let t_vignette = print_toggle("4", "Vignette", is_vignette);
        let t_bloom = print_toggle("5", "Bloom", is_bloom);
        let t_grain = print_toggle("6", "Film Grain", is_grain);
        let t_chromatic = print_toggle("7", "Chromatic Ab.", is_chromatic);
        let t_fxaa = print_toggle("8", "FXAA", is_fxaa);
        let t_sharpen = print_toggle("9", "Sharpen", is_sharpen);
        let t_tonemap = print_toggle("0", "Tone Mapping", is_tonemap);
        let t_ssgi = print_toggle("Z", "SSGI", is_ssgi);
        let t_fog = print_toggle("X", "Vol. Fog", is_fog);
        let t_lut = print_toggle("C", "LUT Grade", is_lut);
        let t_ssr = print_toggle("T", "SSR", is_ssr);
        let t_pt = print_toggle("Y", "PT Resolve", is_pt);
        let t_flares = print_toggle("U", "Neon Flares", is_flares);

        println!(
            "  \x1b[38;2;180;0;0m║\x1b[0m    {}          {}    \x1b[38;2;180;0;0m║\x1b[0m",
            t_vignette, t_bloom
        );
        println!(
            "  \x1b[38;2;180;0;0m║\x1b[0m    {}          {}    \x1b[38;2;180;0;0m║\x1b[0m",
            t_grain, t_chromatic
        );
        println!(
            "  \x1b[38;2;180;0;0m║\x1b[0m    {}          {}    \x1b[38;2;180;0;0m║\x1b[0m",
            t_fxaa, t_sharpen
        );
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    {}                                          \x1b[38;2;180;0;0m║\x1b[0m", t_tonemap);
        println!(
            "  \x1b[38;2;180;0;0m║\x1b[0m    {}          {}    \x1b[38;2;180;0;0m║\x1b[0m",
            t_ssgi, t_fog
        );
        println!(
            "  \x1b[38;2;180;0;0m║\x1b[0m    {}          {}    \x1b[38;2;180;0;0m║\x1b[0m",
            t_lut, t_ssr
        );
        println!(
            "  \x1b[38;2;180;0;0m║\x1b[0m    {}          {}    \x1b[38;2;180;0;0m║\x1b[0m",
            t_pt, t_flares
        );
        println!("  \x1b[38;2;180;0;0m║                                                                          ║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[0m   \x1b[1m■ VALUE ADJUSTMENTS\x1b[0m                                                    \x1b[38;2;180;0;0m║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    [G] Exposure: \x1b[93m{:<4}\x1b[0m   [B] Bloom Int: \x1b[92m{:<4}\x1b[0m   [N] Grain Int: \x1b[95m{:<4}\x1b[0m        \x1b[38;2;180;0;0m║\x1b[0m", exposure, bloom_intensity, grain_intensity);
        println!("  \x1b[38;2;180;0;0m╠══════════════════════════════════════════════════════════════════════════╣\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[97m\x1b[1m  ■ VULKAN RENDERING SYSTEM\x1b[0m                                              \x1b[38;2;180;0;0m║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    VSync: \x1b[96m{:<14}\x1b[0m | MSAA: \x1b[95m{:<3}\x1b[0m | Performance: \x1b[92m{:>6} FPS\x1b[0m           \x1b[38;2;180;0;0m║\x1b[0m", vsync_display, msaa_display, fps_display);
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    Pixel Inteligente: \x1b[92m{:<48}\x1b[0m   \x1b[38;2;180;0;0m║\x1b[0m", pixel_display);
        println!("  \x1b[38;2;180;0;0m╠══════════════════════════════════════════════════════════════════════════╣\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[97m\x1b[1m  ■ ACTIVE BUILD CARDS\x1b[0m                                                    \x1b[38;2;180;0;0m║\x1b[0m");

        if self.build.cards_collected.is_empty() {
            println!("  \x1b[38;2;180;0;0m║\x1b[0m    \x1b[90mNo active cards. Survive a wave to choose a card!\x1b[0m                     \x1b[38;2;180;0;0m║\x1b[0m");
        } else {
            for card in &self.build.cards_collected {
                let clean_name = match card {
                    CardType::DoubleTap => "DOBLE TAP",
                    CardType::PiercingRounds => "RONDAS PERFORANTES",
                    CardType::ExplosiveShot => "DISPARO EXPLOSIVO",
                    CardType::ArmorPlating => "BLINDAJE",
                    CardType::Regeneration => "REGENERACION",
                    CardType::QuickReload => "RECARGA RAPIDA",
                    CardType::ExtendedMag => "CARGADOR EXTENDIDO",
                    CardType::ComboMaster => "MAESTRO DEL COMBO",
                    CardType::ScoreBonus => "BONUS DE PUNTOS",
                };
                let card_line = format!("    • {}: {}", clean_name, card.description());
                let spaces_needed = 74 - card_line.chars().count();
                let pad = " ".repeat(spaces_needed);
                println!(
                    "  \x1b[38;2;180;0;0m║\x1b[0m{}{}\x1b[38;2;180;0;0m║\x1b[0m",
                    card_line, pad
                );
            }
        }

        println!("  \x1b[38;2;180;0;0m╠══════════════════════════════════════════════════════════════════════════╣\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[97m\x1b[1m  ■ CONTROLS\x1b[0m                                                               \x1b[38;2;180;0;0m║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    P \x1b[91m→\x1b[0m Resume Game  |  V \x1b[91m→\x1b[0m Toggle VSync  |  F \x1b[91m→\x1b[0m Toggle Fullscreen       \x1b[38;2;180;0;0m║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    Esc \x1b[91m→\x1b[0m Quit Game  |  4-0 \x1b[91m→\x1b[0m Toggle Post-Process Effects                   \x1b[38;2;180;0;0m║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    G/B/N \x1b[91m→\x1b[0m Cycle Exposure/Bloom/Grain  |  I \x1b[91m→\x1b[0m Pixel Inteligente              \x1b[38;2;180;0;0m║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    Z/X/C/T \x1b[91m→\x1b[0m Toggle SSGI/Fog/LUT/SSR                                      \x1b[38;2;180;0;0m║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m║\x1b[0m    Y/U \x1b[91m→\x1b[0m Toggle PT Resolve/Neon Flares                                  \x1b[38;2;180;0;0m║\x1b[0m");
        println!("  \x1b[38;2;180;0;0m╚══════════════════════════════════════════════════════════════════════════╝\x1b[0m");
        println!();
    }
}

// =============================================================================
// REACTOR APP — Lifecycle del juego
// =============================================================================
impl ReactorApp for Xenofall {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("⚡ XENOFALL — Rail Shooter Roguelite")
            .with_size(3840, 2160)
            .with_vsync(false)
            .with_msaa(4)
            .with_renderer(RendererMode::Forward)
            .with_physics_hz(60)
    }

    fn on_event(&mut self, ctx: &mut ReactorContext, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput { event, .. } = event {
            if event.state == ElementState::Pressed && !event.repeat {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape | KeyCode::KeyP) => {
                        self.toggle_pause_config(ctx);
                        self.pause_event_consumed = true;
                        return true;
                    }
                    PhysicalKey::Code(KeyCode::KeyQ) if self.state == GameState::Paused => {
                        ctx.reactor.exit_requested = true;
                    }
                    _ => {}
                }
            }
        }

        false
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        print_banner();

        ctx.reactor
            .set_pixel_intelligent_profile(PixelIntelligentProfile::Performance);
        if ctx.reactor.pixel_intelligent_enabled() {
            Log::engine("Pixel Inteligente VRS activo para Xenofall");
        } else {
            Log::engine("Pixel Inteligente preparado; esta GPU usara shading nativo 1x1");
        }

        // ── Audio ──
        Log::audio("Cargando audio...");
        self.audio.load_all(&mut ctx.audio);

        // ── Cámara (human eye level 1.7m) ──
        ctx.camera.position = Vec3::new(0.0, CAMERA_Y, 0.0);
        ctx.camera.set_rotation(0.0, 0.0);

        self.apply_render_showcase_profile(ctx);

        // ── Iluminación atmosférica (Casa abandonada, Rumania) ──
        ctx.add_sun();
        ctx.add_directional_light(
            Vec3::new(-0.3, -1.0, -0.5),
            Vec3::new(0.45, 0.38, 0.32),
            0.45,
        );

        // Luces a lo largo del corredor — just below the ceiling
        for i in 0..12 {
            let z = -(i as f32 * 8.0 + 4.0);
            let flicker_color = if i % 3 == 0 {
                Vec3::new(0.45, 0.95, 0.78)
            } else {
                Vec3::new(1.0, 0.68, 0.34)
            };
            let intensity = if i % 4 == 0 { 4.8 } else { 3.2 };
            ctx.add_point_light(
                Vec3::new(0.0, CORRIDOR_HEIGHT - 0.3, z),
                flicker_color,
                intensity,
                11.5,
            );
        }

        // Luz roja de emergencia al fondo
        ctx.add_point_light(
            Vec3::new(0.0, CORRIDOR_HEIGHT * 0.8, -85.0),
            Vec3::new(1.0, 0.08, 0.025),
            7.5,
            28.0,
        );

        // ── Construir escenario ──
        self.build_corridor(ctx);
        self.build_pools(ctx);
        self.apply_render_showcase_materials(ctx);

        // ── Visuales de Interfaz (Crosshair y Overlays) ──
        if let Ok(idx) = ctx.spawn_colored_sphere(Vec3::ZERO, 0.02, 255, 0, 0, 255) {
            self.crosshair_index = Some(idx);
        }

        // Game Over Screen Overlay
        if let Ok(idx) = ctx.spawn_textured_quad(
            "assets/textures/game_over.png",
            Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
        ) {
            self.game_over_index = Some(idx);
        }

        // Victory Screen Overlay
        if let Ok(idx) = ctx.spawn_textured_quad(
            "assets/textures/victory.png",
            Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
        ) {
            self.victory_index = Some(idx);
        }

        Log::asset(&format!(
            "Corredor: {} segmentos, {} charcos cargados",
            self.world.floor_count(),
            self.world.puddle_count()
        ));
        Log::engine(&format!(
            "Pools: {} trazadores, {} impactos listos",
            self.vfx.tracer_count(),
            self.vfx.impact_count()
        ));
        Log::game(&format!("{} oleadas cargadas", self.waves.len()));
        Log::game("Sistema de cartas roguelite activo");
        self.print_render_showcase_budget(ctx);
        xenofall::visual_features::log_visual_feature_roadmap();
        Log::section("¡Sobrevive al corredor, Contractor!");
        println!();
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();
        self.t += dt;

        self.update_rail(dt);
        self.update_camera(ctx);
        self.update_shooting(ctx);
        self.update_waves(ctx);
        self.update_enemies(ctx);
        self.update_tracers(ctx);
        self.update_impacts(ctx);
        self.update_player(ctx);
        self.update_audio(ctx);
        self.update_hud(ctx);
        self.update_interface(ctx);
    }

    fn on_exit(&mut self, _ctx: &mut ReactorContext) {
        println!();
        Log::header("⚡ XENOFALL — After Action Report ⚡");
        let outcome = match self.state {
            GameState::Victory => "🏆 ¡VICTORIA!",
            GameState::GameOver => "💀 GAME OVER",
            _ => "🏁 Abandonado",
        };
        Log::kv("Resultado", outcome);
        Log::kv("Puntuación", &self.score.to_string());
        Log::kv("Kills", &self.kills.to_string());
        Log::kv("Headshots", &self.headshots.to_string());
        Log::kv("Disparos", &self.shots_fired.to_string());
        let acc = if self.shots_fired > 0 {
            (self.shots_hit as f32 / self.shots_fired as f32 * 100.0) as u32
        } else {
            0
        };
        Log::kv("Precisión", &format!("{}%", acc));
        Log::kv(
            "Oleadas",
            &format!("{}/{}", self.current_wave, self.waves.len()),
        );

        if !self.build.cards_collected.is_empty() {
            Log::section("Cartas Coleccionadas (Build)");
            for card in &self.build.cards_collected {
                Log::info(&format!("• {}", card.name()));
            }
        }
        println!();
    }
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    reactor_vulkan::run(Xenofall::new());
}

// =============================================================================
// BANNER
// =============================================================================

fn print_banner() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                                                                  ║");
    println!("║     ██╗  ██╗███████╗███╗   ██╗ ██████╗ ███████╗ ██████╗ _/       ║");
    println!("║     ╚██╗██╔╝██╔════╝████╗  ██║██╔═══██╗██╔════╝██╔═══██╗         ║");
    println!("║      ╚███╔╝ █████╗  ██╔██╗ ██║██║   ██║█████╗  ██║   ██║         ║");
    println!("║      ██╔██╗ ██╔══╝  ██║╚██╗██║██║   ██║██╔══╝  ██║   ██║         ║");
    println!("║     ██╔╝ ██╗███████╗██║ ╚████║╚██████╔╝██║     ╚██████╔╝         ║");
    println!("║     ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝      ╚═════╝          ║");
    println!("║                                                                  ║");
    println!("║     R A I L   S H O O T E R   R O G U E L I T E                  ║");
    println!("║     REACTOR 1.6.0 · Vulkan · Rumania, Día 47                     ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Controles:                                                      ║");
    println!("║    Mouse           → Apuntar (el cursor es la mira)              ║");
    println!("║    Click Izquierdo → TAP rápido = x2 dmg / HOLD = x0.5 dmg       ║");
    println!("║    R               → Recargar                                    ║");
    println!("║    P               → Pausar                                      ║");
    println!("║    V               → Alternar VSync (¡Desbloquear FPS!)          ║");
    println!("║    F / F11         → Alternar Pantalla Completa                  ║");
    println!("║    1/2/3           → Seleccionar carta                           ║");
    println!("║    Esc             → Salir                                       ║");
    println!("║                                                                  ║");
    println!("║  Mecánicas:                                                      ║");
    println!("║    • TAP rápido (click sucesivo < 250ms) → daño x2               ║");
    println!("║    • HOLD (mantener > 350ms) → daño x0.5 pero sin spread         ║");
    println!("║    • Combo hasta x10 → multiplicador de score                    ║");
    println!("║    • Cartas roguelite cada 2 oleadas → modifica tu build         ║");
    println!("║    • Score cap: 9,999,999                                        ║");
    println!("║                                                                  ║");
    println!("║  Lore: Nave alienígena cayó en Rumania.                          ║");
    println!("║  Laboratorio ADN. Infección Día 47. The Contractor enviado solo. ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
}
