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
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::platform::input::Input;
use crate::platform::time::Time;
use crate::reactor::Reactor;
use crate::resources::{
    AssetDatabase, AssetHotReloadManager, AssetId, AssetLoaderQueue, AssetManager, GltfLoader,
    Handle,
};

use std::sync::Arc;

// =============================================================================
// ReactorConfig — Application configuration
// =============================================================================

/// Renderer backend selection.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum RendererMode {
    #[default]
    Forward,
    Deferred,
    RayTracing,
}

/// Configuration for a REACTOR application.
///
/// # Rust (builder pattern)
/// ```rust,no_run
/// # use reactor_vulkan::prelude::*;
/// let _config = ReactorConfig::new("My Game")
///     .with_size(1920, 1080)
///     .with_vsync(true)
///     .with_renderer(RendererMode::RayTracing)
///     .with_scene("assets/level1.gltf");
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
    pub maximized: bool,
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

    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
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
            maximized: false,
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
/// use reactor_vulkan::prelude::*;
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
///     reactor_vulkan::run(MyGame { rotation: 0.0 });
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

// =============================================================================
// 🧊 Tipos para spawning inteligente de modelos Blender → REACTOR
// =============================================================================

/// Dimensiones nativas (sin re-escalar) de un modelo glTF.
///
/// Devuelto por [`ReactorContext::gltf_bounds`]. Útil para que el juego
/// decida tamaños sin tener que calcular AABB manualmente.
#[derive(Clone, Copy, Debug)]
pub struct GltfBounds {
    /// Esquina mínima del bounding box (XYZ).
    pub min: glam::Vec3,
    /// Esquina máxima del bounding box (XYZ).
    pub max: glam::Vec3,
    /// Centro geométrico del bounding box.
    pub center: glam::Vec3,
    /// Tamaño en cada eje (`max − min`).
    pub size: glam::Vec3,
    /// Altura total (`max.y − min.y`). Atajo común.
    pub height: f32,
}

/// Opciones declarativas para [`ReactorContext::spawn_gltf_smart`].
///
/// Builder estilo Bevy: empezar con `GltfSpawn::at(pos)` y encadenar.
#[derive(Clone, Copy, Debug)]
pub struct GltfSpawn {
    /// Posición donde colocar el modelo (los pies, no el pivot).
    pub position: glam::Vec3,
    /// Si `Some(h)`, re-escala el modelo para que su altura final sea `h` metros.
    pub target_height: Option<f32>,
    /// Si `Some(dir)`, rota el modelo en torno a Y para que su frente apunte a `dir`.
    pub face_direction: Option<glam::Vec3>,
    /// Si `true`, coloca los pies del modelo en `position.y` (en vez del pivot).
    pub feet_at_position: bool,
}

impl Default for GltfSpawn {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            target_height: None,
            face_direction: None,
            feet_at_position: true,
        }
    }
}

impl GltfSpawn {
    /// Empezar un builder con la posición dada (pies del modelo).
    pub fn at(position: glam::Vec3) -> Self {
        Self { position, ..Default::default() }
    }

    /// Auto-escalar el modelo para que su altura final sea `meters` metros.
    pub fn with_height(mut self, meters: f32) -> Self {
        self.target_height = Some(meters);
        self
    }

    /// Auto-orientar el modelo para que su frente apunte hacia `dir`.
    pub fn facing(mut self, dir: glam::Vec3) -> Self {
        self.face_direction = Some(dir);
        self
    }

    /// Si `false`, el pivot del modelo (no sus pies) va en `position`.
    pub fn with_pivot_at_position(mut self, on: bool) -> Self {
        self.feet_at_position = !on;
        self
    }
}

/// Información devuelta tras un [`ReactorContext::spawn_gltf_smart`].
///
/// Incluye los índices de los objetos en la escena (uno por mesh-node del glTF),
/// más metadatos útiles para que el juego ajuste hit-boxes o lógica.
#[derive(Clone, Debug)]
pub struct ModelSpawnInfo {
    /// Índices `usize` en `ctx.scene` — uno por mesh-node del glTF.
    pub indices: Vec<usize>,
    /// Factor de escala aplicado para alcanzar la altura objetivo.
    pub applied_scale: f32,
    /// Rotación aplicada para apuntar hacia `face_direction`.
    pub applied_rotation: glam::Quat,
    /// Altura nativa del modelo (antes de escalar).
    pub native_height: f32,
    /// Altura final tras escalar (debe coincidir con `target_height` si se dio).
    pub world_height: f32,
    /// Esquina mínima del AABB en coordenadas de mundo.
    pub world_bounds_min: glam::Vec3,
    /// Esquina máxima del AABB en coordenadas de mundo.
    pub world_bounds_max: glam::Vec3,
}

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
    pub camera: crate::scene::camera::Camera,
    pub scene: crate::systems::scene::Scene,
    pub lighting: crate::systems::lighting::LightingSystem,
    pub physics: crate::systems::physics::PhysicsWorld,
    pub culling: crate::systems::frustum::CullingSystem,
    pub debug: crate::graphics::debug_renderer::DebugRenderer,

    // 🎨 Asset Pipeline (Fase 3)
    pub asset_manager: AssetManager,
    pub gltf_loader: GltfLoader,
    pub asset_db: AssetDatabase,
    pub asset_hot_reload: Option<AssetHotReloadManager>,
    pub asset_loader_queue: AssetLoaderQueue,
    pub audio: crate::systems::audio::AudioSystem,
    pub event_bus: crate::systems::event_bus::EventBus,
    pub(crate) hot_reload_rx: Option<
        tokio::sync::mpsc::UnboundedReceiver<crate::resources::asset_hot_reload::AssetReloadEvent>,
    >,

    // 🌑 Blob shadows (Fase 4.3 — fallback HOTD-style sin shadow maps GPU)
    pub(crate) blob_shadow_mesh: Option<std::sync::Arc<crate::resources::mesh::Mesh>>,
    pub(crate) blob_shadow_material: Option<std::sync::Arc<crate::resources::material::Material>>,

    // Internal
    fixed_accumulator: f32,
}

