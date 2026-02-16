// =============================================================================
// physics_camera.rs — FPS Camera with Physics (Gravity + Collision)
// =============================================================================
// Demonstrates CharacterController with gravity, jumping, and ground collision.
// =============================================================================

use reactor::prelude::*;
use reactor::Vertex;
use reactor::resources::texture::Texture;
use reactor::systems::physics::CharacterController;
use std::sync::Arc;
use winit::keyboard::KeyCode;

// =============================================================================
// PHYSICS CAMERA DEMO
// =============================================================================

struct PhysicsCameraDemo {
    texture: Option<Texture>,
    controller: CharacterController,
    yaw: f32,
    pitch: f32,
    mouse_sensitivity: f32,
}

impl Default for PhysicsCameraDemo {
    fn default() -> Self {
        let mut controller = CharacterController::default();
        controller.position = Vec3::new(0.0, 2.0, 5.0);
        controller.move_speed = 5.0;
        controller.jump_force = 6.0;
        
        Self {
            texture: None,
            controller,
            yaw: 0.0,
            pitch: 0.0,
            mouse_sensitivity: 0.003,
        }
    }
}

impl ReactorApp for PhysicsCameraDemo {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("🎮 Physics Camera Demo - REACTOR")
            .with_size(1280, 720)
            .with_vsync(true)
            .with_msaa(4)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        // Camera initial setup
        ctx.camera.fov = 70.0;

        // Lighting
        ctx.lighting.add_light(Light::directional(
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Vec3::new(1.0, 0.98, 0.95),
            1.0,
        ));

        // Load texture
        let texture = ctx.load_texture("assets/textures/container.jpg")
            .expect("Failed to load texture");
        println!("✅ Loaded texture: {}x{}", texture.width, texture.height);

        // Create textured cube mesh
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

        // Add cubes at various positions
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0)));
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(3.0, 0.5, 0.0)));
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(-3.0, 0.5, 0.0)));
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(0.0, 0.5, 3.0)));
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::from_translation(Vec3::new(0.0, 0.5, -3.0)));

        // Floor (large flat cube)
        ctx.scene.add_object(
            mesh.clone(), 
            material.clone(), 
            Mat4::from_scale_rotation_translation(
                Vec3::new(20.0, 0.2, 20.0),
                glam::Quat::IDENTITY,
                Vec3::new(0.0, -0.1, 0.0),
            )
        );

        // Some platforms to jump on
        ctx.scene.add_object(
            mesh.clone(), 
            material.clone(), 
            Mat4::from_scale_rotation_translation(
                Vec3::new(2.0, 0.3, 2.0),
                glam::Quat::IDENTITY,
                Vec3::new(5.0, 1.0, 0.0),
            )
        );
        ctx.scene.add_object(
            mesh.clone(), 
            material.clone(), 
            Mat4::from_scale_rotation_translation(
                Vec3::new(2.0, 0.3, 2.0),
                glam::Quat::IDENTITY,
                Vec3::new(7.0, 2.0, 0.0),
            )
        );

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              🎮 PHYSICS CAMERA DEMO                          ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Controls:                                                   ║");
        println!("║    WASD      - Move                                          ║");
        println!("║    SPACE     - Jump                                          ║");
        println!("║    SHIFT     - Sprint                                        ║");
        println!("║    Mouse     - Look around                                   ║");
        println!("║    ESC       - Exit                                          ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Physics: Gravity, Ground collision, Jumping                 ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        let dt = ctx.time.delta();
        
        // Collect input
        let escape_down;
        let shift_down;
        let w_down;
        let s_down;
        let a_down;
        let d_down;
        let space_down;
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
            mouse_delta = input.mouse_delta();
        }

        // ESC to exit
        if escape_down {
            std::process::exit(0);
        }

        // Mouse look
        if mouse_delta.x != 0.0 || mouse_delta.y != 0.0 {
            self.yaw -= mouse_delta.x * self.mouse_sensitivity;
            self.pitch -= mouse_delta.y * self.mouse_sensitivity;
            self.pitch = self.pitch.clamp(-1.4, 1.4);
        }

        // Calculate movement direction relative to camera
        let forward = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());

        let mut move_input = Vec3::ZERO;
        if w_down { move_input += forward; }
        if s_down { move_input -= forward; }
        if d_down { move_input += right; }
        if a_down { move_input -= right; }

        // Sprint
        if shift_down {
            self.controller.move_speed = 10.0;
        } else {
            self.controller.move_speed = 5.0;
        }

        // Update physics controller (ground at Y=0)
        self.controller.update(dt, move_input, space_down, 0.0);

        // Update camera position from controller
        ctx.camera.position = self.controller.eye_position();
        ctx.camera.set_rotation(self.yaw, self.pitch);

        // Rotate cubes for visual interest
        let time = ctx.time.elapsed();
        for (i, obj) in ctx.scene.objects.iter_mut().enumerate() {
            if i < 5 {
                let base_pos = match i {
                    0 => Vec3::new(0.0, 0.5, 0.0),
                    1 => Vec3::new(3.0, 0.5, 0.0),
                    2 => Vec3::new(-3.0, 0.5, 0.0),
                    3 => Vec3::new(0.0, 0.5, 3.0),
                    4 => Vec3::new(0.0, 0.5, -3.0),
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
    reactor::run(PhysicsCameraDemo::default());
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
