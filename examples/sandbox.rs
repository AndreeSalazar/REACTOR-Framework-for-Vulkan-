use reactor::{Reactor, Vertex, ResolutionDetector, CPUDetector, Scene};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
    keyboard::KeyCode,
    dpi::LogicalSize,
};
use std::sync::Arc;
use glam::{Vec3, Mat4, Vec2};

// Configuration: Easy to modify resolution
const TARGET_WIDTH: f32 = 800.0;
const TARGET_HEIGHT: f32 = 600.0;
// const TARGET_WIDTH: f32 = 7680.0; // 8K Example
// const TARGET_HEIGHT: f32 = 4320.0;

struct AppState {
    // Drop order matters: fields are dropped in declaration order.
    // Scene (holding Meshes/Materials) must be dropped BEFORE Reactor (which owns the Device).
    scene: Scene,
    reactor: Reactor,
    window: Arc<Window>,
}

struct App {
    state: Option<AppState>,
    position: Vec3,
    rotation_y: f32,
    sun_rotation: f32,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() { return; }

        // Detect Hardware
        CPUDetector::detect();

        // Smart Resolution Logic using ResolutionDetector
        let (width, height) = ResolutionDetector::get_smart_resolution(
            event_loop,
            TARGET_WIDTH,
            TARGET_HEIGHT,
        );

        let window_attributes = Window::default_attributes()
            .with_title("REACTOR Sandbox (Rust)")
            .with_inner_size(LogicalSize::new(width, height));
            
