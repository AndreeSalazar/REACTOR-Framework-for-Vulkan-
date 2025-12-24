// =============================================================================
// ADead-GPU Demo - REACTOR Framework
// =============================================================================
// Este ejemplo demuestra TODAS las características de ADead-GPU:
// - ADead-ISR: Intelligent Shading Rate (75% GPU savings)
// - ADead-SDF: Signed Distance Functions
// - ADead-RT: Ray Marching sin RT Cores
// - ADead-AA: Anti-Aliasing perfecto
// - ADead-Hybrid: Rendering híbrido SDF + Meshes
// =============================================================================

use reactor::{
    Reactor, Scene,
    // ADead-GPU
    IntelligentShadingRate, ISRConfig, ISRBenchmark,
    SDFPrimitive, SDFScene, RayMarcher,
    SDFAntiAliasing, AAComparison,
    HybridRenderer, ADeadBenchmark,
    // Systems
    Camera, Time,
    CPUDetector, ResolutionDetector,
};
use reactor::Vertex;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
    keyboard::KeyCode,
    dpi::LogicalSize,
};
use std::sync::Arc;
use glam::{Vec2, Vec3, Vec4, Mat4};

// =============================================================================
// Configuration
// =============================================================================
const TARGET_WIDTH: f32 = 1280.0;
const TARGET_HEIGHT: f32 = 720.0;
const CAMERA_SPEED: f32 = 5.0;

// =============================================================================
// Demo Mode
// =============================================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DemoMode {
    ISR,        // Intelligent Shading Rate visualization
    SDF,        // SDF primitives and CSG
    RayMarch,   // Ray marching demo
    AA,         // Anti-aliasing comparison
    Hybrid,     // Hybrid rendering
    Benchmark,  // Full benchmark
}

impl DemoMode {
    fn next(&self) -> Self {
        match self {
            DemoMode::ISR => DemoMode::SDF,
            DemoMode::SDF => DemoMode::RayMarch,
            DemoMode::RayMarch => DemoMode::AA,
            DemoMode::AA => DemoMode::Hybrid,
            DemoMode::Hybrid => DemoMode::Benchmark,
            DemoMode::Benchmark => DemoMode::ISR,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            DemoMode::ISR => "ADead-ISR (Intelligent Shading Rate)",
            DemoMode::SDF => "ADead-SDF (Signed Distance Functions)",
            DemoMode::RayMarch => "ADead-RT (Ray Marching)",
            DemoMode::AA => "ADead-AA (Anti-Aliasing)",
            DemoMode::Hybrid => "ADead-Hybrid (SDF + Mesh)",
            DemoMode::Benchmark => "Full Benchmark vs DLSS",
        }
    }
}

// =============================================================================
// Application State
// =============================================================================
struct ADeadDemo {
    state: Option<AppState>,
    
    // Camera
    camera: Camera,
    camera_yaw: f32,
    camera_pitch: f32,
    
    // Time
    time: Time,
    
    // ADead Systems
    isr: IntelligentShadingRate,
    sdf_scene: SDFScene,
    ray_marcher: RayMarcher,
    aa: SDFAntiAliasing,
    hybrid: HybridRenderer,
    
    // Demo state
    current_mode: DemoMode,
    animation_time: f32,
    show_stats: bool,
    isr_preset: u8, // 0=balanced, 1=performance, 2=quality, 3=vr
}

struct AppState {
    scene: Scene,
    reactor: Reactor,
    window: Arc<Window>,
}

