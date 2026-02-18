// =============================================================================
// cube.rs — Simple Rotating Cube Example
// =============================================================================
// The simplest possible 3D example: a colored cube rotating in space.
// Demonstrates:
//   - ReactorApp trait implementation
//   - Mesh creation with vertices and indices
//   - Material/shader loading
//   - Scene management
//   - Camera setup
//   - Basic input handling
//   - Cambios
// =============================================================================

use reactor::prelude::*;
use reactor::Vertex;
use std::sync::Arc;
use winit::keyboard::KeyCode;

/// Simple cube application
struct CubeDemo {
    rotation: f32,
}

impl CubeDemo {
    fn new() -> Self {
        Self { rotation: 0.0 }
    }

    /// Create cube vertices with colors
    fn cube_vertices() -> [Vertex; 8] {
        [
            // Front face (z = 0.5)
            Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::ZERO), // Red
            Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::ZERO), // Green
            Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::ZERO), // Blue
            Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO), // Yellow
            // Back face (z = -0.5)
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(1.0, 0.0, 1.0), Vec2::ZERO), // Magenta
            Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(0.0, 1.0, 1.0), Vec2::ZERO), // Cyan
            Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO), // White
            Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(0.5, 0.5, 0.5), Vec2::ZERO), // Gray
        ]
    }

    /// Create cube indices (36 indices for 12 triangles)
    fn cube_indices() -> [u32; 36] {
        [
            // Front face
            0, 1, 2,  2, 3, 0,
            // Right face
            1, 5, 6,  6, 2, 1,
            // Back face
            5, 4, 7,  7, 6, 5,
            // Left face
            4, 0, 3,  3, 7, 4,
            // Top face
            3, 2, 6,  6, 7, 3,
            // Bottom face
            4, 5, 1,  1, 0, 4,
        ]
    }
}

impl ReactorApp for CubeDemo {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("🎲 REACTOR Cube Demo")
            .with_size(1280, 720)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              🎲 REACTOR Cube Demo                            ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Controls:                                                   ║");
        println!("║    ESC     - Exit                                            ║");
        println!("║    WASD    - Move camera                                     ║");
        println!("║    Space   - Move up                                         ║");
        println!("║    Shift   - Move down                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");

        // Setup camera - looking at origin from a distance
        ctx.camera.position = Vec3::new(0.0, 2.0, 4.0);
        ctx.camera.set_rotation(-0.3, 0.0); // Look slightly down

        // Setup lighting
        ctx.lighting.add_light(Light::directional(
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Vec3::new(1.0, 0.98, 0.95), // Warm white
            1.0,
        ));

        // Add a second directional light for fill
        ctx.lighting.add_light(Light::directional(
            Vec3::new(0.5, -0.5, 0.5).normalize(),
            Vec3::new(0.3, 0.3, 0.4), // Cool fill
            0.3,
        ));

        // Create cube mesh
        let vertices = Self::cube_vertices();
        let indices = Self::cube_indices();

        match ctx.create_mesh(&vertices, &indices) {
            Ok(mesh) => {
                let mesh = Arc::new(mesh);

                // Load shaders
                let vert = ash::util::read_spv(&mut std::io::Cursor::new(
                    include_bytes!("../shaders/vert.spv")
                )).expect("Failed to load vertex shader");

                let frag = ash::util::read_spv(&mut std::io::Cursor::new(
                    include_bytes!("../shaders/frag.spv")
                )).expect("Failed to load fragment shader");

                match ctx.create_material(&vert, &frag) {
                    Ok(material) => {
                        let material = Arc::new(material);

                        // Add cube to scene at origin
                        ctx.scene.add_object(mesh, material, Mat4::IDENTITY);
                        println!("[CUBE] ✓ Cube added to scene");
                    }
                    Err(e) => eprintln!("[CUBE] ✗ Failed to create material: {}", e),
                }
            }
            Err(e) => eprintln!("[CUBE] ✗ Failed to create mesh: {}", e),
        }
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.time.delta();

        // Exit on Escape
        if ctx.input().is_key_down(KeyCode::Escape) {
            std::process::exit(0);
        }

        // Camera movement
        let speed = 3.0 * dt;
        if ctx.input().is_key_down(KeyCode::KeyW) {
            ctx.camera.position.z -= speed;
        }
        if ctx.input().is_key_down(KeyCode::KeyS) {
            ctx.camera.position.z += speed;
        }
        if ctx.input().is_key_down(KeyCode::KeyA) {
            ctx.camera.position.x -= speed;
        }
        if ctx.input().is_key_down(KeyCode::KeyD) {
            ctx.camera.position.x += speed;
        }
        if ctx.input().is_key_down(KeyCode::Space) {
            ctx.camera.position.y += speed;
        }
        if ctx.input().is_key_down(KeyCode::ShiftLeft) {
            ctx.camera.position.y -= speed;
        }

        // Rotate cube
        self.rotation += dt * 1.5;
        let rotation = Mat4::from_rotation_y(self.rotation) 
                     * Mat4::from_rotation_x(self.rotation * 0.7);

        // Update cube transform in scene
        if !ctx.scene.objects.is_empty() {
            ctx.scene.objects[0].transform = rotation;
        }

        // Update window title with FPS
        ctx.set_title(&format!(
            "🎲 REACTOR Cube Demo | FPS: {:.0} | Rotation: {:.1}°",
            ctx.fps(),
            self.rotation.to_degrees() % 360.0
        ));
    }

    // render() uses default implementation which renders ctx.scene with ctx.camera
}

fn main() {
    println!("\n🚀 Starting REACTOR Cube Demo...\n");
    reactor::run(CubeDemo::new());
}
