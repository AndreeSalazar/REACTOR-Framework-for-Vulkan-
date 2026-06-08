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
    pub(crate) fn new() -> Self {
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
            waves: crate::xenofall::waves::build_waves(),
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

    pub(crate) fn toggle_pause_config(&mut self, ctx: &mut ReactorContext) {
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
}
