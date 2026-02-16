// =============================================================================
// ReactorApp - Application Trait for REACTOR Framework
// =============================================================================
// This is the main entry point for building games with REACTOR.
// Users implement this trait and call `reactor::run()` to start.
//
// Architecture:
//   User Game (impl ReactorApp) → Reactor (engine) → VulkanContext (GPU)
//
// This gives users a clean, simple API while Reactor handles all Vulkan
// complexity internally.
// =============================================================================

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
    dpi::LogicalSize,
};

use crate::reactor::Reactor;
use crate::input::Input;
use crate::utils::time::Time;

use std::sync::Arc;

// =============================================================================
// ReactorConfig — Application configuration
// =============================================================================

/// Renderer backend selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RendererMode {
    Forward,
    Deferred,
    RayTracing,
}

impl Default for RendererMode {
    fn default() -> Self { Self::Forward }
}

/// Configuration for a REACTOR application.
///
/// # Rust (builder pattern)
/// ```rust,no_run
/// ReactorConfig::new("My Game")
///     .with_size(1920, 1080)
///     .with_vsync(true)
///     .with_renderer(RendererMode::RayTracing)
///     .with_scene("assets/level1.gltf")
/// ```
///
/// # C++ (designated initializers)
/// ```cpp
/// ReactorApp({
///     .title = "My Game",
///     .resolution = {1920, 1080},
///     .vsync = true,
///     .renderer = RayTracing,
///     .scene = "assets/level1.gltf"
/// });
/// ```
#[derive(Debug, Clone)]
pub struct ReactorConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub fullscreen: bool,
    pub resizable: bool,
    pub msaa_samples: u32,
    pub renderer: RendererMode,
    pub physics_hz: u32,
    pub scene: Option<String>,
}

impl ReactorConfig {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Default::default()
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn with_msaa(mut self, samples: u32) -> Self {
        self.msaa_samples = samples;
        self
    }

    pub fn with_renderer(mut self, mode: RendererMode) -> Self {
        self.renderer = mode;
        self
    }

    pub fn with_physics_hz(mut self, hz: u32) -> Self {
        self.physics_hz = hz;
        self
    }

    pub fn with_scene(mut self, path: &str) -> Self {
        self.scene = Some(path.to_string());
        self
    }
}

impl Default for ReactorConfig {
    fn default() -> Self {
        Self {
            title: "REACTOR Application".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
            fullscreen: false,
            resizable: true,
            msaa_samples: 4,
            renderer: RendererMode::default(),
            physics_hz: 60,
            scene: None,
        }
    }
}

// =============================================================================
// ReactorApp Trait — The "inheritance" point for users
// =============================================================================

/// Trait that users implement to create a REACTOR application.
///
/// # Example
/// ```rust,no_run
/// use reactor::app::{ReactorApp, ReactorContext, ReactorConfig};
///
/// struct MyGame {
///     rotation: f32,
/// }
///
/// impl ReactorApp for MyGame {
///     fn config(&self) -> ReactorConfig {
///         ReactorConfig::new("My Game")
///             .with_size(1280, 720)
///     }
///
///     fn init(&mut self, ctx: &mut ReactorContext) {
///         // Load meshes, materials, setup scene
///     }
///
///     fn update(&mut self, ctx: &mut ReactorContext) {
///         self.rotation += ctx.time.delta();
///     }
/// }
///
/// fn main() {
///     reactor::run(MyGame { rotation: 0.0 });
/// }
/// ```
pub trait ReactorApp {
    /// Return the configuration for this application.
    /// Called once before initialization.
    fn config(&self) -> ReactorConfig {
        ReactorConfig::default()
    }

    /// Called once after Vulkan and the window are initialized.
    /// Use this to load assets, create meshes, setup the scene.
    fn init(&mut self, ctx: &mut ReactorContext);

    /// Called every frame for game logic.
    /// Access input, time, and scene through the context.
    fn update(&mut self, ctx: &mut ReactorContext);

    /// Called every frame for rendering.
    /// Default: renders the built-in scene with the built-in camera.
    fn render(&mut self, ctx: &mut ReactorContext) {
        ctx.render_scene();
    }

    /// Called on fixed timestep for physics updates.
    /// Default: does nothing. Override for physics.
    fn fixed_update(&mut self, _ctx: &mut ReactorContext, _fixed_dt: f32) {}

    /// Called when the window is resized.
    fn on_resize(&mut self, _ctx: &mut ReactorContext, _width: u32, _height: u32) {}

