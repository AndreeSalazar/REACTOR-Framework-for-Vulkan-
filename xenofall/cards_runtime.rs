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
    pub(crate) fn enter_card_select(&mut self) {
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

    pub(crate) fn select_card(&mut self, index: usize, ctx: &mut ReactorContext) {
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


}
