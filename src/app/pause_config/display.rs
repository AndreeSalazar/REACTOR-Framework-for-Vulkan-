use crate::app::context::ReactorContext;
use crate::graphics::post_process::{PostProcessEffect, PostProcessSettings};

use super::utils::{msaa_display, on_off, pixel_display, pixel_rate};
use super::{PauseConfig, PauseConfigPage};

pub(super) fn print(config: &PauseConfig, ctx: &ReactorContext) {
    let rows = config.rows(ctx);
    let width = 92usize;
    let line = "=".repeat(width);
    let page_label = format!(
        "{}  |  page {}/{}",
        config.page.title(),
        config.page_index() + 1,
        PauseConfigPage::ALL.len()
    );

    println!();
    println!("+{}+", line);
    println!("| {:<90} |", config.title);
    println!("| {:<90} |", page_label);
    println!("+{}+", line);
    println!(
        "| FPS {:>7.1} | VSync {:<3} | PP {:<3} | Pixel Inteligente {:<23} |",
        ctx.fps(),
        on_off(ctx.reactor.vsync),
        on_off(ctx.reactor.post_process.enabled),
        pixel_display(ctx)
    );
    println!("+{}+", line);

    for (i, row) in rows.iter().enumerate() {
        let cursor = if i == config.selected { ">" } else { " " };
        println!(
            "| {} {:<28} {:<18} {:<39} |",
            cursor, row.name, row.value, row.hint
        );
    }

    println!("+{}+", line);
    println!("| F1-F5 pages | Up/Down select | Left/Right adjust | Enter/Space toggle/apply     |");
    println!("| V VSync | F Fullscreen | O PostFX | I Pixel | 4-0 legacy FX | Z X C T Y U FX |");
    println!("| G Exposure | B Bloom | N Grain | Esc/P resume | Q quit                         |");
    println!("+{}+", line);
    println!();
}

struct Row {
    name: String,
    value: String,
    hint: String,
}

impl Row {
    fn bool(name: &str, value: bool, hint: &str) -> Self {
        Self::text(name, on_off(value), hint)
    }

    fn value(name: &str, value: f32, hint: &str) -> Self {
        Self::text(name, format!("{value:.3}"), hint)
    }

    fn effect(
        name: &str,
        settings: &PostProcessSettings,
        effect: PostProcessEffect,
        hint: &str,
    ) -> Self {
        Self::bool(name, settings.is_effect_enabled(effect), hint)
    }

    fn text(name: &str, value: impl Into<String>, hint: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.into(),
            hint: hint.to_string(),
        }
    }
}

