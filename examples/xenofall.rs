// =============================================================================
// XENOFALL — Rail Shooter / Light Gun Shooter sobre REACTOR
// =============================================================================
// Inspirado en "House of the Dead" — plantilla base de videojuego REACTOR.
//
// 🎮 Mecánicas implementadas:
//   ✓ Cámara sobre rieles (avanza automáticamente por el corredor)
//   ✓ Apuntado con mouse (el cursor ES la mira)
//   ✓ Disparo con click izquierdo + trazador visible
//   ✓ Recarga con R (8 balas por cargador)
//   ✓ Enemigos zombie en oleadas (cubos que se acercan al jugador)
//   ✓ Sistema de vida (el jugador pierde si los zombies llegan)
//   ✓ Puntuación con multiplicador por combo
//   ✓ Efectos visuales: trazadores, impactos, animación de muerte
//   ✓ HUD completo en el título de la ventana
//   ✓ 8 oleadas con dificultad creciente
//   ✓ Sin un solo `unsafe` en el código de gameplay
//
// 🕹️ Controles:
//   Mouse       → Apuntar (el cursor es la mira)
//   Click Izq.  → Disparar
//   R           → Recargar
//   P           → Pausar
//   Esc         → Salir
//
// 💡 Para usar como plantilla:
//   1. Copia este archivo como `mi_juego.rs`
//   2. Añade [[example]] en Cargo.toml
//   3. Modifica las constantes y las oleadas
//   4. Reemplaza las primitivas por modelos 3D reales
// =============================================================================

use reactor_vulkan::prelude::*;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

// =============================================================================
// CONSTANTES DE GAMEPLAY — ajústalas a tu gusto
// =============================================================================

const RAIL_SPEED: f32 = 3.5; // Velocidad de avance (m/s)
const RAIL_LENGTH: f32 = 90.0; // Longitud total del corredor

const AIM_FOV_SCALE: f32 = 0.55; // Escala del FOV para el raycast del mouse
const MAX_AMMO: u32 = 8; // Balas por cargador
const RELOAD_TIME: f32 = 1.2; // Segundos para recargar
const FIRE_COOLDOWN: f32 = 0.18; // Segundos entre disparos

const TRACER_SPEED: f32 = 150.0; // Velocidad del trazador (m/s)
const TRACER_LIFETIME: f32 = 0.4; // Tiempo de vida del trazador (s)
const TRACER_POOL_SIZE: usize = 12; // Trazadores simultáneos
const IMPACT_POOL_SIZE: usize = 20; // Impactos simultáneos
const IMPACT_LIFETIME: f32 = 0.35; // Duración del impacto (s)

const ENEMY_BASE_SPEED: f32 = 2.0; // Velocidad base de los zombies (m/s)
const ENEMY_HIT_RADIUS: f32 = 0.9; // Radio de colisión del enemigo
const ENEMY_ATTACK_DIST: f32 = 2.5; // Distancia a la que hacen daño
const ENEMY_ATTACK_COOLDOWN: f32 = 1.2; // Cooldown entre ataques del zombie
const ENEMY_DEATH_DURATION: f32 = 0.6; // Duración de la animación de muerte

const PLAYER_MAX_HP: i32 = 100;
const DAMAGE_PER_HIT: i32 = 12; // Daño que hace un zombie al jugador
const SCORE_PER_KILL: u32 = 100;
const COMBO_TIMEOUT: f32 = 2.5; // Segundos antes de resetear el combo
const HEADSHOT_MULTIPLIER: u32 = 3; // Multiplicador por headshot

const MUZZLE_FLASH_DURATION: f32 = 0.06; // Duración del flash del disparo

// =============================================================================
// TIPOS
// =============================================================================

#[derive(Clone, Copy, PartialEq)]
enum EnemyState {
    Alive,
    Dying,
    Dead,
}

