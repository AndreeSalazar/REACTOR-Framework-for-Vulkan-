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
        crate::xenofall::ui::print_banner();

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
        crate::xenofall::visual_features::log_visual_feature_roadmap();
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
