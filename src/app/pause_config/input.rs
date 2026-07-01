use winit::keyboard::KeyCode;

use super::utils::{cycle_float, cycle_pixel_intelligent, toggle_effect, toggle_fullscreen, toggle_vsync};
use super::PauseConfigPage;
use crate::app::context::ReactorContext;
use crate::graphics::post_process::{PostProcessEffect, PostProcessSettings};

pub(super) struct PauseConfigInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub activate: bool,
    pub prev_page: bool,
    pub next_page: bool,
    pub page: Option<PauseConfigPage>,
    pub resume: bool,
    pub quit: bool,
    pub toggle_vsync: bool,
    pub toggle_fullscreen: bool,
    pub toggle_post_process: bool,
    pub pixel_intelligent: bool,
    pub cycle_exposure: bool,
    pub cycle_bloom: bool,
    pub cycle_grain: bool,
    pub k4: bool,
    pub k5: bool,
    pub k6: bool,
    pub k7: bool,
    pub k8: bool,
    pub k9: bool,
    pub k0: bool,
    pub k_z: bool,
    pub k_x: bool,
    pub k_c: bool,
    pub k_t: bool,
    pub k_y: bool,
    pub k_u: bool,
}

impl PauseConfigInput {
    pub fn capture(ctx: &ReactorContext) -> Self {
        let input = ctx.input();
        let page = if input.is_key_just_pressed(KeyCode::F1) {
            Some(PauseConfigPage::Display)
        } else if input.is_key_just_pressed(KeyCode::F2) {
            Some(PauseConfigPage::Lighting)
        } else if input.is_key_just_pressed(KeyCode::F3) {
            Some(PauseConfigPage::Color)
        } else if input.is_key_just_pressed(KeyCode::F4) {
            Some(PauseConfigPage::Performance)
        } else if input.is_key_just_pressed(KeyCode::F5) {
            Some(PauseConfigPage::Presets)
        } else {
            None
        };

        Self {
            up: input.is_key_just_pressed(KeyCode::ArrowUp),
            down: input.is_key_just_pressed(KeyCode::ArrowDown),
            left: input.is_key_just_pressed(KeyCode::ArrowLeft),
            right: input.is_key_just_pressed(KeyCode::ArrowRight),
            activate: input.is_key_just_pressed(KeyCode::Enter) || input.is_key_just_pressed(KeyCode::Space),
            prev_page: input.is_key_just_pressed(KeyCode::BracketLeft),
            next_page: input.is_key_just_pressed(KeyCode::BracketRight),
            page,
            resume: input.is_key_just_pressed(KeyCode::KeyP) || input.is_key_just_pressed(KeyCode::Escape),
            quit: input.is_key_just_pressed(KeyCode::KeyQ),
            toggle_vsync: input.is_key_just_pressed(KeyCode::KeyV),
            toggle_fullscreen: input.is_key_just_pressed(KeyCode::F11) || input.is_key_just_pressed(KeyCode::KeyF),
            toggle_post_process: input.is_key_just_pressed(KeyCode::KeyO),
            pixel_intelligent: input.is_key_just_pressed(KeyCode::KeyI),
            cycle_exposure: input.is_key_just_pressed(KeyCode::KeyG),
            cycle_bloom: input.is_key_just_pressed(KeyCode::KeyB),
            cycle_grain: input.is_key_just_pressed(KeyCode::KeyN),
            k4: input.is_key_just_pressed(KeyCode::Digit4),
            k5: input.is_key_just_pressed(KeyCode::Digit5),
            k6: input.is_key_just_pressed(KeyCode::Digit6),
            k7: input.is_key_just_pressed(KeyCode::Digit7),
            k8: input.is_key_just_pressed(KeyCode::Digit8),
            k9: input.is_key_just_pressed(KeyCode::Digit9),
            k0: input.is_key_just_pressed(KeyCode::Digit0),
            k_z: input.is_key_just_pressed(KeyCode::KeyZ),
            k_x: input.is_key_just_pressed(KeyCode::KeyX),
            k_c: input.is_key_just_pressed(KeyCode::KeyC),
            k_t: input.is_key_just_pressed(KeyCode::KeyT),
            k_y: input.is_key_just_pressed(KeyCode::KeyY),
            k_u: input.is_key_just_pressed(KeyCode::KeyU),
        }
    }
}

impl super::PauseConfig {
    pub(super) fn apply_direct_hotkeys(&mut self, ctx: &mut ReactorContext, input: &PauseConfigInput) -> bool {
        let mut changed = false;

        if input.toggle_vsync { changed |= toggle_vsync(ctx); }
        if input.toggle_fullscreen { changed |= toggle_fullscreen(ctx); }
        if input.toggle_post_process { ctx.reactor.post_process.enabled = !ctx.reactor.post_process.enabled; changed = true; }
        if input.pixel_intelligent { changed |= cycle_pixel_intelligent(ctx); }

        let s = &mut ctx.reactor.post_process.settings;
        changed |= input.k4 && toggle_effect(s, PostProcessEffect::Vignette);
        changed |= input.k5 && toggle_effect(s, PostProcessEffect::Bloom);
        changed |= input.k6 && toggle_effect(s, PostProcessEffect::FilmGrain);
        changed |= input.k7 && toggle_effect(s, PostProcessEffect::ChromaticAberration);
        changed |= input.k8 && toggle_effect(s, PostProcessEffect::FXAA);
        changed |= input.k9 && toggle_effect(s, PostProcessEffect::Sharpen);
        changed |= input.k0 && toggle_effect(s, PostProcessEffect::ToneMapping);
        changed |= input.k_z && toggle_effect(s, PostProcessEffect::SSGI);
        changed |= input.k_x && toggle_effect(s, PostProcessEffect::VolumetricFog);
        changed |= input.k_c && toggle_effect(s, PostProcessEffect::LutColorGrading);
        changed |= input.k_t && toggle_effect(s, PostProcessEffect::SSR);
        changed |= input.k_y && toggle_effect(s, PostProcessEffect::PathTracedLighting);
        changed |= input.k_u && toggle_effect(s, PostProcessEffect::AnamorphicFlares);

        if input.cycle_exposure { s.exposure = cycle_float(s.exposure, &[0.80, 1.00, 1.15, 1.50, 2.00]); changed = true; }
        if input.cycle_bloom { s.bloom_intensity = cycle_float(s.bloom_intensity, &[0.10, 0.20, 0.35, 0.50, 0.80]); changed = true; }
        if input.cycle_grain { s.grain_intensity = cycle_float(s.grain_intensity, &[0.00, 0.003, 0.006, 0.012]); changed = true; }

        changed
    }
}
