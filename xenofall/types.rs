// Extracted from xenofall.rs — Types
// =============================================================================
// TIPOS — Enums and game entity structs
// =============================================================================

use reactor_vulkan::prelude::*;

// =============================================================================
// ENUMS
// =============================================================================

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyState {
    Alive,
    Dying,
    Dead,
}

/// Estado global del juego
#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    Playing,
    Paused,
    CardSelect,
    GameOver,
    Victory,
}

/// Tipo de disparo detectado
#[derive(Clone, Copy, PartialEq)]
pub enum FireMode {
    Tap,
    Hold,
    Normal,
}

// =============================================================================
// STRUCTS
// =============================================================================

pub struct Enemy {
    pub _scene_indices: Vec<usize>,
    pub position: Vec3,
    pub health: i32,
    pub max_health: i32,
    pub state: EnemyState,
    pub death_timer: f32,
    pub attack_timer: f32,
    pub speed: f32,
    pub is_gltf: bool,
    pub _id: u32,
    /// Índice del blob shadow asociado (Fase 4.3 HOTD-style).
    pub blob_shadow: Option<usize>,
    /// Escala glTF aplicada por `spawn_gltf_smart` (auto-calculada por REACTOR
    /// a partir de la altura nativa del modelo en Blender).
    pub _gltf_scale: f32,
    /// Transforms locales originales de cada malla para mantener la jerarquía al mover el zombie
    pub initial_transforms: Vec<(usize, Mat4)>,
}

pub struct Tracer {
    pub pool_index: usize,
    pub position: Vec3,
    pub direction: Vec3,
    pub lifetime: f32,
}

pub struct Impact {
    pub pool_index: usize,
    pub position: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Definición de una oleada de enemigos
#[derive(Clone, Copy)]
pub struct WaveDef {
    pub trigger_z: f32,
    pub count: u32,
    pub spread: f32,
    pub depth: f32,
    pub _height_range: (f32, f32),
    pub speed_mult: f32,
    pub enemy_hp: i32,
}
