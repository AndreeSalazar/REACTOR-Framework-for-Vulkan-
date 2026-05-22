// =============================================================================
// quick.rs — API corto de REACTOR
// =============================================================================
// 3 formas de arrancar un juego, de la MÁS CORTA a la más declarativa.
// Edita MODE (1, 2 o 3) para probar cada forma.
// =============================================================================

use reactor_vulkan::prelude::*;

const MODE: u8 = 1;

fn main() {
    match MODE {
        // ── 1) reactor::quick — la más corta posible ────────────────────────
        1 => reactor_vulkan::quick("REACTOR — Quick", 1280, 720, |ctx| {
            let t = ctx.elapsed();
            let eye = Vec3::new(t.cos() * 5.0, 2.0, t.sin() * 5.0);
            ctx.look_at(eye, Vec3::ZERO);
        }),

        // ── 2) reactor::quick_with — init + update ──────────────────────────
        2 => reactor_vulkan::quick_with(
            ReactorConfig::new("REACTOR — Quick With")
                .with_size(1600, 900)
                .with_vsync(true)
                .with_msaa(4),
            |ctx| {
                ctx.look_at(Vec3::new(0.0, 2.0, 5.0), Vec3::ZERO);
                ctx.add_sun();
                ctx.add_point_light(
                    Vec3::new(3.0, 4.0, 2.0),
                    Vec3::new(1.0, 0.7, 0.4),
                    2.0,
                    15.0,
                );
            },
            |ctx| {
                let _dt = ctx.delta();
            },
        ),

        // ── 3) macro reactor::game! — declarativa ───────────────────────────
        3 => reactor_vulkan::game! {
            title: "REACTOR — game! macro",
            size: (1280, 720),
            vsync: true,
            msaa: 4,
            init: |ctx| {
                ctx.look_at(Vec3::new(0.0, 2.0, 5.0), Vec3::ZERO);
                ctx.add_sun();
            },
            update: |ctx| {
                let _ = ctx.fps();
            }
        },

        _ => eprintln!("MODE debe ser 1, 2 o 3"),
    }
}