    /// Called when the application is about to exit.
    fn on_exit(&mut self, _ctx: &mut ReactorContext) {}

    /// Called for custom event handling (advanced).
    /// Return true to consume the event, false to let Reactor handle it.
    fn on_event(&mut self, _ctx: &mut ReactorContext, _event: &WindowEvent) -> bool {
        false
    }
}

// =============================================================================
// ReactorContext — Everything the user needs in one place
// =============================================================================

/// Context passed to all ReactorApp callbacks.
/// Contains the ENTIRE engine — all systems inherited and ready to use.
///
/// This is the "inheritance" — ReactorContext inherits:
///   VulkanContext → Reactor → Camera + Scene + Lighting + Physics + Debug
pub struct ReactorContext {
    // Engine core
    pub reactor: Reactor,
    pub window: Arc<Window>,
    pub time: Time,
    pub config: ReactorConfig,

    // Game systems — ALL inherited and ready
    pub camera: crate::systems::camera::Camera,
    pub scene: crate::scene::Scene,
    pub lighting: crate::systems::lighting::LightingSystem,
    pub physics: crate::systems::physics::PhysicsWorld,
    pub culling: crate::systems::frustum::CullingSystem,
    pub debug: crate::graphics::debug_renderer::DebugRenderer,

    // Internal
    fixed_accumulator: f32,
}

impl Drop for ReactorContext {
    fn drop(&mut self) {
        // CRITICAL: Clear scene BEFORE reactor is dropped
        // This releases Arc references to Mesh/Material which contain Vulkan resources
        // that need the allocator (which is inside reactor) to be freed
        self.scene.clear();
        
        // Wait for GPU to finish before cleanup
        unsafe {
            let _ = self.reactor.context.device.device_wait_idle();
        }
    }
}

impl ReactorContext {
    // =========================================================================
    // Input
    // =========================================================================

    /// Get current input state
    pub fn input(&self) -> &Input {
        &self.reactor.input
    }

    // =========================================================================
    // Window
    // =========================================================================

    /// Get window aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        let size = self.window.inner_size();
        if size.height == 0 { return 1.0; }
        size.width as f32 / size.height as f32
    }

    /// Get window size
    pub fn window_size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    /// Set window title
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    // =========================================================================
    // Resource creation — delegate to Reactor
    // =========================================================================

    /// Create a mesh from vertices and indices
    pub fn create_mesh(&self, vertices: &[crate::vertex::Vertex], indices: &[u32]) -> Result<crate::mesh::Mesh, Box<dyn std::error::Error>> {
        self.reactor.create_mesh(vertices, indices)
    }

    /// Create a material from SPIR-V shader code
    pub fn create_material(&self, vert_code: &[u32], frag_code: &[u32]) -> Result<crate::material::Material, Box<dyn std::error::Error>> {
        self.reactor.create_material(vert_code, frag_code)
    }

    /// Load texture from file (PNG, JPG, BMP, etc.)
    pub fn load_texture(&self, path: &str) -> Result<crate::resources::texture::Texture, Box<dyn std::error::Error>> {
        self.reactor.load_texture(path)
    }

    /// Load texture from embedded bytes
    pub fn load_texture_bytes(&self, bytes: &[u8]) -> Result<crate::resources::texture::Texture, Box<dyn std::error::Error>> {
        self.reactor.load_texture_bytes(bytes)
    }

    /// Create a solid color texture
    pub fn create_solid_texture(&self, r: u8, g: u8, b: u8, a: u8) -> Result<crate::resources::texture::Texture, Box<dyn std::error::Error>> {
        self.reactor.create_solid_texture(r, g, b, a)
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Render the scene using the built-in camera
    pub fn render_scene(&mut self) {
        self.camera.set_aspect_ratio(
            self.window.inner_size().width as f32,
            self.window.inner_size().height as f32,
        );
        let vp = self.camera.view_projection_matrix();
        if let Err(e) = self.reactor.draw_scene(&self.scene, &vp) {
            eprintln!("REACTOR draw error: {}", e);
        }
    }

    /// Render the scene with a custom view-projection matrix
    pub fn draw_scene_with_vp(&mut self, view_projection: &glam::Mat4) {
        if let Err(e) = self.reactor.draw_scene(&self.scene, view_projection) {
            eprintln!("REACTOR draw error: {}", e);
        }
    }

    /// Render a custom scene (not the built-in one)
    pub fn draw_scene(&mut self, scene: &crate::scene::Scene, view_projection: &glam::Mat4) {
        if let Err(e) = self.reactor.draw_scene(scene, view_projection) {
            eprintln!("REACTOR draw error: {}", e);
        }
    }

    /// Render a single mesh with transform
    pub fn draw(&mut self, mesh: &crate::mesh::Mesh, material: &crate::material::Material, transform: &glam::Mat4) {
        if let Err(e) = self.reactor.draw_frame(mesh, material, transform) {
            eprintln!("REACTOR draw error: {}", e);
        }
    }

    // =========================================================================
    // Delta time shortcut
    // =========================================================================

    /// Get delta time in seconds (shortcut for ctx.time.delta())
    pub fn delta(&self) -> f32 {
        self.time.delta()
    }

    /// Get FPS (shortcut for ctx.time.fps())
    pub fn fps(&self) -> f32 {
        self.time.fps()
    }
}