impl Drop for ReactorContext {
    fn drop(&mut self) {
        // CRITICAL: Clear scene BEFORE reactor is dropped
        // This releases Arc references to Mesh/Material which contain Vulkan resources
        // that need the allocator (which is inside reactor) to be freed
        self.scene.clear();
        self.blob_shadow_mesh = None;
        self.blob_shadow_material = None;
        self.asset_manager.clear();

        // SAFETY: device_wait_idle() blocks until all GPU operations complete.
        // This is safe to call at any time and has no aliasing requirements.
        // We call it here to ensure all GPU operations complete before
        // Vulkan resources are dropped by the Reactor destructor.
        // The device handle is still valid at this point (Drop hasn't run yet).
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
        if size.height == 0 {
            return 1.0;
        }
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
    pub fn create_mesh(
        &self,
        vertices: &[crate::resources::vertex::Vertex],
        indices: &[u32],
    ) -> crate::core::error::ReactorResult<crate::resources::mesh::Mesh> {
        self.reactor
            .create_mesh(vertices, indices)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    /// Create a material from SPIR-V shader code
    pub fn create_material(
        &self,
        vert_code: &[u32],
        frag_code: &[u32],
    ) -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        self.reactor
            .create_material(vert_code, frag_code)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    /// Load texture from file (PNG, JPG, BMP, etc.)
    pub fn load_texture(
        &self,
        path: &str,
    ) -> crate::core::error::ReactorResult<crate::resources::texture::Texture> {
        self.reactor
            .load_texture(path)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    /// Load texture from embedded bytes
    pub fn load_texture_bytes(
        &self,
        bytes: &[u8],
    ) -> crate::core::error::ReactorResult<crate::resources::texture::Texture> {
        self.reactor
            .load_texture_bytes(bytes)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    /// Create a solid color texture
    pub fn create_solid_texture(
        &self,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> crate::core::error::ReactorResult<crate::resources::texture::Texture> {
        self.reactor
            .create_solid_texture(r, g, b, a)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    /// Create a textured material with a diffuse texture
    pub fn create_textured_material(
        &self,
        vert_code: &[u32],
        frag_code: &[u32],
        texture: &crate::resources::texture::Texture,
    ) -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        self.reactor
            .create_textured_material(vert_code, frag_code, texture)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    // =========================================================================
    // Model Loading (OBJ)
    // =========================================================================

    /// Load an OBJ file and return the mesh
    pub fn load_obj(
        &self,
        path: &str,
    ) -> crate::core::error::ReactorResult<crate::resources::mesh::Mesh> {
        use crate::resources::model::ObjData;

        let obj = ObjData::load(path)
            .map_err(|_e| crate::core::error::ReactorError::file_not_found(path))?;
        if obj.vertices.is_empty() {
            return Err(crate::core::error::ReactorError::invalid_format(
                "OBJ file contains no vertices",
            ));
        }

        println!(
            "📦 Loaded OBJ: {} vertices, {} triangles",
            obj.vertex_count(),
            obj.triangle_count()
        );

        self.reactor
            .create_mesh(&obj.vertices, &obj.indices)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    /// Load an OBJ file and create a mesh with material, returning a scene object index
    pub fn load_obj_with_material(
        &mut self,
        path: &str,
        material: std::sync::Arc<crate::resources::material::Material>,
    ) -> crate::core::error::ReactorResult<u32> {
        let mesh = self.load_obj(path)?;
        let mesh_arc = std::sync::Arc::new(mesh);
        let index = self.scene.objects.len() as u32;
        self.scene
            .add_object(mesh_arc, material, glam::Mat4::IDENTITY);
        Ok(index)
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
    pub fn draw_scene(
        &mut self,
        scene: &crate::systems::scene::Scene,
        view_projection: &glam::Mat4,
    ) {
        if let Err(e) = self.reactor.draw_scene(scene, view_projection) {
            eprintln!("REACTOR draw error: {}", e);
        }
    }

    /// Render a single mesh with transform
    pub fn draw(
        &mut self,
        mesh: &crate::resources::mesh::Mesh,
        material: &crate::resources::material::Material,
        transform: &glam::Mat4,
    ) {
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

    /// Tiempo total transcurrido desde el arranque (shortcut for ctx.time.elapsed()).
    pub fn elapsed(&self) -> f32 {
        self.time.elapsed()
    }

    // =========================================================================
    // 🎥 Camera shortcuts — facilitan el uso sin tocar self.camera
    // =========================================================================

    /// Coloca la cámara en `eye` y la apunta a `target`. Forma corta.
    ///
    /// ```rust,no_run
    /// # use reactor_vulkan::prelude::*;
    /// # fn demo(ctx: &mut reactor_vulkan::app::ReactorContext) {
    /// ctx.look_at(Vec3::new(0.0, 2.0, 5.0), Vec3::ZERO);
    /// # }
    /// ```
    pub fn look_at(&mut self, eye: glam::Vec3, target: glam::Vec3) -> &mut Self {
        self.camera.aim_at(eye, target);
        self
    }

    /// Mueve la cámara sin cambiar su orientación.
    pub fn move_camera_to(&mut self, position: glam::Vec3) -> &mut Self {
        self.camera.position = position;
        self
    }

    // =========================================================================
    // 💡 Lighting shortcuts
    // =========================================================================

    /// Añade un sol direccional con valores por defecto agradables.
    ///
    /// ```rust,no_run
    /// # fn demo(ctx: &mut reactor_vulkan::app::ReactorContext) {
    /// ctx.add_sun();
    /// # }
    /// ```
    pub fn add_sun(&mut self) -> usize {
        self.lighting
            .add_light(crate::systems::lighting::Light::sun())
    }

    /// Añade una luz direccional personalizada.
    pub fn add_directional_light(
        &mut self,
        direction: glam::Vec3,
        color: glam::Vec3,
        intensity: f32,
    ) -> usize {
        self.lighting
            .add_light(crate::systems::lighting::Light::directional(
                direction, color, intensity,
            ))
    }

    /// Añade una luz puntual.
    pub fn add_point_light(
        &mut self,
        position: glam::Vec3,
        color: glam::Vec3,
        intensity: f32,
        range: f32,
    ) -> usize {
        self.lighting
            .add_light(crate::systems::lighting::Light::point(
                position, color, intensity, range,
            ))
    }

    /// Añade un foco (spotlight).
    pub fn add_spot_light(
        &mut self,
        position: glam::Vec3,
        direction: glam::Vec3,
        color: glam::Vec3,
        intensity: f32,
        range: f32,
        angle_degrees: f32,
    ) -> usize {
        self.lighting
            .add_light(crate::systems::lighting::Light::spot(
                position,
                direction,
                color,
                intensity,
                range,
                angle_degrees,
            ))
    }

    // =========================================================================
    // 🎬 Scene shortcuts
    // =========================================================================

    /// Añade un objeto a la escena y devuelve su índice.
    pub fn spawn(
        &mut self,
        mesh: std::sync::Arc<crate::resources::mesh::Mesh>,
        material: std::sync::Arc<crate::resources::material::Material>,
        transform: glam::Mat4,
    ) -> usize {
        self.scene.add_object(mesh, material, transform)
    }

    /// Actualiza el transform del objeto en `index`.
    pub fn set_transform(&mut self, index: usize, transform: glam::Mat4) {
        if let Some(obj) = self.scene.objects.get_mut(index) {
            obj.transform = transform;
        }
    }

    /// Obtiene el transform actual del objeto en `index`.
    pub fn get_transform(&self, index: usize) -> Option<glam::Mat4> {
        self.scene.objects.get(index).map(|obj| obj.transform)
    }

    // =========================================================================
    // 🧱 Default Material + Primitive Spawning (UE5-style helpers)
    // =========================================================================

    /// Crea un material con los shaders SPIR-V embebidos en REACTOR
    /// (vertex color + iluminación básica). Listo para usar sin tocar disco.
    ///
    /// ```rust,no_run
    /// # fn demo(ctx: &mut reactor_vulkan::app::ReactorContext) -> Result<(), reactor_vulkan::ReactorError> {
    /// let mat = ctx.default_material()?;
    /// # Ok(()) }
    /// ```
    pub fn default_material(
        &self,
    ) -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        let vert = crate::builtin_shaders::vert_default();
        let frag = crate::builtin_shaders::frag_default();
        self.reactor
            .create_material(&vert, &frag)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    /// Spawn-helper: crea un cubo unitario en `position` y lo añade a la escena.
    /// Retorna el índice del objeto.
    pub fn spawn_cube(&mut self, position: glam::Vec3) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::cube();
        self.spawn_primitive(&v, &i, glam::Mat4::from_translation(position))
    }

    /// Spawn-helper: crea una esfera (32×16 segmentos) en `position` y la añade.
    pub fn spawn_sphere(
        &mut self,
        position: glam::Vec3,
        _radius: f32,
    ) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::sphere(32, 16);
        let xf = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(_radius.max(0.001)),
            glam::Quat::IDENTITY,
            position,
        );
        self.spawn_primitive(&v, &i, xf)
    }

    /// Spawn-helper: crea un plano (suelo) centrado en `position` con tamaño `size`.
    pub fn spawn_plane(
        &mut self,
        position: glam::Vec3,
        size: f32,
    ) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::plane(1);
        let xf = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(size, 1.0, size),
            glam::Quat::IDENTITY,
            position,
        );
        self.spawn_primitive(&v, &i, xf)
    }

    // =========================================================================
    // 🌑 Blob shadows (Fase 4.3 — sombras estilo House of the Dead)
    // =========================================================================
    //
    // Rail-shooters arcade clásicos (HOTD, Time Crisis) usan "blob shadows":
    // un disco oscuro plano debajo de cada entidad dinámica. Es 100 % CPU,
    // no requiere shadow-map GPU y aporta inmediatamente profundidad visual.
    //
    // Estos helpers crean/actualizan/ocultan un blob sin que el juego tenga
    // que tocar el mesh ni el material.

    /// Crea una sombra blob (disco oscuro plano) en el suelo bajo `position`.
    /// `radius` en metros. Devuelve el `usize` para usar con
    /// [`move_blob_shadow`](Self::move_blob_shadow) y
    /// [`hide_blob_shadow`](Self::hide_blob_shadow).
    ///
    /// El mesh + material se inicializan la primera vez y se reutilizan,
    /// así que llamar a este método miles de veces sólo allocata 1 mesh y 1
    /// material en GPU.
    pub fn spawn_blob_shadow(
        &mut self,
        position: glam::Vec3,
        radius: f32,
    ) -> crate::core::error::ReactorResult<usize> {
        use crate::resources::primitives::Primitives;

        // ── Lazy-init del mesh (esfera baja-poly compartida) ──
        if self.blob_shadow_mesh.is_none() {
            let (v, i) = Primitives::sphere(12, 6);
            let mesh = self
                .reactor
                .create_mesh(&v, &i)
                .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?;
            self.blob_shadow_mesh = Some(std::sync::Arc::new(mesh));
        }

        // ── Lazy-init del material (textura 1×1 oscura semi-transparente) ──
        if self.blob_shadow_material.is_none() {
            let dark_tex = self
                .reactor
                .create_solid_texture(8, 8, 10, 200)
                .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?;
            let mat = self
                .reactor
                .create_textured_material(
                    &crate::builtin_shaders::vert_textured(),
                    &crate::builtin_shaders::frag_textured(),
                    &dark_tex,
                )
                .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?
                .with_kept_texture(dark_tex);
            self.blob_shadow_material = Some(std::sync::Arc::new(mat));
        }

        let mesh = self.blob_shadow_mesh.clone().unwrap();
        let mat = self.blob_shadow_material.clone().unwrap();
        let xf = Self::blob_xf(position, radius);
        Ok(self.scene.add_object(mesh, mat, xf))
    }

    /// Mueve y re-escala un blob existente para que siga a su entidad.
    ///
    /// Llamar cada frame con la posición actual de la entidad propietaria.
    pub fn move_blob_shadow(&mut self, index: usize, position: glam::Vec3, radius: f32) {
        self.set_transform(index, Self::blob_xf(position, radius));
    }

    /// Oculta un blob (lo manda lejos del frustum). Útil al morir la entidad.
    ///
    /// El blob sigue ocupando su slot en la escena para permitir reutilización
    /// inmediata con [`move_blob_shadow`](Self::move_blob_shadow).
    pub fn hide_blob_shadow(&mut self, index: usize) {
        self.set_transform(
            index,
            glam::Mat4::from_translation(glam::Vec3::new(0.0, -1000.0, 0.0)),
        );
    }

    /// Construye el transform de un blob: aplanado en Y, escalado a `radius`
    /// en XZ, levantado 2 cm sobre el suelo para evitar z-fighting con el piso.
    fn blob_xf(position: glam::Vec3, radius: f32) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(radius, 0.02, radius),
            glam::Quat::IDENTITY,
            glam::Vec3::new(position.x, 0.02, position.z),
        )
    }

    /// Helper interno: crea mesh + material por defecto y añade a la escena.
    fn spawn_primitive(
        &mut self,
        vertices: &[crate::resources::vertex::Vertex],
        indices: &[u32],
        transform: glam::Mat4,
    ) -> crate::core::error::ReactorResult<usize> {
        // Las dos definiciones de Vertex (legacy + nueva) son ABI-idénticas
        // (repr(C) Pod con position/color/uv). Re-interpretamos sin copia.
        let legacy: &[crate::resources::vertex::Vertex] = bytemuck::cast_slice(vertices);
        let mesh = std::sync::Arc::new(
            self.reactor
                .create_mesh(legacy, indices)
                .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?,
        );
        let material = std::sync::Arc::new(self.default_material()?);
        Ok(self.scene.add_object(mesh, material, transform))
    }

    // =========================================================================
    // 🎯 Template Helpers — Spawning de alto nivel (1 línea en el juego)
    // =========================================================================

    /// Crea una esfera con color sólido y la añade a la escena.
    ///
    /// Ideal para crosshairs, indicadores, debug markers, etc.
    /// La textura se mantiene viva dentro del material (no requiere `keep_textures`).
    ///
    /// ```ignore
    /// let crosshair = ctx.spawn_colored_sphere(pos, 0.02, 255, 0, 0, 255)?;
    /// ```
    pub fn spawn_colored_sphere(
        &mut self,
        position: glam::Vec3,
        radius: f32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::sphere(16, 8);
        let mesh = std::sync::Arc::new(
            self.reactor
                .create_mesh(&v, &i)
                .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?,
        );
        let mat = self.create_colored_material(r, g, b, a)?;
        let mat_arc = std::sync::Arc::new(mat);
        let xf = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(radius.max(0.001)),
            glam::Quat::IDENTITY,
            position,
        );
        Ok(self.scene.add_object(mesh, mat_arc, xf))
    }

    /// Crea un quad (plano rectangular) con una textura de archivo y lo añade a la escena.
    ///
    /// Ideal para overlays (Game Over, Victoria), carteles, HUD, splash screens.
    /// La textura se mantiene viva dentro del material.
    ///
    /// ```ignore
    /// let go_idx = ctx.spawn_textured_quad("assets/textures/game_over.png", hidden_xf)?;
    /// ```
    pub fn spawn_textured_quad(
        &mut self,
        texture_path: &str,
        transform: glam::Mat4,
    ) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::quad();
        let mesh = std::sync::Arc::new(
            self.reactor
                .create_mesh(&v, &i)
                .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?,
        );
        let texture = self.load_texture(texture_path)?;
        let mat = self
            .reactor
            .create_textured_material(
                &crate::builtin_shaders::vert_textured(),
                &crate::builtin_shaders::frag_textured(),
                &texture,
            )
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?
            .with_kept_texture(texture);
        let mat_arc = std::sync::Arc::new(mat);
        Ok(self.scene.add_object(mesh, mat_arc, transform))
    }