struct Enemy {
    scene_index: usize,
    position: Vec3,
    health: i32,
    max_health: i32,
    state: EnemyState,
    death_timer: f32,
    attack_timer: f32,
    speed: f32,
    id: u32, // para pseudo-random determinista
}

struct Tracer {
    pool_index: usize,
    position: Vec3,
    direction: Vec3,
    lifetime: f32,
}

struct Impact {
    pool_index: usize,
    position: Vec3,
    lifetime: f32,
    max_lifetime: f32,
}

/// Definición de una oleada de enemigos
#[derive(Clone, Copy)]
struct WaveDef {
    /// Distancia Z del riel donde se activa esta oleada
    trigger_z: f32,
    /// Cuántos enemigos
    count: u32,
    /// Dispersión lateral (±metros)
    spread: f32,
    /// Profundidad de la formación (metros delante del trigger)
    depth: f32,
    /// Rango de altura (min, max)
    height_range: (f32, f32),
    /// Multiplicador de velocidad
    speed_mult: f32,
    /// Vida de cada enemigo
    enemy_hp: i32,
}

/// Estado global del juego
#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    Paused,
    GameOver,
    Victory,
}

// =============================================================================
// HELPERS
// =============================================================================

/// Hash pseudo-random determinista (no necesita crate `rand`)
fn hash_rand(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(2654435761);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85ebca6b);
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2ae35);
    x ^= x >> 16;
    (x % 10000) as f32 / 10000.0
}

/// Hash pseudo-random en rango [-1, 1]
fn hash_rand_signed(seed: u32) -> f32 {
    hash_rand(seed) * 2.0 - 1.0
}

