// =============================================================================
// hello_reactor.rs — The simplest REACTOR application possible
// =============================================================================
// ReactorContext INHERITS everything:
//   ctx.reactor  → Vulkan engine
//   ctx.camera   → 3D camera (ready)
//   ctx.scene    → Scene graph (ready)
//   ctx.lighting → Lighting system (ready)
//   ctx.physics  → Physics world (ready)
//   ctx.debug    → Debug renderer (ready)
//   ctx.time     → Frame timing (ready)
//   ctx.input()  → Keyboard/Mouse (ready)
//
// You just implement init() + update(). That's it.
// =============================================================================

use reactor::prelude::*;
use reactor::Vertex; // Legacy vertex type for create_mesh
use std::sync::Arc;
use winit::keyboard::KeyCode;

struct HelloReactor {
    cube_mesh: Option<Arc<reactor::Mesh>>,
    material: Option<Arc<reactor::Material>>,
}

impl ReactorApp for HelloReactor {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("🚀 Hello REACTOR!")
            .with_size(1280, 720)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        // Setup camera
        ctx.camera.position = Vec3::new(0.0, 2.0, 5.0);
        ctx.camera.set_rotation(0.0, -0.3);

        // Setup lighting
        ctx.lighting.add_light(Light::directional(
            Vec3::new(-0.5, -1.0, -0.3),
            Vec3::new(1.0, 0.95, 0.8),
            1.0,
        ));

        // Create cube
        let vertices = [
            Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::ZERO),
            Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(1.0, 0.0, 1.0), Vec2::ZERO),
            Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(0.0, 1.0, 1.0), Vec2::ZERO),
            Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
            Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(0.5, 0.5, 0.5), Vec2::ZERO),
        ];
        let indices: [u32; 36] = [
            0,1,2, 2,3,0, 1,5,6, 6,2,1,
            5,4,7, 7,6,5, 4,0,3, 3,7,4,
            3,2,6, 6,7,3, 4,5,1, 1,0,4,
        ];

        let mesh = Arc::new(ctx.create_mesh(&vertices, &indices).unwrap());
        let vert = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../shaders/vert.spv"))).unwrap();
        let frag = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!("../shaders/frag.spv"))).unwrap();
        let material = Arc::new(ctx.create_material(&vert, &frag).unwrap());

        // Add cube to scene — scene is already part of ctx!
        ctx.scene.add_object(mesh.clone(), material.clone(), Mat4::IDENTITY);

        self.cube_mesh = Some(mesh);
        self.material = Some(material);
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        // Escape to quit
        if ctx.input().is_key_down(KeyCode::Escape) {
            std::process::exit(0);
        }

        // Rotate cube
        let t = ctx.time.elapsed();
        let rotation = Mat4::from_rotation_y(t * 1.5) * Mat4::from_rotation_x(t * 0.7);
        ctx.scene.objects[0].transform = rotation;

        // FPS in title
        ctx.set_title(&format!("🚀 Hello REACTOR! | FPS: {:.0}", ctx.fps()));
    }

    // render() is NOT needed! Default renders ctx.scene with ctx.camera automatically.
}

fn main() {
    reactor::run(HelloReactor {
        cube_mesh: None,
        material: None,
    });
}
