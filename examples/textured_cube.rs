// =============================================================================
// textured_cube.rs — Texture Loading Demo
// =============================================================================
// Demonstrates loading textures and applying them to 3D objects.
// This example shows the texture API in action.
// =============================================================================

use reactor::prelude::*;
use reactor::Vertex;
use std::sync::Arc;

// =============================================================================
// YOUR GAME — Textured Cube Demo
// =============================================================================

struct TexturedCube {
    rotation: f32,
}

impl ReactorApp for TexturedCube {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("🖼️ Textured Cube Demo")
            .with_size(1280, 720)
            .with_vsync(true)
            .with_msaa(4)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        // Camera setup
        ctx.camera.position = Vec3::new(0.0, 2.0, 4.0);
        ctx.camera.set_rotation(-0.3, 0.0);

        // Lighting
        ctx.lighting.add_light(Light::directional(
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Vec3::ONE,
            1.0,
        ));

        // Create a cube mesh with UV coordinates
        let mesh = Arc::new(ctx.create_mesh(&cube_vertices_uv(), &cube_indices()).unwrap());
        
        // Load shaders
        let vert = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../shaders/vert.spv"))).unwrap();
        let frag = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../shaders/frag.spv"))).unwrap();
        let material = Arc::new(ctx.create_material(&vert, &frag).unwrap());

        // Add cube to scene
        ctx.scene.add_object(mesh, material, Mat4::IDENTITY);

        // Load a texture (example - would need actual texture file)
        // let texture = ctx.load_texture("assets/texture.png").unwrap();
        
        // Or create a solid color texture
        let _solid_texture = ctx.create_solid_texture(255, 128, 64, 255);
        
        println!("🖼️ Textured Cube initialized!");
        println!("   - Texture API ready for use");
        println!("   - Use ctx.load_texture(\"path.png\") to load textures");
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        // Rotate cube
        self.rotation += ctx.time.delta() * 1.0;
        let transform = Mat4::from_rotation_y(self.rotation) * Mat4::from_rotation_x(self.rotation * 0.5);
        
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
    reactor::run(TexturedCube { rotation: 0.0 });
}

// =============================================================================
// CUBE DATA WITH UV COORDINATES
// =============================================================================

fn cube_vertices_uv() -> [Vertex; 24] {
    // Each face has 4 vertices with proper UVs for texturing
    [
        // Front face (Z+)
        Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::new(0.0, 0.0)),
        
        // Back face (Z-)
        Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(0.0, 0.0, -1.0), Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.0, 0.0, -1.0), Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(0.0, 0.0, -1.0), Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(0.0, 0.0, -1.0), Vec2::new(0.0, 0.0)),
        
        // Right face (X+)
        Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::new(0.0, 0.0)),
        
        // Left face (X-)
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(-1.0, 0.0, 0.0), Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(-1.0, 0.0, 0.0), Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(-1.0, 0.0, 0.0), Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(-1.0, 0.0, 0.0), Vec2::new(0.0, 0.0)),
        
        // Top face (Y+)
        Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::new(0.0, 0.0)),
        
        // Bottom face (Y-)
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.0, -1.0, 0.0), Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(0.0, -1.0, 0.0), Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(0.0, -1.0, 0.0), Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(0.0, -1.0, 0.0), Vec2::new(0.0, 0.0)),
    ]
}

fn cube_indices() -> [u32; 36] {
    [
        0, 1, 2,  2, 3, 0,      // Front
        4, 5, 6,  6, 7, 4,      // Back
        8, 9, 10, 10, 11, 8,    // Right
        12, 13, 14, 14, 15, 12, // Left
        16, 17, 18, 18, 19, 16, // Top
        20, 21, 22, 22, 23, 20, // Bottom
    ]
}
