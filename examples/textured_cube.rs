// =============================================================================
// textured_cube.rs — Texture Loading Demo
// =============================================================================
// Demonstrates loading textures and applying them to 3D objects.
// This example shows the texture API in action.
// =============================================================================

use reactor::prelude::*;
use reactor::Vertex;
use reactor::resources::texture::Texture;
use std::sync::Arc;

// =============================================================================
// YOUR GAME — Textured Cube Demo
// =============================================================================

struct TexturedCube {
    rotation: f32,
    texture: Option<Texture>, // Keep texture alive for the lifetime of the app
}

impl ReactorApp for TexturedCube {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("🖼️ Textured Cube Demo")
            .with_size(1280, 720)
            .with_vsync(true)
            .with_msaa(4)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        // Camera setup - closer to see the textured cube properly
        ctx.camera.position = Vec3::new(0.0, 0.5, 2.5);
        ctx.camera.set_rotation(-0.2, 0.0);
        ctx.camera.fov = 45.0;

        // Lighting
        ctx.lighting.add_light(Light::directional(
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Vec3::ONE,
            1.0,
        ));

        // Create a cube mesh with UV coordinates
        let mesh = Arc::new(ctx.create_mesh(&cube_vertices_uv(), &cube_indices()).unwrap());
        
        // =====================================================================
        // TEXTURED MATERIAL - Load texture and create material with it
        // =====================================================================
        
        // Load texture from file and store it in self to keep it alive
        let texture = ctx.load_texture("assets/textures/container.jpg")
            .expect("Failed to load container.jpg texture");
        println!("✅ Loaded container.jpg: {}x{}", texture.width, texture.height);

        // Load texture shaders (with sampler support)
        let vert = ash::util::read_spv(&mut std::io::Cursor::new(
            include_bytes!("../shaders/texture_vert.spv")
        )).unwrap();
        let frag = ash::util::read_spv(&mut std::io::Cursor::new(
            include_bytes!("../shaders/texture_frag.spv")
        )).unwrap();

        // Create textured material
        let material = Arc::new(
            ctx.create_textured_material(&vert, &frag, &texture)
                .expect("Failed to create textured material")
        );

        // Store texture to keep it alive for the lifetime of the app
        self.texture = Some(texture);

        // Add cube to scene with textured material
        ctx.scene.add_object(mesh, material, Mat4::IDENTITY);
        
        println!("🖼️ Textured Cube initialized!");
        println!("   - Texture applied to cube material");
        println!("   - Using texture shaders with sampler");
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
    reactor::run(TexturedCube { rotation: 0.0, texture: None });
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
