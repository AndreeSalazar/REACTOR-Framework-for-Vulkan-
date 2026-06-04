// Extracted from xenofall.rs — Constants
// =============================================================================
// CONSTANTES DE GAMEPLAY
// =============================================================================

use reactor_vulkan::prelude::*;

// =============================================================================
// UE5-STANDARD PROPORTIONS (1 unit ≈ 1 meter)
// =============================================================================
// Model data (measured): zombie_basic.glb
//   Native vertex height: 178 cm (centimeter-scale model)
//   Root node scale in glTF: 0.017 (auto-converts cm → m)
//   Effective height at scale 1.0: 178 × 0.017 = 3.03m
//   For 1.8m zombie: scale = 1.8 / 3.03 ≈ 0.59 → use 0.6
//
// Corridor: 7m wide × 3.5m tall (realistic lab hallway)
// Human:    ~1.8m tall at eye level 1.7m
// =============================================================================

pub const RAIL_SPEED: f32 = 3.0;
pub const RAIL_LENGTH: f32 = 90.0;

pub const AIM_FOV_SCALE: f32 = 0.55;
pub const MAX_AMMO: u32 = 8;
pub const RELOAD_TIME: f32 = 1.2;
pub const FIRE_COOLDOWN: f32 = 0.18;

// TAP / HOLD thresholds
pub const TAP_WINDOW: f32 = 0.25;
pub const HOLD_THRESHOLD: f32 = 0.35;
pub const TAP_DAMAGE_MULT: f32 = 2.0;
pub const HOLD_DAMAGE_MULT: f32 = 0.5;

pub const TRACER_SPEED: f32 = 150.0;
pub const TRACER_LIFETIME: f32 = 0.4;
pub const TRACER_POOL_SIZE: usize = 12;
pub const IMPACT_POOL_SIZE: usize = 20;
pub const IMPACT_LIFETIME: f32 = 0.35;

// Corridor geometry constants
pub const CORRIDOR_HALF_WIDTH: f32 = 3.5; // Total 7m wide
pub const CORRIDOR_HEIGHT: f32 = 3.5;     // 3.5m ceiling
pub const PILLAR_X: f32 = 2.8;            // Pillar distance from center
pub const CAMERA_Y: f32 = 1.7;            // Human eye level (1.7m)

// Enemy constants — tuned for UE5-scale humans
pub const ENEMY_BASE_SPEED: f32 = 2.5;
pub const ENEMY_HIT_RADIUS: f32 = 0.5;       // Hitbox radius for ~1.8m tall zombie
pub const ENEMY_ATTACK_DIST: f32 = 2.5;
pub const ENEMY_ATTACK_COOLDOWN: f32 = 1.2;
pub const ENEMY_DEATH_DURATION: f32 = 0.6;

// Zombie model scale — measured: 178cm native × 0.017 root = 3.03m at 1.0
// For a 1.8m tall zombie: 0.6x. Using 0.6 for proper human height.
// Cube fallback dimensions (human proportions: 0.5m wide × 1.8m tall × 0.35m deep)
pub const ZOMBIE_CUBE_SCALE: Vec3 = Vec3::new(0.5, 1.8, 0.35);
// Y offset: zombie feet at floor (Y=0). Hitbox center at ~0.9m.
pub const ZOMBIE_GROUND_Y: f32 = 0.9;
// zombie_basic.glb faces +Z in its bind pose. Keep this at 0 so enemies face the player.
pub const ZOMBIE_MODEL_YAW_OFFSET: f32 = 0.0;

/// How far ahead of the camera enemies spawn (close enough to see immediately)
pub const ENEMY_SPAWN_DIST_MIN: f32 = 5.0;
pub const ENEMY_SPAWN_DIST_MAX: f32 = 12.0;
/// Brief delay after clearing a wave before camera resumes
pub const WAVE_CLEAR_DELAY: f32 = 1.0;

pub const PLAYER_MAX_HP: i32 = 100;
pub const DAMAGE_PER_HIT: i32 = 12;
pub const SCORE_PER_KILL: u32 = 100;
pub const COMBO_TIMEOUT: f32 = 2.5;
pub const MAX_COMBO: u32 = 10;
pub const HEADSHOT_MULTIPLIER: u32 = 3;
pub const SCORE_CAP: u32 = 9_999_999;

pub const MUZZLE_FLASH_DURATION: f32 = 0.06;