    /// Crea un material con color sólido (textura 1×1 interna).
    ///
    /// La textura se mantiene viva dentro del material — no requiere
    /// guardarla externamente en un `Vec<Texture>`.
    ///
    /// ```ignore
    /// let red_mat = ctx.create_colored_material(255, 0, 0, 255)?;
    /// ```
    pub fn create_colored_material(
        &self,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        let texture = self.create_solid_texture(r, g, b, a)?;
        let mat = self
            .reactor
            .create_textured_material(
                &crate::builtin_shaders::vert_textured(),
                &crate::builtin_shaders::frag_textured(),
                &texture,
            )
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?
            .with_kept_texture(texture);
        Ok(mat)
    }

    // =========================================================================
    // 📦 Asset Pipeline (Fase 3) — Carga de modelos glTF y assets
    // =========================================================================

    /// Carga un modelo glTF/GLB y devuelve el GltfModel completo
    pub fn load_gltf<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> crate::core::error::ReactorResult<crate::resources::GltfModel> {
        self.gltf_loader
            .load(path)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    /// Carga un modelo glTF de forma asíncrona
    pub async fn load_gltf_async<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> crate::core::error::ReactorResult<crate::resources::GltfModel> {
        let path_buf = path.as_ref().to_path_buf();
        let mut loader = self.gltf_loader.clone();
        tokio::task::spawn_blocking(move || loader.load(path_buf))
            .await
            .map_err(|e| {
                crate::core::error::ReactorError::internal(format!("Blocking task failed: {}", e))
            })?
    }

    /// Carga un modelo glTF en la cola asíncrona (background)
    /// Retorna un Receiver para obtener el resultado cuando esté listo
    pub fn load_gltf_queued<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        priority: crate::resources::LoadPriority,
    ) -> tokio::sync::oneshot::Receiver<
        crate::core::error::ReactorResult<Handle<crate::resources::GltfModel>>,
    > {
        let path_buf = path.as_ref().to_path_buf();
        let id = AssetId::from_path(&path_buf);
        self.asset_loader_queue.enqueue_gltf(id, path_buf, priority)
    }

