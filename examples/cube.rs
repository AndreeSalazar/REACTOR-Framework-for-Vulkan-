// =============================================================================
// cube.rs — Simple Rotating Cube Example
// =============================================================================
// The simplest possible 3D example: a colored cube rotating in space.
// All engine API is imported from `reactorapp::*` — the single high-level
// entry point of REACTOR. No other imports needed.
// =============================================================================

#[path = "shared/mod.rs"]
mod shared;

use reactor_vulkan::reactorapp::*;
use reactor_vulkan::ReactorApp;
use shared::camera_input::{CameraInput, CameraInputSettings, CameraMode};
use shared::fps_counter::FpsCounter;

fn cube_vertices() -> [Vertex; 8] {
    use glam::Vec2;
    [
        Vertex::new(Vec3::new(-0.5, -0.5, 0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::ZERO),
        Vertex::new(Vec3::new(0.5, -0.5, 0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::ZERO),
        Vertex::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5, 0.5, 0.5), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(1.0, 0.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(0.5, -0.5, -0.5), Vec3::new(0.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(0.5, 0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5, 0.5, -0.5), Vec3::new(0.5, 0.5, 0.5), Vec2::ZERO),
    ]
}

fn cube_indices() -> [u32; 36] {
    [
        0, 1, 2, 2, 3, 0,
        1, 5, 6, 6, 2, 1,
        5, 4, 7, 7, 6, 5,
        4, 0, 3, 3, 7, 4,
        3, 2, 6, 6, 7, 3,
        4, 5, 1, 1, 0, 4,
    ]
}

pub struct CubeDemo {
    camera_input: CameraInput,
    fps: FpsCounter,
    cube_index: Option<usize>,
    rotation: f32,
}

impl CubeDemo {
    pub fn new() -> Self {
        let mut settings = CameraInputSettings::default();
        settings.mode = CameraMode::Orbit;
        settings.orbit_radius = 4.0;
        settings.orbit_speed = 0.6;
        Self {
            camera_input: CameraInput::new(settings),
            fps: FpsCounter::default(),
            cube_index: None,
            rotation: 0.0,
        }
    }
}

impl Default for CubeDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactorApp for CubeDemo {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("🎲 REACTOR Cube Demo").with_size(1280, 720)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              🎲 REACTOR Cube Demo                            ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  ESC         — Exit                                          ║");
        println!("║  Camera      — auto-orbiting around origin                   ║");
        println!("╚══════════════════════════════════════════════════════════════╝");

        let mut app = App::new(ctx);
        app.camera()
            .look_at(Vec3::new(0.0, 2.0, 4.0), Vec3::ZERO, 45.0);
        app.lighting().default_three_point();

        self.cube_index = Some(
            app.mesh()
                .vertices(&cube_vertices())
                .indices(&cube_indices())
                .use_cookbook_forward_material()
                .name("cube")
                .transform(Mat4::IDENTITY)
                .spawn()
                .expect("cube spawn"),
        );
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        self.camera_input.update(ctx);

        let dt = ctx.time.delta();
        self.rotation += dt * 1.5;
        let transform = Mat4::from_rotation_y(self.rotation) * Mat4::from_rotation_x(self.rotation * 0.7);

        if let Some(idx) = self.cube_index {
            if let Some(obj) = ctx.scene.get_mut(idx) {
                obj.transform = transform;
            }
        }

        ctx.set_title(&self.fps.format_title(ctx, "🎲 REACTOR Cube Demo"));
    }
}

fn main() {
    println!("\n🚀 Starting REACTOR Cube Demo...\n");
    reactor_vulkan::reactorapp::launch(CubeDemo::new());
}
