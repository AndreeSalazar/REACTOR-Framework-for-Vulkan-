// =============================================================================
// REACTOR Elite Sandbox - Demostración Completa del Framework
// =============================================================================
// Este ejemplo demuestra TODAS las características élite de REACTOR:
// - Sistema de Iluminación (Directional, Point, Spot)
// - Sistema de Partículas (Fuego, Humo, Explosiones)
// - Animaciones y Tweens con Easing
// - Física básica (RigidBody, Raycasting)
// - Frustum Culling
// - Debug Renderer
// - Cámara FPS con controles
// - Primitivas procedurales
// - Sistema de Tiempo
// =============================================================================

use reactor::{
    Reactor, Vertex, Scene,
    LightingSystem, Light, LightType,
    ParticleSystem, 
    Tween, EasingFunction,
    Camera,
    AABB, Ray, PhysicsWorld,
    CullingSystem,
    DebugRenderer,
    Time,
    CPUDetector, ResolutionDetector,
};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
    keyboard::KeyCode,
    dpi::LogicalSize,
};
use std::sync::Arc;
use glam::{Vec3, Vec4, Mat4, Vec2};

// =============================================================================
// Configuration
// =============================================================================
const TARGET_WIDTH: f32 = 1280.0;
const TARGET_HEIGHT: f32 = 720.0;
const CAMERA_SPEED: f32 = 5.0;
const _MOUSE_SENSITIVITY: f32 = 0.003;

// =============================================================================
// Application State
// =============================================================================
struct EliteApp {
    state: Option<AppState>,
    
    // Camera
    camera: Camera,
    camera_yaw: f32,
    camera_pitch: f32,
    
    // Time
    time: Time,
    
    // Systems
    lighting: LightingSystem,
    particles: Vec<ParticleSystem>,
    physics: PhysicsWorld,
    culling: CullingSystem,
    debug: DebugRenderer,
    
    // Animation
    cube_tween: Option<Tween<f32>>,
    light_orbit_angle: f32,
    
    // Demo state
    show_debug: bool,
    paused: bool,
    selected_object: Option<usize>,
}

struct AppState {
    scene: Scene,
    reactor: Reactor,
    window: Arc<Window>,
    object_bounds: Vec<AABB>, // Bounds for culling/picking
    base_object_count: usize, // Number of static objects (before particles)
    particle_mesh: Arc<reactor::Mesh>,
    particle_material: Arc<reactor::Material>,
}