// =============================================================================
// Internal Application Runner
// =============================================================================

struct AppRunner<A: ReactorApp> {
    app: A,
    context: Option<ReactorContext>,
}

impl<A: ReactorApp> ApplicationHandler for AppRunner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.is_some() { return; }

        let config = self.app.config();

        // Create window
        let window_attributes = Window::default_attributes()
            .with_title(&config.title)
            .with_inner_size(LogicalSize::new(config.width, config.height));

        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Initialize Reactor
        let reactor = match Reactor::init(&window) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to initialize Reactor: {}", e);
                event_loop.exit();
                return;
            }
        };

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              🚀 REACTOR Framework Initialized                ║");
        println!("║  Title: {:52} ║", config.title);
        println!("║  Resolution: {}x{:<44} ║",
            window.inner_size().width,
            format!("{}", window.inner_size().height));
        println!("║  MSAA: {:?}{:<49} ║",
            reactor.msaa_samples, "");
        println!("║  Ray Tracing: {:<47} ║",
            if reactor.ray_tracing.is_some() { "✅ Enabled" } else { "❌ Not available" });
        println!("╚══════════════════════════════════════════════════════════════╝");

        let aspect = window.inner_size().width as f32 / window.inner_size().height.max(1) as f32;
        let mut ctx = ReactorContext {
            reactor,
            window,
            time: Time::new(),
            config: config.clone(),
            camera: crate::systems::camera::Camera::perspective(60.0, aspect, 0.1, 1000.0),
            scene: crate::scene::Scene::new(),
            lighting: crate::systems::lighting::LightingSystem::new(),
            physics: crate::systems::physics::PhysicsWorld::new(),
            culling: crate::systems::frustum::CullingSystem::new(),
            debug: crate::graphics::debug_renderer::DebugRenderer::new(),
            fixed_accumulator: 0.0,
        };

        // Call user init
        self.app.init(&mut ctx);

        self.context = Some(ctx);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(ctx) = &mut self.context else { return };

        // Let Reactor handle input
        ctx.reactor.handle_event(&event);

        // Let user handle event first
        if self.app.on_event(ctx, &event) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.app.on_exit(ctx);
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.app.on_resize(ctx, size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                // Update time
                ctx.time.update();
                ctx.reactor.input.begin_frame();

                let dt = ctx.time.delta();

                // Fixed timestep for physics
                if ctx.config.physics_hz > 0 {
                    let fixed_dt = 1.0 / ctx.config.physics_hz as f32;
                    ctx.fixed_accumulator += dt;
                    while ctx.fixed_accumulator >= fixed_dt {
                        self.app.fixed_update(ctx, fixed_dt);
                        ctx.fixed_accumulator -= fixed_dt;
                    }
                }

                // Update game logic
                self.app.update(ctx);

                // Render
                self.app.render(ctx);

                // Request next frame
                ctx.window.request_redraw();
            }

            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ctx) = &mut self.context {
            self.app.on_exit(ctx);
            unsafe {
                ctx.reactor.context.device.device_wait_idle().unwrap();
            }
        }
    }
}

// =============================================================================
// Public API — The entry point
// =============================================================================

/// Run a REACTOR application. This is the main entry point.
///
/// # Example
/// ```rust,no_run
/// use reactor::app::{ReactorApp, ReactorContext, ReactorConfig};
///
/// struct MyGame;
///
/// impl ReactorApp for MyGame {
///     fn init(&mut self, _ctx: &mut ReactorContext) {}
///     fn update(&mut self, _ctx: &mut ReactorContext) {}
/// }
///
/// fn main() {
///     reactor::run(MyGame);
/// }
/// ```
pub fn run<A: ReactorApp + 'static>(app: A) {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = AppRunner {
        app,
        context: None,
    };

    event_loop.run_app(&mut runner).expect("Event loop error");
}
