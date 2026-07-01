use winit::window::Fullscreen;

use crate::app::context::ReactorContext;
use crate::core::PixelIntelligentProfile;
use crate::graphics::post_process::{PostProcessEffect, PostProcessSettings};

pub(super) fn toggle_effect(settings: &mut PostProcessSettings, effect: PostProcessEffect) -> bool {
    if settings.is_effect_enabled(effect) {
        settings.disable_effect(effect);
    } else {
        settings.enable_effect(effect);
    }
    true
}

pub(super) fn step_f32(value: &mut f32, dir: i32, step: f32, min: f32, max: f32) -> bool {
    *value = (*value + step * dir as f32).clamp(min, max);
    true
}

pub(super) fn cycle_float(current: f32, values: &[f32]) -> f32 {
    values.iter().copied().find(|value| current < *value - 0.001).unwrap_or(values[0])
}

pub(super) fn cycle_pixel_intelligent(ctx: &mut ReactorContext) -> bool {
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

pub(super) fn toggle_vsync(ctx: &mut ReactorContext) -> bool {
    ctx.reactor.vsync = !ctx.reactor.vsync;
    if let Err(err) = ctx.reactor.recreate_swapchain() {
        eprintln!("REACTOR PauseConfig: failed to toggle VSync: {err}");
        ctx.reactor.resized = true;
    }
    true
}

pub(super) fn toggle_fullscreen(ctx: &mut ReactorContext) -> bool {
    if ctx.window.fullscreen().is_some() {
        ctx.window.set_fullscreen(None);
    } else {
        ctx.window.set_fullscreen(Some(Fullscreen::Borderless(None)));
    }
    true
}

pub(super) fn on_off(value: bool) -> &'static str {
    if value { "ON" } else { "OFF" }
}

pub(super) fn pixel_display(ctx: &ReactorContext) -> String {
    if ctx.reactor.pixel_intelligent_enabled() {
        let rate = ctx.reactor.pixel_intelligent_rate();
        format!("{:?} {}x{}", ctx.reactor.pixel_intelligent.profile, rate.width, rate.height)
    } else if ctx.reactor.context.supports_fragment_shading_rate() {
        "OFF".to_string()
    } else {
        "No VRS HW".to_string()
    }
}

pub(super) fn pixel_rate(ctx: &ReactorContext) -> String {
    let rate = ctx.reactor.pixel_intelligent_rate();
    format!("{}x{}", rate.width, rate.height)
}

pub(super) fn msaa_display(ctx: &ReactorContext) -> String {
    let raw = ctx.reactor.msaa_samples.as_raw();
    if raw <= 1 { "1x".to_string() } else { format!("{raw}x") }
}