// =============================================================================
// Application Implementation
// =============================================================================
impl ApplicationHandler for EliteApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() { return; }

        // Hardware Detection
        let cpu_info = CPUDetector::detect();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║           REACTOR Elite Sandbox - Vulkan Engine              ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║ CPU Cores: {:2} | {}                              ║", 
            cpu_info.logical_cores, cpu_info.recommendation);

        // Smart Resolution
        let (width, height) = ResolutionDetector::get_smart_resolution(
            event_loop, TARGET_WIDTH, TARGET_HEIGHT
        );

        let window_attributes = Window::default_attributes()
            .with_title("🚀 REACTOR Elite Sandbox")
            .with_inner_size(LogicalSize::new(width, height));
            
        let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));
        let reactor = Reactor::init(&window).expect("Failed to initialize Reactor");
        
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║ Controls:                                                    ║");
        println!("║   WASD      - Move camera                                    ║");
        println!("║   Mouse     - Look around (hold right click)                 ║");
        println!("║   Space/C   - Move up/down                                   ║");
        println!("║   F         - Toggle debug rendering                         ║");
        println!("║   P         - Pause/Resume                                   ║");
        println!("║   1-3       - Spawn particles (Fire/Smoke/Explosion)         ║");
        println!("║   Click     - Select object (raycast)                        ║");
        println!("║   ESC       - Exit                                           ║");
        println!("╚══════════════════════════════════════════════════════════════╝");

        // =================================================================
        // Create Scene with Primitives
        // =================================================================
        let mut scene = Scene::new();
        let mut object_bounds = Vec::new();

        // Load Shaders
        let vert_code = include_bytes!("../shaders/vert.spv");
        let frag_code = include_bytes!("../shaders/frag.spv");
        let vert_decoded = ash::util::read_spv(&mut std::io::Cursor::new(&vert_code[..])).unwrap();
        let frag_decoded = ash::util::read_spv(&mut std::io::Cursor::new(&frag_code[..])).unwrap();
        let material = Arc::new(reactor.create_material(&vert_decoded, &frag_decoded).unwrap());

        // 1. Floor (Plane) - Using legacy Vertex type
        let floor_verts = create_floor_vertices();
        let floor_indices = create_floor_indices();
        let floor_mesh = Arc::new(reactor.create_mesh(&floor_verts, &floor_indices).unwrap());
        scene.add_object(
            floor_mesh,
            material.clone(),
            Mat4::from_scale_rotation_translation(
                Vec3::new(20.0, 1.0, 20.0),
                glam::Quat::IDENTITY,
                Vec3::new(0.0, -1.0, 0.0)
            )
        );
        object_bounds.push(AABB::from_center_size(Vec3::new(0.0, -1.0, 0.0), Vec3::new(20.0, 0.1, 20.0)));

        // 2. Central Cube (Animated)
        let (cube_verts, cube_indices) = create_cube();
        let cube_mesh = Arc::new(reactor.create_mesh(&cube_verts, &cube_indices).unwrap());
        scene.add_object(cube_mesh.clone(), material.clone(), Mat4::IDENTITY);
        object_bounds.push(AABB::from_center_size(Vec3::ZERO, Vec3::ONE));

        // 3. Sphere (using cube for now - sphere needs proper vertex format)
        let sphere_mesh = cube_mesh.clone();
        scene.add_object(
            sphere_mesh.clone(),
            material.clone(),
            Mat4::from_translation(Vec3::new(3.0, 0.0, 0.0))
        );
        object_bounds.push(AABB::from_center_size(Vec3::new(3.0, 0.0, 0.0), Vec3::ONE));

        // 4. Cylinder (using cube for demo)
        let cyl_mesh = cube_mesh.clone();
        scene.add_object(
            cyl_mesh,
            material.clone(),
            Mat4::from_translation(Vec3::new(-3.0, 0.0, 0.0))
        );
        object_bounds.push(AABB::from_center_size(Vec3::new(-3.0, 0.0, 0.0), Vec3::new(1.0, 2.0, 1.0)));

        // 5. Torus (using cube for demo)
        let torus_mesh = cube_mesh.clone();
        scene.add_object(
            torus_mesh,
            material.clone(),
            Mat4::from_translation(Vec3::new(0.0, 0.0, 3.0))
        );
        object_bounds.push(AABB::from_center_size(Vec3::new(0.0, 0.0, 3.0), Vec3::new(2.6, 0.6, 2.6)));

        // 6. Cone (using cube for demo)
        let cone_mesh = cube_mesh.clone();
        scene.add_object(
            cone_mesh,
            material.clone(),
            Mat4::from_translation(Vec3::new(0.0, 0.0, -3.0))
        );
        object_bounds.push(AABB::from_center_size(Vec3::new(0.0, 0.0, -3.0), Vec3::new(1.0, 1.5, 1.0)));

        // 7-12. Grid of smaller cubes for culling demo
        for x in -2..=2 {
            for z in -2..=2 {
                if x == 0 && z == 0 { continue; } // Skip center
                let pos = Vec3::new(x as f32 * 2.0, 0.5, z as f32 * 2.0 + 8.0);
                scene.add_object(
                    cube_mesh.clone(),
                    material.clone(),
                    Mat4::from_scale_rotation_translation(
                        Vec3::splat(0.3),
                        glam::Quat::IDENTITY,
                        pos
                    )
                );
                object_bounds.push(AABB::from_center_size(pos, Vec3::splat(0.3)));
            }
        }

        // =================================================================
        // Setup Lighting
        // =================================================================
        self.lighting = LightingSystem::new();
        self.lighting.set_ambient(Vec3::new(0.1, 0.1, 0.15), 1.0);
        
        // Sun (Directional)
        self.lighting.add_light(Light::directional(
            Vec3::new(-0.5, -1.0, -0.3),
            Vec3::new(1.0, 0.95, 0.8),
            0.8
        ));

        // Orbiting point light
        self.lighting.add_light(Light::point(
            Vec3::new(2.0, 2.0, 0.0),
            Vec3::new(1.0, 0.5, 0.0),
            2.0,
            10.0
        ));

        // Blue accent light
        self.lighting.add_light(Light::point(
            Vec3::new(-2.0, 1.0, 2.0),
            Vec3::new(0.2, 0.4, 1.0),
            1.5,
            8.0
        ));

        // =================================================================
        // Setup Camera
        // =================================================================
        self.camera = Camera::perspective(60.0, width as f32 / height as f32, 0.1, 100.0);
        self.camera.position = Vec3::new(0.0, 2.0, 8.0);
        self.camera_yaw = 0.0;
        self.camera_pitch = -0.2;

        // =================================================================
        // Setup Animation
        // =================================================================
        self.cube_tween = Some(Tween::new(0.0, std::f32::consts::TAU, 4.0)
            .with_easing(EasingFunction::EaseInOutCubic));

        // Store base object count (before dynamic particle objects)
        let base_object_count = scene.objects.len();

        // Create small particle mesh (tiny cube)
        let particle_verts = create_particle_vertices();
        let particle_indices = create_particle_indices();
        let particle_mesh = Arc::new(reactor.create_mesh(&particle_verts, &particle_indices).unwrap());

        self.state = Some(AppState {
            scene,
            reactor,
            window,
            object_bounds,
            base_object_count,
            particle_mesh,
            particle_material: material,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else { return };
        
        // Process input
        state.reactor.handle_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            
            WindowEvent::RedrawRequested => {
                // Update time
                self.time.update();
                let dt = self.time.delta();

                if !self.paused {
                    // ==========================================================
                    // Update Camera (FPS Style)
                    // ==========================================================
                    let input = &state.reactor.input;
                    let move_speed = CAMERA_SPEED * dt;

                    // Mouse look (hold right mouse button for free look like Blender)
                    if input.is_mouse_down(winit::event::MouseButton::Right) {
                        let mouse_delta = input.mouse_delta();
                        let sensitivity = 0.005;
                        self.camera_yaw -= mouse_delta.x * sensitivity;
                        self.camera_pitch -= mouse_delta.y * sensitivity;
                        // Clamp pitch to avoid gimbal lock
                        self.camera_pitch = self.camera_pitch.clamp(-1.5, 1.5);
                        self.camera.set_rotation(self.camera_yaw, self.camera_pitch);
                    }

                    // Movement
                    if input.is_key_down(KeyCode::KeyW) {
                        self.camera.move_forward(move_speed);
                    }
                    if input.is_key_down(KeyCode::KeyS) {
                        self.camera.move_forward(-move_speed);
                    }
                    if input.is_key_down(KeyCode::KeyA) {
                        self.camera.move_right(-move_speed);
                    }
                    if input.is_key_down(KeyCode::KeyD) {
                        self.camera.move_right(move_speed);
                    }
                    if input.is_key_down(KeyCode::Space) {
                        self.camera.move_up(move_speed);
                    }
                    if input.is_key_down(KeyCode::KeyC) {
                        self.camera.move_up(-move_speed);
                    }

                    // Scroll wheel for zoom
                    let scroll = input.scroll_delta();
                    if scroll.abs() > 0.0 {
                        self.camera.move_forward(scroll * 0.5);
                    }

                    // ==========================================================
                    // Update Animations
                    // ==========================================================
                    
                    // Cube rotation with tween
                    if let Some(tween) = &mut self.cube_tween {
                        let rotation = tween.update(dt);
                        if tween.is_finished() {
                            tween.reset();
                        }
                        
                        let transform = Mat4::from_rotation_y(rotation) 
                            * Mat4::from_rotation_x(rotation * 0.5);
                        state.scene.objects[1].transform = transform;
                    }

                    // Orbiting light
                    self.light_orbit_angle += dt * 0.5;
                    if let Some(light) = self.lighting.get_light_mut(1) {
                        light.position = Vec3::new(
                            self.light_orbit_angle.cos() * 4.0,
                            2.0 + (self.light_orbit_angle * 2.0).sin(),
                            self.light_orbit_angle.sin() * 4.0
                        );
                    }

                    // ==========================================================
                    // Update Particles
                    // ==========================================================
                    for particle_system in &mut self.particles {
                        particle_system.update(dt);
                    }
                    // Remove finished particle systems
                    self.particles.retain(|p| !p.is_finished());

                    // ==========================================================
                    // Render Particles as Scene Objects
                    // ==========================================================
                    // Remove old particle objects (keep only base objects)
                    state.scene.objects.truncate(state.base_object_count);
                    state.object_bounds.truncate(state.base_object_count);

                    // Add particle objects to scene
                    for particle_system in &self.particles {
                        for particle in particle_system.particles() {
                            if particle.alive {
                                let scale = particle.size * 0.3;
                                let transform = Mat4::from_scale_rotation_translation(
                                    Vec3::splat(scale),
                                    glam::Quat::IDENTITY,
                                    particle.position
                                );
                                state.scene.add_object(
                                    state.particle_mesh.clone(),
                                    state.particle_material.clone(),
                                    transform
                                );
                            }
                        }
                    }

                    // ==========================================================
                    // Update Physics
                    // ==========================================================
                    let _physics_steps = self.physics.step(dt);

                    // ==========================================================
                    // Frustum Culling
                    // ==========================================================
                    let vp = self.camera.view_projection_matrix();
                    self.culling.update_frustum(vp);
                    
                    for (i, bounds) in state.object_bounds.iter().enumerate() {
                        state.scene.objects[i].visible = self.culling.is_visible_aabb(bounds);
                    }
                }

                // ==========================================================
                // Debug Rendering
                // ==========================================================
                self.debug.clear();
                
                // Always show selected object highlight (green outline)
                if let Some(idx) = self.selected_object {
                    if idx < state.object_bounds.len() {
                        let bounds = &state.object_bounds[idx];
                        self.debug.aabb(
                            &reactor::DebugAABB { min: bounds.min, max: bounds.max },
                            Vec4::new(0.0, 1.0, 0.0, 1.0)
                        );
                    }
                }

                // Draw particle positions as colored points
                for particle_system in &self.particles {
                    for particle in particle_system.particles() {
                        // Draw small cross at particle position
                        let size = particle.size * 0.1;
                        self.debug.line(
                            particle.position - Vec3::X * size,
                            particle.position + Vec3::X * size,
                            particle.color
                        );
                        self.debug.line(
                            particle.position - Vec3::Y * size,
                            particle.position + Vec3::Y * size,
                            particle.color
                        );
                    }
                }

                if self.show_debug {
                    // Draw grid
                    self.debug.grid(Vec3::new(0.0, -1.0, 0.0), 20.0, 20, Vec4::new(0.3, 0.3, 0.3, 1.0));
                    
                    // Draw world axes
                    self.debug.axes(Vec3::ZERO, 2.0);
                    
                    // Draw light positions
                    for light in &self.lighting.lights {
                        if light.enabled {
                            let color = Vec4::new(light.color.x, light.color.y, light.color.z, 1.0);
                            match light.light_type {
                                LightType::Point => {
                                    self.debug.sphere(
                                        &reactor::DebugSphere { center: light.position, radius: 0.1 },
                                        color,
                                        8
                                    );
                                }
                                LightType::Directional => {
                                    self.debug.line(Vec3::ZERO, -light.direction * 3.0, color);
                                }
                                LightType::Spot => {
                                    self.debug.line(light.position, light.position + light.direction * 2.0, color);
                                }
                            }
                        }
                    }

                    // Draw camera info
                    let cam_info = format!(
                        "Camera: ({:.1}, {:.1}, {:.1}) Yaw: {:.1}° Pitch: {:.1}°",
                        self.camera.position.x, self.camera.position.y, self.camera.position.z,
                        self.camera_yaw.to_degrees(), self.camera_pitch.to_degrees()
                    );
                    println!("\r{}", cam_info);
                }

                // ==========================================================
                // Render
                // ==========================================================
                let vp = self.camera.view_projection_matrix();
                
                if let Err(e) = state.reactor.draw_scene(&state.scene, &vp) {
                    eprintln!("Draw error: {}", e);
                }

                // Update window title with stats
                let title = format!(
                    "🚀 REACTOR Elite | FPS: {:.0} | Objects: {}/{} | Particles: {} | Lights: {}",
                    self.time.fps(),
                    self.culling.visible_count(),
                    state.scene.objects.len(),
                    self.particles.iter().map(|p| p.alive_count()).sum::<usize>(),
                    self.lighting.light_count()
                );
                state.window.set_title(&title);
                
                state.window.request_redraw();
            }

            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if key_event.state == winit::event::ElementState::Pressed {
                    match key_event.physical_key {
                        winit::keyboard::PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                        winit::keyboard::PhysicalKey::Code(KeyCode::KeyF) => {
                            self.show_debug = !self.show_debug;
                            println!("Debug rendering: {}", if self.show_debug { "ON" } else { "OFF" });
                        }
                        winit::keyboard::PhysicalKey::Code(KeyCode::KeyP) => {
                            self.paused = !self.paused;
                            println!("{}", if self.paused { "PAUSED" } else { "RESUMED" });
                        }
                        winit::keyboard::PhysicalKey::Code(KeyCode::Digit1) => {
                            let spawn_pos = self.camera.position + self.camera.forward() * 3.0;
                            let mut fire = ParticleSystem::fire();
                            fire.position = spawn_pos;
                            self.particles.push(fire);
                            println!("🔥 FIRE spawned at ({:.1}, {:.1}, {:.1}) - {} total particle systems", 
                                spawn_pos.x, spawn_pos.y, spawn_pos.z, self.particles.len());
                        }
                        winit::keyboard::PhysicalKey::Code(KeyCode::Digit2) => {
                            let spawn_pos = self.camera.position + self.camera.forward() * 3.0;
                            let mut smoke = ParticleSystem::smoke();
                            smoke.position = spawn_pos;
                            self.particles.push(smoke);
                            println!("💨 SMOKE spawned at ({:.1}, {:.1}, {:.1}) - {} total particle systems", 
                                spawn_pos.x, spawn_pos.y, spawn_pos.z, self.particles.len());
                        }
                        winit::keyboard::PhysicalKey::Code(KeyCode::Digit3) => {
                            let spawn_pos = self.camera.position + self.camera.forward() * 4.0;
                            let mut explosion = ParticleSystem::explosion();
                            explosion.position = spawn_pos;
                            explosion.play();
                            self.particles.push(explosion);
                            println!("💥 EXPLOSION spawned at ({:.1}, {:.1}, {:.1}) - {} total particle systems", 
                                spawn_pos.x, spawn_pos.y, spawn_pos.z, self.particles.len());
                        }
                        _ => {}
                    }
                }
            }

            WindowEvent::MouseInput { state: button_state, button, .. } => {
                if button == winit::event::MouseButton::Left 
                    && button_state == winit::event::ElementState::Pressed 
                {
                    // Raycast for object selection
                    let window_size = state.window.inner_size();
                    let mouse_pos = state.reactor.input.mouse_position();
                    
                    let inv_vp = self.camera.view_projection_matrix().inverse();
                    let ray = Ray::from_screen(
                        mouse_pos.x,
                        mouse_pos.y,
                        window_size.width as f32,
                        window_size.height as f32,
                        inv_vp
                    );

                    // Find closest hit
                    let mut closest_hit: Option<(usize, f32)> = None;
                    for (i, bounds) in state.object_bounds.iter().enumerate() {
                        if let Some(t) = ray.intersects_aabb(bounds) {
                            if closest_hit.is_none() || t < closest_hit.unwrap().1 {
                                closest_hit = Some((i, t));
                            }
                        }
                    }

                    if let Some((idx, t)) = closest_hit {
                        // Deselect previous object (reset scale)
                        if let Some(prev_idx) = self.selected_object {
                            if prev_idx < state.scene.objects.len() && prev_idx != idx {
                                // Reset previous object transform (remove scale boost)
                                let current = state.scene.objects[prev_idx].transform;
                                state.scene.objects[prev_idx].transform = 
                                    Mat4::from_scale(Vec3::splat(1.0 / 1.2)) * current;
                            }
                        }
                        
                        self.selected_object = Some(idx);
                        let hit_point = ray.point_at(t);
                        
                        // Scale up selected object to show selection
                        let current = state.scene.objects[idx].transform;
                        state.scene.objects[idx].transform = 
                            Mat4::from_scale(Vec3::splat(1.2)) * current;
                        
                        let name = state.scene.objects[idx].name.as_deref().unwrap_or("unnamed");
                        println!("✓ Selected object {} ({}) at ({:.2}, {:.2}, {:.2})", 
                            idx, name, hit_point.x, hit_point.y, hit_point.z);
                    } else {
                        // Deselect current object
                        if let Some(prev_idx) = self.selected_object {
                            if prev_idx < state.scene.objects.len() {
                                let current = state.scene.objects[prev_idx].transform;
                                state.scene.objects[prev_idx].transform = 
                                    Mat4::from_scale(Vec3::splat(1.0 / 1.2)) * current;
                            }
                        }
                        self.selected_object = None;
                        println!("✗ No object selected");
                    }
                }
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.camera.set_aspect_ratio(size.width as f32, size.height as f32);
                }
            }

            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║                    Shutting down REACTOR...                  ║");
            println!("╚══════════════════════════════════════════════════════════════╝");
            unsafe {
                state.reactor.context.device.device_wait_idle().unwrap();
            }
        }
    }
}