        let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));

        let reactor = Reactor::init(&window).expect("Failed to initialize Reactor");
        println!("REACTOR initialized successfully with Vulkan!");

        // --- Create Meshes ---

        // 1. Rotating Cube (Multi-colored)
        let cube_vertices = [
            // Front face (Z+)
            Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::ZERO), // Red
            Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::ZERO), // Green
            Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::ZERO), // Blue
            Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO), // Yellow
            // Back face (Z-)
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(1.0, 0.0, 1.0), Vec2::ZERO), // Magenta
            Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(0.0, 1.0, 1.0), Vec2::ZERO), // Cyan
            Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO), // White
            Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(0.0, 0.0, 0.0), Vec2::ZERO), // Black
        ];

        let cube_indices = [
            0, 1, 2, 2, 3, 0, // Front
            1, 5, 6, 6, 2, 1, // Right
            5, 4, 7, 7, 6, 5, // Back
            4, 0, 3, 3, 7, 4, // Left
            3, 2, 6, 6, 7, 3, // Top
            4, 5, 1, 1, 0, 4, // Bottom
        ];

        let cube_mesh = Arc::new(reactor.create_mesh(&cube_vertices, &cube_indices).expect("Failed to create cube mesh"));

        // 2. Floor (Large, Green/Gray)
        // Flattened cube
        let floor_vertices = [
            Vertex::new(Vec3::new(-10.0, -0.1,  10.0), Vec3::new(0.2, 0.3, 0.2), Vec2::ZERO),
            Vertex::new(Vec3::new( 10.0, -0.1,  10.0), Vec3::new(0.2, 0.3, 0.2), Vec2::ZERO),
            Vertex::new(Vec3::new( 10.0,  0.0,  10.0), Vec3::new(0.2, 0.3, 0.2), Vec2::ZERO),
            Vertex::new(Vec3::new(-10.0,  0.0,  10.0), Vec3::new(0.2, 0.3, 0.2), Vec2::ZERO),
            Vertex::new(Vec3::new(-10.0, -0.1, -10.0), Vec3::new(0.2, 0.3, 0.2), Vec2::ZERO),
            Vertex::new(Vec3::new( 10.0, -0.1, -10.0), Vec3::new(0.2, 0.3, 0.2), Vec2::ZERO),
            Vertex::new(Vec3::new( 10.0,  0.0, -10.0), Vec3::new(0.2, 0.3, 0.2), Vec2::ZERO),
            Vertex::new(Vec3::new(-10.0,  0.0, -10.0), Vec3::new(0.2, 0.3, 0.2), Vec2::ZERO),
        ];
        let floor_mesh = Arc::new(reactor.create_mesh(&floor_vertices, &cube_indices).expect("Failed to create floor mesh"));

        // 3. Sun (Small, Yellow/Bright)
        let sun_vertices = [
            Vertex::new(Vec3::new(-0.2, -0.2,  0.2), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new( 0.2, -0.2,  0.2), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new( 0.2,  0.2,  0.2), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new(-0.2,  0.2,  0.2), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new(-0.2, -0.2, -0.2), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new( 0.2, -0.2, -0.2), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new( 0.2,  0.2, -0.2), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
            Vertex::new(Vec3::new(-0.2,  0.2, -0.2), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
        ];
        let sun_mesh = Arc::new(reactor.create_mesh(&sun_vertices, &cube_indices).expect("Failed to create sun mesh"));


        // Load Shaders (Shared Material for now)
        let vert_code = include_bytes!("../shaders/vert.spv");
        let frag_code = include_bytes!("../shaders/frag.spv");

        let vert_decoded = ash::util::read_spv(&mut std::io::Cursor::new(&vert_code[..])).expect("Failed to read vert spv");
        let frag_decoded = ash::util::read_spv(&mut std::io::Cursor::new(&frag_code[..])).expect("Failed to read frag spv");

        let material = Arc::new(reactor.create_material(&vert_decoded, &frag_decoded).expect("Failed to create material"));

        // --- Build Scene ---
        let mut scene = Scene::new();

        // 1. Floor (Static)
        scene.add_object(
            floor_mesh,
            material.clone(),
            Mat4::from_translation(Vec3::new(0.0, -2.0, 0.0)) // Move floor down
        );

        // 2. Sun (Will animate)
        scene.add_object(
            sun_mesh,
            material.clone(),
            Mat4::from_translation(Vec3::new(2.0, 2.0, -2.0))
        );

        // 3. Rotating Cube (Will animate)
        scene.add_object(
            cube_mesh,
            material.clone(),
            Mat4::IDENTITY
        );

        self.state = Some(AppState {
            scene,
            reactor,
            window,
        });
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            println!("Exiting... Waiting for GPU to finish...");
            unsafe {
                state.reactor.context.device.device_wait_idle().unwrap();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let Some(state) = &mut self.state {
            state.reactor.handle_event(&event);
            match event {
                WindowEvent::CloseRequested => {
                    println!("Closing...");
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    // Update Logic
                    
                    // 1. Rotate Main Cube (Index 2)
                    self.rotation_y += 0.02;
                    if self.rotation_y > std::f32::consts::TAU {
                        self.rotation_y -= std::f32::consts::TAU;
                    }

                    // 2. Animate Sun (Index 1) - Slowly orbit or pulse
                    self.sun_rotation += 0.005;
                    
                    // Handle Input
                    if state.reactor.input.is_key_down(KeyCode::ArrowRight) { self.position.x += 0.05; }
                    if state.reactor.input.is_key_down(KeyCode::ArrowLeft) { self.position.x -= 0.05; }
                    if state.reactor.input.is_key_down(KeyCode::ArrowUp) { self.position.y -= 0.05; }
                    if state.reactor.input.is_key_down(KeyCode::ArrowDown) { self.position.y += 0.05; }

                    // Camera / View Projection
                    let width = state.window.inner_size().width as f32;
                    let height = state.window.inner_size().height as f32;
                    let aspect = width / height;

                    let mut projection = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
                    projection.y_axis.y *= -1.0; // Vulkan Y-flip

                    let view = Mat4::look_at_rh(
                        Vec3::new(0.0, 1.0, 5.0), // Camera slightly up and back
                        Vec3::ZERO,               // Looking at center
                        Vec3::Y,
                    );
                    
                    let view_proj = projection * view;

                    // Update Scene Objects
                    // Object 0: Floor (Static, but could move)
                    // Object 1: Sun
                    let sun_x = self.sun_rotation.cos() * 3.0;
                    let sun_z = self.sun_rotation.sin() * 3.0;
                    state.scene.objects[1].transform = Mat4::from_translation(Vec3::new(sun_x, 2.0, sun_z));

                    // Object 2: Rotating Cube
                    let rotation = Mat4::from_rotation_y(self.rotation_y) * Mat4::from_rotation_x(self.rotation_y * 0.5);
                    let model = Mat4::from_translation(self.position) * rotation;
                    state.scene.objects[2].transform = model;

                    // Draw Scene
                    if let Err(e) = state.reactor.draw_scene(&state.scene, &view_proj) {
                        eprintln!("Draw error: {}", e);
                    }
                    
                    state.window.request_redraw();
                }
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                     if key_event.state == winit::event::ElementState::Pressed {
                         match key_event.physical_key {
                             winit::keyboard::PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                             _ => (),
                         }
                     }
                }
                _ => (),
            }
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App {
        state: None,
        position: Vec3::ZERO,
        rotation_y: 0.0,
        sun_rotation: 0.0,
    };
    
    event_loop.run_app(&mut app).unwrap();
}
