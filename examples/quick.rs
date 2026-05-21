// =============================================================================
// quick.rs — Ejemplo del API corto de REACTOR
// =============================================================================
// Muestra las 3 formas más cortas de arrancar un juego:
//   1. reactor::quick(...)          → 1 línea
//   2. reactor::quick_with(...)     → init + update separados
//   3. reactor::game! { ... }       → macro declarativa
//
// Edita la constante MODE para probar cada una.
// =============================================================================

use reactor::prelude::*;

const MODE: u8 = 1; // 1 = quick, 2 = quick_with, 3 = game!

fn main() {
    match MODE {
        // ── Modo 1: reactor::quick ──────────────────────────────────────────
        1 => {
            reactor::quick("REACTOR — Quick", 1280, 720, |ctx| {
                // Cámara orbital simple usando elapsed
                let t = ctx.time.elapsed();
                let eye = Vec3::new(t.cos() * 5.0, 2.0, t.sin() * 5.0);
                ctx.camera = std::mem::take(&mut ctx.camera).look_at(eye, Vec3::ZERO, Vec3::Y);
            });
        }

        // ── Modo 2: reactor::quick_with ─────────────────────────────────────
        2 => {
            let cfg = ReactorConfig::new("REACTOR — Quick With")
                .with_size(1600, 900)
                .with_vsync(true)
                .with_msaa(4);

            reactor::quick_with(
                cfg,
                |ctx| {
                    // init — una vez al arranque
                    ctx.camera.position = Vec3::new(0.0, 2.0, 5.0);
                    ctx.lighting.add_light(Light::directional(
                        Vec3::new(-0.5, -1.0, -0.3).normalize(),
                        Vec3::ONE,
                        1.0,
                    ));
                },
                |ctx| {
                    // update — cada frame
                    let _dt = ctx.time.delta();
                },
            );
        }

        // ── Modo 3: macro reactor::game! ────────────────────────────────────
        3 => {
            reactor::game! {
                title: "REACTOR — game! macro",
                size: (1280, 720),
                vsync: true,
                msaa: 4,
                init: |ctx| {
                    ctx.camera.position = Vec3::new(0.0, 2.0, 5.0);
                },
                update: |ctx| {
                    let _dt = ctx.time.delta();
                }
            }
        }

        _ => eprintln!("MODE debe ser 1, 2 o 3"),
    }
}
