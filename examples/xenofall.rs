// =============================================================================
// XENOFALL — Plantilla base de videojuego con REACTOR
// =============================================================================
// Esta es la PLANTILLA OFICIAL para empezar tu juego sobre REACTOR.
// Está pensada para ser HEREDADA: copia este archivo, renombra `Xenofall`
// con el nombre de tu juego, y rellena los TODO.
//
// Demuestra TODAS las facilidades del SDK 1.1.0 sin tocar Vulkan:
//   ✓ Configuración 1080p / vsync / MSAA 4×
//   ✓ Cámara FPS con FpsController (WASD + flechas + Space/Ctrl + Shift)
//   ✓ Iluminación: sol direccional + 2 puntuales coloreadas
//   ✓ Spawn de primitivas en 1 línea (suelo + cubos + esferas)
//   ✓ Animación de transforms por entidad
//   ✓ HUD básico: FPS + posición de cámara en el título
//   ✓ Sin un solo `unsafe` en el código de gameplay
// =============================================================================

use reactor_vulkan::prelude::*;
use reactor_vulkan::systems::fps_controller::FpsController;

// =============================================================================
// GAME STATE — el "Pawn / GameMode" en términos de Unreal
// =============================================================================

struct Xenofall {
    /// Controlador de cámara en primera persona (puedes sustituir por uno propio).
    fps: FpsController,
    /// Índices en la escena de los objetos que animamos.
    spinning: Vec<usize>,
    bouncing: Vec<usize>,
    /// Tiempo acumulado para animaciones.
    t: f32,
}

impl Xenofall {
    fn new() -> Self {
        Self {
            fps: FpsController::default(),
            spinning: Vec::new(),
            bouncing: Vec::new(),
            t: 0.0,
        }
    }
}

// =============================================================================
// REACTOR APP — Lifecycle del juego
// =============================================================================

impl ReactorApp for Xenofall {
    // ── Configuración del juego ─────────────────────────────────────────────
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("⚡ XENOFALL — REACTOR base")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_msaa(4)
            .with_renderer(RendererMode::Forward)
            .with_physics_hz(60)
    }

    // ── Setup inicial (una sola vez) ────────────────────────────────────────
    fn init(&mut self, ctx: &mut ReactorContext) {
        print_banner();

        // 1) Cámara: posición inicial mirando al origen.
        ctx.look_at(Vec3::new(0.0, 4.0, 12.0), Vec3::ZERO);
        // Alinear el FPS controller con la orientación inicial
        self.fps.yaw = ctx.camera.yaw();
        self.fps.pitch = ctx.camera.pitch();

        // 2) Iluminación cinematográfica.
        ctx.add_sun();
        ctx.add_point_light(
            Vec3::new(-6.0, 4.0, 2.0),
            Vec3::new(0.4, 0.6, 1.0),   // azul frío
            5.0,
            20.0,
        );
        ctx.add_point_light(
            Vec3::new( 6.0, 4.0, 2.0),
            Vec3::new(1.0, 0.5, 0.2),   // naranja cálido
            5.0,
            20.0,
        );

        // 3) Geometría del nivel.
        //    Suelo grande, una fila de cubos giratorios y esferas saltarinas.
        let _floor = ctx
            .spawn_plane(Vec3::new(0.0, -1.0, 0.0), 30.0)
            .expect("spawn floor");

        for i in -3..=3 {
            let x = i as f32 * 2.5;
            // Cubos giratorios en fondo
            if let Ok(idx) = ctx.spawn_cube(Vec3::new(x, 0.5, -4.0)) {
                self.spinning.push(idx);
            }
            // Esferas saltarinas al frente
            if let Ok(idx) = ctx.spawn_sphere(Vec3::new(x, 0.5, 2.0), 0.5) {
                self.bouncing.push(idx);
            }
        }

        println!(
            "[XENOFALL] Nivel listo · {} cubos · {} esferas",
            self.spinning.len(),
            self.bouncing.len()
        );
    }

    // ── Lógica de juego (cada frame) ────────────────────────────────────────
    fn update(&mut self, ctx: &mut ReactorContext) {
        self.t += ctx.delta();

        // Cámara FPS (ESC sale, WASD/flechas, Shift = boost).
        self.fps.update(ctx);

        // Animación: cubos giran sobre Y, esferas rebotan en Y.
        for (n, &idx) in self.spinning.iter().enumerate() {
            let base = Vec3::new((n as f32 - 3.0) * 2.5, 0.5, -4.0);
            let rot = Mat4::from_rotation_y(self.t * (0.6 + 0.1 * n as f32));
            let xf = Mat4::from_translation(base) * rot;
            ctx.set_transform(idx, xf);
        }
        for (n, &idx) in self.bouncing.iter().enumerate() {
            let base_x = (n as f32 - 3.0) * 2.5;
            let y = 0.5 + (self.t * 2.0 + n as f32 * 0.7).sin().abs() * 1.5;
            let xf = Mat4::from_translation(Vec3::new(base_x, y, 2.0));
            ctx.set_transform(idx, xf);
        }

        // HUD en el título de la ventana.
        let p = ctx.camera.position;
        ctx.set_title(&format!(
            "⚡ XENOFALL · {:5.0} FPS · pos ({:>5.1}, {:>5.1}, {:>5.1})",
            ctx.fps(), p.x, p.y, p.z
        ));
    }

    // ── Física fija (60 Hz) ─────────────────────────────────────────────────
    fn fixed_update(&mut self, _ctx: &mut ReactorContext, _dt: f32) {
        // TODO: integra aquí tu sistema de física determinista
    }

    // ── Cleanup ─────────────────────────────────────────────────────────────
    fn on_exit(&mut self, _ctx: &mut ReactorContext) {
        println!("[XENOFALL] ¡Hasta la próxima, comandante!");
    }
}

// =============================================================================
// MAIN — 1 línea
// =============================================================================

fn main() {
    reactor_vulkan::run(Xenofall::new());
}

// =============================================================================
fn print_banner() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║               ⚡  X E N O F A L L  ⚡                       ║");
    println!("║         Plantilla base de juego sobre REACTOR 1.1.0          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Controles:                                                  ║");
    println!("║    WASD / Flechas → mover y mirar                            ║");
    println!("║    Space / Ctrl   → subir / bajar                            ║");
    println!("║    Shift          → boost de velocidad                       ║");
    println!("║    Esc            → salir                                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