    /// Spawn de un modelo glTF en la escena con transform
    pub fn spawn_gltf<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        transform: glam::Mat4,
    ) -> crate::core::error::ReactorResult<Vec<usize>> {
        let model = self.load_gltf(path)?;
        self.spawn_gltf_model(&model, transform)
    }

    // =========================================================================
    // 🧊 Spawning inteligente de modelos Blender (auto-escala + auto-orientación)
    // =========================================================================

    /// Inspecciona un glTF/GLB y devuelve sus dimensiones nativas sin spawnearlo.
    ///
    /// Útil para conocer la altura/anchura "real" exportada desde Blender antes
    /// de decidir cómo posicionarlo.
    pub fn gltf_bounds<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> crate::core::error::ReactorResult<GltfBounds> {
        let model = self.load_gltf(path)?;
        let (min, max) = model.bounds().ok_or_else(|| {
            crate::core::error::ReactorError::asset_load("glTF model has no meshes")
        })?;
        Ok(GltfBounds {
            min,
            max,
            center: (min + max) * 0.5,
            size: max - min,
            height: max.y - min.y,
        })
    }

    /// Spawn "inteligente" de un modelo glTF: auto-escala a la altura objetivo,
    /// auto-orienta hacia una dirección, y coloca los pies del modelo en
    /// `position` (no el pivot, que en Blender suele estar mal calibrado).
    ///
    /// ```ignore
    /// let info = ctx.spawn_gltf_smart(
    ///     "assets/models/zombie.glb",
    ///     GltfSpawn::at(Vec3::new(0.0, 0.0, -10.0))
    ///         .with_height(1.8)              // re-escala a 1.8 m de alto
    ///         .facing(Vec3::new(0.0, 0.0, 1.0)), // mira hacia +Z (la cámara)
    /// )?;
    /// println!("Zombie spawned: scale {:.2}, height {:.2}m",
    ///     info.applied_scale, info.world_height);
    /// ```
    pub fn spawn_gltf_smart<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        spawn: GltfSpawn,
    ) -> crate::core::error::ReactorResult<ModelSpawnInfo> {
        let model = self.load_gltf(path)?;

        // ── 1. Bounds nativos ──
        let (min, max) = model.bounds().ok_or_else(|| {
            crate::core::error::ReactorError::asset_load("glTF model has no meshes")
        })?;
        let native_height = max.y - min.y;
        let native_center = (min + max) * 0.5;

        // ── 2. Auto-escala ──
        let scale = if let Some(target_h) = spawn.target_height {
            if native_height > 1e-6 {
                target_h / native_height
            } else {
                1.0
            }
        } else {
            1.0
        };

        // ── 3. Auto-orientación (rotación Y para mirar hacia `face_direction`) ──
        let rotation = if let Some(dir) = spawn.face_direction {
            let flat = glam::Vec3::new(dir.x, 0.0, dir.z);
            if flat.length_squared() > 1e-6 {
                let n = flat.normalize();
                // atan2(x, z) coloca el modelo (mira por defecto a -Z) en `n`.
                let yaw = (-n.x).atan2(-n.z);
                glam::Quat::from_rotation_y(yaw)
            } else {
                glam::Quat::IDENTITY
            }
        } else {
            glam::Quat::IDENTITY
        };

        // ── 4. Offset: pies del modelo en position.y (no su pivot) ──
        //    Después de escalar, los pies del modelo están en min.y * scale.
        //    Para subirlos a position.y, sumamos -min.y * scale al transform.
        let feet_offset = if spawn.feet_at_position {
            glam::Vec3::new(0.0, -min.y * scale, 0.0)
        } else {
            glam::Vec3::ZERO
        };

        let final_pos = spawn.position + feet_offset;
        let transform = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(scale),
            rotation,
            final_pos,
        );

        // ── 5. Spawn real ──
        let indices = self.spawn_gltf_model(&model, transform)?;

        let world_min = (min - native_center) * scale
            + spawn.position
            + glam::Vec3::Y * (native_center.y * scale - min.y * scale);
        let world_max = (max - native_center) * scale
            + spawn.position
            + glam::Vec3::Y * (native_center.y * scale - min.y * scale);

        Ok(ModelSpawnInfo {
            indices,
            applied_scale: scale,
            applied_rotation: rotation,
            native_height,
            world_height: native_height * scale,
            world_bounds_min: world_min,
            world_bounds_max: world_max,
        })
    }

    /// Spawn de un GltfModel ya cargado en la escena
    pub fn spawn_gltf_model(
        &mut self,
        model: &crate::resources::GltfModel,
        parent_transform: glam::Mat4,
    ) -> crate::core::error::ReactorResult<Vec<usize>> {
        let mut indices = Vec::new();
        self.spawn_gltf_node_recursive(&model.root_node, model, parent_transform, &mut indices)?;
        Ok(indices)
    }

    /// Recorre recursivamente la jerarquía de nodos glTF y los añade a la escena
    fn spawn_gltf_node_recursive(
        &mut self,
        node: &crate::resources::GltfNode,
        model: &crate::resources::GltfModel,
        parent_transform: glam::Mat4,
        indices: &mut Vec<usize>,
    ) -> crate::core::error::ReactorResult<()> {
        let world_transform = parent_transform * node.transform;

        // Si el nodo tiene mesh, crear entidad en escena
        if let Some(mesh_idx) = node.mesh_index {
            if let Some(mesh_data) = model.meshes.get(mesh_idx) {
                // Crear mesh Vulkan en GPU
                let vulkan_mesh = crate::resources::mesh::Mesh::new(
                    &self.reactor.context,
                    &self.reactor.allocator,
                    &mesh_data.vertices,
                    &mesh_data.indices,
                )?;
                let mesh_arc = std::sync::Arc::new(vulkan_mesh);

                // Determinar material
                let material_arc = if let Some(mat_idx) = mesh_data.material_index {
                    if let Some(mat_data) = model.materials.get(mat_idx) {
                        if let Some(tex_idx) = mat_data.base_color_texture_index {
                            if let Some(tex_data) = model.textures.get(tex_idx) {
                                // Subir textura a GPU
                                let texture = crate::resources::texture::Texture::from_rgba(
                                    &self.reactor.context,
                                    self.reactor.allocator.clone(),
                                    &tex_data.pixels,
                                    tex_data.width,
                                    tex_data.height,
                                    true,
                                )?;
                                // Crear material con textura
                                let vert = crate::builtin_shaders::vert_textured();
                                let frag = crate::builtin_shaders::frag_textured();
                                let mat = self
                                    .reactor
                                    .create_textured_material(&vert, &frag, &texture)
                                    .map_err(|e| {
                                        crate::core::error::ReactorError::internal(e.to_string())
                                    })?
                                    .with_kept_texture(texture);
                                std::sync::Arc::new(mat)
                            } else {
                                std::sync::Arc::new(self.default_material()?)
                            }
                        } else {
                            std::sync::Arc::new(self.default_material()?)
                        }
                    } else {
                        std::sync::Arc::new(self.default_material()?)
                    }
                } else {
                    std::sync::Arc::new(self.default_material()?)
                };

                let obj_idx = self
                    .scene
                    .add_object(mesh_arc, material_arc, world_transform);
                indices.push(obj_idx);
            }
        }

        // Recursar hijos
        for child in &node.children {
            self.spawn_gltf_node_recursive(child, model, world_transform, indices)?;
        }

        Ok(())
    }

    /// Trackear un asset para hot-reload
    pub fn track_asset_for_reload<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        asset_type: crate::resources::AssetType,
    ) -> crate::core::error::ReactorResult<AssetId> {
        let path = path.as_ref();
        let id = AssetId::from_path(path);

        if let Some(ref mut hot_reload) = self.asset_hot_reload {
            hot_reload
                .track_asset(id, path, asset_type)
                .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?;
        }

        Ok(id)
    }

    /// Obtener estadísticas del asset pipeline
    pub fn asset_stats(&self) -> AssetPipelineStats {
        AssetPipelineStats {
            loader_queue: self.asset_loader_queue.stats(),
            hot_reload: self.asset_hot_reload.as_ref().map(|hr| hr.stats()),
            db: self.asset_db.stats(),
            gltf_cache: self.gltf_loader.cache_stats(),
        }
    }
}

