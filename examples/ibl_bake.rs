// =============================================================================
// REACTOR · ejemplo `ibl_bake` — cocina IBL (cubemap HDR + LUT) en GPU
// =============================================================================
// Demuestra el pipeline completo de Image-Based Lighting:
//
//   1. Crea / carga un equirect HDR.
//   2. Despacha los 4 compute shaders: equirect→cube, irradiance, prefilter,
//      BRDF LUT.
//   3. Reporta tiempo total + tamaños VRAM aproximados.
//
// Uso:
//   cargo run --release --example ibl_bake
//     → usa un sky procedural (no necesita HDR en disco)
//
//   $env:IBL_HDR = "assets/skybox.hdr"; cargo run --release --example ibl_bake
//     → bake desde un Radiance .hdr equirectangular real
//
// El descriptor set producido vive en `ibl.descriptor_set` y queda listo para
// bindearse a un fragment shader PBR que use `lib/ibl_textures.glsl`.
// =============================================================================

use reactor_vulkan::graphics::IblBaker;
use reactor_vulkan::prelude::*;
use std::time::Instant;

struct IblBakeDemo;

impl ReactorApp for IblBakeDemo {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("REACTOR · IBL Bake Test")
            .with_size(1280, 720)
            .with_vsync(true)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        println!("\x1b[38;2;200;200;0m╔════════════════════════════════════════════╗");
        println!("║       REACTOR · IBL HDR Bake Test         ║");
        println!("╚════════════════════════════════════════════╝\x1b[0m");

        let vk_ctx = &ctx.reactor.context;
        let allocator = ctx.reactor.allocator.clone();

        let t0 = Instant::now();
        let ibl = match std::env::var("IBL_HDR") {
            Ok(path) => {
                println!("\x1b[36m  → Baking IBL from HDR:\x1b[0m {}", path);
                IblBaker::bake_from_equirect_file(vk_ctx, allocator, &path)
            }
            Err(_) => {
                println!("\x1b[36m  → Baking IBL from procedural studio sky\x1b[0m");
                println!("\x1b[90m    (tip: set $env:IBL_HDR=\"path.hdr\" para usar uno real)\x1b[0m");
                IblBaker::bake_procedural(vk_ctx, allocator)
            }
        };

        match ibl {
            Ok(textures) => {
                let dt = t0.elapsed();
                let radiance_mb = radiance_vram_mb();
                println!(
                    "\x1b[32m  ✓ IBL cocinado en {:.2} ms\x1b[0m",
                    dt.as_secs_f64() * 1000.0
                );
                println!("\x1b[90m    ─ Irradiance:  32×32×6  RGBA16F   ≈ 96 KB\x1b[0m");
                println!("\x1b[90m    ─ Prefiltered: 128×128×6 (5 mips) RGBA16F ≈ {:.0} KB\x1b[0m", radiance_mb);
                println!("\x1b[90m    ─ BRDF LUT:    512×512   RG16F     ≈ 512 KB\x1b[0m");
                println!("\x1b[90m    ─ max_mip_level expuesto: {}\x1b[0m", textures.max_mip_level);
                println!("\x1b[90m    ─ Descriptor set listo en set = 1 (FRAGMENT)\x1b[0m");
                println!();
                println!("\x1b[33m  Bindings del shader que lo consuma:\x1b[0m");
                println!("\x1b[90m    layout(set=1, binding=0) uniform samplerCube u_ibl_irradiance;\x1b[0m");
                println!("\x1b[90m    layout(set=1, binding=1) uniform samplerCube u_ibl_prefiltered;\x1b[0m");
                println!("\x1b[90m    layout(set=1, binding=2) uniform sampler2D   u_ibl_brdf_lut;\x1b[0m");
                println!("\x1b[90m    layout(set=1, binding=3) uniform IblParams {{ float max_mip; }};\x1b[0m");
                println!();
                println!("\x1b[32m  ESC para salir.\x1b[0m");
                // Conserva las texturas hasta el shutdown — el Drop libera los handles.
                std::mem::forget(textures);
            }
            Err(e) => {
                eprintln!("\x1b[31m  ✗ Error cocinando IBL: {}\x1b[0m", e);
            }
        }
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        if ctx.input().is_key_just_pressed(winit::keyboard::KeyCode::Escape) {
            ctx.reactor.exit_requested = true;
        }
    }
}

fn radiance_vram_mb() -> f32 {
    // 128² + 64² + 32² + 16² + 8² = 16384+4096+1024+256+64 = 21824 texels/face
    // × 6 caras × 8 bytes (RGBA16F) = ≈1 MB
    let mut texels = 0u32;
    let mut s = 128u32;
    for _ in 0..5 { texels += s * s; s /= 2; }
    (texels * 6 * 8) as f32 / 1024.0
}

fn main() {
    reactor_vulkan::run(IblBakeDemo);
}
