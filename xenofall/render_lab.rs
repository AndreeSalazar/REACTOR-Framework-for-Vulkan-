// =============================================================================
// XENOFALL — REACTOR Professional Render Lab
// =============================================================================
// Fase 1 goal:
// - Keep XENOFALL as the playable test scene.
// - Centralize the professional shader/post stack outside the monolith.
// - Make the renderer budget visible so future delta-rendering work can be measured.
// =============================================================================

use reactor_vulkan::graphics::post_process::PostProcessEffect;
use reactor_vulkan::prelude::*;

/// Render profile used by XENOFALL as REACTOR's first professional test scene.
pub struct RenderLabProfile {
    pub name: &'static str,
    pub target_resolution: (u32, u32),
    pub target_fps: u32,
    pub persistent_vram_gib: f32,
    pub dynamic_vram_gib: f32,
    pub system_vram_gib: f32,
}

impl RenderLabProfile {
    pub const fn xenofall_phase_one() -> Self {
        Self {
            name: "XENOFALL Phase 1 — Professional Shader Lab",
            target_resolution: (3840, 2160),
            target_fps: 60,
            persistent_vram_gib: 6.0,
            dynamic_vram_gib: 4.0,
            system_vram_gib: 2.0,
        }
    }
}

/// Apply the cinematic shader stack used to stress REACTOR's professional rendering path.
pub fn apply_professional_profile(ctx: &mut ReactorContext) {
    let mut shader = BaseShaderCookbook::xenofall_showcase();
    let s = &mut shader.post_settings;

    // Cinematic horror baseline: dark, glossy floor, controlled bloom, stable TAA.
    s.exposure = 0.72;
    s.gamma = 2.2;
    s.bloom_threshold = 1.2;
    s.bloom_intensity = 0.32;
    s.grain_intensity = 0.003;
    s.chromatic_intensity = 0.0014;
    s.vignette_intensity = 0.42;
    s.sharpen_intensity = 0.18;
    s.ssgi_intensity = 0.42;
    s.ssgi_radius = 14.0;
    s.ssr_strength = 0.46;
    s.fog_density = 0.10;
    s.fog_scatter = 0.15;
    s.flare_intensity = 0.18;
    s.highlight_recovery = 0.82;
    s.dof_focus_distance = 12.0;
    s.dof_aperture = 0.015;

    // Professional stack under observation. These are the passes we will later
    // classify into static cache, dynamic delta, history, and temporal resolve.
    s.enable_effect(PostProcessEffect::Bloom);
    s.enable_effect(PostProcessEffect::SSGI);
    s.enable_effect(PostProcessEffect::SSR);
    s.enable_effect(PostProcessEffect::VolumetricFog);
    s.enable_effect(PostProcessEffect::LutColorGrading);
    s.enable_effect(PostProcessEffect::ToneMapping);
    s.enable_effect(PostProcessEffect::AnamorphicFlares);
    s.disable_effect(PostProcessEffect::FXAA);
    s.enable_effect(PostProcessEffect::TAA);
    s.enable_effect(PostProcessEffect::FilmGrain);
    s.enable_effect(PostProcessEffect::ChromaticAberration);
    s.enable_effect(PostProcessEffect::ContactShadows);
    s.enable_effect(PostProcessEffect::SSSDiffusion);
    s.enable_effect(PostProcessEffect::DepthOfField);
    s.enable_effect(PostProcessEffect::AutoExposure);

    ctx.apply_base_shader(&shader);

    let sun_dir = Vec3::new(-0.22, -0.86, -0.45).normalize();
    let moon_cold = Vec3::new(0.42, 0.55, 0.82);
    ctx.scene.set_sun(sun_dir, moon_cold * 1.65);
    ctx.scene.set_ambient(Vec3::new(0.012, 0.016, 0.014));
    ctx.add_directional_light(sun_dir, moon_cold, 1.35);

    Log::engine("Render Lab: cinematic AAA shader profile active for XENOFALL");
}

/// Print the Phase 1 render budget and the concrete signals that future delta
/// rendering work must improve.
pub fn log_phase_one_budget(ctx: &ReactorContext) {
    let profile = RenderLabProfile::xenofall_phase_one();
    let gbuffer_mib = ctx
        .reactor
        .gbuffer
        .as_ref()
        .map(|g| g.estimated_bytes() as f32 / (1024.0 * 1024.0))
        .unwrap_or(0.0);
    let history_mib = ctx
        .reactor
        .temporal_history
        .as_ref()
        .map(|h| h.estimated_bytes() as f32 / (1024.0 * 1024.0))
        .unwrap_or(0.0);

    Log::section("REACTOR Render Lab — Phase 1 Foundation");
    Log::kv("Profile", profile.name);
    Log::kv(
        "Target",
        &format!(
            "{}x{} @ {} FPS",
            profile.target_resolution.0, profile.target_resolution.1, profile.target_fps
        ),
    );
    Log::kv("G-Buffer 4 attachments", &format!("{:.1} MiB", gbuffer_mib));
    Log::kv(
        "TAA history color/depth",
        &format!("{:.1} MiB", history_mib),
    );
    Log::kv("Depth resolve MSAA", "R32F sampleable");
    Log::kv("GTAO/SSR source", "depth + normal/material G-Buffer");
    Log::kv(
        "VRAM target split",
        &format!(
            "{:.0}GB persistent / {:.0}GB dynamic / {:.0}GB system",
            profile.persistent_vram_gib, profile.dynamic_vram_gib, profile.system_vram_gib
        ),
    );
    Log::kv(
        "Phase 1 mission",
        "measure professional shader behavior before delta rendering",
    );
    Log::kv(
        "Next required",
        "dirty tile overlay + history validation + sector/static cache metrics",
    );
}