/// Estadísticas consolidadas del Asset Pipeline
#[derive(Clone, Debug)]
pub struct AssetPipelineStats {
    pub loader_queue: crate::resources::LoaderStats,
    pub hot_reload: Option<crate::resources::HotReloadStats>,
    pub db: crate::resources::AssetDbStats,
    pub gltf_cache: crate::resources::GltfCacheStats,
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
        if self.context.is_some() {
            return;
        }

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
        let reactor = match Reactor::init(&window, config.msaa_samples) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to initialize Reactor: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Enable ANSI colors on Windows terminal
        crate::systems::console::init();

        crate::systems::console::ReactorBanner::print_init(
            &config.title,
            &format!("{}×{}", window.inner_size().width, window.inner_size().height),
            &format!("{:?}", reactor.msaa_samples),
            reactor.ray_tracing.is_some(),
            &format!("{}", crate::systems::console::gpu_name_short(&reactor.context)),
        );

        let aspect = window.inner_size().width as f32 / window.inner_size().height.max(1) as f32;

        // Initialize Asset Pipeline (Fase 3)
        let asset_manager = AssetManager::new();
        let gltf_loader = GltfLoader::new("assets");
        let asset_db = AssetDatabase::open(".reactor/assets.db")
            .unwrap_or_else(|_| AssetDatabase::in_memory().unwrap());
        let asset_loader_queue = AssetLoaderQueue::new().unwrap_or_else(|_| {
            AssetLoaderQueue::with_config(crate::resources::LoaderQueueConfig {
                num_workers: 2,
                ..Default::default()
            })
            .unwrap()
        });