// =============================================================================
// Main Entry Point
// =============================================================================
fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = EliteApp {
        state: None,
        camera: Camera::new(),
        camera_yaw: 0.0,
        camera_pitch: 0.0,
        time: Time::new(),
        lighting: LightingSystem::new(),
        particles: Vec::new(),
        physics: PhysicsWorld::new(),
        culling: CullingSystem::new(),
        debug: DebugRenderer::new(),
        cube_tween: None,
        light_orbit_angle: 0.0,
        show_debug: false,
        paused: false,
        selected_object: None,
    };
    
    event_loop.run_app(&mut app).unwrap();
}

// =============================================================================
// Helper Functions - Create Meshes with Legacy Vertex Format
// =============================================================================

fn create_cube() -> (Vec<Vertex>, Vec<u32>) {
    let vertices = vec![
        // Front face (Z+)
        Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::ZERO),
        Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::ZERO),
        Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(1.0, 1.0, 0.0), Vec2::ZERO),
        // Back face (Z-)
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(1.0, 0.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(0.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(0.5, 0.5, 0.5), Vec2::ZERO),
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
    let color = Vec3::new(0.3, 0.4, 0.3);
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

fn create_particle_vertices() -> Vec<Vertex> {
    // Small bright particle cube
    let color = Vec3::new(1.0, 0.8, 0.2); // Orange/yellow
    vec![
        // Front face
        Vertex::new(Vec3::new(-0.05, -0.05,  0.05), color, Vec2::ZERO),
        Vertex::new(Vec3::new( 0.05, -0.05,  0.05), color, Vec2::ZERO),
        Vertex::new(Vec3::new( 0.05,  0.05,  0.05), color, Vec2::ZERO),
        Vertex::new(Vec3::new(-0.05,  0.05,  0.05), color, Vec2::ZERO),
        // Back face
        Vertex::new(Vec3::new(-0.05, -0.05, -0.05), color, Vec2::ZERO),
        Vertex::new(Vec3::new( 0.05, -0.05, -0.05), color, Vec2::ZERO),
        Vertex::new(Vec3::new( 0.05,  0.05, -0.05), color, Vec2::ZERO),
        Vertex::new(Vec3::new(-0.05,  0.05, -0.05), color, Vec2::ZERO),
    ]
}

fn create_particle_indices() -> Vec<u32> {
    vec![
        0, 1, 2, 2, 3, 0, // Front
        1, 5, 6, 6, 2, 1, // Right
        5, 4, 7, 7, 6, 5, // Back
        4, 0, 3, 3, 7, 4, // Left
        3, 2, 6, 6, 7, 3, // Top
        4, 5, 1, 1, 0, 4, // Bottom
    ]
}
