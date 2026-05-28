use super::app::ReactorContext;
use crate::core::PixelIntelligentProfile;
use crate::graphics::post_process::{PostProcessEffect, PostProcessSettings};
use winit::keyboard::KeyCode;
use winit::window::Fullscreen;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseConfigPage {
    Display,
    Lighting,
    Color,
    Performance,
    Presets,
}

impl PauseConfigPage {
    const ALL: [Self; 5] = [
        Self::Display,
        Self::Lighting,
        Self::Color,
        Self::Performance,
        Self::Presets,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Display => "DISPLAY",
            Self::Lighting => "LIGHTING / PT",
            Self::Color => "COLOR / POST FX",
            Self::Performance => "PERFORMANCE",
            Self::Presets => "PRESETS",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PauseConfigResult {
    pub changed: bool,
    pub requested_resume: bool,
    pub requested_quit: bool,
}

#[derive(Debug, Clone)]
pub struct PauseConfig {
    pub title: String,
    pub page: PauseConfigPage,
    pub selected: usize,
    pub print_on_change: bool,
    dirty: bool,
}

impl Default for PauseConfig {
    fn default() -> Self {
        Self {
            title: "REACTOR PAUSE CONFIG".to_string(),
            page: PauseConfigPage::Display,
            selected: 0,
            print_on_change: true,
            dirty: true,
        }
    }
}

pub type PauseConfiguration = PauseConfig;
pub type PauseConfiguracion = PauseConfig;

impl PauseConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn update(&mut self, ctx: &mut ReactorContext) -> PauseConfigResult {
        let input = PauseConfigInput::capture(ctx);
        let mut result = PauseConfigResult::default();

        if input.resume {
            result.requested_resume = true;
        }
        if input.quit {
            result.requested_quit = true;
        }

        if let Some(page) = input.page {
            self.page = page;
            self.selected = self.selected.min(self.row_count().saturating_sub(1));
            self.dirty = true;
        }

        if input.prev_page || input.next_page {
            self.step_page(if input.next_page { 1 } else { -1 });
            self.dirty = true;
        }

        if input.up || input.down {
            self.step_selected(if input.down { 1 } else { -1 });
            self.dirty = true;
        }

        if input.left || input.right || input.activate {
            let dir = if input.left { -1 } else { 1 };
            result.changed |= self.adjust_selected(ctx, dir, input.activate);
        }

        result.changed |= self.apply_direct_hotkeys(ctx, &input);
        if result.changed {
            self.dirty = true;
        }

        if self.print_on_change && self.dirty {
            self.print(ctx);
            self.dirty = false;
        }

        result
    }

    pub fn print(&self, ctx: &ReactorContext) {
        let rows = self.rows(ctx);
        let width = 92usize;
        let line = "=".repeat(width);
        let page_label = format!(
            "{}  |  page {}/{}",
            self.page.title(),
            self.page_index() + 1,
            PauseConfigPage::ALL.len()
        );

        println!();
        println!("+{}+", line);
        println!("| {:<90} |", self.title);
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
            let cursor = if i == self.selected { ">" } else { " " };
            println!(
                "| {} {:<28} {:<18} {:<39} |",
                cursor, row.name, row.value, row.hint
            );
        }

        println!("+{}+", line);
        println!(
            "| F1-F5 pages | Up/Down select | Left/Right adjust | Enter/Space toggle/apply     |"
        );
        println!(
            "| V VSync | F Fullscreen | O PostFX | I Pixel | 4-0 legacy FX | Z X C T Y U FX |"
        );
        println!(
            "| G Exposure | B Bloom | N Grain | P resume | Esc quit                         |"
        );
        println!("+{}+", line);
        println!();
    }

    fn row_count(&self) -> usize {
        match self.page {
            PauseConfigPage::Display => 11,
            PauseConfigPage::Lighting => 13,
            PauseConfigPage::Color => 13,
            PauseConfigPage::Performance => 10,
            PauseConfigPage::Presets => 5,
        }
    }

    fn page_index(&self) -> usize {
        PauseConfigPage::ALL
            .iter()
            .position(|p| *p == self.page)
            .unwrap_or(0)
    }

    fn step_page(&mut self, dir: i32) {
        let len = PauseConfigPage::ALL.len() as i32;
        let next = (self.page_index() as i32 + dir).rem_euclid(len) as usize;
        self.page = PauseConfigPage::ALL[next];
        self.selected = self.selected.min(self.row_count().saturating_sub(1));
    }

    fn step_selected(&mut self, dir: i32) {
        let len = self.row_count() as i32;
        self.selected = (self.selected as i32 + dir).rem_euclid(len) as usize;
    }