// =============================================================================
// Application Implementation
// =============================================================================
impl ApplicationHandler for ADeadDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() { return; }

        // Hardware Detection
        let cpu_info = CPUDetector::detect();
        
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║           🔥 ADead-GPU Demo - REACTOR Framework 🔥               ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║ CPU: {} cores | {}                          ║", 
            cpu_info.logical_cores, cpu_info.recommendation);

        // Smart Resolution
        let (width, height) = ResolutionDetector::get_smart_resolution(
            event_loop, TARGET_WIDTH, TARGET_HEIGHT
        );

        let window_attributes = Window::default_attributes()
            .with_title("🔥 ADead-GPU Demo")
            .with_inner_size(LogicalSize::new(width, height));
            
        let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));
        let reactor = Reactor::init(&window).expect("Failed to initialize Reactor");
        
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║ Controls:                                                        ║");
        println!("║   TAB       - Switch demo mode                                   ║");
        println!("║   1-4       - ISR Presets (Balanced/Performance/Quality/VR)      ║");
        println!("║   WASD      - Move camera                                        ║");
        println!("║   Mouse     - Look around (hold right click)                     ║");
        println!("║   S         - Toggle stats                                       ║");
        println!("║   B         - Run benchmark                                      ║");
        println!("║   C         - Compare AA methods                                 ║");
        println!("║   ESC       - Exit                                               ║");
        println!("╚══════════════════════════════════════════════════════════════════╝");

        // Create scene with visual content
        let mut scene = Scene::new();

        // Load shaders
        let vert_code = include_bytes!("../shaders/vert.spv");
        let frag_code = include_bytes!("../shaders/frag.spv");
        let vert_decoded = ash::util::read_spv(&mut std::io::Cursor::new(&vert_code[..])).unwrap();
        let frag_decoded = ash::util::read_spv(&mut std::io::Cursor::new(&frag_code[..])).unwrap();
        let material = std::sync::Arc::new(reactor.create_material(&vert_decoded, &frag_decoded).unwrap());

        // Create floor
        let floor_verts = create_floor_vertices();
        let floor_indices = create_floor_indices();
        let floor_mesh = std::sync::Arc::new(reactor.create_mesh(&floor_verts, &floor_indices).unwrap());
        scene.add_object(
            floor_mesh,
            material.clone(),
            glam::Mat4::from_scale_rotation_translation(
                Vec3::new(20.0, 1.0, 20.0),
                glam::Quat::IDENTITY,
                Vec3::new(0.0, -1.0, 0.0)
            )
        );

        // Create cubes representing SDF objects visually
        let (cube_verts, cube_indices) = create_cube();
        let cube_mesh = std::sync::Arc::new(reactor.create_mesh(&cube_verts, &cube_indices).unwrap());

        // Red sphere representation (cube for now)
        scene.add_object(
            cube_mesh.clone(),
            material.clone(),
            glam::Mat4::from_scale_rotation_translation(
                Vec3::splat(1.0),
                glam::Quat::IDENTITY,
                Vec3::new(0.0, 1.0, 0.0)
            )
        );

        // Green cube
        scene.add_object(
            cube_mesh.clone(),
            material.clone(),
            glam::Mat4::from_scale_rotation_translation(
                Vec3::splat(0.7),
                glam::Quat::IDENTITY,
                Vec3::new(3.0, 0.5, 0.0)
            )
        );

        // Blue cylinder representation
        scene.add_object(
            cube_mesh.clone(),
            material.clone(),
            glam::Mat4::from_scale_rotation_translation(
                Vec3::new(0.5, 1.5, 0.5),
                glam::Quat::IDENTITY,
                Vec3::new(-3.0, 1.0, 0.0)
            )
        );

        // Yellow torus representation
        scene.add_object(
            cube_mesh.clone(),
            material.clone(),
            glam::Mat4::from_scale_rotation_translation(
                Vec3::new(1.2, 0.3, 1.2),
                glam::Quat::IDENTITY,
                Vec3::new(0.0, 1.0, 3.0)
            )
        );

        // Additional objects for visual richness
        for i in 0..5 {
            let angle = i as f32 * std::f32::consts::TAU / 5.0;
            let x = angle.cos() * 6.0;
            let z = angle.sin() * 6.0;
            scene.add_object(
                cube_mesh.clone(),
                material.clone(),
                glam::Mat4::from_scale_rotation_translation(
                    Vec3::splat(0.5),
                    glam::Quat::from_rotation_y(angle),
                    Vec3::new(x, 0.5, z)
                )
            );
        }

        // Initialize ADead systems
        self.isr = IntelligentShadingRate::new(width as u32, height as u32);
        self.setup_sdf_scene();
        self.setup_hybrid_scene();

        // Setup camera
        self.camera = Camera::new();
        self.camera.position = Vec3::new(0.0, 3.0, 10.0);
        self.camera_yaw = 0.0;
        self.camera_pitch = -0.2;
        self.camera.set_rotation(self.camera_yaw, self.camera_pitch);

        self.state = Some(AppState {
            scene,
            reactor,
            window,
        });

        // Print initial mode
        println!("\n🎮 Current Mode: {}", self.current_mode.name());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        // Handle event first
        if let Some(state) = &mut self.state {
            state.reactor.handle_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            
            WindowEvent::RedrawRequested => {
                self.time.update();
                let dt = self.time.delta();
                self.animation_time += dt;

                // Get input state before borrowing state mutably
                let (mouse_right, mouse_delta, key_w, key_s, key_a, key_d, key_space, key_c) = {
                    if let Some(state) = &self.state {
                        let input = &state.reactor.input;
                        (
                            input.is_mouse_down(winit::event::MouseButton::Right),
                            input.mouse_delta(),
                            input.is_key_down(KeyCode::KeyW),
                            input.is_key_down(KeyCode::KeyS),
                            input.is_key_down(KeyCode::KeyA),
                            input.is_key_down(KeyCode::KeyD),
                            input.is_key_down(KeyCode::Space),
                            input.is_key_down(KeyCode::KeyC),
                        )
                    } else {
                        return;
                    }
                };

                let move_speed = CAMERA_SPEED * dt;

                // Mouse look
                if mouse_right {
                    let sensitivity = 0.005;
                    self.camera_yaw -= mouse_delta.x * sensitivity;
                    self.camera_pitch -= mouse_delta.y * sensitivity;
                    self.camera_pitch = self.camera_pitch.clamp(-1.5, 1.5);
                    self.camera.set_rotation(self.camera_yaw, self.camera_pitch);
                }

                // Movement
                if key_w { self.camera.move_forward(move_speed); }
                if key_s { self.camera.move_forward(-move_speed); }
                if key_a { self.camera.move_right(-move_speed); }
                if key_d { self.camera.move_right(move_speed); }
                if key_space { self.camera.move_up(move_speed); }
                if key_c { self.camera.move_up(-move_speed); }

                // Update current demo mode
                self.update_demo_mode(dt);

                // Render
                let vp = self.camera.view_projection_matrix();
                let mode_name = self.current_mode.name().to_string();
                let fps = self.time.fps();
                
                if let Some(state) = &mut self.state {
                    if let Err(e) = state.reactor.draw_scene(&state.scene, &vp) {
                        eprintln!("Draw error: {}", e);
                    }

                    // Update window title
                    let title = format!("🔥 ADead-GPU | Mode: {} | FPS: {:.0}", mode_name, fps);
                    state.window.set_title(&title);
                    
                    state.window.request_redraw();
                }
            }

            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if key_event.state == winit::event::ElementState::Pressed {
                    match key_event.physical_key {
                        winit::keyboard::PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                        
                        winit::keyboard::PhysicalKey::Code(KeyCode::Tab) => {
                            self.current_mode = self.current_mode.next();
                            println!("\n🎮 Switched to: {}", self.current_mode.name());
                            self.print_mode_info();
                        }
                        
                        winit::keyboard::PhysicalKey::Code(KeyCode::Digit1) => {
                            self.isr.config = ISRConfig::default();
                            self.isr_preset = 0;
                            println!("📊 ISR Preset: Balanced");
                        }
                        winit::keyboard::PhysicalKey::Code(KeyCode::Digit2) => {
                            self.isr.config = IntelligentShadingRate::preset_performance();
                            self.isr_preset = 1;
                            println!("📊 ISR Preset: Performance (Max GPU Savings)");
                        }
                        winit::keyboard::PhysicalKey::Code(KeyCode::Digit3) => {
                            self.isr.config = IntelligentShadingRate::preset_quality();
                            self.isr_preset = 2;
                            println!("📊 ISR Preset: Quality (Max Visual Quality)");
                        }
                        winit::keyboard::PhysicalKey::Code(KeyCode::Digit4) => {
                            self.isr.config = IntelligentShadingRate::preset_vr();
                            self.isr_preset = 3;
                            println!("📊 ISR Preset: VR (Foveated Rendering)");
                        }
                        
                        winit::keyboard::PhysicalKey::Code(KeyCode::KeyB) => {
                            self.run_benchmark();
                        }
                        
                        winit::keyboard::PhysicalKey::Code(KeyCode::KeyP) => {
                            AAComparison::print_comparison();
                        }
                        
                        winit::keyboard::PhysicalKey::Code(KeyCode::KeyI) => {
                            self.show_stats = !self.show_stats;
                            if self.show_stats {
                                self.print_stats();
                            }
                        }
                        
                        _ => {}
                    }
                }
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.camera.set_aspect_ratio(size.width as f32, size.height as f32);
                    self.isr.resize(size.width, size.height);
                    self.hybrid.resize(size.width, size.height);
                }
            }

            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            println!("\n╔══════════════════════════════════════════════════════════════════╗");
            println!("║                    Shutting down ADead-GPU...                    ║");
            println!("╚══════════════════════════════════════════════════════════════════╝");
            unsafe {
                state.reactor.context.device.device_wait_idle().unwrap();
            }
        }
    }
}

