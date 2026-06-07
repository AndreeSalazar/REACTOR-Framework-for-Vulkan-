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

fn main() {
    reactor_vulkan::run(crate::xenofall::new());
}

// =============================================================================
// BANNER
// =============================================================================