        // Hot-reload setup (optional, can fail if notify not supported)
        let (hot_reload_tx, hot_reload_rx) = tokio::sync::mpsc::unbounded_channel();
        let asset_hot_reload =
            AssetHotReloadManager::new(crate::resources::HotReloadConfig::default(), hot_reload_tx)
                .ok();
        let hot_reload_rx = if asset_hot_reload.is_some() {
            Some(hot_reload_rx)
        } else {
            None
        };

        let mut ctx = ReactorContext {
            reactor,
            window,
            time: Time::new(),
            config: config.clone(),
            camera: crate::scene::camera::Camera::perspective(60.0, aspect, 0.1, 1000.0),
            scene: crate::systems::scene::Scene::new(),
            lighting: crate::systems::lighting::LightingSystem::new(),
            physics: crate::systems::physics::PhysicsWorld::new(),
            culling: crate::systems::frustum::CullingSystem::new(),
            debug: crate::graphics::debug_renderer::DebugRenderer::new(),
            asset_manager,
            gltf_loader,
            asset_db,
            asset_hot_reload,
            asset_loader_queue,
            audio: crate::systems::audio::AudioSystem::new(),
            event_bus: crate::systems::event_bus::EventBus::new(),
            hot_reload_rx,
            blob_shadow_mesh: None,
            blob_shadow_material: None,
            fixed_accumulator: 0.0,
        };