/// Intersección rayo–esfera. Retorna la distancia t o None.
fn ray_sphere_intersect(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = origin - center;
    let a = dir.dot(dir);
    let b = 2.0 * oc.dot(dir);
    let c = oc.dot(oc) - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        None
    } else {
        let t = (-b - disc.sqrt()) / (2.0 * a);
        if t > 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

/// Intersección rayo–esfera más pequeña (headshot). Retorna distancia t o None.
fn ray_headshot_intersect(origin: Vec3, dir: Vec3, center: Vec3) -> Option<f32> {
    // Headshot: esfera más pequeña en la parte superior del enemigo
    let head_center = center + Vec3::new(0.0, 0.5, 0.0);
    ray_sphere_intersect(origin, dir, head_center, 0.3)
}

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

    // Entidades
    enemies: Vec<Enemy>,
    active_tracers: Vec<Tracer>,
    active_impacts: Vec<Impact>,

    // Pools de objetos pre-creados (para evitar alloc en runtime)
    tracer_pool: Vec<usize>, // scene indices
    impact_pool: Vec<usize>, // scene indices
    muzzle_flash_index: Option<usize>,

    // Escenario
    floor_indices: Vec<usize>,
    wall_indices: Vec<usize>,
    pillar_indices: Vec<usize>,
    light_indices: Vec<usize>,

    // Oleadas
    waves: Vec<WaveDef>,
    wave_index: usize,
    total_enemies_alive: u32,

    // ID counter para pseudo-random
    next_enemy_id: u32,

    // Timing
    t: f32,

    // Pending damage from enemies (accumulated in update_enemies, applied in update_player)
    damage_pending: i32,
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
            enemies: Vec::new(),
            active_tracers: Vec::new(),
            active_impacts: Vec::new(),
            tracer_pool: Vec::new(),
            impact_pool: Vec::new(),
            muzzle_flash_index: None,
            floor_indices: Vec::new(),
            wall_indices: Vec::new(),
            pillar_indices: Vec::new(),
            light_indices: Vec::new(),
            waves: Self::build_waves(),
            wave_index: 0,
            total_enemies_alive: 0,
            next_enemy_id: 0,
            t: 0.0,
            damage_pending: 0,
        }
    }

    /// Definición de las 8 oleadas del juego
    fn build_waves() -> Vec<WaveDef> {
        vec![
            // Oleada 1: Introducción — pocos zombies lentos
            WaveDef {
                trigger_z: 8.0,
                count: 3,
                spread: 3.0,
                depth: 5.0,
                height_range: (0.8, 0.8),
                speed_mult: 0.7,
                enemy_hp: 1,
            },
            // Oleada 2: Más enemigos, un poco más rápidos
            WaveDef {
                trigger_z: 18.0,
                count: 5,
                spread: 4.0,
                depth: 6.0,
                height_range: (0.8, 1.0),
                speed_mult: 0.8,
                enemy_hp: 1,
            },
            // Oleada 3: Mezcla de alturas
            WaveDef {
                trigger_z: 28.0,
                count: 4,
                spread: 5.0,
                depth: 4.0,
                height_range: (0.8, 2.0),
                speed_mult: 0.9,
                enemy_hp: 2,
            },
            // Oleada 4: Emboscada lateral
            WaveDef {
                trigger_z: 38.0,
                count: 7,
                spread: 5.5,
                depth: 8.0,
                height_range: (0.8, 0.8),
                speed_mult: 1.0,
                enemy_hp: 2,
            },
            // Oleada 5: Horda densa
            WaveDef {
                trigger_z: 48.0,
                count: 6,
                spread: 6.0,
                depth: 5.0,
                height_range: (0.8, 1.5),
                speed_mult: 1.1,
                enemy_hp: 2,
            },
            // Oleada 6: Rápidos y resistentes
            WaveDef {
                trigger_z: 58.0,
                count: 8,
                spread: 5.0,
                depth: 10.0,
                height_range: (0.8, 2.0),
                speed_mult: 1.2,
                enemy_hp: 3,
            },
            // Oleada 7: Caos total
            WaveDef {
                trigger_z: 68.0,
                count: 7,
                spread: 6.0,
                depth: 8.0,
                height_range: (0.8, 2.5),
                speed_mult: 1.3,
                enemy_hp: 3,
            },
            // Oleada 8: JEFE FINAL — muchos zombies fuertes
            WaveDef {
                trigger_z: 78.0,
                count: 12,
                spread: 7.0,
                depth: 12.0,
                height_range: (0.8, 1.5),
                speed_mult: 1.0,
                enemy_hp: 4,
            },
        ]
    }

    // =========================================================================
    // SPAWN HELPERS
    // =========================================================================

    fn build_corridor(&mut self, ctx: &mut ReactorContext) {
        // ── Suelo del corredor (varios segmentos a lo largo del riel) ──
        for i in 0..10 {
            let z = -(i as f32 * 10.0 + 5.0);
            if let Ok(idx) = ctx.spawn_plane(Vec3::new(0.0, 0.0, z), 12.0) {
                self.floor_indices.push(idx);
            }
        }

        // ── Paredes laterales ──
        for i in 0..18 {
            let z = -(i as f32 * 5.0 + 2.5);
            // Pared izquierda
            if let Ok(left) = ctx.spawn_cube(Vec3::ZERO) {
                ctx.set_transform(
                    left,
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(0.3, 5.0, 5.0),
                        Quat::IDENTITY,
                        Vec3::new(-6.5, 2.5, z),
                    ),
                );
                self.wall_indices.push(left);
            }
            // Pared derecha
            if let Ok(right) = ctx.spawn_cube(Vec3::ZERO) {
                ctx.set_transform(
                    right,
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(0.3, 5.0, 5.0),
                        Quat::IDENTITY,
                        Vec3::new(6.5, 2.5, z),
                    ),
                );
                self.wall_indices.push(right);
            }
        }

        // ── Pilares decorativos ──
        for i in 0..9 {
            let z = -(i as f32 * 10.0 + 7.0);
            if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
                ctx.set_transform(
                    idx,
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(0.6, 4.5, 0.6),
                        Quat::IDENTITY,
                        Vec3::new(-5.0, 2.25, z),
                    ),
                );
                self.pillar_indices.push(idx);
            }
            if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
                ctx.set_transform(
                    idx,
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(0.6, 4.5, 0.6),
                        Quat::IDENTITY,
                        Vec3::new(5.0, 2.25, z),
                    ),
                );
                self.pillar_indices.push(idx);
            }
        }

        // ── Techo (barras transversales para atmósfera) ──
        for i in 0..9 {
            let z = -(i as f32 * 10.0 + 5.0);
            if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
                ctx.set_transform(
                    idx,
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(13.0, 0.2, 0.5),
                        Quat::IDENTITY,
                        Vec3::new(0.0, 5.0, z),
                    ),
                );
                self.wall_indices.push(idx);
            }
        }
    }

    fn build_pools(&mut self, ctx: &mut ReactorContext) {
        // Pool de trazadores (esferas amarillas pequeñas)
        for _ in 0..TRACER_POOL_SIZE {
            if let Ok(idx) = ctx.spawn_sphere(Vec3::new(0.0, -1000.0, 0.0), 0.04) {
                self.tracer_pool.push(idx);
            }
        }

        // Pool de impactos (esferas rojas)
        for _ in 0..IMPACT_POOL_SIZE {
            if let Ok(idx) = ctx.spawn_sphere(Vec3::new(0.0, -1000.0, 0.0), 0.12) {
                self.impact_pool.push(idx);
            }
        }

        // Muzzle flash (esfera blanca frente a la cámara)
        if let Ok(idx) = ctx.spawn_sphere(Vec3::new(0.0, -1000.0, 0.0), 0.15) {
            self.muzzle_flash_index = Some(idx);
        }
    }

    fn spawn_enemy(&mut self, ctx: &mut ReactorContext, pos: Vec3, hp: i32, speed: f32) {
        let id = self.next_enemy_id;
        self.next_enemy_id += 1;

        if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
            // Escalar el cubo para que parezca un zombie (alto y delgado)
            let xf = Mat4::from_scale_rotation_translation(
                Vec3::new(0.6, 1.6, 0.4),
                Quat::IDENTITY,
                pos,
            );
            ctx.set_transform(idx, xf);

            self.enemies.push(Enemy {
                scene_index: idx,
                position: pos,
                health: hp,
                max_health: hp,
                state: EnemyState::Alive,
                death_timer: 0.0,
                attack_timer: 0.0,
                speed,
                id,
            });
            self.total_enemies_alive += 1;
        }
    }

    // =========================================================================
    // COMBAT
    // =========================================================================

    fn fire_weapon(&mut self, ctx: &mut ReactorContext, ray_origin: Vec3, ray_dir: Vec3) {
        if self.ammo == 0 || self.reloading || self.fire_cooldown > 0.0 {
            return;
        }

        self.ammo -= 1;
        self.fire_cooldown = FIRE_COOLDOWN;
        self.muzzle_flash_timer = MUZZLE_FLASH_DURATION;
        self.shots_fired += 1;

        // Spawn tracer
        if let Some(pool_idx) = self.find_free_tracer() {
            let tracer_pos = ray_origin + ray_dir * 0.5;
            ctx.set_transform(
                self.tracer_pool[pool_idx],
                Mat4::from_translation(tracer_pos),
            );
            self.active_tracers.push(Tracer {
                pool_index: pool_idx,
                position: tracer_pos,
                direction: ray_dir,
                lifetime: TRACER_LIFETIME,
            });
        }

        // Check hit against enemies
        let mut hit_enemy_idx: Option<usize> = None;
        let mut hit_point = ray_origin + ray_dir * 100.0; // default far point
        let mut is_headshot = false;

        // Check headshots first (smaller hitbox, higher priority)
        let mut closest_headshot = f32::MAX;
        for (i, enemy) in self.enemies.iter().enumerate() {
            if enemy.state != EnemyState::Alive {
                continue;
            }
            if let Some(t) = ray_headshot_intersect(ray_origin, ray_dir, enemy.position) {
                if t < closest_headshot && t < 100.0 {
                    closest_headshot = t;
                    hit_enemy_idx = Some(i);
                    hit_point = ray_origin + ray_dir * t;
                    is_headshot = true;
                }
            }
        }

        // If no headshot, check body hits
        if hit_enemy_idx.is_none() {
            let mut closest_body = f32::MAX;
            for (i, enemy) in self.enemies.iter().enumerate() {
                if enemy.state != EnemyState::Alive {
                    continue;
                }
                if let Some(t) =
                    ray_sphere_intersect(ray_origin, ray_dir, enemy.position, ENEMY_HIT_RADIUS)
                {
                    if t < closest_body && t < 100.0 {
                        closest_body = t;
                        hit_enemy_idx = Some(i);
                        hit_point = ray_origin + ray_dir * t;
                        is_headshot = false;
                    }
                }
            }
        }

        // Apply hit
        if let Some(idx) = hit_enemy_idx {
            self.shots_hit += 1;

            // Extraer datos del enemigo antes de cualquier llamada a &mut self
            let (_damage, died, was_headshot) = {
                let enemy = &mut self.enemies[idx];
                let damage = if is_headshot { enemy.max_health } else { 1 };
                enemy.health -= damage;
                let died = enemy.health <= 0;
                if died {
                    enemy.state = EnemyState::Dying;
                    enemy.death_timer = 0.0;
                }
                (damage, died, is_headshot)
            };

            // Spawn impact effect (ahora podemos usar &mut self sin conflictos)
            self.spawn_impact(ctx, hit_point);

            if died {
                self.kills += 1;
                self.total_enemies_alive = self.total_enemies_alive.saturating_sub(1);

                // Score calculation
                let combo_mult = 1 + self.combo;
                let headshot_bonus = if was_headshot { HEADSHOT_MULTIPLIER } else { 1 };
                self.score += SCORE_PER_KILL * combo_mult * headshot_bonus;
                if was_headshot {
                    self.headshots += 1;
                }
                self.combo += 1;
                self.combo_timer = COMBO_TIMEOUT;
            }
        } else {
            // Impact on wall/floor (at the end of the ray or at some distance)
            self.spawn_impact(ctx, hit_point);
        }

        // Auto-reload when empty
        if self.ammo == 0 {
            self.reloading = true;
            self.reload_timer = RELOAD_TIME;
        }
    }

    fn spawn_impact(&mut self, ctx: &mut ReactorContext, position: Vec3) {
        // Find a free impact from the pool
        let used_pools: Vec<usize> = self.active_impacts.iter().map(|i| i.pool_index).collect();
        for (pool_idx, &scene_idx) in self.impact_pool.iter().enumerate() {
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
        for pool_idx in 0..self.tracer_pool.len() {
            if !used.contains(&pool_idx) {
                return Some(pool_idx);
            }
        }
        None
    }

    // =========================================================================
    // UPDATE SUB-SYSTEMS
    // =========================================================================

    fn update_rail(&mut self, dt: f32) {
        if self.state != GameState::Playing {
            return;
        }
        if self.rail_progress < RAIL_LENGTH {
            self.rail_progress += RAIL_SPEED * dt;
        }
    }

    fn update_camera(&mut self, ctx: &mut ReactorContext) {
        // Camera follows the rail (moves along -Z)
        let cam_pos = Vec3::new(0.0, 2.0, -self.rail_progress);
        ctx.camera.position = cam_pos;

        // Camera looks straight ahead with slight bob
        let bob = (self.t * 3.0).sin() * 0.03;
        ctx.camera.position.y += bob;
        ctx.camera.set_rotation(0.0, 0.0);
    }

    fn get_aim_ray(&self, ctx: &ReactorContext) -> (Vec3, Vec3) {
        let mouse_pos = ctx.input().mouse_position();
        let (w, h) = ctx.window_size();
        let (w, h) = (w as f32, h as f32);

        // NDC coordinates (center = 0,0; edges = ±1)
        let ndc_x = (mouse_pos.x / w) * 2.0 - 1.0;
        let ndc_y = 1.0 - (mouse_pos.y / h) * 2.0;

        // Scale by FOV to get aim direction
        let fov_h = ctx.camera.fov * AIM_FOV_SCALE * ctx.camera.aspect_ratio;
        let fov_v = ctx.camera.fov * AIM_FOV_SCALE;

        let aim_yaw = ndc_x * fov_h * 0.5;
        let aim_pitch = ndc_y * fov_v * 0.5;

        // Build ray from camera position
        let forward = Vec3::new(0.0, 0.0, -1.0); // camera faces -Z
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
                self.ammo = MAX_AMMO;
            }
        }

        // Manual reload
        if ctx.input().is_key_just_pressed(KeyCode::KeyR) && !self.reloading && self.ammo < MAX_AMMO
        {
            self.reloading = true;
            self.reload_timer = RELOAD_TIME;
        }

        // Pause toggle
        if ctx.input().is_key_just_pressed(KeyCode::KeyP) {
            self.state = match self.state {
                GameState::Playing => GameState::Paused,
                GameState::Paused => GameState::Playing,
                other => other,
            };
        }

        // Restart on game over
        if self.state == GameState::GameOver || self.state == GameState::Victory {
            if ctx.input().is_key_just_pressed(KeyCode::Space) {
                // Simple restart: just exit and let the user re-run
                std::process::exit(0);
            }
        }

        // ESC to quit
        if ctx.input().is_key_just_pressed(KeyCode::Escape) {
            std::process::exit(0);
        }

        if self.state != GameState::Playing {
            return;
        }

        // Fire on click
        let (ray_origin, ray_dir) = self.get_aim_ray(ctx);
        if ctx.input().is_mouse_button_down(MouseButton::Left) {
            self.fire_weapon(ctx, ray_origin, ray_dir);
        }

        // Muzzle flash visual
        if let Some(flash_idx) = self.muzzle_flash_index {
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
                // Hide tracer
                ctx.set_transform(
                    self.tracer_pool[tracer.pool_index],
                    Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
                );
                to_remove.push(i);
            } else {
                ctx.set_transform(
                    self.tracer_pool[tracer.pool_index],
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
                    self.impact_pool[impact.pool_index],
                    Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
                );
                to_remove.push(i);
            } else {
                // Shrink over time
                let scale = (impact.lifetime / impact.max_lifetime) * 0.12;
                ctx.set_transform(
                    self.impact_pool[impact.pool_index],
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

        for enemy in &mut self.enemies {
            match enemy.state {
                EnemyState::Alive => {
                    // Move toward camera
                    let to_player = cam_pos - enemy.position;
                    let dist = to_player.length();

                    if dist > ENEMY_ATTACK_DIST {
                        let move_dir = to_player.normalize();
                        enemy.position += move_dir * enemy.speed * dt;

                        // Face the player
                        let facing = Quat::from_rotation_y((-move_dir.x).atan2(-move_dir.z));
                        ctx.set_transform(
                            enemy.scene_index,
                            Mat4::from_scale_rotation_translation(
                                Vec3::new(0.6, 1.6, 0.4),
                                facing,
                                enemy.position,
                            ),
                        );
                    } else {
                        // Attack the player
                        enemy.attack_timer -= dt;
                        if enemy.attack_timer <= 0.0 {
                            enemy.attack_timer = ENEMY_ATTACK_COOLDOWN;
                            // Signal that this enemy attacked (damage applied in update_player via flag)
                            self.damage_pending += DAMAGE_PER_HIT;
                        }
                    }
                }

                EnemyState::Dying => {
                    enemy.death_timer += dt;
                    let t = enemy.death_timer / ENEMY_DEATH_DURATION;

                    // Fall backward animation
                    let fall_angle = t * std::f32::consts::FRAC_PI_2;
                    let sink = t * 0.8;
                    let xf = Mat4::from_translation(enemy.position + Vec3::new(0.0, -sink, 0.0))
                        * Mat4::from_rotation_x(fall_angle)
                        * Mat4::from_scale(Vec3::new(0.6, 1.6, 0.4));
                    ctx.set_transform(enemy.scene_index, xf);

                    if enemy.death_timer >= ENEMY_DEATH_DURATION {
                        enemy.state = EnemyState::Dead;
                        ctx.set_transform(
                            enemy.scene_index,
                            Mat4::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
                        );
                    }
                }

                EnemyState::Dead => {
                    // Already hidden
                }
            }
        }

        // Clean up dead enemies from the list periodically
        self.enemies.retain(|e| e.state != EnemyState::Dead);
    }

    fn update_player(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();

        // Damage from enemies (accumulated in update_enemies)
        if self.state != GameState::Playing {
            return;
        }

        if self.damage_pending > 0 {
            self.hp -= self.damage_pending;
            self.damage_flash = 0.3;
            self.damage_pending = 0;
        }

        // Damage flash decay
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
        }
    }

    fn update_waves(&mut self, ctx: &mut ReactorContext) {
        if self.state != GameState::Playing {
            return;
        }
        if self.wave_index >= self.waves.len() {
            return;
        }

        let wave = self.waves[self.wave_index]; // Copy, no referencia

        // Check if we've reached the trigger point
        if self.rail_progress >= wave.trigger_z {
            self.current_wave = self.wave_index as u32 + 1;

            // Spawn all enemies for this wave
            for i in 0..wave.count {
                let seed = self.next_enemy_id + i;
                let x_offset = hash_rand_signed(seed * 7 + 1) * wave.spread * 0.5;
                let z_offset = hash_rand(seed * 13 + 3) * wave.depth;
                let y = wave.height_range.0
                    + hash_rand(seed * 19 + 5) * (wave.height_range.1 - wave.height_range.0);

                let pos = Vec3::new(x_offset, y, -(self.rail_progress + 12.0 + z_offset));

                let speed =
                    ENEMY_BASE_SPEED * wave.speed_mult * (0.8 + hash_rand(seed * 23 + 7) * 0.4);

                self.spawn_enemy(ctx, pos, wave.enemy_hp, speed);
            }

            self.wave_index += 1;
        }
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
            GameState::GameOver => "💀",
            GameState::Victory => "🏆",
        };

        let ammo_str = if self.reloading {
            format!("⟳ {:.1}s", self.reload_timer)
        } else {
            format!("{}/{}", self.ammo, MAX_AMMO)
        };

        let combo_str = if self.combo > 1 {
            format!(" x{}", self.combo)
        } else {
            String::new()
        };

        let flash = if self.damage_flash > 0.0 {
            "⚠️ "
        } else {
            ""
        };

        ctx.set_title(&format!(
            "{}{} XENOFALL · {} HP · {} balas · {} pts{} · {} kills · {}% · Ola {}/{} · {:.0} FPS",
            flash,
            state_icon,
            self.hp,
            ammo_str,
            self.score,
            combo_str,
            self.kills,
            accuracy,
            self.current_wave,
            self.waves.len(),
            ctx.fps(),
        ));

        // Print game over / victory messages
        if self.state == GameState::GameOver {
            if (self.t * 2.0).sin() > 0.0 {
                // Blink effect in console
                eprint!("");
            }
        }
    }
}

