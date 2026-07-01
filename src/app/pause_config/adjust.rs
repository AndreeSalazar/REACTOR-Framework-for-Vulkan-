use crate::app::context::ReactorContext;
use crate::core::PixelIntelligentProfile;
use crate::graphics::post_process::{PostProcessEffect, PostProcessSettings};

use super::utils::{cycle_pixel_intelligent, step_f32, toggle_effect, toggle_fullscreen, toggle_vsync};
use super::PauseConfigPage;

impl super::PauseConfig {
    pub(super) fn adjust_selected(&mut self, ctx: &mut ReactorContext, dir: i32, activate: bool) -> bool {
        match self.page {
            PauseConfigPage::Display => self.adjust_display(ctx, dir, activate),
            PauseConfigPage::Lighting => self.adjust_lighting(ctx, dir, activate),
            PauseConfigPage::Color => self.adjust_color(ctx, dir, activate),
            PauseConfigPage::Performance => self.adjust_performance(ctx, dir, activate),
            PauseConfigPage::Presets => self.apply_preset(ctx, self.selected),
        }
    }

    fn adjust_display(&mut self, ctx: &mut ReactorContext, dir: i32, activate: bool) -> bool {
        match self.selected {
            0 => { ctx.reactor.post_process.enabled = !ctx.reactor.post_process.enabled; true }
            1 => step_f32(&mut self.overlay_alpha, dir, 0.05, 0.20, 1.00),
            2 => toggle_vsync(ctx),
            3 => toggle_fullscreen(ctx),
            4 => step_f32(&mut ctx.reactor.post_process.settings.exposure, dir, 0.05, 0.40, 2.50),
            5 => step_f32(&mut ctx.reactor.post_process.settings.gamma, dir, 0.05, 1.60, 2.80),
            6 => step_f32(&mut ctx.reactor.post_process.settings.bloom_intensity, dir, 0.05, 0.00, 2.00),
            7 => step_f32(&mut ctx.reactor.post_process.settings.bloom_threshold, dir, 0.02, 0.30, 1.60),
            8 => step_f32(&mut ctx.reactor.post_process.settings.grain_intensity, dir, 0.003, 0.00, 0.12),
            9 => step_f32(&mut ctx.reactor.post_process.settings.chromatic_intensity, dir, 0.0005, 0.00, 0.015),
            10 => step_f32(&mut ctx.reactor.post_process.settings.vignette_intensity, dir, 0.03, 0.00, 0.90),
            11 => step_f32(&mut ctx.reactor.post_process.settings.sharpen_intensity, dir, 0.03, 0.00, 1.50),
            _ => activate,
        }
    }

    fn adjust_lighting(&self, ctx: &mut ReactorContext, dir: i32, activate: bool) -> bool {
        let s = &mut ctx.reactor.post_process.settings;
        match self.selected {
            0 => toggle_effect(s, PostProcessEffect::SSGI),
            1 => step_f32(&mut s.ssgi_intensity, dir, 0.04, 0.00, 1.50),
            2 => step_f32(&mut s.ssgi_radius, dir, 1.00, 1.00, 40.00),
            3 => toggle_effect(s, PostProcessEffect::PathTracedLighting),
            4 => step_f32(&mut s.pathtrace_intensity, dir, 0.04, 0.00, 1.50),
            5 => toggle_effect(s, PostProcessEffect::SSR),
            6 => step_f32(&mut s.ssr_strength, dir, 0.04, 0.00, 1.50),
            7 => toggle_effect(s, PostProcessEffect::VolumetricFog),
            8 => step_f32(&mut s.fog_density, dir, 0.02, 0.00, 1.20),
            9 => step_f32(&mut s.fog_scatter, dir, 0.04, 0.00, 2.00),
            10 => toggle_effect(s, PostProcessEffect::AnamorphicFlares),
            11 => step_f32(&mut s.flare_intensity, dir, 0.04, 0.00, 2.00),
            12 => step_f32(&mut s.highlight_recovery, dir, 0.04, 0.00, 2.00),
            _ => activate,
        }
    }