        // Call user init
        self.app.init(&mut ctx);

        self.context = Some(ctx);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(ctx) = &mut self.context else { return };

        // Let Reactor handle input
        ctx.reactor.handle_event(&event);

        // Let user handle event first
        if self.app.on_event(ctx, &event) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                // No llamamos a `on_exit` aquí — winit invocará `exiting()` cuando
                // el event loop termine, y ese es el único punto donde corremos el
                // shutdown del usuario. Evita duplicar el "After Action Report" y
                // las llamadas Vulkan tras `device_wait_idle`.
                event_loop.exit();
            }

            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                self.app.on_resize(ctx, size.width, size.height);
            }

            WindowEvent::RedrawRequested => {
                // Drain hot-reload events and emit to EventBus
                if let Some(ref mut rx) = ctx.hot_reload_rx {
                    while let Ok(event) = rx.try_recv() {
                        ctx.event_bus.emit(event);
                    }
                }

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

                // Si el dispositivo Vulkan se perdió, detenemos el loop para evitar spam de errores
                if ctx.reactor.device_lost {
                    event_loop.exit();
                    return;
                }

                // Request next frame
                ctx.window.request_redraw();
            }

            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ctx) = &mut self.context {
            // 1. Esperar a que toda la GPU termine ANTES del cleanup del usuario
            //    y del Drop de los recursos. Esto evita los típicos errores de
            //    validation layer del estilo "X is in use" al cerrar la ventana.
            //    SAFETY: el dispositivo es válido mientras `ctx` siga vivo;
            //    ignoramos `DEVICE_LOST` para permitir cleanup grácil.
            unsafe {
                let _ = ctx.reactor.context.device.device_wait_idle();
            }

            // 2. Callback de usuario para volcar estadísticas / liberar handles.
            self.app.on_exit(ctx);

            // 3. Re-sincronizar por si `on_exit` lanzó trabajo a la GPU
            //    (poco común pero gratis, evita race condition en el Drop).
            unsafe {
                let _ = ctx.reactor.context.device.device_wait_idle();
            }
        }

        // 4. Soltar el contexto explícitamente para que los Drops corran AHORA
        //    (orden inverso de creación), en vez de cuando winit decida tirar
        //    el `AppRunner`. Esto fuerza un cleanup determinista de Vulkan.
        self.context.take();
    }
}