    fn rows(&self, ctx: &ReactorContext) -> Vec<Row> {
        let s = &ctx.reactor.post_process.settings;
        match self.page {
            PauseConfigPage::Display => vec![
                Row::bool(
                    "Post Process",
                    ctx.reactor.post_process.enabled,
                    "Full post stack",
                ),
                Row::bool("VSync", ctx.reactor.vsync, "Recreates swapchain"),
                Row::bool(
                    "Fullscreen",
                    ctx.window.fullscreen().is_some(),
                    "Borderless",
                ),
                Row::value("Exposure", s.exposure, "HDR brightness"),
                Row::value("Gamma", s.gamma, "Output curve"),
                Row::value("Bloom Intensity", s.bloom_intensity, "Neon bleed"),
                Row::value("Bloom Threshold", s.bloom_threshold, "Bright cutoff"),
                Row::value("Film Grain", s.grain_intensity, "Cinematic noise"),
                Row::value(
                    "Chromatic Aberration",
                    s.chromatic_intensity,
                    "Lens dispersion",
                ),
                Row::value("Vignette", s.vignette_intensity, "Edge darkening"),
                Row::value("Sharpen", s.sharpen_intensity, "Micro contrast"),
            ],
            PauseConfigPage::Lighting => vec![
                Row::effect("SSGI", s, PostProcessEffect::SSGI, "Screen-space GI"),
                Row::value("SSGI Intensity", s.ssgi_intensity, "Indirect light"),
                Row::value("SSGI Radius", s.ssgi_radius, "Bounce reach"),
                Row::effect(
                    "PT Resolve",
                    s,
                    PostProcessEffect::PathTracedLighting,
                    "Multi-bounce resolve",
                ),
                Row::value("PT Intensity", s.pathtrace_intensity, "Cyberpunk-style GI"),
                Row::effect("SSR", s, PostProcessEffect::SSR, "Wet reflections"),
                Row::value("SSR Strength", s.ssr_strength, "Reflection mix"),
                Row::effect(
                    "Volumetric Fog",
                    s,
                    PostProcessEffect::VolumetricFog,
                    "Light scattering",
                ),
                Row::value("Fog Density", s.fog_density, "Atmosphere"),
                Row::value("Fog Scatter", s.fog_scatter, "Shaft brightness"),
                Row::effect(
                    "Neon Flares",
                    s,
                    PostProcessEffect::AnamorphicFlares,
                    "Anamorphic lens",
                ),
                Row::value("Flare Intensity", s.flare_intensity, "Horizontal streaks"),
                Row::value("Highlight Recovery", s.highlight_recovery, "Anti blowout"),
            ],
            PauseConfigPage::Color => vec![
                Row::effect(
                    "LUT Color Grade",
                    s,
                    PostProcessEffect::LutColorGrading,
                    "Final look",
                ),
                Row::value("LUT Strength", s.lut_strength, "Grade amount"),
                Row::effect(
                    "Tone Mapping",
                    s,
                    PostProcessEffect::ToneMapping,
                    "ACES curve",
                ),
                Row::effect("Bloom", s, PostProcessEffect::Bloom, "Glow pass"),
                Row::effect(
                    "Vignette",
                    s,
                    PostProcessEffect::Vignette,
                    "Cinematic frame",
                ),
                Row::effect(
                    "Chromatic Aberration",
                    s,
                    PostProcessEffect::ChromaticAberration,
                    "Lens split",
                ),
                Row::effect(
                    "Film Grain",
                    s,
                    PostProcessEffect::FilmGrain,
                    "Noise overlay",
                ),
                Row::effect("FXAA", s, PostProcessEffect::FXAA, "Fast AA"),
                Row::effect("Sharpen", s, PostProcessEffect::Sharpen, "Detail"),
                Row::effect("Grayscale", s, PostProcessEffect::Grayscale, "Debug/look"),
                Row::effect("Sepia", s, PostProcessEffect::Sepia, "Vintage look"),
                Row::effect("Invert", s, PostProcessEffect::Invert, "Debug/look"),
                Row::effect("Blur", s, PostProcessEffect::Blur, "Soft focus"),
            ],
            PauseConfigPage::Performance => vec![
                Row::text(
                    "Pixel Inteligente",
                    format!("{:?}", ctx.reactor.pixel_intelligent.profile),
                    "VRS profile",
                ),
                Row::text(
                    "Current VRS Rate",
                    pixel_rate(ctx),
                    "Native fallback if unsupported",
                ),
                Row::bool("Post Process", ctx.reactor.post_process.enabled, "GPU cost"),
                Row::effect("FXAA", s, PostProcessEffect::FXAA, "Cheap AA"),
                Row::effect(
                    "PT Resolve",
                    s,
                    PostProcessEffect::PathTracedLighting,
                    "High cost",
                ),
                Row::effect("SSR", s, PostProcessEffect::SSR, "Medium cost"),
                Row::effect(
                    "Volumetric Fog",
                    s,
                    PostProcessEffect::VolumetricFog,
                    "Medium cost",
                ),
                Row::effect(
                    "Neon Flares",
                    s,
                    PostProcessEffect::AnamorphicFlares,
                    "Medium cost",
                ),
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

    fn adjust_selected(&mut self, ctx: &mut ReactorContext, dir: i32, activate: bool) -> bool {
        match self.page {
            PauseConfigPage::Display => self.adjust_display(ctx, dir, activate),
            PauseConfigPage::Lighting => self.adjust_lighting(ctx, dir, activate),
            PauseConfigPage::Color => self.adjust_color(ctx, dir, activate),
            PauseConfigPage::Performance => self.adjust_performance(ctx, dir, activate),
            PauseConfigPage::Presets => self.apply_preset(ctx, self.selected),
        }
    }

    fn adjust_display(&self, ctx: &mut ReactorContext, dir: i32, activate: bool) -> bool {
        match self.selected {
            0 => {
                ctx.reactor.post_process.enabled = !ctx.reactor.post_process.enabled;
                true
            }
            1 => toggle_vsync(ctx),
            2 => toggle_fullscreen(ctx),
            3 => step_f32(
                &mut ctx.reactor.post_process.settings.exposure,
                dir,
                0.05,
                0.40,
                2.50,
            ),
            4 => step_f32(
                &mut ctx.reactor.post_process.settings.gamma,
                dir,
                0.05,
                1.60,
                2.80,
            ),
            5 => step_f32(
                &mut ctx.reactor.post_process.settings.bloom_intensity,
                dir,
                0.05,
                0.00,
                2.00,
            ),
            6 => step_f32(
                &mut ctx.reactor.post_process.settings.bloom_threshold,
                dir,
                0.02,
                0.30,
                1.60,
            ),
            7 => step_f32(
                &mut ctx.reactor.post_process.settings.grain_intensity,
                dir,
                0.003,
                0.00,
                0.12,
            ),
            8 => step_f32(
                &mut ctx.reactor.post_process.settings.chromatic_intensity,
                dir,
                0.0005,
                0.00,
                0.015,
            ),
            9 => step_f32(
                &mut ctx.reactor.post_process.settings.vignette_intensity,
                dir,
                0.03,
                0.00,
                0.90,
            ),
            10 => step_f32(
                &mut ctx.reactor.post_process.settings.sharpen_intensity,
                dir,
                0.03,
                0.00,
                1.50,
            ),
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
            2 => {
                ctx.reactor.post_process.enabled = !ctx.reactor.post_process.enabled;
                true
            }
            3 => toggle_effect(
                &mut ctx.reactor.post_process.settings,
                PostProcessEffect::FXAA,
            ),
            4 => toggle_effect(
                &mut ctx.reactor.post_process.settings,
                PostProcessEffect::PathTracedLighting,
            ),
            5 => toggle_effect(
                &mut ctx.reactor.post_process.settings,
                PostProcessEffect::SSR,
            ),
            6 => toggle_effect(
                &mut ctx.reactor.post_process.settings,
                PostProcessEffect::VolumetricFog,
            ),
            7 => toggle_effect(
                &mut ctx.reactor.post_process.settings,
                PostProcessEffect::AnamorphicFlares,
            ),
            8 => toggle_effect(
                &mut ctx.reactor.post_process.settings,
                PostProcessEffect::FilmGrain,
            ),
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
                ctx.reactor
                    .set_pixel_intelligent_profile(PixelIntelligentProfile::Performance);
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
            _ => {
                *s = PostProcessSettings::default();
            }
        }
        true
    }

    fn apply_direct_hotkeys(&mut self, ctx: &mut ReactorContext, input: &PauseConfigInput) -> bool {
        let mut changed = false;

        if input.toggle_vsync {
            changed |= toggle_vsync(ctx);
        }
        if input.toggle_fullscreen {
            changed |= toggle_fullscreen(ctx);
        }
        if input.toggle_post_process {
            ctx.reactor.post_process.enabled = !ctx.reactor.post_process.enabled;
            changed = true;
        }
        if input.pixel_intelligent {
            changed |= cycle_pixel_intelligent(ctx);
        }

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

        if input.cycle_exposure {
            s.exposure = cycle_float(s.exposure, &[0.80, 1.00, 1.15, 1.50, 2.00]);
            changed = true;
        }
        if input.cycle_bloom {
            s.bloom_intensity = cycle_float(s.bloom_intensity, &[0.10, 0.20, 0.35, 0.50, 0.80]);
            changed = true;
        }
        if input.cycle_grain {
            s.grain_intensity = cycle_float(s.grain_intensity, &[0.00, 0.003, 0.006, 0.012]);
            changed = true;
        }

        changed
    }
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

#[derive(Default)]
struct PauseConfigInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    activate: bool,
    prev_page: bool,
    next_page: bool,
    page: Option<PauseConfigPage>,
    resume: bool,
    quit: bool,
    toggle_vsync: bool,
    toggle_fullscreen: bool,
    toggle_post_process: bool,
    pixel_intelligent: bool,
    cycle_exposure: bool,
    cycle_bloom: bool,
    cycle_grain: bool,
    k4: bool,
    k5: bool,
    k6: bool,
    k7: bool,
    k8: bool,
    k9: bool,
    k0: bool,
    k_z: bool,
    k_x: bool,
    k_c: bool,
    k_t: bool,
    k_y: bool,
    k_u: bool,
}

impl PauseConfigInput {
    fn capture(ctx: &ReactorContext) -> Self {
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
            activate: input.is_key_just_pressed(KeyCode::Enter)
                || input.is_key_just_pressed(KeyCode::Space),
            prev_page: input.is_key_just_pressed(KeyCode::BracketLeft),
            next_page: input.is_key_just_pressed(KeyCode::BracketRight),
            page,
            resume: input.is_key_just_pressed(KeyCode::KeyP),
            quit: input.is_key_just_pressed(KeyCode::Escape),
            toggle_vsync: input.is_key_just_pressed(KeyCode::KeyV),
            toggle_fullscreen: input.is_key_just_pressed(KeyCode::F11)
                || input.is_key_just_pressed(KeyCode::KeyF),
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

fn toggle_effect(settings: &mut PostProcessSettings, effect: PostProcessEffect) -> bool {
    if settings.is_effect_enabled(effect) {
        settings.disable_effect(effect);
    } else {
        settings.enable_effect(effect);
    }
    true
}

fn step_f32(value: &mut f32, dir: i32, step: f32, min: f32, max: f32) -> bool {
    *value = (*value + step * dir as f32).clamp(min, max);
    true
}

fn cycle_float(current: f32, values: &[f32]) -> f32 {
    values
        .iter()
        .copied()
        .find(|value| current < *value - 0.001)
        .unwrap_or(values[0])
}

fn cycle_pixel_intelligent(ctx: &mut ReactorContext) -> bool {
    let next = match ctx.reactor.pixel_intelligent.profile {
        PixelIntelligentProfile::Off => PixelIntelligentProfile::Quality,
        PixelIntelligentProfile::Quality => PixelIntelligentProfile::Balanced,
        PixelIntelligentProfile::Balanced => PixelIntelligentProfile::Performance,
        PixelIntelligentProfile::Performance => PixelIntelligentProfile::UltraPerformance,
        PixelIntelligentProfile::UltraPerformance => PixelIntelligentProfile::Off,
    };
    ctx.reactor.set_pixel_intelligent_profile(next);
    true
}

fn toggle_vsync(ctx: &mut ReactorContext) -> bool {
    ctx.reactor.vsync = !ctx.reactor.vsync;
    if let Err(err) = ctx.reactor.recreate_swapchain() {
        eprintln!("REACTOR PauseConfig: failed to toggle VSync: {err}");
        ctx.reactor.resized = true;
    }
    true
}

fn toggle_fullscreen(ctx: &mut ReactorContext) -> bool {
    if ctx.window.fullscreen().is_some() {
        ctx.window.set_fullscreen(None);
    } else {
        ctx.window
            .set_fullscreen(Some(Fullscreen::Borderless(None)));
    }
    true
}

fn on_off(value: bool) -> &'static str {
    if value {
        "ON"
    } else {
        "OFF"
    }
}

fn pixel_display(ctx: &ReactorContext) -> String {
    if ctx.reactor.pixel_intelligent_enabled() {
        let rate = ctx.reactor.pixel_intelligent_rate();
        format!(
            "{:?} {}x{}",
            ctx.reactor.pixel_intelligent.profile, rate.width, rate.height
        )
    } else if ctx.reactor.context.supports_fragment_shading_rate() {
        "OFF".to_string()
    } else {
        "No VRS HW".to_string()
    }
}

fn pixel_rate(ctx: &ReactorContext) -> String {
    let rate = ctx.reactor.pixel_intelligent_rate();
    format!("{}x{}", rate.width, rate.height)
}

fn msaa_display(ctx: &ReactorContext) -> String {
    let raw = ctx.reactor.msaa_samples.as_raw();
    if raw <= 1 {
        "1x".to_string()
    } else {
        format!("{raw}x")
    }
}