impl ADeadDemo {
    fn new() -> Self {
        Self {
            state: None,
            camera: Camera::new(),
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            time: Time::new(),
            isr: IntelligentShadingRate::new(1280, 720),
            sdf_scene: SDFScene::new(),
            ray_marcher: RayMarcher::new(),
            aa: SDFAntiAliasing::new(),
            hybrid: HybridRenderer::new(1280, 720),
            current_mode: DemoMode::ISR,
            animation_time: 0.0,
            show_stats: false,
            isr_preset: 0,
        }
    }

    fn setup_sdf_scene(&mut self) {
        self.sdf_scene = SDFScene::new();
        self.sdf_scene.light_direction = Vec3::new(-0.5, -1.0, -0.3).normalize();
        self.sdf_scene.light_color = Vec3::new(1.0, 0.95, 0.9);
        self.sdf_scene.ambient_intensity = 0.15;
        self.sdf_scene.background_color = Vec4::new(0.4, 0.6, 0.9, 1.0);

        // Add SDF primitives
        self.sdf_scene.add(SDFPrimitive::sphere(Vec3::new(0.0, 1.0, 0.0), 1.0)
            .with_color(Vec4::new(1.0, 0.3, 0.3, 1.0)));
        
        self.sdf_scene.add(SDFPrimitive::cube(Vec3::new(3.0, 0.5, 0.0), Vec3::new(0.5, 0.5, 0.5))
            .with_color(Vec4::new(0.3, 1.0, 0.3, 1.0)));
        
        self.sdf_scene.add(SDFPrimitive::cylinder(Vec3::new(-3.0, 1.0, 0.0), 1.0, 0.5)
            .with_color(Vec4::new(0.3, 0.3, 1.0, 1.0)));
        
        self.sdf_scene.add(SDFPrimitive::torus(Vec3::new(0.0, 1.0, 3.0), 1.0, 0.3)
            .with_color(Vec4::new(1.0, 1.0, 0.3, 1.0)));

        // Ground plane (large box)
        self.sdf_scene.add(SDFPrimitive::cube(Vec3::new(0.0, -0.5, 0.0), Vec3::new(10.0, 0.5, 10.0))
            .with_color(Vec4::new(0.5, 0.5, 0.5, 1.0)));
    }