    fn adjust_color(&self, ctx: &mut ReactorContext, dir: i32, activate: bool) -> bool {
        let s = &mut ctx.reactor.post_process.settings;
        match self.selected {
            0 => toggle_effect(s, PostProcessEffect::LutColorGrading),
            1 => step_f32(&mut s.lut_strength, dir, 0.04, 0.00, 1.50),
            2 => toggle_effect(s, PostProcessEffect::ToneMapping),
            3 => toggle_effect(s, PostProcessEffect::Bloom),
            4 => toggle_effect(s, PostProcessEffect::Vignette),
            5 => toggle_effect(s, PostProcessEffect::ChromaticAberration),
            6 => toggle_effect(s, PostProcessEffect::FilmGrain),
            7 => toggle_effect(s, PostProcessEffect::FXAA),
            8 => toggle_effect(s, PostProcessEffect::Sharpen),
            9 => toggle_effect(s, PostProcessEffect::Grayscale),
            10 => toggle_effect(s, PostProcessEffect::Sepia),
            11 => toggle_effect(s, PostProcessEffect::Invert),
            12 => toggle_effect(s, PostProcessEffect::Blur),
            _ => activate,
        }
    }

    fn adjust_performance(&self, ctx: &mut ReactorContext, _dir: i32, activate: bool) -> bool {
        match self.selected {
            0 => cycle_pixel_intelligent(ctx),
            2 => { ctx.reactor.post_process.enabled = !ctx.reactor.post_process.enabled; true }
            3 => toggle_effect(&mut ctx.reactor.post_process.settings, PostProcessEffect::FXAA),
            4 => toggle_effect(&mut ctx.reactor.post_process.settings, PostProcessEffect::PathTracedLighting),
            5 => toggle_effect(&mut ctx.reactor.post_process.settings, PostProcessEffect::SSR),
            6 => toggle_effect(&mut ctx.reactor.post_process.settings, PostProcessEffect::VolumetricFog),
            7 => toggle_effect(&mut ctx.reactor.post_process.settings, PostProcessEffect::AnamorphicFlares),
            8 => toggle_effect(&mut ctx.reactor.post_process.settings, PostProcessEffect::FilmGrain),
            _ => activate,
        }
    }

    fn apply_preset(&self, ctx: &mut ReactorContext, preset: usize) -> bool {
        let s = &mut ctx.reactor.post_process.settings;
        match preset {
            0 => {
                *s = PostProcessSettings::cinematic();
                s.exposure = 1.02;
                s.pathtrace_intensity = 0.68;
                s.ssr_strength = 0.42;
                s.flare_intensity = 0.52;
                s.highlight_recovery = 0.72;
            }
            1 => {
                *s = PostProcessSettings::default();
                s.disable_effect(PostProcessEffect::PathTracedLighting);
                s.disable_effect(PostProcessEffect::SSR);
                s.disable_effect(PostProcessEffect::VolumetricFog);
                s.disable_effect(PostProcessEffect::AnamorphicFlares);
                s.grain_intensity = 0.0;
                s.chromatic_intensity = 0.0;
                s.ssgi_intensity = 0.16;
                ctx.reactor.set_pixel_intelligent_profile(PixelIntelligentProfile::Performance);
            }
            2 => {
                *s = PostProcessSettings::default();
                s.disable_effect(PostProcessEffect::FilmGrain);
                s.disable_effect(PostProcessEffect::ChromaticAberration);
                s.disable_effect(PostProcessEffect::Blur);
                s.exposure = 0.98;
                s.grain_intensity = 0.0;
                s.chromatic_intensity = 0.0;
                s.highlight_recovery = 0.78;
            }
            3 => {
                *s = PostProcessSettings::cinematic();
                s.enable_effect(PostProcessEffect::Bloom);
                s.enable_effect(PostProcessEffect::AnamorphicFlares);
                s.bloom_intensity = 0.85;
                s.bloom_threshold = 0.68;
                s.flare_intensity = 0.95;
                s.fog_scatter = 0.82;
                s.lut_strength = 0.95;
            }
            _ => { *s = PostProcessSettings::default(); }
        }
        true
    }
}
