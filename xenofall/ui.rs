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
    pub(crate) fn update_hud(&self, ctx: &mut ReactorContext) {
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

    pub(crate) fn update_interface(&mut self, ctx: &mut ReactorContext) {
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
    pub(crate) fn print_config_pause(&self, ctx: &ReactorContext) {
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

pub(crate) fn print_banner() {
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
