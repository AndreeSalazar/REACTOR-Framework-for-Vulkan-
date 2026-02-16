// =============================================================================
// free_camera_test.rs — Free Camera Test for 3D Visualization
// =============================================================================
// Test scene with free camera controls (WASD + Mouse) for inspecting 3D objects.
// This is useful for testing textures, models, and lighting.
// =============================================================================

use reactor::prelude::*;
use reactor::Vertex;
use reactor::resources::texture::Texture;
use std::sync::Arc;
use winit::keyboard::KeyCode;

// =============================================================================
// FREE CAMERA TEST
// =============================================================================

struct FreeCameraTest {
    texture: Option<Texture>,
    camera_speed: f32,
    mouse_sensitivity: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for FreeCameraTest {
    fn default() -> Self {
        Self {
            texture: None,
            camera_speed: 3.0,
            mouse_sensitivity: 0.003,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl ReactorApp for FreeCameraTest {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("🎮 Free Camera Test - REACTOR")
            .with_size(1280, 720)
            .with_vsync(true)
            .with_msaa(4)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        // Camera setup - start position
        ctx.camera.position = Vec3::new(0.0, 1.0, 5.0);
        ctx.camera.fov = 60.0;
        self.yaw = 0.0;
        self.pitch = 0.0;

        // Lighting - multiple lights for better visualization
        ctx.lighting.add_light(Light::directional(
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Vec3::new(1.0, 0.98, 0.95),
            1.0,
        ));
        ctx.lighting.add_light(Light::directional(
            Vec3::new(0.5, -0.5, 0.5).normalize(),
            Vec3::new(0.3, 0.4, 0.5),
            0.3,
        ));

        // Load texture
        let texture = ctx.load_texture("assets/textures/container.jpg")
            .expect("Failed to load texture");
        println!("✅ Loaded texture: {}x{}", texture.width, texture.height);

        // Create textured cube
        let mesh = Arc::new(ctx.create_mesh(&cube_vertices_uv(), &cube_indices()).unwrap());
        
        let vert = ash::util::read_spv(&mut std::io::Cursor::new(
            include_bytes!("../shaders/texture_vert.spv")
        )).unwrap();
        let frag = ash::util::read_spv(&mut std::io::Cursor::new(
            include_bytes!("../shaders/texture_frag.spv")
        )).unwrap();

        let material = Arc::new(
            ctx.create_textured_material(&vert, &frag, &texture)
                .expect("Failed to create textured material")
        );
        self.texture = Some(texture);

        // Add multiple cubes for testing
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)));
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(3.0, 0.0, 0.0)));
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(-3.0, 0.0, 0.0)));
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(0.0, 0.0, 3.0)));
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(0.0, 0.0, -3.0)));

        // Floor cube (scaled)
        ctx.scene.add_object(
            mesh.clone(), 
            material.clone(), 
            Mat4::from_scale_rotation_translation(
                Vec3::new(10.0, 0.1, 10.0),
                glam::Quat::IDENTITY,
                Vec3::new(0.0, -1.0, 0.0),
            )
        );

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              🎮 FREE CAMERA TEST                             ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Controls:                                                   ║");
        println!("║    WASD      - Move camera                                   ║");
        println!("║    Space     - Move up                                       ║");
        println!("║    Shift     - Move down                                     ║");
        println!("║    Mouse     - Look around (click to capture)                ║");
        println!("║    ESC       - Release mouse / Exit                          ║");
        println!("║    Q/E       - Roll camera                                   ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.time.delta();
        
        // Collect input state first to avoid borrow conflicts
        let escape_down;
        let shift_down;
        let w_down;
        let s_down;
        let a_down;
        let d_down;
        let space_down;
        let ctrl_down;
        let mouse_delta;
        {
            let input = ctx.input();
            escape_down = input.is_key_down(KeyCode::Escape);
            shift_down = input.is_key_down(KeyCode::ShiftLeft);
            w_down = input.is_key_down(KeyCode::KeyW);
            s_down = input.is_key_down(KeyCode::KeyS);
            a_down = input.is_key_down(KeyCode::KeyA);
            d_down = input.is_key_down(KeyCode::KeyD);
            space_down = input.is_key_down(KeyCode::Space);
            ctrl_down = input.is_key_down(KeyCode::ControlLeft);
            mouse_delta = input.mouse_delta();
        }

        // ESC to exit
        if escape_down {
            std::process::exit(0);
        }

        // Mouse look - use mouse delta for smooth camera control
        if mouse_delta.x != 0.0 || mouse_delta.y != 0.0 {
            self.yaw -= mouse_delta.x * self.mouse_sensitivity;
            self.pitch -= mouse_delta.y * self.mouse_sensitivity;

            // Clamp pitch to avoid gimbal lock
            self.pitch = self.pitch.clamp(-1.5, 1.5);
        }

        // Apply rotation to camera (yaw, pitch order)
        ctx.camera.set_rotation(self.yaw, self.pitch);

        // Calculate movement vectors
        let forward = ctx.camera.forward();
        let right = ctx.camera.right();
        let up = Vec3::Y;

        let mut velocity = Vec3::ZERO;
        let speed = if shift_down {
            self.camera_speed * 2.0 // Sprint
        } else {
            self.camera_speed
        };

        // WASD movement
        if w_down {
            velocity += forward;
        }
        if s_down {
            velocity -= forward;
        }
        if a_down {
            velocity -= right;
        }
        if d_down {
            velocity += right;
        }
        if space_down {
            velocity += up;
        }
        if ctrl_down {
            velocity -= up;
        }

        // Apply movement
        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize() * speed * dt;
            ctx.camera.position += velocity;
        }

        // Rotate objects slowly for visual interest
        let time = ctx.time.elapsed();
        for (i, obj) in ctx.scene.objects.iter_mut().enumerate() {
            if i < 5 { // Only rotate the cubes, not the floor
                let base_pos = match i {
                    0 => Vec3::new(0.0, 0.0, 0.0),
                    1 => Vec3::new(3.0, 0.0, 0.0),
                    2 => Vec3::new(-3.0, 0.0, 0.0),
                    3 => Vec3::new(0.0, 0.0, 3.0),
                    4 => Vec3::new(0.0, 0.0, -3.0),
                    _ => Vec3::ZERO,
                };
                let rotation = glam::Quat::from_rotation_y(time + i as f32 * 0.5);
                obj.transform = Mat4::from_rotation_translation(rotation, base_pos);
            }
        }
    }

    fn render(&mut self, ctx: &mut ReactorContext) {
        ctx.render_scene();
    }
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    reactor::run(FreeCameraTest::default());
}

// =============================================================================
// CUBE DATA WITH UV COORDINATES
// =============================================================================

fn cube_vertices_uv() -> [Vertex; 24] {
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
    ]
}

fn cube_indices() -> [u32; 36] {
    [
        0, 1, 2, 2, 3, 0,       // Front
        4, 5, 6, 6, 7, 4,       // Back
        8, 9, 10, 10, 11, 8,    // Top
        12, 13, 14, 14, 15, 12, // Bottom
        16, 17, 18, 18, 19, 16, // Right
        20, 21, 22, 22, 23, 20, // Left
    ]
}