impl PauseConfig {
    pub(super) fn rows(&self, ctx: &ReactorContext) -> Vec<Row> {
        let s = &ctx.reactor.post_process.settings;
        match self.page {
            PauseConfigPage::Display => vec![
                Row::bool(
                    "Post Process",
                    ctx.reactor.post_process.enabled,
                    "Full post stack",
                ),
                Row::value(
                    "HUD Opacity",
                    self.overlay_alpha,
                    "Transparent pause overlay",
                ),
                Row::bool("VSync", ctx.reactor.vsync, "Recreates swapchain"),
                Row::bool("Fullscreen", ctx.window.fullscreen().is_some(), "Borderless"),
                Row::value("Exposure", s.exposure, "HDR brightness"),
                Row::value("Gamma", s.gamma, "Output curve"),
                Row::value("Bloom Intensity", s.bloom_intensity, "Neon bleed"),
                Row::value("Bloom Threshold", s.bloom_threshold, "Bright cutoff"),
                Row::value("Film Grain", s.grain_intensity, "Cinematic noise"),
                Row::value("Chromatic Aberration", s.chromatic_intensity, "Lens dispersion"),
                Row::value("Vignette", s.vignette_intensity, "Edge darkening"),
                Row::value("Sharpen", s.sharpen_intensity, "Micro contrast"),
            ],
            PauseConfigPage::Lighting => vec![
                Row::effect("SSGI", s, PostProcessEffect::SSGI, "Screen-space GI"),
                Row::value("SSGI Intensity", s.ssgi_intensity, "Indirect light"),
                Row::value("SSGI Radius", s.ssgi_radius, "Bounce reach"),
                Row::effect("PT Resolve", s, PostProcessEffect::PathTracedLighting, "Multi-bounce resolve"),
                Row::value("PT Intensity", s.pathtrace_intensity, "Cyberpunk-style GI"),
                Row::effect("SSR", s, PostProcessEffect::SSR, "Wet reflections"),
                Row::value("SSR Strength", s.ssr_strength, "Reflection mix"),
                Row::effect("Volumetric Fog", s, PostProcessEffect::VolumetricFog, "Light scattering"),
                Row::value("Fog Density", s.fog_density, "Atmosphere"),
                Row::value("Fog Scatter", s.fog_scatter, "Shaft brightness"),
                Row::effect("Neon Flares", s, PostProcessEffect::AnamorphicFlares, "Anamorphic lens"),
                Row::value("Flare Intensity", s.flare_intensity, "Horizontal streaks"),
                Row::value("Highlight Recovery", s.highlight_recovery, "Anti blowout"),
            ],
            PauseConfigPage::Color => vec![
                Row::effect("LUT Color Grade", s, PostProcessEffect::LutColorGrading, "Final look"),
                Row::value("LUT Strength", s.lut_strength, "Grade amount"),
                Row::effect("Tone Mapping", s, PostProcessEffect::ToneMapping, "ACES curve"),
                Row::effect("Bloom", s, PostProcessEffect::Bloom, "Glow pass"),
                Row::effect("Vignette", s, PostProcessEffect::Vignette, "Cinematic frame"),
                Row::effect("Chromatic Aberration", s, PostProcessEffect::ChromaticAberration, "Lens split"),
                Row::effect("Film Grain", s, PostProcessEffect::FilmGrain, "Noise overlay"),
                Row::effect("FXAA", s, PostProcessEffect::FXAA, "Fast AA"),
                Row::effect("Sharpen", s, PostProcessEffect::Sharpen, "Detail"),
                Row::effect("Grayscale", s, PostProcessEffect::Grayscale, "Debug/look"),
                Row::effect("Sepia", s, PostProcessEffect::Sepia, "Vintage look"),
                Row::effect("Invert", s, PostProcessEffect::Invert, "Debug/look"),
                Row::effect("Blur", s, PostProcessEffect::Blur, "Soft focus"),
            ],
            PauseConfigPage::Performance => vec![
                Row::text("Pixel Inteligente", format!("{:?}", ctx.reactor.pixel_intelligent.profile), "VRS profile"),
                Row::text("Current VRS Rate", pixel_rate(ctx), "Native fallback if unsupported"),
                Row::bool("Post Process", ctx.reactor.post_process.enabled, "GPU cost"),
                Row::effect("FXAA", s, PostProcessEffect::FXAA, "Cheap AA"),
                Row::effect("PT Resolve", s, PostProcessEffect::PathTracedLighting, "High cost"),
                Row::effect("SSR", s, PostProcessEffect::SSR, "Medium cost"),
                Row::effect("Volumetric Fog", s, PostProcessEffect::VolumetricFog, "Medium cost"),
                Row::effect("Neon Flares", s, PostProcessEffect::AnamorphicFlares, "Medium cost"),
                Row::effect("Film Grain", s, PostProcessEffect::FilmGrain, "Tiny cost"),
                Row::text("MSAA", msaa_display(ctx), "Fixed at pipeline creation"),
            ],
            PauseConfigPage::Presets => vec![
                Row::text("Cinematic Ultra", "APPLY", "PT, SSR, Fog, LUT, flares"),
                Row::text("Performance", "APPLY", "Keeps image clean and fast"),
                Row::text("Clean Competitive", "APPLY", "No grain/chromatic noise"),
                Row::text("Neon Overdrive", "APPLY", "Cyberpunk punch"),
                Row::text("Reset Defaults", "APPLY", "REACTOR defaults"),
            ],
        }
    }
}