    fn setup_hybrid_scene(&mut self) {
        self.hybrid = HybridRenderer::new(1280, 720);
        
        // Add SDF objects
        self.hybrid.add_sphere("Sun", Vec3::new(0.0, 5.0, -10.0), 2.0, Vec4::new(1.0, 0.9, 0.0, 1.0));
        self.hybrid.add_cube("Building1", Vec3::new(-5.0, 2.0, 0.0), Vec3::new(1.0, 2.0, 1.0), Vec4::new(0.7, 0.7, 0.8, 1.0));
        self.hybrid.add_cube("Building2", Vec3::new(5.0, 3.0, 0.0), Vec3::new(1.5, 3.0, 1.5), Vec4::new(0.8, 0.7, 0.7, 1.0));
        self.hybrid.add_sphere("Tree1", Vec3::new(-3.0, 1.5, 5.0), 1.5, Vec4::new(0.2, 0.6, 0.2, 1.0));
        self.hybrid.add_sphere("Tree2", Vec3::new(3.0, 1.2, 5.0), 1.2, Vec4::new(0.3, 0.7, 0.3, 1.0));
    }

    fn update_demo_mode(&mut self, _dt: f32) {
        match self.current_mode {
            DemoMode::ISR => {
                // Update ISR importance map
                self.update_isr_demo();
            }
            DemoMode::SDF => {
                // Animate SDF objects
                self.update_sdf_demo();
            }
            DemoMode::RayMarch => {
                // Ray marching demo
                self.update_raymarching_demo();
            }
            DemoMode::AA => {
                // AA comparison
            }
            DemoMode::Hybrid => {
                // Update hybrid renderer
                self.hybrid.update(self.camera.position, _dt);
            }
            DemoMode::Benchmark => {
                // Benchmark mode
            }
        }
    }

