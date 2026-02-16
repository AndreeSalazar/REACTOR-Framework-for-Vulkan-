// =============================================================================
// simple_cube.rs — THE ONE CALL Pattern Demo
// =============================================================================
// The absolute minimum code to render a 3D cube with REACTOR.
// This is the "Hello World" of game engines.
//
// PATTERN: Inherit → Override → Run
//   1. Create struct with your state
//   2. Implement ReactorApp trait
//   3. Call reactor::run()
//
// That's it. No boilerplate. No Vulkan. No window management.
// =============================================================================

use reactor::prelude::*;
use reactor::Vertex;
use std::sync::Arc;

// =============================================================================
// YOUR GAME — Just state + logic
// =============================================================================

struct SimpleCube {
    rotation: f32,
}

impl ReactorApp for SimpleCube {
    // -------------------------------------------------------------------------
    // CONFIG — One place to configure everything
    // -------------------------------------------------------------------------
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Cube Example")
            .with_size(1980, 1080)
            .with_vsync(false)
            .with_msaa(4)
    }

    // -------------------------------------------------------------------------
    // INIT — Setup your scene once
    // -------------------------------------------------------------------------
    fn init(&mut self, ctx: &mut ReactorContext) {
        // Camera
        ctx.camera.position = Vec3::new(0.0, 2.0, 4.0);
        ctx.camera.set_rotation(-0.3, 0.0);

        // Light
        ctx.lighting.add_light(Light::directional(
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Vec3::ONE,
            1.0,
        ));

        // Cube mesh
        let mesh = Arc::new(ctx.create_mesh(&cube_vertices(), &cube_indices()).unwrap());
        
        // Material (shaders)
        let vert = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../shaders/vert.spv"))).unwrap();
        let frag = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../shaders/frag.spv"))).unwrap();
        let material = Arc::new(ctx.create_material(&vert, &frag).unwrap());

        // Add to scene
        ctx.scene.add_object(mesh, material, Mat4::IDENTITY);
    }

    // -------------------------------------------------------------------------
    // UPDATE — Game logic every frame
    // -------------------------------------------------------------------------
    fn update(&mut self, ctx: &mut ReactorContext) {
        // Rotate cube
        self.rotation += ctx.time.delta() * 1.5;
        let transform = Mat4::from_rotation_y(self.rotation) * Mat4::from_rotation_x(self.rotation * 0.7);
        
        if !ctx.scene.objects.is_empty() {
            ctx.scene.objects[0].transform = transform;
        }

        // ESC to exit
        if ctx.input().is_key_down(winit::keyboard::KeyCode::Escape) {
            std::process::exit(0);
        }
    }
}

// =============================================================================
// MAIN — THE ONE CALL
// =============================================================================

fn main() {
    reactor::run(SimpleCube { rotation: 0.0 });
}

// =============================================================================
// CUBE DATA — Could be loaded from file in real game
// =============================================================================

fn cube_vertices() -> [Vertex; 8] {
    [
        Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::ZERO),
        Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::ZERO),
        Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(1.0, 0.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(0.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(0.5, 0.5, 0.5), Vec2::ZERO),
    ]
}

fn cube_indices() -> [u32; 36] {
    [
        0, 1, 2,  2, 3, 0,  // Front
        1, 5, 6,  6, 2, 1,  // Right
        5, 4, 7,  7, 6, 5,  // Back
        4, 0, 3,  3, 7, 4,  // Left
        3, 2, 6,  6, 7, 3,  // Top
        4, 5, 1,  1, 0, 4,  // Bottom
    ]
}