// =============================================================================
// Public API — The entry point
// =============================================================================

/// Run a REACTOR application. This is the main entry point.
///
/// # Example
/// ```rust,no_run
/// use reactor_vulkan::prelude::*;
///
/// struct MyGame;
///
/// impl ReactorApp for MyGame {
///     fn init(&mut self, _ctx: &mut ReactorContext) {}
///     fn update(&mut self, _ctx: &mut ReactorContext) {}
/// }
///
/// fn main() {
///     reactor_vulkan::run(MyGame);
/// }
/// ```
pub fn run<A: ReactorApp + 'static>(app: A) {
    // env_logger::init() panics if called twice (eg in tests).
    let _ = env_logger::try_init();

    // Crear un runtime de Tokio multi-threaded y entrar a su contexto
    // para que la cola de carga de assets y hot-reload puedan usar tokio::spawn y temporizadores.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");
    let _guard = rt.enter();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = AppRunner { app, context: None };

    event_loop.run_app(&mut runner).expect("Event loop error");
}

// =============================================================================
// SHORT API — minimal-boilerplate helpers
// =============================================================================

/// Closure-based app generated on the fly from `quick(...)` / `quick_with(...)`.
struct QuickApp<I, U>
where
    I: FnMut(&mut ReactorContext) + 'static,
    U: FnMut(&mut ReactorContext) + 'static,
{
    config: ReactorConfig,
    init: Option<I>,
    update: U,
}

impl<I, U> ReactorApp for QuickApp<I, U>
where
    I: FnMut(&mut ReactorContext) + 'static,
    U: FnMut(&mut ReactorContext) + 'static,
{
    fn config(&self) -> ReactorConfig {
        self.config.clone()
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        if let Some(mut f) = self.init.take() {
            f(ctx);
        }
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        (self.update)(ctx);
    }
}

/// **One-call game launcher** — el camino MÁS corto para arrancar un juego REACTOR.
///
/// ```rust,no_run
/// reactor_vulkan::quick("Mi Juego", 1280, 720, |ctx| {
///     // se ejecuta cada frame
///     ctx.camera.position.x = ctx.time.elapsed().sin() * 5.0;
/// });
/// ```
pub fn quick<U>(title: &str, width: u32, height: u32, update: U)
where
    U: FnMut(&mut ReactorContext) + 'static,
{
    run(QuickApp {
        config: ReactorConfig::new(title).with_size(width, height),
        init: None::<fn(&mut ReactorContext)>,
        update,
    });
}

/// Como [`quick`] pero con un closure de inicialización adicional.
///
/// ```rust,no_run
/// # use reactor_vulkan::prelude::*;
/// reactor_vulkan::quick_with(
///     ReactorConfig::new("Mi Juego").with_size(1280, 720).with_msaa(4),
///     |ctx| {
///         ctx.camera.position = Vec3::new(0.0, 2.0, 5.0);
///     },
///     |ctx| {
///         // update cada frame
///         let _dt = ctx.time.delta();
///     },
/// );
/// ```
pub fn quick_with<I, U>(config: ReactorConfig, init: I, update: U)
where
    I: FnMut(&mut ReactorContext) + 'static,
    U: FnMut(&mut ReactorContext) + 'static,
{
    run(QuickApp { config, init: Some(init), update });
}

/// Helper to call closure inside macro with type inference.
#[inline(always)]
pub fn call_init<F>(mut f: F, ctx: &mut ReactorContext)
where
    F: FnMut(&mut ReactorContext),
{
    f(ctx);
}

/// Helper to call closure inside macro with type inference.
#[inline(always)]
pub fn call_update<F>(mut f: F, ctx: &mut ReactorContext)
where
    F: FnMut(&mut ReactorContext),
{
    f(ctx);
}