// =============================================================================
// REACTOR APP — Lifecycle del juego
// =============================================================================

impl ReactorApp for Xenofall {
    // ── Configuración ────────────────────────────────────────────────────────
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("⚡ XENOFALL — Rail Shooter")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_msaa(4)
            .with_renderer(RendererMode::Forward)
            .with_physics_hz(60)
    }

    // ── Inicialización ───────────────────────────────────────────────────────
    fn init(&mut self, ctx: &mut ReactorContext) {
        print_banner();

        // Cámara inicial
        ctx.camera.position = Vec3::new(0.0, 2.0, 0.0);
        ctx.camera.set_rotation(0.0, 0.0);

        // Iluminación atmosférica
        ctx.add_sun();
        ctx.add_directional_light(Vec3::new(-0.3, -1.0, -0.5), Vec3::new(0.6, 0.5, 0.4), 0.8);

        // Luces a lo largo del corredor
        for i in 0..10 {
            let z = -(i as f32 * 9.0 + 4.5);
            ctx.add_point_light(
                Vec3::new(0.0, 4.5, z),
                Vec3::new(1.0, 0.7, 0.4), // Luz cálida tipo antorcha
                2.5,
                12.0,
            );
        }

        // Luz roja de emergencia al fondo
        ctx.add_point_light(
            Vec3::new(0.0, 3.0, -85.0),
            Vec3::new(1.0, 0.2, 0.1),
            5.0,
            25.0,
        );

        // Construir escenario
        self.build_corridor(ctx);
        self.build_pools(ctx);

        println!(
            "[XENOFALL] Corredor construido · {} segmentos de suelo",
            self.floor_indices.len()
        );
        println!(
            "[XENOFALL] Pools: {} trazadores, {} impactos",
            self.tracer_pool.len(),
            self.impact_pool.len()
        );
        println!("[XENOFALL] {} oleadas cargadas", self.waves.len());
        println!("[XENOFALL] ¡Sobrevive al corredor, comandante!");
    }

    // ── Loop principal ───────────────────────────────────────────────────────
    fn update(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.delta();
        self.t += dt;

        // Actualizar subsistemas
        self.update_rail(dt);
        self.update_camera(ctx);
        self.update_shooting(ctx);
        self.update_waves(ctx);
        self.update_enemies(ctx);
        self.update_tracers(ctx);
        self.update_impacts(ctx);
        self.update_player(ctx);
        self.update_hud(ctx);
    }

    // ── Cleanup ──────────────────────────────────────────────────────────────
    fn on_exit(&mut self, _ctx: &mut ReactorContext) {
        println!();
        println!("╔═══════════════════════════════════════════════════╗");
        println!("║         ⚡ XENOFALL — After Action Report ⚡     ║");
        println!("╠═══════════════════════════════════════════════════╣");
        println!(
            "║  Resultado:  {}",
            match self.state {
                GameState::Victory => "🏆 ¡VICTORIA!",
                GameState::GameOver => "💀 GAME OVER",
                _ => "🏁 Abandonado",
            }
        );
        println!("║  Puntuación: {:>37} ║", self.score);
        println!("║  Kills:      {:>37} ║", self.kills);
        println!("║  Headshots:  {:>37} ║", self.headshots);
        println!("║  Disparos:   {:>37} ║", self.shots_fired);
        let acc = if self.shots_fired > 0 {
            (self.shots_hit as f32 / self.shots_fired as f32 * 100.0) as u32
        } else {
            0
        };
        println!("║  Precisión:  {:>36}% ║", acc);
        println!(
            "║  Oleadas:    {:>37} ║",
            format!("{}/{}", self.current_wave, self.waves.len())
        );
        println!("╚═══════════════════════════════════════════════════╝");
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
    println!("║          R A I L   S H O O T E R  ·  REACTOR 1.1.0               ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Controles:                                                      ║");
    println!("║    🖱️  Mouse          → Apuntar (el cursor es la mira)           ║");
    println!("║    🔫 Click Izquierdo → Disparar                                 ║");
    println!("║    🔄 R              → Recargar                                  ║");
    println!("║    ⏸️  P              → Pausar                                   ║");
    println!("║    🚪 Esc            → Salir                                     ║");
    println!("║                                                                  ║");
    println!("║  Objetivo: Sobrevive 8 oleadas a través del corredor.            ║");
    println!("║  ¡Apunta a la cabeza para daño triple!                           ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
}