    fn update_isr_demo(&mut self) {
        // Simulate importance calculation for demo
        let center = Vec2::new(0.5, 0.5);
        
        for y in 0..self.isr.importance_map.height {
            for x in 0..self.isr.importance_map.width {
                let uv = Vec2::new(
                    x as f32 / self.isr.importance_map.width as f32,
                    y as f32 / self.isr.importance_map.height as f32,
                );
                
                // Simulate edge detection (circular pattern)
                let dist = (uv - center).length();
                let edge_factor = ((dist * 10.0 + self.animation_time).sin().abs());
                
                // Calculate importance
                let importance = edge_factor * 0.8 + 0.2;
                self.isr.update_tile_importance(x, y, importance);
            }
        }
    }

    fn update_sdf_demo(&mut self) {
        // Animate the sphere position
        if let Some(prim) = self.sdf_scene.primitives.get_mut(0) {
            prim.position.y = 1.0 + (self.animation_time * 2.0).sin() * 0.5;
        }
        
        // Rotate the torus
        if let Some(prim) = self.sdf_scene.primitives.get_mut(3) {
            prim.rotation = glam::Quat::from_rotation_y(self.animation_time);
        }
    }

    fn update_raymarching_demo(&mut self) {
        // Sample ray march from camera
        let ray_dir = self.camera.forward();
        let hit = self.ray_marcher.march(&self.sdf_scene, self.camera.position, ray_dir);
        
        if hit.hit && self.show_stats {
            println!("Ray hit at {:?}, steps: {}", hit.position, hit.steps);
        }
    }

    fn print_mode_info(&self) {
        match self.current_mode {
            DemoMode::ISR => {
                println!("  📊 Intelligent Shading Rate - Adaptive resolution rendering");
                println!("  Press 1-4 to change presets, I for stats");
            }
            DemoMode::SDF => {
                println!("  🔷 Signed Distance Functions - Mathematical primitives");
                println!("  Watch the animated sphere and rotating torus");
            }
            DemoMode::RayMarch => {
                println!("  🔦 Ray Marching - Ray tracing without RT Cores");
                println!("  Press I to see ray hit info");
            }
            DemoMode::AA => {
                println!("  ✨ Anti-Aliasing - SDF-based perfect edges");
                println!("  Press C to compare AA methods");
            }
            DemoMode::Hybrid => {
                println!("  🔀 Hybrid Rendering - SDF + Mesh combined");
                println!("  Automatic LOD based on distance");
            }
            DemoMode::Benchmark => {
                println!("  📈 Benchmark Mode - Press B to run full benchmark");
            }
        }
    }

    fn print_stats(&mut self) {
        println!("\n╔══════════════════════════════════════════════════════════════════╗");
        println!("║                     ADead-GPU Statistics                         ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        
        // ISR Stats
        let isr_stats = self.isr.stats();
        println!("║ ISR:                                                             ║");
        println!("║   Total Pixels:    {:10}                                    ║", isr_stats.total_pixels);
        println!("║   Rendered Pixels: {:10}                                    ║", isr_stats.rendered_pixels);
        println!("║   GPU Savings:     {:5.1}%                                        ║", isr_stats.savings_percent * 100.0);
        println!("║   Distribution:                                                  ║");
        println!("║     Critical: {:4.1}%  High: {:4.1}%  Medium: {:4.1}%              ║", 
            isr_stats.distribution[0] * 100.0,
            isr_stats.distribution[1] * 100.0,
            isr_stats.distribution[2] * 100.0);
        println!("║     Low: {:4.1}%  Minimal: {:4.1}%                                 ║",
            isr_stats.distribution[3] * 100.0,
            isr_stats.distribution[4] * 100.0);
        
        // Hybrid Stats
        println!("║                                                                  ║");
        println!("║ Hybrid Renderer:                                                 ║");
        println!("║   SDF Objects:    {:4}                                           ║", self.hybrid.stats.sdf_objects);
        println!("║   Mesh Objects:   {:4}                                           ║", self.hybrid.stats.mesh_objects);
        println!("║   Hybrid Objects: {:4}                                           ║", self.hybrid.stats.hybrid_objects);
        
        // Ray Marcher Stats
        println!("║                                                                  ║");
        println!("║ Ray Marcher:                                                     ║");
        println!("║   Max Steps: {:4}                                                ║", self.ray_marcher.config.max_steps);
        println!("║   Max Distance: {:6.1}                                           ║", self.ray_marcher.config.max_distance);
        
        println!("╚══════════════════════════════════════════════════════════════════╝");
    }

    fn run_benchmark(&mut self) {
        println!("\n🏁 Running ADead-GPU Benchmark...\n");
        
        // ISR Benchmark
        let isr_bench = ISRBenchmark::calculate(&mut self.isr, 16.6);
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                   ISR Benchmark Results                          ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║ Traditional FPS:  {:6.1}                                         ║", isr_bench.traditional_fps);
        println!("║ ADead-ISR FPS:    {:6.1}                                         ║", isr_bench.isr_fps);
        println!("║ Speedup:          {:5.2}x                                          ║", isr_bench.speedup);
        println!("║ GPU Savings:      {:5.1}%                                          ║", isr_bench.gpu_savings);
        println!("║ Quality:          {:5.1}%                                          ║", isr_bench.quality_estimate);
        println!("╚══════════════════════════════════════════════════════════════════╝");
        
        // Hybrid Benchmark
        let hybrid_bench = ADeadBenchmark::run("Demo Scene", &mut self.hybrid, 16.6);
        hybrid_bench.print();
        hybrid_bench.compare_with_dlss();
        
        // AA Comparison
        println!("\n");
        AAComparison::print_comparison();
    }
}

// =============================================================================
// Helper Functions for Mesh Creation
// =============================================================================

fn create_cube() -> (Vec<Vertex>, Vec<u32>) {
    let color = Vec3::new(0.8, 0.6, 0.4);
    let vertices = vec![
        // Front face
        Vertex::new(Vec3::new(-0.5, -0.5,  0.5), color, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new( 0.5, -0.5,  0.5), color, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new( 0.5,  0.5,  0.5), color, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-0.5,  0.5,  0.5), color, Vec2::new(0.0, 1.0)),
        // Back face
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), color, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new( 0.5, -0.5, -0.5), color, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new( 0.5,  0.5, -0.5), color, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-0.5,  0.5, -0.5), color, Vec2::new(0.0, 1.0)),
    ];
    let indices = vec![
        0, 1, 2, 2, 3, 0, // Front
        1, 5, 6, 6, 2, 1, // Right
        5, 4, 7, 7, 6, 5, // Back
        4, 0, 3, 3, 7, 4, // Left
        3, 2, 6, 6, 7, 3, // Top
        4, 5, 1, 1, 0, 4, // Bottom
    ];
    (vertices, indices)
}

fn create_floor_vertices() -> Vec<Vertex> {
    let color = Vec3::new(0.4, 0.5, 0.4);
    vec![
        Vertex::new(Vec3::new(-0.5, 0.0,  0.5), color, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new( 0.5, 0.0,  0.5), color, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new( 0.5, 0.0, -0.5), color, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-0.5, 0.0, -0.5), color, Vec2::new(0.0, 1.0)),
    ]
}

fn create_floor_indices() -> Vec<u32> {
    vec![0, 1, 2, 2, 3, 0]
}

// =============================================================================
// Main Entry Point
// =============================================================================
fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = ADeadDemo::new();
    
    event_loop.run_app(&mut app).unwrap();
}
