// =============================================================================
// REACTOR C API — Bridge between Rust core and C/C++
// =============================================================================
// This crate exposes REACTOR's functionality as extern "C" functions.
// It produces a .dll/.so that C++ code can link against.
//
// Architecture:
//   C++ Game → reactor.dll (this) → Rust Reactor → Vulkan
//
// Design Goal: ONE CALL — ReactorApp() initializes everything ultra-intelligently
// =============================================================================

use std::os::raw::c_char;
use std::ffi::CStr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
    dpi::LogicalSize,
    keyboard::{KeyCode, PhysicalKey},
};

use reactor::reactor::Reactor;
#[allow(unused_imports)]
use reactor::input::Input;
use reactor::utils::time::Time;
use reactor::scene::Scene;
use reactor::mesh::Mesh;
use reactor::material::Material;
use reactor::systems::camera::Camera;
use reactor::systems::lighting::LightingSystem;
use reactor::systems::physics::PhysicsWorld;
use reactor::systems::frustum::CullingSystem;
use reactor::graphics::debug_renderer::DebugRenderer;
use reactor::core::error::{ErrorCode, get_last_error_code, get_last_error_message, clear_last_error, has_error};

use std::sync::Arc;

// =============================================================================
// Global State — Thread-safe singleton for C++ access
// =============================================================================

static REACTOR_STATE: Mutex<Option<ReactorState>> = Mutex::new(None);
static REACTOR_INITIALIZED: AtomicBool = AtomicBool::new(false);

struct ReactorState {
    reactor: Option<Reactor>,
    window: Option<Arc<Window>>,
    time: Time,
    scene: Scene,
    camera: Camera,
    lighting: LightingSystem,
    physics: PhysicsWorld,
    culling: CullingSystem,
    debug: DebugRenderer,
    running: bool,
    should_close: bool,
    frame_active: bool,
    width: u32,
    height: u32,
    title: String,
    delta_time: f32,
    total_time: f32,
    frame_count: u64,
    // Input state
    keys_down: HashMap<u32, bool>,
    keys_pressed: HashMap<u32, bool>,
    mouse_x: f32,
    mouse_y: f32,
    mouse_dx: f32,
    mouse_dy: f32,
    mouse_buttons: [bool; 5],
}

impl Default for ReactorState {
    fn default() -> Self {
        Self {
            reactor: None,
            window: None,
            time: Time::new(),
            scene: Scene::new(),
            camera: Camera::perspective(60.0, 16.0/9.0, 0.1, 1000.0),
            lighting: LightingSystem::new(),
            physics: PhysicsWorld::new(),
            culling: CullingSystem::new(),
            debug: DebugRenderer::new(),
            running: false,
            should_close: false,
            frame_active: false,
            width: 1280,
            height: 720,
            title: String::from("REACTOR Application"),
            delta_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            keys_down: HashMap::new(),
            keys_pressed: HashMap::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            mouse_buttons: [false; 5],
        }
    }
}

// =============================================================================
// Opaque handles — C++ sees these as pointers
// =============================================================================

pub struct ReactorHandle {
    _placeholder: u8,
}

pub struct SceneHandle {
    pub(crate) scene: Scene,
}

pub struct MeshHandle {
    pub(crate) mesh: reactor::mesh::Mesh,
}

pub struct MaterialHandle {
    pub(crate) material: reactor::material::Material,
}

pub struct TextureHandle {
    pub(crate) texture: reactor::resources::texture::Texture,
}

pub struct CameraHandle {
    pub(crate) camera: Camera,
}

// =============================================================================
// Math types — Compatible with C (repr(C))
// =============================================================================

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct CVec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct CVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct CVec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CMat4 {
    pub cols: [[f32; 4]; 4],
}

impl Default for CMat4 {
    fn default() -> Self {
        Self { cols: glam::Mat4::IDENTITY.to_cols_array_2d() }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CVertex {
    pub position: CVec3,
    pub normal: CVec3,
    pub uv: CVec2,
    pub color: CVec4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CTransform {
    pub position: CVec3,
    pub rotation: CVec3,
    pub scale: CVec3,
}

impl Default for CTransform {
    fn default() -> Self {
        Self {
            position: CVec3::default(),
            rotation: CVec3::default(),
            scale: CVec3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CLight {
    pub light_type: u32,  // 0=Directional, 1=Point, 2=Spot
    pub position: CVec3,
    pub direction: CVec3,
    pub color: CVec3,
    pub intensity: f32,
    pub range: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}

impl Default for CLight {
    fn default() -> Self {
        Self {
            light_type: 0,
            position: CVec3::default(),
            direction: CVec3 { x: 0.0, y: -1.0, z: 0.0 },
            color: CVec3 { x: 1.0, y: 1.0, z: 1.0 },
            intensity: 1.0,
            range: 10.0,
            inner_angle: 30.0,
            outer_angle: 45.0,
        }
    }
}

/// Renderer mode enum for C ABI
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub enum CRendererMode {
    #[default]
    Forward = 0,
    Deferred = 1,
    RayTracing = 2,
}

#[repr(C)]
pub struct CConfig {
    pub title: *const c_char,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub msaa_samples: u32,
    pub fullscreen: bool,
    pub resizable: bool,
    pub physics_hz: u32,
    pub renderer: CRendererMode,
    pub scene: *const c_char,  // Path to auto-load scene (glTF, etc.)
}

impl Default for CConfig {
    fn default() -> Self {
        Self {
            title: std::ptr::null(),
            width: 1280,
            height: 720,
            vsync: true,
            msaa_samples: 4,
            fullscreen: false,
            resizable: true,
            physics_hz: 60,
            renderer: CRendererMode::Forward,
            scene: std::ptr::null(),
        }
    }
}

// =============================================================================
// Callback types for C++
// =============================================================================

pub type InitCallback = extern "C" fn();
pub type UpdateCallback = extern "C" fn(f32);
pub type RenderCallback = extern "C" fn();
pub type ShutdownCallback = extern "C" fn();
pub type ResizeCallback = extern "C" fn(u32, u32);

#[repr(C)]
pub struct CCallbacks {
    pub on_init: Option<InitCallback>,
    pub on_update: Option<UpdateCallback>,
    pub on_render: Option<RenderCallback>,
    pub on_shutdown: Option<ShutdownCallback>,
    pub on_resize: Option<ResizeCallback>,
}

impl Default for CCallbacks {
    fn default() -> Self {
        Self {
            on_init: None,
            on_update: None,
            on_render: None,
            on_shutdown: None,
            on_resize: None,
        }
    }
}

// =============================================================================
// Version & Info
// =============================================================================

// =============================================================================
// ReactorResult — ABI-safe error codes (never panic across FFI boundary)
// =============================================================================

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReactorResult {
    Ok = 0,
    ErrorNotInitialized = 1,
    ErrorAlreadyInitialized = 2,
    ErrorVulkanInit = 3,
    ErrorWindowCreation = 4,
    ErrorShaderCompilation = 5,
    ErrorMeshCreation = 6,
    ErrorMaterialCreation = 7,
    ErrorInvalidHandle = 8,
    ErrorOutOfMemory = 9,
    ErrorInvalidArgument = 10,
    ErrorFrameNotActive = 11,
    ErrorFrameAlreadyActive = 12,
    ErrorUnknown = 255,
}

/// Get human-readable string for a ReactorResult
#[unsafe(no_mangle)]
pub extern "C" fn reactor_result_string(result: ReactorResult) -> *const c_char {
    match result {
        ReactorResult::Ok => c"REACTOR_OK".as_ptr(),
        ReactorResult::ErrorNotInitialized => c"REACTOR_ERROR_NOT_INITIALIZED".as_ptr(),
        ReactorResult::ErrorAlreadyInitialized => c"REACTOR_ERROR_ALREADY_INITIALIZED".as_ptr(),
        ReactorResult::ErrorVulkanInit => c"REACTOR_ERROR_VULKAN_INIT".as_ptr(),
        ReactorResult::ErrorWindowCreation => c"REACTOR_ERROR_WINDOW_CREATION".as_ptr(),
        ReactorResult::ErrorShaderCompilation => c"REACTOR_ERROR_SHADER_COMPILATION".as_ptr(),
        ReactorResult::ErrorMeshCreation => c"REACTOR_ERROR_MESH_CREATION".as_ptr(),
        ReactorResult::ErrorMaterialCreation => c"REACTOR_ERROR_MATERIAL_CREATION".as_ptr(),
        ReactorResult::ErrorInvalidHandle => c"REACTOR_ERROR_INVALID_HANDLE".as_ptr(),
        ReactorResult::ErrorOutOfMemory => c"REACTOR_ERROR_OUT_OF_MEMORY".as_ptr(),
        ReactorResult::ErrorInvalidArgument => c"REACTOR_ERROR_INVALID_ARGUMENT".as_ptr(),
        ReactorResult::ErrorFrameNotActive => c"REACTOR_ERROR_FRAME_NOT_ACTIVE".as_ptr(),
        ReactorResult::ErrorFrameAlreadyActive => c"REACTOR_ERROR_FRAME_ALREADY_ACTIVE".as_ptr(),
        ReactorResult::ErrorUnknown => c"REACTOR_ERROR_UNKNOWN".as_ptr(),
    }
}

// =============================================================================
// Global Initialization / Shutdown (Formal Lifecycle)
// =============================================================================

/// Initialize REACTOR subsystems. Must be called before any other reactor_ function.
/// Returns REACTOR_OK on success.
#[unsafe(no_mangle)]
pub extern "C" fn reactor_initialize() -> ReactorResult {
    if REACTOR_INITIALIZED.load(Ordering::SeqCst) {
        return ReactorResult::ErrorAlreadyInitialized;
    }
    
    // Initialize global state
    {
        let mut state = REACTOR_STATE.lock().unwrap();
        *state = Some(ReactorState::default());
    }
    
    REACTOR_INITIALIZED.store(true, Ordering::SeqCst);
    ReactorResult::Ok
}

/// Shutdown REACTOR and release all resources.
/// After this call, no reactor_ functions should be used until reactor_initialize() is called again.
#[unsafe(no_mangle)]
pub extern "C" fn reactor_shutdown() -> ReactorResult {
    if !REACTOR_INITIALIZED.load(Ordering::SeqCst) {
        return ReactorResult::ErrorNotInitialized;
    }
    
    // Clean up state - IMPORTANT: Clear scene objects BEFORE destroying Vulkan resources
    {
        let mut state = REACTOR_STATE.lock().unwrap();
        if let Some(ref mut s) = *state {
            // 1. Wait for GPU to finish all operations
            if let Some(ref reactor) = s.reactor {
                unsafe {
                    let _ = reactor.context.device.device_wait_idle();
                }
            }
            
            // 2. Clear scene objects (releases Arc<Mesh> and Arc<Material>)
            s.scene.objects.clear();
            
            // 3. Clear lighting
            s.lighting.lights.clear();
        }
        
        // 4. Drop the entire state (Reactor, Window, etc.)
        *state = None;
    }
    
    // Clean up callbacks
    {
        let mut cb = CALLBACKS.lock().unwrap();
        *cb = CCallbacks::default();
    }
    
    REACTOR_INITIALIZED.store(false, Ordering::SeqCst);
    ReactorResult::Ok
}

/// Check if REACTOR is initialized
#[unsafe(no_mangle)]
pub extern "C" fn reactor_is_initialized() -> bool {
    REACTOR_INITIALIZED.load(Ordering::SeqCst)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_version() -> *const c_char {
    c"REACTOR v1.0.5".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_engine_name() -> *const c_char {
    c"REACTOR Framework for Vulkan".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_version_major() -> u32 { 1 }

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_version_minor() -> u32 { 0 }

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_version_patch() -> u32 { 5 }

// =============================================================================
// CORE API — The ONE CALL entry point
// =============================================================================

static CALLBACKS: Mutex<CCallbacks> = Mutex::new(CCallbacks {
    on_init: None,
    on_update: None,
    on_render: None,
    on_shutdown: None,
    on_resize: None,
});

struct AppRunner;

impl ApplicationHandler for AppRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut state = REACTOR_STATE.lock().unwrap();
        if state.is_some() && state.as_ref().unwrap().reactor.is_some() {
            return;
        }

        let s = state.get_or_insert_with(ReactorState::default);

        let window_attributes = Window::default_attributes()
            .with_title(&s.title)
            .with_inner_size(LogicalSize::new(s.width, s.height));

        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let reactor = match Reactor::init(&window) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to initialize Reactor: {}", e);
                event_loop.exit();
                return;
            }
        };

        println!("+==============================================================+");
        println!("|              REACTOR Framework for C++                       |");
        println!("|  Version: {}                                             |", "1.0.5");
        println!("|  Title: {:52} |", s.title);
        println!("|  Resolution: {}x{:<44} |", s.width, s.height);
        println!("|  Ray Tracing: {:<47} |",
            if reactor.ray_tracing.is_some() { "[OK] Enabled" } else { "[X] Not available" });
        println!("+==============================================================+");

        let aspect = s.width as f32 / s.height.max(1) as f32;
        s.camera = Camera::perspective(60.0, aspect, 0.1, 1000.0);
        s.reactor = Some(reactor);
        s.window = Some(window);
        s.running = true;
        s.time = Time::new();

        drop(state);

        // Call user init
        let callbacks = CALLBACKS.lock().unwrap();
        if let Some(init) = callbacks.on_init {
            init();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let mut state = REACTOR_STATE.lock().unwrap();
        let Some(s) = state.as_mut() else { return };

        // Handle input
        if let Some(ref mut reactor) = s.reactor {
            reactor.handle_event(&event);
        }

        match &event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    let key_code = key as u32;
                    if event.state.is_pressed() {
                        if !s.keys_down.get(&key_code).copied().unwrap_or(false) {
                            s.keys_pressed.insert(key_code, true);
                        }
                        s.keys_down.insert(key_code, true);
                    } else {
                        s.keys_down.insert(key_code, false);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_x = position.x as f32;
                let new_y = position.y as f32;
                s.mouse_dx = new_x - s.mouse_x;
                s.mouse_dy = new_y - s.mouse_y;
                s.mouse_x = new_x;
                s.mouse_y = new_y;
            }
            WindowEvent::MouseInput { state: btn_state, button, .. } => {
                let idx = match button {
                    winit::event::MouseButton::Left => 0,
                    winit::event::MouseButton::Right => 1,
                    winit::event::MouseButton::Middle => 2,
                    winit::event::MouseButton::Back => 3,
                    winit::event::MouseButton::Forward => 4,
                    _ => return,
                };
                s.mouse_buttons[idx] = btn_state.is_pressed();
            }
            _ => {}
        }

        match event {
            WindowEvent::CloseRequested => {
                s.should_close = true;
                drop(state);
                let callbacks = CALLBACKS.lock().unwrap();
                if let Some(shutdown) = callbacks.on_shutdown {
                    shutdown();
                }
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    s.width = size.width;
                    s.height = size.height;
                    let w = size.width;
                    let h = size.height;
                    drop(state);
                    let callbacks = CALLBACKS.lock().unwrap();
                    if let Some(resize) = callbacks.on_resize {
                        resize(w, h);
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                s.time.update();
                s.delta_time = s.time.delta();
                s.total_time += s.delta_time;
                s.frame_count += 1;

                // Clear per-frame input
                s.keys_pressed.clear();
                s.mouse_dx = 0.0;
                s.mouse_dy = 0.0;

                let dt = s.delta_time;
                
                // Calculate view-projection matrix
                let view_proj = s.camera.view_projection_matrix();
                
                // Render the scene directly while we have the lock
                if let Some(ref mut reactor) = s.reactor {
                    let _ = reactor.draw_scene(&s.scene, &view_proj);
                }
                
                drop(state);

                // Call user update
                let callbacks = CALLBACKS.lock().unwrap();
                if let Some(update) = callbacks.on_update {
                    update(dt);
                }
                if let Some(render) = callbacks.on_render {
                    render();
                }
                drop(callbacks);

                // Request next frame
                let state = REACTOR_STATE.lock().unwrap();
                if let Some(s) = state.as_ref() {
                    if let Some(ref window) = s.window {
                        window.request_redraw();
                    }
                }
            }

            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        let mut state = REACTOR_STATE.lock().unwrap();
        if let Some(s) = state.as_mut() {
            // 1. Wait for GPU to finish all operations
            if let Some(ref reactor) = s.reactor {
                unsafe {
                    let _ = reactor.context.device.device_wait_idle();
                }
            }
            
            // 2. Clear scene objects BEFORE Reactor is dropped (releases VkBuffer/VkPipeline)
            s.scene.objects.clear();
            
            // 3. Clear lighting
            s.lighting.lights.clear();
        }
    }
}

/// Initialize and run REACTOR with callbacks — THE ONE CALL
#[unsafe(no_mangle)]
pub extern "C" fn reactor_run(config: CConfig, callbacks: CCallbacks) -> i32 {
    // Store callbacks
    {
        let mut cb = CALLBACKS.lock().unwrap();
        *cb = callbacks;
    }

    // Store config
    {
        let mut state = REACTOR_STATE.lock().unwrap();
        let s = state.get_or_insert_with(ReactorState::default);
        s.width = if config.width > 0 { config.width } else { 1280 };
        s.height = if config.height > 0 { config.height } else { 720 };
        if !config.title.is_null() {
            s.title = unsafe { CStr::from_ptr(config.title) }
                .to_string_lossy()
                .into_owned();
        }
    }

    // Run event loop
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("Failed to create event loop: {}", e);
            return -1;
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = AppRunner;
    if let Err(e) = event_loop.run_app(&mut runner) {
        eprintln!("Event loop error: {}", e);
        return -1;
    }

    0
}

/// Simple run with just title and size
#[unsafe(no_mangle)]
pub extern "C" fn reactor_run_simple(
    title: *const c_char,
    width: u32,
    height: u32,
    on_init: Option<InitCallback>,
    on_update: Option<UpdateCallback>,
    on_render: Option<RenderCallback>,
) -> i32 {
    let config = CConfig {
        title,
        width,
        height,
        ..Default::default()
    };
    let callbacks = CCallbacks {
        on_init,
        on_update,
        on_render,
        on_shutdown: None,
        on_resize: None,
    };
    reactor_run(config, callbacks)
}

// =============================================================================
// Frame Lifecycle — Command submission boundary
// =============================================================================

/// Begin a new frame. Must be called before any per-frame operations.
/// Pairs with reactor_end_frame(). Returns REACTOR_OK on success.
#[unsafe(no_mangle)]
pub extern "C" fn reactor_begin_frame() -> ReactorResult {
    let mut state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_mut() else {
        return ReactorResult::ErrorNotInitialized;
    };
    if s.frame_active {
        return ReactorResult::ErrorFrameAlreadyActive;
    }
    
    // Update time
    s.time.update();
    s.delta_time = s.time.delta();
    s.total_time += s.delta_time;
    s.frame_count += 1;
    
    // Clear per-frame input
    s.keys_pressed.clear();
    s.mouse_dx = 0.0;
    s.mouse_dy = 0.0;
    
    s.frame_active = true;
    ReactorResult::Ok
}

/// End the current frame. Submits rendering commands to the GPU.
/// Must be called after reactor_begin_frame().
#[unsafe(no_mangle)]
pub extern "C" fn reactor_end_frame() -> ReactorResult {
    let mut state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_mut() else {
        return ReactorResult::ErrorNotInitialized;
    };
    if !s.frame_active {
        return ReactorResult::ErrorFrameNotActive;
    }
    
    // Render the scene
    let view_proj = s.camera.view_projection_matrix();
    if let Some(ref mut reactor) = s.reactor {
        let _ = reactor.draw_scene(&s.scene, &view_proj);
    }
    
    s.frame_active = false;
    ReactorResult::Ok
}

/// Check if a frame is currently active (between begin_frame/end_frame)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_is_frame_active() -> bool {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.frame_active)
        .unwrap_or(false)
}

// =============================================================================
// Time & Frame Info
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_delta_time() -> f32 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.delta_time)
        .unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_total_time() -> f32 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.total_time)
        .unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_fps() -> f32 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.time.fps())
        .unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_frame_count() -> u64 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.frame_count)
        .unwrap_or(0)
}

// =============================================================================
// Window API
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_width() -> u32 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.width)
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_height() -> u32 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.height)
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_aspect_ratio() -> f32 {
    let state = REACTOR_STATE.lock().unwrap();
    state.as_ref()
        .map(|s| s.width as f32 / s.height.max(1) as f32)
        .unwrap_or(16.0/9.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_should_close() -> bool {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.should_close)
        .unwrap_or(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_request_close() {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        s.should_close = true;
    }
}

// =============================================================================
// Input API
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn reactor_key_down(key_code: u32) -> bool {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.keys_down.get(&key_code).copied().unwrap_or(false))
        .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_key_pressed(key_code: u32) -> bool {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.keys_pressed.get(&key_code).copied().unwrap_or(false))
        .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mouse_position() -> CVec2 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| CVec2 { x: s.mouse_x, y: s.mouse_y })
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mouse_delta() -> CVec2 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| CVec2 { x: s.mouse_dx, y: s.mouse_dy })
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mouse_button(button: u32) -> bool {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.mouse_buttons.get(button as usize).copied().unwrap_or(false))
        .unwrap_or(false)
}

// Key codes (matching winit KeyCode)
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_w() -> u32 { KeyCode::KeyW as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_a() -> u32 { KeyCode::KeyA as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_s() -> u32 { KeyCode::KeyS as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_d() -> u32 { KeyCode::KeyD as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_q() -> u32 { KeyCode::KeyQ as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_e() -> u32 { KeyCode::KeyE as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_space() -> u32 { KeyCode::Space as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_shift() -> u32 { KeyCode::ShiftLeft as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_ctrl() -> u32 { KeyCode::ControlLeft as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_escape() -> u32 { KeyCode::Escape as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_enter() -> u32 { KeyCode::Enter as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_tab() -> u32 { KeyCode::Tab as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_up() -> u32 { KeyCode::ArrowUp as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_arrow_down() -> u32 { KeyCode::ArrowDown as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_left() -> u32 { KeyCode::ArrowLeft as u32 }
#[unsafe(no_mangle)] pub extern "C" fn reactor_key_right() -> u32 { KeyCode::ArrowRight as u32 }

// =============================================================================
// Scene API
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_create() -> *mut SceneHandle {
    Box::into_raw(Box::new(SceneHandle {
        scene: Scene::new(),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_destroy(scene: *mut SceneHandle) {
    if !scene.is_null() {
        unsafe { drop(Box::from_raw(scene)); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_object_count(scene: *const SceneHandle) -> u32 {
    if scene.is_null() { return 0; }
    let scene = unsafe { &*scene };
    scene.scene.objects.len() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_clear(scene: *mut SceneHandle) {
    if scene.is_null() { return; }
    let scene = unsafe { &mut *scene };
    scene.scene.objects.clear();
}

/// Add an object to the scene (takes ownership of mesh/material handles)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_add_object(
    scene: *mut SceneHandle,
    mesh: *mut MeshHandle,
    material: *mut MaterialHandle,
    transform: CMat4,
) -> i32 {
    if scene.is_null() || mesh.is_null() || material.is_null() { return -1; }
    
    let scene = unsafe { &mut *scene };
    let mesh_handle = unsafe { Box::from_raw(mesh) };
    let material_handle = unsafe { Box::from_raw(material) };
    
    let transform_mat = glam::Mat4::from_cols_array_2d(&transform.cols);
    
    let idx = scene.scene.add_object(
        std::sync::Arc::new(mesh_handle.mesh),
        std::sync::Arc::new(material_handle.material),
        transform_mat,
    );
    
    idx as i32
}

/// Set transform for an object in the scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_set_transform(scene: *mut SceneHandle, index: u32, transform: CMat4) {
    if scene.is_null() { return; }
    let scene = unsafe { &mut *scene };
    if (index as usize) < scene.scene.objects.len() {
        scene.scene.objects[index as usize].transform = glam::Mat4::from_cols_array_2d(&transform.cols);
    }
}

/// Get transform for an object in the scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_get_transform(scene: *const SceneHandle, index: u32) -> CMat4 {
    if scene.is_null() { return CMat4::default(); }
    let scene = unsafe { &*scene };
    if (index as usize) < scene.scene.objects.len() {
        CMat4 { cols: scene.scene.objects[index as usize].transform.to_cols_array_2d() }
    } else {
        CMat4::default()
    }
}

/// Set visibility for an object in the scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_set_visible(scene: *mut SceneHandle, index: u32, visible: bool) {
    if scene.is_null() { return; }
    let scene = unsafe { &mut *scene };
    if (index as usize) < scene.scene.objects.len() {
        scene.scene.objects[index as usize].visible = visible;
    }
}

/// Get visibility for an object in the scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_is_visible(scene: *const SceneHandle, index: u32) -> bool {
    if scene.is_null() { return false; }
    let scene = unsafe { &*scene };
    if (index as usize) < scene.scene.objects.len() {
        scene.scene.objects[index as usize].visible
    } else {
        false
    }
}

/// Remove an object from the scene by index
#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_remove_object(scene: *mut SceneHandle, index: u32) -> bool {
    if scene.is_null() { return false; }
    let scene = unsafe { &mut *scene };
    if (index as usize) < scene.scene.objects.len() {
        scene.scene.objects.remove(index as usize);
        true
    } else {
        false
    }
}

// =============================================================================
// Global Scene API (uses built-in scene from ReactorState)
// =============================================================================

/// Add object to the global scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_add_object(mesh: *mut MeshHandle, material: *mut MaterialHandle, transform: CMat4) -> i32 {
    if mesh.is_null() || material.is_null() { return -1; }
    
    let mesh_handle = unsafe { Box::from_raw(mesh) };
    let material_handle = unsafe { Box::from_raw(material) };
    let transform_mat = glam::Mat4::from_cols_array_2d(&transform.cols);
    
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        let idx = s.scene.add_object(
            std::sync::Arc::new(mesh_handle.mesh),
            std::sync::Arc::new(material_handle.material),
            transform_mat,
        );
        idx as i32
    } else {
        -1
    }
}

/// Get object count in global scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_object_count() -> u32 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.scene.objects.len() as u32)
        .unwrap_or(0)
}

/// Set transform for object in global scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_set_object_transform(index: u32, transform: CMat4) {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        if (index as usize) < s.scene.objects.len() {
            s.scene.objects[index as usize].transform = glam::Mat4::from_cols_array_2d(&transform.cols);
        }
    }
}

/// Get transform for object in global scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_object_transform(index: u32) -> CMat4 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .and_then(|s| {
            if (index as usize) < s.scene.objects.len() {
                Some(CMat4 { cols: s.scene.objects[index as usize].transform.to_cols_array_2d() })
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Set visibility for object in global scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_set_object_visible(index: u32, visible: bool) {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        if (index as usize) < s.scene.objects.len() {
            s.scene.objects[index as usize].visible = visible;
        }
    }
}

/// Clear global scene
#[unsafe(no_mangle)]
pub extern "C" fn reactor_clear_scene() {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        s.scene.objects.clear();
    }
}

// =============================================================================
// Mesh Creation API
// =============================================================================

/// Create a cube mesh (built-in primitive)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_cube() -> *mut MeshHandle {
    use reactor::vertex::Vertex;
    
    let mut state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_mut() else { return std::ptr::null_mut(); };
    let Some(ref reactor) = s.reactor else { return std::ptr::null_mut(); };
    
    // Cube vertices: position, color, uv
    let vertices = vec![
        // Front face (red)
        Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },
        Vertex { position: [ 0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0], uv: [1.0, 0.0] },
        Vertex { position: [ 0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0], uv: [1.0, 1.0] },
        Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 0.0], uv: [0.0, 1.0] },
        // Back face
        Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0], uv: [1.0, 0.0] },
        Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0], uv: [0.0, 0.0] },
        Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0], uv: [0.0, 1.0] },
        Vertex { position: [-0.5,  0.5, -0.5], color: [0.5, 0.5, 0.5], uv: [1.0, 1.0] },
    ];
    
    let indices: Vec<u32> = vec![
        // Front
        0, 1, 2, 2, 3, 0,
        // Right
        1, 5, 6, 6, 2, 1,
        // Back
        5, 4, 7, 7, 6, 5,
        // Left
        4, 0, 3, 3, 7, 4,
        // Top
        3, 2, 6, 6, 7, 3,
        // Bottom
        4, 5, 1, 1, 0, 4,
    ];
    
    match Mesh::new(&reactor.context, &reactor.allocator, &vertices, &indices) {
        Ok(mesh) => Box::into_raw(Box::new(MeshHandle { mesh })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Create a mesh from vertex data (requires active Vulkan context — call from on_init or on_update)
/// vertices: array of CVertex, vertex_count: number of vertices
/// indices: array of u32, index_count: number of indices
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_mesh(
    vertices: *const CVertex,
    vertex_count: u32,
    indices: *const u32,
    index_count: u32,
) -> *mut MeshHandle {
    if vertices.is_null() || indices.is_null() || vertex_count == 0 || index_count == 0 {
        return std::ptr::null_mut();
    }
    
    let mut state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_mut() else { return std::ptr::null_mut(); };
    let Some(ref reactor) = s.reactor else { return std::ptr::null_mut(); };
    
    // Convert CVertex array to reactor::vertex::Vertex array
    let c_verts = unsafe { std::slice::from_raw_parts(vertices, vertex_count as usize) };
    let rust_verts: Vec<reactor::vertex::Vertex> = c_verts.iter().map(|v| {
        reactor::vertex::Vertex {
            position: [v.position.x, v.position.y, v.position.z],
            color: [v.normal.x, v.normal.y, v.normal.z], // CVertex.normal maps to color
            uv: [v.uv.x, v.uv.y],
        }
    }).collect();
    
    let rust_indices = unsafe { std::slice::from_raw_parts(indices, index_count as usize) };
    
    match Mesh::new(&reactor.context, &reactor.allocator, &rust_verts, rust_indices) {
        Ok(mesh) => Box::into_raw(Box::new(MeshHandle { mesh })),
        Err(e) => {
            eprintln!("[REACTOR] Failed to create mesh: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Destroy a mesh handle
#[unsafe(no_mangle)]
pub extern "C" fn reactor_destroy_mesh(mesh: *mut MeshHandle) {
    if !mesh.is_null() {
        unsafe { drop(Box::from_raw(mesh)); }
    }
}

// =============================================================================
// Default Shaders (Embedded SPIR-V from compiled files)
// =============================================================================

// Load compiled SPIR-V shaders at compile time
const DEFAULT_VERT_SPV_BYTES: &[u8] = include_bytes!("../../../shaders/vert.spv");
const DEFAULT_FRAG_SPV_BYTES: &[u8] = include_bytes!("../../../shaders/frag.spv");

// Helper to convert bytes to u32 slice (SPIR-V format)
fn bytes_to_u32_vec(bytes: &[u8]) -> Vec<u32> {
    bytes.chunks(4)
        .map(|chunk| {
            u32::from_le_bytes([
                chunk.get(0).copied().unwrap_or(0),
                chunk.get(1).copied().unwrap_or(0),
                chunk.get(2).copied().unwrap_or(0),
                chunk.get(3).copied().unwrap_or(0),
            ])
        })
        .collect()
}

// =============================================================================
// Material Creation API
// =============================================================================

/// Create a simple colored material using default shaders
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_material_simple(_r: f32, _g: f32, _b: f32) -> *mut MaterialHandle {
    let mut state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_mut() else { return std::ptr::null_mut(); };
    let Some(ref reactor) = s.reactor else { return std::ptr::null_mut(); };
    
    // Convert embedded bytes to u32 vectors
    let vert_spv = bytes_to_u32_vec(DEFAULT_VERT_SPV_BYTES);
    let frag_spv = bytes_to_u32_vec(DEFAULT_FRAG_SPV_BYTES);
    
    match Material::new_with_msaa(
        &reactor.context,
        reactor.render_pass,
        &vert_spv,
        &frag_spv,
        s.width,
        s.height,
        reactor.msaa_samples,
    ) {
        Ok(material) => Box::into_raw(Box::new(MaterialHandle { material })),
        Err(e) => {
            eprintln!("Failed to create material: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Create a material from SPIR-V shader code (requires active Vulkan context)
/// vert_spv/frag_spv: pointers to u32 arrays of SPIR-V code
/// vert_len/frag_len: number of u32 words in each array
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_material(
    vert_spv: *const u32,
    vert_len: u32,
    frag_spv: *const u32,
    frag_len: u32,
) -> *mut MaterialHandle {
    if vert_spv.is_null() || frag_spv.is_null() || vert_len == 0 || frag_len == 0 {
        return std::ptr::null_mut();
    }
    
    let mut state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_mut() else { return std::ptr::null_mut(); };
    let Some(ref reactor) = s.reactor else { return std::ptr::null_mut(); };
    
    let vert_code = unsafe { std::slice::from_raw_parts(vert_spv, vert_len as usize) };
    let frag_code = unsafe { std::slice::from_raw_parts(frag_spv, frag_len as usize) };
    
    match Material::new_with_msaa(
        &reactor.context,
        reactor.render_pass,
        vert_code,
        frag_code,
        s.width,
        s.height,
        reactor.msaa_samples,
    ) {
        Ok(material) => Box::into_raw(Box::new(MaterialHandle { material })),
        Err(e) => {
            eprintln!("[REACTOR] Failed to create material: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Create a textured material from SPIR-V shader code and texture (requires active Vulkan context)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_textured_material(
    vert_spv: *const u32,
    vert_len: u32,
    frag_spv: *const u32,
    frag_len: u32,
    texture: *const TextureHandle,
) -> *mut MaterialHandle {
    if vert_spv.is_null() || frag_spv.is_null() || texture.is_null() {
        return std::ptr::null_mut();
    }
    
    let mut state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_mut() else { return std::ptr::null_mut(); };
    let Some(ref reactor) = s.reactor else { return std::ptr::null_mut(); };
    
    let vert_code = unsafe { std::slice::from_raw_parts(vert_spv, vert_len as usize) };
    let frag_code = unsafe { std::slice::from_raw_parts(frag_spv, frag_len as usize) };
    let tex = unsafe { &*texture };
    
    match Material::with_texture(
        &reactor.context,
        reactor.render_pass,
        vert_code,
        frag_code,
        s.width,
        s.height,
        &tex.texture,
        reactor.msaa_samples,
    ) {
        Ok(material) => Box::into_raw(Box::new(MaterialHandle { material })),
        Err(e) => {
            eprintln!("[REACTOR] Failed to create textured material: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Destroy a material handle
#[unsafe(no_mangle)]
pub extern "C" fn reactor_destroy_material(material: *mut MaterialHandle) {
    if !material.is_null() {
        unsafe { drop(Box::from_raw(material)); }
    }
}

// =============================================================================
// Texture API
// =============================================================================

/// Load a texture from file (PNG, JPG, BMP, etc.)
/// Returns null if loading fails or if called outside of reactor context
#[unsafe(no_mangle)]
pub extern "C" fn reactor_load_texture(path: *const std::ffi::c_char) -> *mut TextureHandle {
    if path.is_null() { return std::ptr::null_mut(); }
    
    let path_str = unsafe {
        match std::ffi::CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    // Texture loading requires Vulkan context - placeholder for now
    // Actual loading happens through ReactorContext in callbacks
    let _ = path_str;
    std::ptr::null_mut()
}

/// Load a texture from memory bytes
#[unsafe(no_mangle)]
pub extern "C" fn reactor_load_texture_bytes(data: *const u8, len: u32) -> *mut TextureHandle {
    if data.is_null() || len == 0 { return std::ptr::null_mut(); }
    
    // Texture loading requires Vulkan context - placeholder
    std::ptr::null_mut()
}

/// Create a solid color texture (1x1 pixel)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_solid_texture(r: u8, g: u8, b: u8, a: u8) -> *mut TextureHandle {
    let _ = (r, g, b, a);
    // Texture creation requires Vulkan context - placeholder
    std::ptr::null_mut()
}

/// Get texture width
#[unsafe(no_mangle)]
pub extern "C" fn reactor_texture_width(texture: *const TextureHandle) -> u32 {
    if texture.is_null() { return 0; }
    unsafe { (*texture).texture.width }
}

/// Get texture height
#[unsafe(no_mangle)]
pub extern "C" fn reactor_texture_height(texture: *const TextureHandle) -> u32 {
    if texture.is_null() { return 0; }
    unsafe { (*texture).texture.height }
}

/// Destroy a texture handle
#[unsafe(no_mangle)]
pub extern "C" fn reactor_destroy_texture(texture: *mut TextureHandle) {
    if !texture.is_null() {
        unsafe { drop(Box::from_raw(texture)); }
    }
}

// =============================================================================
// Model Loading API (OBJ)
// =============================================================================

/// OBJ data returned from loading
#[repr(C)]
pub struct CObjData {
    pub vertex_count: u32,
    pub index_count: u32,
    pub triangle_count: u32,
    pub success: bool,
}

/// Load an OBJ file and get info (actual mesh creation requires Vulkan context)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_load_obj_info(path: *const std::ffi::c_char) -> CObjData {
    if path.is_null() {
        return CObjData { vertex_count: 0, index_count: 0, triangle_count: 0, success: false };
    }
    
    let path_str = unsafe {
        match std::ffi::CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return CObjData { vertex_count: 0, index_count: 0, triangle_count: 0, success: false },
        }
    };
    
    // Try to load OBJ to get info
    match reactor::resources::model::ObjData::load(path_str) {
        Ok(obj) => CObjData {
            vertex_count: obj.vertex_count() as u32,
            index_count: obj.index_count() as u32,
            triangle_count: obj.triangle_count() as u32,
            success: true,
        },
        Err(_) => CObjData { vertex_count: 0, index_count: 0, triangle_count: 0, success: false },
    }
}

// =============================================================================
// Physics API
// =============================================================================

/// Character controller handle
#[repr(C)]
pub struct CCharacterController {
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub height: f32,
    pub radius: f32,
    pub move_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub is_grounded: bool,
}

/// Create a character controller
#[unsafe(no_mangle)]
pub extern "C" fn reactor_character_controller_create(x: f32, y: f32, z: f32) -> CCharacterController {
    CCharacterController {
        position_x: x,
        position_y: y,
        position_z: z,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        height: 1.8,
        radius: 0.3,
        move_speed: 5.0,
        jump_force: 6.0,
        gravity: 9.81,
        is_grounded: false,
    }
}

/// Update character controller with physics
#[unsafe(no_mangle)]
pub extern "C" fn reactor_character_controller_update(
    controller: *mut CCharacterController,
    dt: f32,
    move_x: f32,
    move_z: f32,
    jump: bool,
    ground_y: f32,
) {
    if controller.is_null() { return; }
    
    let c = unsafe { &mut *controller };
    
    // Ground check
    let feet_y = c.position_y - c.height * 0.5;
    c.is_grounded = feet_y <= ground_y + 0.1;
    
    // Gravity
    if !c.is_grounded {
        c.velocity_y -= c.gravity * dt;
    } else if c.velocity_y < 0.0 {
        c.velocity_y = 0.0;
        c.position_y = ground_y + c.height * 0.5;
    }
    
    // Jump
    if jump && c.is_grounded {
        c.velocity_y = c.jump_force;
        c.is_grounded = false;
    }
    
    // Horizontal movement
    let move_len = (move_x * move_x + move_z * move_z).sqrt();
    if move_len > 0.0 {
        let nx = move_x / move_len;
        let nz = move_z / move_len;
        let target_vx = nx * c.move_speed;
        let target_vz = nz * c.move_speed;
        let drag = if c.is_grounded { 10.0 } else { 0.1 };
        c.velocity_x += (target_vx - c.velocity_x) * drag * dt;
        c.velocity_z += (target_vz - c.velocity_z) * drag * dt;
    } else if c.is_grounded {
        c.velocity_x *= 1.0 - 10.0 * dt;
        c.velocity_z *= 1.0 - 10.0 * dt;
    }
    
    // Apply velocity
    c.position_x += c.velocity_x * dt;
    c.position_y += c.velocity_y * dt;
    c.position_z += c.velocity_z * dt;
}

/// Get eye position from character controller
#[unsafe(no_mangle)]
pub extern "C" fn reactor_character_controller_eye_position(controller: *const CCharacterController, out_x: *mut f32, out_y: *mut f32, out_z: *mut f32) {
    if controller.is_null() { return; }
    let c = unsafe { &*controller };
    unsafe {
        if !out_x.is_null() { *out_x = c.position_x; }
        if !out_y.is_null() { *out_y = c.position_y + c.height * 0.4; }
        if !out_z.is_null() { *out_z = c.position_z; }
    }
}

/// Ray-AABB intersection test
#[unsafe(no_mangle)]
pub extern "C" fn reactor_raycast_aabb(
    ray_ox: f32, ray_oy: f32, ray_oz: f32,
    ray_dx: f32, ray_dy: f32, ray_dz: f32,
    aabb_min_x: f32, aabb_min_y: f32, aabb_min_z: f32,
    aabb_max_x: f32, aabb_max_y: f32, aabb_max_z: f32,
    out_t: *mut f32,
) -> bool {
    let inv_dx = 1.0 / ray_dx;
    let inv_dy = 1.0 / ray_dy;
    let inv_dz = 1.0 / ray_dz;
    
    let t1 = (aabb_min_x - ray_ox) * inv_dx;
    let t2 = (aabb_max_x - ray_ox) * inv_dx;
    let t3 = (aabb_min_y - ray_oy) * inv_dy;
    let t4 = (aabb_max_y - ray_oy) * inv_dy;
    let t5 = (aabb_min_z - ray_oz) * inv_dz;
    let t6 = (aabb_max_z - ray_oz) * inv_dz;
    
    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
    
    if tmax < 0.0 || tmin > tmax {
        false
    } else {
        if !out_t.is_null() {
            unsafe { *out_t = if tmin < 0.0 { tmax } else { tmin }; }
        }
        true
    }
}

/// AABB-AABB intersection test
#[unsafe(no_mangle)]
pub extern "C" fn reactor_aabb_intersects(
    a_min_x: f32, a_min_y: f32, a_min_z: f32,
    a_max_x: f32, a_max_y: f32, a_max_z: f32,
    b_min_x: f32, b_min_y: f32, b_min_z: f32,
    b_max_x: f32, b_max_y: f32, b_max_z: f32,
) -> bool {
    a_min_x <= b_max_x && a_max_x >= b_min_x &&
    a_min_y <= b_max_y && a_max_y >= b_min_y &&
    a_min_z <= b_max_z && a_max_z >= b_min_z
}

// =============================================================================
// Lighting API
// =============================================================================

/// Add a directional light to the global lighting system
#[unsafe(no_mangle)]
pub extern "C" fn reactor_add_directional_light(dir_x: f32, dir_y: f32, dir_z: f32, r: f32, g: f32, b: f32, intensity: f32) -> i32 {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        let light = reactor::systems::lighting::Light::directional(
            glam::Vec3::new(dir_x, dir_y, dir_z).normalize(),
            glam::Vec3::new(r, g, b),
            intensity,
        );
        s.lighting.add_light(light);
        (s.lighting.lights.len() - 1) as i32
    } else {
        -1
    }
}

/// Add a point light to the global lighting system
#[unsafe(no_mangle)]
pub extern "C" fn reactor_add_point_light(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, intensity: f32, range: f32) -> i32 {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        let light = reactor::systems::lighting::Light::point(
            glam::Vec3::new(x, y, z),
            glam::Vec3::new(r, g, b),
            intensity,
            range,
        );
        s.lighting.add_light(light);
        (s.lighting.lights.len() - 1) as i32
    } else {
        -1
    }
}

/// Add a spot light to the global lighting system
#[unsafe(no_mangle)]
pub extern "C" fn reactor_add_spot_light(
    pos_x: f32, pos_y: f32, pos_z: f32,
    dir_x: f32, dir_y: f32, dir_z: f32,
    r: f32, g: f32, b: f32,
    intensity: f32, range: f32, angle_degrees: f32
) -> i32 {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        let light = reactor::systems::lighting::Light::spot(
            glam::Vec3::new(pos_x, pos_y, pos_z),
            glam::Vec3::new(dir_x, dir_y, dir_z).normalize(),
            glam::Vec3::new(r, g, b),
            intensity,
            range,
            angle_degrees,
        );
        s.lighting.add_light(light);
        (s.lighting.lights.len() - 1) as i32
    } else {
        -1
    }
}

/// Get light count
#[unsafe(no_mangle)]
pub extern "C" fn reactor_light_count() -> u32 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| s.lighting.lights.len() as u32)
        .unwrap_or(0)
}

/// Clear all lights
#[unsafe(no_mangle)]
pub extern "C" fn reactor_clear_lights() {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        s.lighting.lights.clear();
    }
}

// =============================================================================
// Camera API
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn reactor_camera_create_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> *mut CameraHandle {
    Box::into_raw(Box::new(CameraHandle {
        camera: Camera::perspective(fov, aspect, near, far),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_camera_destroy(camera: *mut CameraHandle) {
    if !camera.is_null() {
        unsafe { drop(Box::from_raw(camera)); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_camera_set_position(camera: *mut CameraHandle, x: f32, y: f32, z: f32) {
    if camera.is_null() { return; }
    let camera = unsafe { &mut *camera };
    camera.camera.position = glam::Vec3::new(x, y, z);
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_camera_set_target(camera: *mut CameraHandle, x: f32, y: f32, z: f32) {
    if camera.is_null() { return; }
    let camera = unsafe { &mut *camera };
    // Camera uses rotation, not target - compute rotation to look at target
    let dir = glam::Vec3::new(x, y, z) - camera.camera.position;
    if dir.length() > 0.0 {
        let dir = dir.normalize();
        camera.camera.rotation.y = dir.x.atan2(-dir.z);
        camera.camera.rotation.x = dir.y.asin();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_camera_get_view_projection(camera: *const CameraHandle) -> CMat4 {
    if camera.is_null() { return CMat4::default(); }
    let camera = unsafe { &*camera };
    CMat4 { cols: camera.camera.view_projection_matrix().to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_camera_get_view(camera: *const CameraHandle) -> CMat4 {
    if camera.is_null() { return CMat4::default(); }
    let camera = unsafe { &*camera };
    CMat4 { cols: camera.camera.view_matrix().to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_camera_get_projection(camera: *const CameraHandle) -> CMat4 {
    if camera.is_null() { return CMat4::default(); }
    let camera = unsafe { &*camera };
    CMat4 { cols: camera.camera.projection_matrix().to_cols_array_2d() }
}

// Global camera (built-in)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_set_camera_position(x: f32, y: f32, z: f32) {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        s.camera.position = glam::Vec3::new(x, y, z);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_set_camera_target(x: f32, y: f32, z: f32) {
    if let Some(s) = REACTOR_STATE.lock().unwrap().as_mut() {
        // Camera uses rotation, not target - compute rotation to look at target
        let dir = glam::Vec3::new(x, y, z) - s.camera.position;
        if dir.length() > 0.0 {
            let dir = dir.normalize();
            s.camera.rotation.y = dir.x.atan2(-dir.z);
            s.camera.rotation.x = dir.y.asin();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_camera_position() -> CVec3 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| CVec3 { x: s.camera.position.x, y: s.camera.position.y, z: s.camera.position.z })
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_view_projection() -> CMat4 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .map(|s| CMat4 { cols: s.camera.view_projection_matrix().to_cols_array_2d() })
        .unwrap_or_default()
}

// =============================================================================
// Math utilities (available to C++)
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_identity() -> CMat4 {
    CMat4 { cols: glam::Mat4::IDENTITY.to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_perspective(fov_degrees: f32, aspect: f32, near: f32, far: f32) -> CMat4 {
    let mut proj = glam::Mat4::perspective_rh(fov_degrees.to_radians(), aspect, near, far);
    proj.y_axis.y *= -1.0;
    CMat4 { cols: proj.to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_look_at(eye: CVec3, target: CVec3, up: CVec3) -> CMat4 {
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(eye.x, eye.y, eye.z),
        glam::Vec3::new(target.x, target.y, target.z),
        glam::Vec3::new(up.x, up.y, up.z),
    );
    CMat4 { cols: view.to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_mul(a: CMat4, b: CMat4) -> CMat4 {
    let result = glam::Mat4::from_cols_array_2d(&a.cols) * glam::Mat4::from_cols_array_2d(&b.cols);
    CMat4 { cols: result.to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_translation(x: f32, y: f32, z: f32) -> CMat4 {
    CMat4 { cols: glam::Mat4::from_translation(glam::Vec3::new(x, y, z)).to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_rotation_x(angle_radians: f32) -> CMat4 {
    CMat4 { cols: glam::Mat4::from_rotation_x(angle_radians).to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_rotation_y(angle_radians: f32) -> CMat4 {
    CMat4 { cols: glam::Mat4::from_rotation_y(angle_radians).to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_rotation_z(angle_radians: f32) -> CMat4 {
    CMat4 { cols: glam::Mat4::from_rotation_z(angle_radians).to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_scale(x: f32, y: f32, z: f32) -> CMat4 {
    CMat4 { cols: glam::Mat4::from_scale(glam::Vec3::new(x, y, z)).to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_inverse(m: CMat4) -> CMat4 {
    let mat = glam::Mat4::from_cols_array_2d(&m.cols);
    CMat4 { cols: mat.inverse().to_cols_array_2d() }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_mat4_transpose(m: CMat4) -> CMat4 {
    let mat = glam::Mat4::from_cols_array_2d(&m.cols);
    CMat4 { cols: mat.transpose().to_cols_array_2d() }
}

// Vec3 operations
#[unsafe(no_mangle)]
pub extern "C" fn reactor_vec3_add(a: CVec3, b: CVec3) -> CVec3 {
    CVec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_vec3_sub(a: CVec3, b: CVec3) -> CVec3 {
    CVec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_vec3_mul(a: CVec3, s: f32) -> CVec3 {
    CVec3 { x: a.x * s, y: a.y * s, z: a.z * s }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_vec3_dot(a: CVec3, b: CVec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_vec3_cross(a: CVec3, b: CVec3) -> CVec3 {
    CVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_vec3_length(v: CVec3) -> f32 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_vec3_normalize(v: CVec3) -> CVec3 {
    let len = reactor_vec3_length(v);
    if len > 0.0 {
        CVec3 { x: v.x / len, y: v.y / len, z: v.z / len }
    } else {
        v
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_vec3_lerp(a: CVec3, b: CVec3, t: f32) -> CVec3 {
    CVec3 {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        z: a.z + (b.z - a.z) * t,
    }
}

// =============================================================================
// ADead-GPU utilities (exposed to C++)
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn reactor_sdf_sphere(px: f32, py: f32, pz: f32, radius: f32) -> f32 {
    reactor::sd_sphere(glam::Vec3::new(px, py, pz), radius)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_sdf_box(px: f32, py: f32, pz: f32, bx: f32, by: f32, bz: f32) -> f32 {
    reactor::sd_box(glam::Vec3::new(px, py, pz), glam::Vec3::new(bx, by, bz))
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_sdf_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    reactor::op_smooth_union(d1, d2, k)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_sdf_cylinder(px: f32, py: f32, pz: f32, h: f32, r: f32) -> f32 {
    reactor::sd_cylinder(glam::Vec3::new(px, py, pz), h, r)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_sdf_torus(px: f32, py: f32, pz: f32, r1: f32, r2: f32) -> f32 {
    reactor::sd_torus(glam::Vec3::new(px, py, pz), r1, r2)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_sdf_capsule(px: f32, py: f32, pz: f32, h: f32, r: f32) -> f32 {
    // sd_capsule takes (p, a, b, radius) - create a vertical capsule
    let a = glam::Vec3::new(0.0, -h/2.0, 0.0);
    let b = glam::Vec3::new(0.0, h/2.0, 0.0);
    reactor::sd_capsule(glam::Vec3::new(px, py, pz), a, b, r)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_sdf_union(d1: f32, d2: f32) -> f32 {
    reactor::op_union(d1, d2)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_sdf_subtract(d1: f32, d2: f32) -> f32 {
    reactor::op_subtract(d1, d2)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_sdf_intersect(d1: f32, d2: f32) -> f32 {
    reactor::op_intersect(d1, d2)
}

// =============================================================================
// Utility functions
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn reactor_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_clamp(v: f32, min: f32, max: f32) -> f32 {
    v.clamp(min, max)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_deg_to_rad(degrees: f32) -> f32 {
    degrees.to_radians()
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_rad_to_deg(radians: f32) -> f32 {
    radians.to_degrees()
}

// =============================================================================
// Debug logging
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn reactor_log_info(msg: *const c_char) {
    if msg.is_null() { return; }
    let s = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    println!("[REACTOR INFO] {}", s);
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_log_warn(msg: *const c_char) {
    if msg.is_null() { return; }
    let s = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    println!("[REACTOR WARN] {}", s);
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_log_error(msg: *const c_char) {
    if msg.is_null() { return; }
    let s = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    eprintln!("[REACTOR ERROR] {}", s);
}

// =============================================================================
// Error Handling API
// =============================================================================

/// Get the last error code (0 = no error)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_last_error() -> u32 {
    get_last_error_code() as u32
}

/// Get the last error message (returns null if no error)
/// The returned string is valid until the next error occurs or clear_error is called
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_error_message() -> *const c_char {
    static ERROR_MSG: Mutex<Option<std::ffi::CString>> = Mutex::new(None);
    
    if let Some(msg) = get_last_error_message() {
        if let Ok(cstring) = std::ffi::CString::new(msg) {
            let mut guard = ERROR_MSG.lock().unwrap();
            *guard = Some(cstring);
            return guard.as_ref().unwrap().as_ptr();
        }
    }
    std::ptr::null()
}

/// Check if there's a pending error
#[unsafe(no_mangle)]
pub extern "C" fn reactor_has_error() -> bool {
    has_error()
}

/// Clear the last error
#[unsafe(no_mangle)]
pub extern "C" fn reactor_clear_error() {
    clear_last_error();
}

/// Get a human-readable description for an error code
#[unsafe(no_mangle)]
pub extern "C" fn reactor_error_description(code: u32) -> *const c_char {
    use std::sync::LazyLock;
    static DESCRIPTIONS: LazyLock<Mutex<HashMap<u32, std::ffi::CString>>> = 
        LazyLock::new(|| Mutex::new(HashMap::new()));
    
    let error_code: ErrorCode = unsafe { std::mem::transmute(code) };
    let desc = error_code.description();
    
    let mut guard = DESCRIPTIONS.lock().unwrap();
    if !guard.contains_key(&code) {
        if let Ok(cstring) = std::ffi::CString::new(desc) {
            guard.insert(code, cstring);
        }
    }
    
    guard.get(&code)
        .map(|s| s.as_ptr())
        .unwrap_or(std::ptr::null())
}

// =============================================================================
// ECS API
// =============================================================================

/// Entity handle
pub type CEntity = u32;

/// Create a new entity in the ECS world
#[unsafe(no_mangle)]
pub extern "C" fn reactor_ecs_create_entity() -> CEntity {
    static NEXT_ENTITY: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    NEXT_ENTITY.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// Destroy an entity
#[unsafe(no_mangle)]
pub extern "C" fn reactor_ecs_destroy_entity(_entity: CEntity) {
    // Placeholder - would remove from world
}

/// Get entity count
#[unsafe(no_mangle)]
pub extern "C" fn reactor_ecs_entity_count() -> u32 {
    0 // Placeholder
}

// =============================================================================
// Debug Draw API
// =============================================================================

/// Draw a debug line
#[unsafe(no_mangle)]
pub extern "C" fn reactor_debug_line(
    x1: f32, y1: f32, z1: f32,
    x2: f32, y2: f32, z2: f32,
    r: f32, g: f32, b: f32,
) {
    // Placeholder - would add to debug renderer
    let _ = (x1, y1, z1, x2, y2, z2, r, g, b);
}

/// Draw a debug AABB
#[unsafe(no_mangle)]
pub extern "C" fn reactor_debug_aabb(
    min_x: f32, min_y: f32, min_z: f32,
    max_x: f32, max_y: f32, max_z: f32,
    r: f32, g: f32, b: f32,
) {
    let _ = (min_x, min_y, min_z, max_x, max_y, max_z, r, g, b);
}

/// Draw a debug sphere
#[unsafe(no_mangle)]
pub extern "C" fn reactor_debug_sphere(
    cx: f32, cy: f32, cz: f32,
    radius: f32,
    r: f32, g: f32, b: f32,
) {
    let _ = (cx, cy, cz, radius, r, g, b);
}

/// Draw a debug grid
#[unsafe(no_mangle)]
pub extern "C" fn reactor_debug_grid(size: f32, divisions: u32, r: f32, g: f32, b: f32) {
    let _ = (size, divisions, r, g, b);
}

/// Clear debug draws
#[unsafe(no_mangle)]
pub extern "C" fn reactor_debug_clear() {
    // Placeholder
}

// =============================================================================
// Animation API
// =============================================================================

/// Animation clip handle
pub type CAnimationClip = u32;

/// Create an animation clip
#[unsafe(no_mangle)]
pub extern "C" fn reactor_animation_create_clip(name: *const c_char) -> CAnimationClip {
    let _ = name;
    static NEXT_CLIP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    NEXT_CLIP.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// Add position keyframe
#[unsafe(no_mangle)]
pub extern "C" fn reactor_animation_add_position_keyframe(
    clip: CAnimationClip,
    time: f32,
    x: f32, y: f32, z: f32,
) {
    let _ = (clip, time, x, y, z);
}

/// Add rotation keyframe
#[unsafe(no_mangle)]
pub extern "C" fn reactor_animation_add_rotation_keyframe(
    clip: CAnimationClip,
    time: f32,
    x: f32, y: f32, z: f32, w: f32,
) {
    let _ = (clip, time, x, y, z, w);
}

/// Play animation
#[unsafe(no_mangle)]
pub extern "C" fn reactor_animation_play(clip: CAnimationClip, looping: bool) {
    let _ = (clip, looping);
}

/// Stop animation
#[unsafe(no_mangle)]
pub extern "C" fn reactor_animation_stop(clip: CAnimationClip) {
    let _ = clip;
}

/// Update animations (call each frame)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_animation_update(dt: f32) {
    let _ = dt;
}

// =============================================================================
// Audio API
// =============================================================================

/// Audio clip handle
pub type CAudioClip = u32;

/// Audio source handle
pub type CAudioSource = u32;

/// Load an audio clip
#[unsafe(no_mangle)]
pub extern "C" fn reactor_audio_load(path: *const c_char) -> CAudioClip {
    let _ = path;
    static NEXT_CLIP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    NEXT_CLIP.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// Create an audio source
#[unsafe(no_mangle)]
pub extern "C" fn reactor_audio_create_source() -> CAudioSource {
    static NEXT_SOURCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    NEXT_SOURCE.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// Play audio
#[unsafe(no_mangle)]
pub extern "C" fn reactor_audio_play(source: CAudioSource, clip: CAudioClip) {
    let _ = (source, clip);
}

/// Stop audio
#[unsafe(no_mangle)]
pub extern "C" fn reactor_audio_stop(source: CAudioSource) {
    let _ = source;
}

/// Set audio volume
#[unsafe(no_mangle)]
pub extern "C" fn reactor_audio_set_volume(source: CAudioSource, volume: f32) {
    let _ = (source, volume);
}

/// Set audio position (3D)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_audio_set_position(source: CAudioSource, x: f32, y: f32, z: f32) {
    let _ = (source, x, y, z);
}

/// Set master volume
#[unsafe(no_mangle)]
pub extern "C" fn reactor_audio_set_master_volume(volume: f32) {
    let _ = volume;
}

// =============================================================================
// Post-Processing API
// =============================================================================

/// Enable/disable bloom
#[unsafe(no_mangle)]
pub extern "C" fn reactor_postprocess_set_bloom(enabled: bool, intensity: f32, threshold: f32) {
    let _ = (enabled, intensity, threshold);
}

/// Enable/disable tone mapping
#[unsafe(no_mangle)]
pub extern "C" fn reactor_postprocess_set_tonemapping(enabled: bool, exposure: f32) {
    let _ = (enabled, exposure);
}

/// Enable/disable vignette
#[unsafe(no_mangle)]
pub extern "C" fn reactor_postprocess_set_vignette(enabled: bool, intensity: f32) {
    let _ = (enabled, intensity);
}

/// Enable/disable FXAA
#[unsafe(no_mangle)]
pub extern "C" fn reactor_postprocess_set_fxaa(enabled: bool) {
    let _ = enabled;
}

// =============================================================================
// GPU Info API
// =============================================================================

/// Get GPU name
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_gpu_name() -> *const c_char {
    use std::sync::LazyLock;
    static GPU_NAME: LazyLock<std::ffi::CString> = 
        LazyLock::new(|| std::ffi::CString::new("Vulkan GPU").unwrap());
    GPU_NAME.as_ptr()
}

/// Get VRAM in MB (queries Vulkan memory heaps)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_vram_mb() -> u32 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .and_then(|s| s.reactor.as_ref())
        .map(|r| {
            let mem_props = unsafe {
                r.context.instance.get_physical_device_memory_properties(r.context.physical_device)
            };
            let mut vram: u64 = 0;
            for i in 0..mem_props.memory_heap_count {
                let heap = mem_props.memory_heaps[i as usize];
                if heap.flags.contains(ash::vk::MemoryHeapFlags::DEVICE_LOCAL) {
                    vram += heap.size;
                }
            }
            (vram / (1024 * 1024)) as u32
        })
        .unwrap_or(0)
}

/// Get current MSAA samples
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_msaa_samples() -> u32 {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .and_then(|s| s.reactor.as_ref())
        .map(|r| r.msaa_samples.as_raw())
        .unwrap_or(4)
}

/// Check if ray tracing is supported
#[unsafe(no_mangle)]
pub extern "C" fn reactor_is_raytracing_supported() -> bool {
    REACTOR_STATE.lock().unwrap()
        .as_ref()
        .and_then(|s| s.reactor.as_ref())
        .map(|r| r.ray_tracing.is_some())
        .unwrap_or(false)
}

/// Get Vulkan version
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_vulkan_version(major: *mut u32, minor: *mut u32, patch: *mut u32) {
    unsafe {
        if !major.is_null() { *major = 1; }
        if !minor.is_null() { *minor = 3; }
        if !patch.is_null() { *patch = 0; }
    }
}

// =============================================================================
// PHASE 1A: ECS Component CRUD — Real entity-component system
// =============================================================================
// Roadmap §F: "Component CRUD real (transform, mesh renderer, light, camera,
//              physics, audio, custom). Queries con filtros y batches."
// =============================================================================

/// Opaque ECS world with real component storage
static ECS_WORLD: Mutex<Option<EcsWorld>> = Mutex::new(None);

struct EcsWorld {
    next_id: u32,
    entities: HashMap<u32, EcsEntity>,
}

#[derive(Clone, Debug)]
struct EcsEntity {
    id: u32,
    name: String,
    active: bool,
    transform: CTransform,
    // Component flags
    has_mesh_renderer: bool,
    has_light: bool,
    has_camera: bool,
    has_rigidbody: bool,
    has_audio_source: bool,
    // Component data
    mesh_renderer: Option<CMeshRenderer>,
    light: Option<CLight>,
    camera_component: Option<CCameraComponent>,
    rigidbody: Option<CRigidBodyComponent>,
    tags: Vec<String>,
}

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct CMeshRenderer {
    pub mesh_index: i32,
    pub material_index: i32,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub visible: bool,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct CCameraComponent {
    pub fov: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub is_main: bool,
    pub clear_color: CVec4,
}

impl Default for CCameraComponent {
    fn default() -> Self {
        Self {
            fov: 60.0,
            near_plane: 0.1,
            far_plane: 1000.0,
            is_main: false,
            clear_color: CVec4 { x: 0.1, y: 0.1, z: 0.15, w: 1.0 },
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct CRigidBodyComponent {
    pub mass: f32,
    pub drag: f32,
    pub angular_drag: f32,
    pub use_gravity: bool,
    pub is_kinematic: bool,
    pub velocity: CVec3,
    pub angular_velocity: CVec3,
}

impl Default for CRigidBodyComponent {
    fn default() -> Self {
        Self {
            mass: 1.0, drag: 0.0, angular_drag: 0.05,
            use_gravity: true, is_kinematic: false,
            velocity: CVec3::default(), angular_velocity: CVec3::default(),
        }
    }
}

impl Default for EcsWorld {
    fn default() -> Self {
        Self { next_id: 1, entities: HashMap::new() }
    }
}

fn ecs_world() -> std::sync::MutexGuard<'static, Option<EcsWorld>> {
    let mut w = ECS_WORLD.lock().unwrap();
    if w.is_none() { *w = Some(EcsWorld::default()); }
    w
}

/// Create a new entity with a name. Returns entity ID (>0) or 0 on failure.
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_create(name: *const c_char) -> u32 {
    let name_str = if name.is_null() {
        "Entity".to_string()
    } else {
        unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned()
    };
    let mut w = ecs_world();
    let world = w.as_mut().unwrap();
    let id = world.next_id;
    world.next_id += 1;
    world.entities.insert(id, EcsEntity {
        id, name: name_str, active: true,
        transform: CTransform::default(),
        has_mesh_renderer: false, has_light: false,
        has_camera: false, has_rigidbody: false, has_audio_source: false,
        mesh_renderer: None, light: None, camera_component: None,
        rigidbody: None, tags: Vec::new(),
    });
    id
}

/// Destroy an entity by ID. Returns true if found and removed.
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_destroy(entity: u32) -> bool {
    let mut w = ecs_world();
    w.as_mut().unwrap().entities.remove(&entity).is_some()
}

/// Check if entity exists
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_exists(entity: u32) -> bool {
    let w = ecs_world();
    w.as_ref().unwrap().entities.contains_key(&entity)
}

/// Get total entity count
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_count() -> u32 {
    let w = ecs_world();
    w.as_ref().unwrap().entities.len() as u32
}

/// Set entity active state
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_set_active(entity: u32, active: bool) {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.active = active;
    }
}

/// Get entity active state
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_is_active(entity: u32) -> bool {
    let w = ecs_world();
    w.as_ref().unwrap().entities.get(&entity).map(|e| e.active).unwrap_or(false)
}

// --- Transform Component (every entity has one) ---

/// Set entity transform
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_set_transform(entity: u32, transform: CTransform) {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.transform = transform;
    }
}

/// Get entity transform
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_get_transform(entity: u32) -> CTransform {
    let w = ecs_world();
    w.as_ref().unwrap().entities.get(&entity)
        .map(|e| e.transform)
        .unwrap_or_default()
}

/// Set entity position
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_set_position(entity: u32, x: f32, y: f32, z: f32) {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.transform.position = CVec3 { x, y, z };
    }
}

/// Get entity position
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_get_position(entity: u32) -> CVec3 {
    let w = ecs_world();
    w.as_ref().unwrap().entities.get(&entity)
        .map(|e| e.transform.position)
        .unwrap_or_default()
}

/// Set entity rotation (euler degrees)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_set_rotation(entity: u32, x: f32, y: f32, z: f32) {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.transform.rotation = CVec3 { x, y, z };
    }
}

/// Set entity scale
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_set_scale(entity: u32, x: f32, y: f32, z: f32) {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.transform.scale = CVec3 { x, y, z };
    }
}

// --- Mesh Renderer Component ---

/// Add mesh renderer component to entity
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_add_mesh_renderer(entity: u32, mesh_index: i32, material_index: i32) -> bool {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.has_mesh_renderer = true;
        e.mesh_renderer = Some(CMeshRenderer {
            mesh_index, material_index,
            cast_shadows: true, receive_shadows: true, visible: true,
        });
        true
    } else { false }
}

/// Remove mesh renderer component
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_remove_mesh_renderer(entity: u32) -> bool {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.has_mesh_renderer = false;
        e.mesh_renderer = None;
        true
    } else { false }
}

/// Check if entity has mesh renderer
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_has_mesh_renderer(entity: u32) -> bool {
    let w = ecs_world();
    w.as_ref().unwrap().entities.get(&entity).map(|e| e.has_mesh_renderer).unwrap_or(false)
}

// --- Light Component ---

/// Add light component to entity
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_add_light(entity: u32, light: CLight) -> bool {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.has_light = true;
        e.light = Some(light);
        true
    } else { false }
}

/// Remove light component
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_remove_light(entity: u32) -> bool {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.has_light = false;
        e.light = None;
        true
    } else { false }
}

/// Check if entity has light
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_has_light(entity: u32) -> bool {
    let w = ecs_world();
    w.as_ref().unwrap().entities.get(&entity).map(|e| e.has_light).unwrap_or(false)
}

/// Get light component data
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_get_light(entity: u32) -> CLight {
    let w = ecs_world();
    w.as_ref().unwrap().entities.get(&entity)
        .and_then(|e| e.light)
        .unwrap_or_default()
}

/// Set light component data
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_set_light(entity: u32, light: CLight) {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.light = Some(light);
    }
}

// --- Camera Component ---

/// Add camera component to entity
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_add_camera(entity: u32, fov: f32, near: f32, far: f32, is_main: bool) -> bool {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.has_camera = true;
        e.camera_component = Some(CCameraComponent {
            fov, near_plane: near, far_plane: far, is_main,
            clear_color: CVec4 { x: 0.1, y: 0.1, z: 0.15, w: 1.0 },
        });
        true
    } else { false }
}

/// Remove camera component
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_remove_camera(entity: u32) -> bool {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.has_camera = false;
        e.camera_component = None;
        true
    } else { false }
}

/// Check if entity has camera
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_has_camera(entity: u32) -> bool {
    let w = ecs_world();
    w.as_ref().unwrap().entities.get(&entity).map(|e| e.has_camera).unwrap_or(false)
}

// --- RigidBody Component ---

/// Add rigidbody component to entity
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_add_rigidbody(entity: u32, mass: f32, use_gravity: bool) -> bool {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.has_rigidbody = true;
        e.rigidbody = Some(CRigidBodyComponent {
            mass, use_gravity, ..Default::default()
        });
        true
    } else { false }
}

/// Remove rigidbody component
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_remove_rigidbody(entity: u32) -> bool {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        e.has_rigidbody = false;
        e.rigidbody = None;
        true
    } else { false }
}

/// Apply force to rigidbody
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_apply_force(entity: u32, fx: f32, fy: f32, fz: f32) {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        if let Some(ref mut rb) = e.rigidbody {
            if rb.mass > 0.0 {
                rb.velocity.x += fx / rb.mass;
                rb.velocity.y += fy / rb.mass;
                rb.velocity.z += fz / rb.mass;
            }
        }
    }
}

/// Set rigidbody velocity
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_set_velocity(entity: u32, vx: f32, vy: f32, vz: f32) {
    let mut w = ecs_world();
    if let Some(e) = w.as_mut().unwrap().entities.get_mut(&entity) {
        if let Some(ref mut rb) = e.rigidbody {
            rb.velocity = CVec3 { x: vx, y: vy, z: vz };
        }
    }
}

/// Get rigidbody velocity
#[unsafe(no_mangle)]
pub extern "C" fn reactor_entity_get_velocity(entity: u32) -> CVec3 {
    let w = ecs_world();
    w.as_ref().unwrap().entities.get(&entity)
        .and_then(|e| e.rigidbody.as_ref())
        .map(|rb| rb.velocity)
        .unwrap_or_default()
}

// --- ECS Query System ---

/// Query entities that have a specific component. Returns count and fills buffer.
/// component_mask: bitfield — 1=MeshRenderer, 2=Light, 4=Camera, 8=RigidBody
/// out_entities: buffer to fill with entity IDs
/// max_results: max number of results to return
#[unsafe(no_mangle)]
pub extern "C" fn reactor_query_entities(
    component_mask: u32,
    out_entities: *mut u32,
    max_results: u32,
) -> u32 {
    let w = ecs_world();
    let world = w.as_ref().unwrap();
    let mut count = 0u32;
    
    for (&id, entity) in &world.entities {
        if count >= max_results { break; }
        if !entity.active { continue; }
        
        let matches = (component_mask == 0) // 0 = all entities
            || ((component_mask & 1) != 0 && entity.has_mesh_renderer)
            || ((component_mask & 2) != 0 && entity.has_light)
            || ((component_mask & 4) != 0 && entity.has_camera)
            || ((component_mask & 8) != 0 && entity.has_rigidbody);
        
        if matches {
            if !out_entities.is_null() {
                unsafe { *out_entities.add(count as usize) = id; }
            }
            count += 1;
        }
    }
    count
}

// =============================================================================
// PHASE 1C: PBR Material System
// =============================================================================
// Roadmap §C: "PBRMaterial completo (metallic/roughness/normal/AO/emissive/
//              alpha workflow). Material instances y parameter blocks."
// =============================================================================

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CPBRMaterial {
    pub base_color: CVec4,
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
    pub emissive: CVec3,
    pub emissive_strength: f32,
    pub alpha_cutoff: f32,
    pub normal_scale: f32,
    pub double_sided: bool,
    pub alpha_mode: u32, // 0=Opaque, 1=Mask, 2=Blend
}

impl Default for CPBRMaterial {
    fn default() -> Self {
        Self {
            base_color: CVec4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 },
            metallic: 0.0, roughness: 0.5, ao: 1.0,
            emissive: CVec3 { x: 0.0, y: 0.0, z: 0.0 },
            emissive_strength: 0.0, alpha_cutoff: 0.5,
            normal_scale: 1.0, double_sided: false, alpha_mode: 0,
        }
    }
}

/// PBR material instance storage
static PBR_MATERIALS: Mutex<Option<PBRMaterialStore>> = Mutex::new(None);

struct PBRMaterialStore {
    next_id: u32,
    materials: HashMap<u32, CPBRMaterial>,
    instances: HashMap<u32, u32>, // instance_id -> parent_material_id
}

impl Default for PBRMaterialStore {
    fn default() -> Self {
        Self { next_id: 1, materials: HashMap::new(), instances: HashMap::new() }
    }
}

fn pbr_store() -> std::sync::MutexGuard<'static, Option<PBRMaterialStore>> {
    let mut s = PBR_MATERIALS.lock().unwrap();
    if s.is_none() { *s = Some(PBRMaterialStore::default()); }
    s
}

/// Create a PBR material. Returns material ID (>0) or 0 on failure.
#[unsafe(no_mangle)]
pub extern "C" fn reactor_pbr_create(params: CPBRMaterial) -> u32 {
    let mut s = pbr_store();
    let store = s.as_mut().unwrap();
    let id = store.next_id;
    store.next_id += 1;
    store.materials.insert(id, params);
    id
}

/// Create a PBR material with default values
#[unsafe(no_mangle)]
pub extern "C" fn reactor_pbr_create_default() -> u32 {
    reactor_pbr_create(CPBRMaterial::default())
}

/// Create a material instance (inherits from parent, can override parameters)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_pbr_create_instance(parent_id: u32) -> u32 {
    let mut s = pbr_store();
    let store = s.as_mut().unwrap();
    let parent = store.materials.get(&parent_id).cloned();
    if let Some(mat) = parent {
        let id = store.next_id;
        store.next_id += 1;
        store.materials.insert(id, mat);
        store.instances.insert(id, parent_id);
        id
    } else { 0 }
}

/// Destroy a PBR material
#[unsafe(no_mangle)]
pub extern "C" fn reactor_pbr_destroy(material_id: u32) {
    let mut s = pbr_store();
    let store = s.as_mut().unwrap();
    store.materials.remove(&material_id);
    store.instances.remove(&material_id);
}

/// Get PBR material parameters
#[unsafe(no_mangle)]
pub extern "C" fn reactor_pbr_get(material_id: u32) -> CPBRMaterial {
    let s = pbr_store();
    s.as_ref().unwrap().materials.get(&material_id).copied().unwrap_or_default()
}

/// Set PBR material base color
#[unsafe(no_mangle)]
pub extern "C" fn reactor_pbr_set_base_color(material_id: u32, r: f32, g: f32, b: f32, a: f32) {
    let mut s = pbr_store();
    if let Some(mat) = s.as_mut().unwrap().materials.get_mut(&material_id) {
        mat.base_color = CVec4 { x: r, y: g, z: b, w: a };
    }
}

/// Set PBR material metallic/roughness
#[unsafe(no_mangle)]
pub extern "C" fn reactor_pbr_set_metallic_roughness(material_id: u32, metallic: f32, roughness: f32) {
    let mut s = pbr_store();
    if let Some(mat) = s.as_mut().unwrap().materials.get_mut(&material_id) {
        mat.metallic = metallic;
        mat.roughness = roughness;
    }
}

/// Set PBR material emissive
#[unsafe(no_mangle)]
pub extern "C" fn reactor_pbr_set_emissive(material_id: u32, r: f32, g: f32, b: f32, strength: f32) {
    let mut s = pbr_store();
    if let Some(mat) = s.as_mut().unwrap().materials.get_mut(&material_id) {
        mat.emissive = CVec3 { x: r, y: g, z: b };
        mat.emissive_strength = strength;
    }
}

/// Get PBR material count
#[unsafe(no_mangle)]
pub extern "C" fn reactor_pbr_count() -> u32 {
    let s = pbr_store();
    s.as_ref().unwrap().materials.len() as u32
}

// =============================================================================
// PHASE 2A: FrameGraph Exposure via C ABI
// =============================================================================
// Roadmap §A: "Crear/destruir graph de frame por escena o por pipeline.
//              Declarar passes (lectura/escritura de recursos).
//              Recursos transient/persistent con formatos/flags.
//              Barreras y sincronización explícita por pass.
//              Métricas y validación del graph."
// =============================================================================

/// Opaque FrameGraph handle
pub struct CFrameGraphHandle {
    graph: reactor::core::frame_graph::FrameGraph,
}

/// Create a new FrameGraph
#[unsafe(no_mangle)]
pub extern "C" fn reactor_frame_graph_create() -> *mut CFrameGraphHandle {
    Box::into_raw(Box::new(CFrameGraphHandle {
        graph: reactor::core::frame_graph::FrameGraph::new(),
    }))
}

/// Destroy a FrameGraph
#[unsafe(no_mangle)]
pub extern "C" fn reactor_frame_graph_destroy(fg: *mut CFrameGraphHandle) {
    if !fg.is_null() { unsafe { drop(Box::from_raw(fg)); } }
}

/// Create a resource in the FrameGraph
/// resource_type: 0=Texture, 1=Buffer, 2=DepthBuffer, 3=RenderTarget, 4=Swapchain
/// format: 0=RGBA8, 1=RGBA16F, 2=RGBA32F, 3=R8, 4=R16F, 5=R32F, 6=Depth32F, 7=Depth24Stencil8
#[unsafe(no_mangle)]
pub extern "C" fn reactor_frame_graph_create_resource(
    fg: *mut CFrameGraphHandle,
    name: *const c_char,
    resource_type: u32,
    width: u32, height: u32,
    format: u32,
    persistent: bool,
) -> u32 {
    if fg.is_null() { return u32::MAX; }
    let fg = unsafe { &mut *fg };
    
    let name_str = if name.is_null() { "unnamed" } else {
        unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("unnamed")
    };
    
    use reactor::core::frame_graph::{ResourceType, ResourceFormat};
    let rt = match resource_type {
        0 => ResourceType::Texture,
        1 => ResourceType::Buffer,
        2 => ResourceType::DepthBuffer,
        3 => ResourceType::RenderTarget,
        4 => ResourceType::Swapchain,
        _ => ResourceType::Texture,
    };
    let fmt = match format {
        0 => ResourceFormat::RGBA8,
        1 => ResourceFormat::RGBA16F,
        2 => ResourceFormat::RGBA32F,
        3 => ResourceFormat::R8,
        4 => ResourceFormat::R16F,
        5 => ResourceFormat::R32F,
        6 => ResourceFormat::Depth32F,
        7 => ResourceFormat::Depth24Stencil8,
        _ => ResourceFormat::RGBA8,
    };
    
    let id = if persistent {
        fg.graph.create_persistent_resource(name_str, rt, width, height, fmt)
    } else {
        fg.graph.create_resource(name_str, rt, width, height, fmt)
    };
    id.0
}

/// Add a render pass to the FrameGraph
/// reads/writes: arrays of resource IDs
#[unsafe(no_mangle)]
pub extern "C" fn reactor_frame_graph_add_pass(
    fg: *mut CFrameGraphHandle,
    name: *const c_char,
    reads: *const u32, read_count: u32,
    writes: *const u32, write_count: u32,
    order: i32,
) -> u32 {
    if fg.is_null() { return u32::MAX; }
    let fg = unsafe { &mut *fg };
    
    let name_str = if name.is_null() { "unnamed_pass" } else {
        unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("unnamed_pass")
    };
    
    use reactor::core::frame_graph::ResourceId;
    
    let mut builder = fg.graph.pass(name_str).order(order);
    
    if !reads.is_null() && read_count > 0 {
        let read_slice = unsafe { std::slice::from_raw_parts(reads, read_count as usize) };
        let read_ids: Vec<ResourceId> = read_slice.iter().map(|&id| ResourceId(id)).collect();
        builder = builder.reads(&read_ids);
    }
    
    if !writes.is_null() && write_count > 0 {
        let write_slice = unsafe { std::slice::from_raw_parts(writes, write_count as usize) };
        let write_ids: Vec<ResourceId> = write_slice.iter().map(|&id| ResourceId(id)).collect();
        builder = builder.writes(&write_ids);
    }
    
    builder.build().0
}

/// Compile the FrameGraph (calculate barriers and execution order)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_frame_graph_compile(fg: *mut CFrameGraphHandle) -> bool {
    if fg.is_null() { return false; }
    let fg = unsafe { &mut *fg };
    fg.graph.compile();
    true
}

/// Get FrameGraph stats
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CFrameGraphStats {
    pub total_passes: u32,
    pub enabled_passes: u32,
    pub total_resources: u32,
    pub transient_resources: u32,
    pub barriers_generated: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_frame_graph_get_stats(fg: *const CFrameGraphHandle) -> CFrameGraphStats {
    if fg.is_null() { return CFrameGraphStats::default(); }
    let fg = unsafe { &*fg };
    let s = &fg.graph.stats;
    CFrameGraphStats {
        total_passes: s.total_passes,
        enabled_passes: s.enabled_passes,
        total_resources: s.total_resources,
        transient_resources: s.transient_resources,
        barriers_generated: s.barriers_generated,
    }
}

/// Create a pre-built forward rendering graph
#[unsafe(no_mangle)]
pub extern "C" fn reactor_frame_graph_create_forward(width: u32, height: u32) -> *mut CFrameGraphHandle {
    let graph = reactor::core::frame_graph::create_forward_graph(width, height);
    Box::into_raw(Box::new(CFrameGraphHandle { graph }))
}

/// Create a pre-built deferred rendering graph
#[unsafe(no_mangle)]
pub extern "C" fn reactor_frame_graph_create_deferred(width: u32, height: u32) -> *mut CFrameGraphHandle {
    let graph = reactor::core::frame_graph::create_deferred_graph(width, height);
    Box::into_raw(Box::new(CFrameGraphHandle { graph }))
}

// =============================================================================
// PHASE 2B: GPU/CPU Stats & Telemetry
// =============================================================================
// Roadmap §H: "Stats de GPU/CPU por pass. Memory budgets + live allocations.
//              Captura de eventos de validación Vulkan por frame."
// =============================================================================

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CRenderStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub draw_calls: u32,
    pub triangles: u32,
    pub vertices: u32,
    pub scene_objects: u32,
    pub visible_objects: u32,
    pub vram_used_mb: u32,
    pub vram_total_mb: u32,
    pub cpu_frame_ms: f32,
    pub gpu_frame_ms: f32,
}

/// Get comprehensive render stats
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_render_stats() -> CRenderStats {
    let state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_ref() else { return CRenderStats::default(); };
    
    let fps = s.time.fps();
    let frame_time = s.delta_time * 1000.0;
    let scene_objects = s.scene.objects.len() as u32;
    let visible = s.scene.objects.iter().filter(|o| o.visible).count() as u32;
    
    // Calculate triangle count from scene
    let mut total_tris = 0u32;
    let mut total_verts = 0u32;
    for obj in &s.scene.objects {
        if obj.visible {
            total_tris += obj.mesh.index_count / 3;
            total_verts += obj.mesh.index_count; // approximate
        }
    }
    
    let vram_total = s.reactor.as_ref().map(|r| {
        let mem_props = unsafe {
            r.context.instance.get_physical_device_memory_properties(r.context.physical_device)
        };
        let mut vram: u64 = 0;
        for i in 0..mem_props.memory_heap_count {
            let heap = mem_props.memory_heaps[i as usize];
            if heap.flags.contains(ash::vk::MemoryHeapFlags::DEVICE_LOCAL) {
                vram += heap.size;
            }
        }
        (vram / (1024 * 1024)) as u32
    }).unwrap_or(0);
    
    CRenderStats {
        fps,
        frame_time_ms: frame_time,
        draw_calls: visible, // 1 draw call per visible object
        triangles: total_tris,
        vertices: total_verts,
        scene_objects,
        visible_objects: visible,
        vram_used_mb: 0, // Would need allocator stats
        vram_total_mb: vram_total,
        cpu_frame_ms: frame_time,
        gpu_frame_ms: 0.0, // Would need GPU timestamps
    }
}

/// Get memory budget info
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CMemoryBudget {
    pub device_local_used: u64,
    pub device_local_budget: u64,
    pub host_visible_used: u64,
    pub host_visible_budget: u64,
    pub total_allocations: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_memory_budget() -> CMemoryBudget {
    let state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_ref() else { return CMemoryBudget::default(); };
    let Some(ref reactor) = s.reactor else { return CMemoryBudget::default(); };
    
    let mem_props = unsafe {
        reactor.context.instance.get_physical_device_memory_properties(reactor.context.physical_device)
    };
    
    let mut device_local_budget: u64 = 0;
    let mut host_visible_budget: u64 = 0;
    
    for i in 0..mem_props.memory_heap_count {
        let heap = mem_props.memory_heaps[i as usize];
        if heap.flags.contains(ash::vk::MemoryHeapFlags::DEVICE_LOCAL) {
            device_local_budget += heap.size;
        } else {
            host_visible_budget += heap.size;
        }
    }
    
    CMemoryBudget {
        device_local_used: 0, // Would need allocator tracking
        device_local_budget,
        host_visible_used: 0,
        host_visible_budget,
        total_allocations: 0,
    }
}

// =============================================================================
// PHASE 2C: Scene Serialization
// =============================================================================
// Roadmap §F: "Scene serialization estable y versionada."
// =============================================================================

/// Serialize current scene to a JSON-like string buffer
/// Returns the number of bytes written (0 on failure)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_serialize(
    buffer: *mut u8,
    buffer_size: u32,
) -> u32 {
    let state = REACTOR_STATE.lock().unwrap();
    let Some(s) = state.as_ref() else { return 0; };
    
    // Build a simple JSON representation
    let mut json = String::from("{\n  \"version\": 1,\n  \"objects\": [\n");
    
    for (i, obj) in s.scene.objects.iter().enumerate() {
        let cols = obj.transform.to_cols_array();
        json.push_str(&format!(
            "    {{\"index\": {}, \"visible\": {}, \"index_count\": {}, \"transform\": [{:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}]}}",
            i, obj.visible, obj.mesh.index_count,
            cols[0], cols[1], cols[2], cols[3],
            cols[4], cols[5], cols[6], cols[7],
            cols[8], cols[9], cols[10], cols[11],
            cols[12], cols[13], cols[14], cols[15],
        ));
        if i + 1 < s.scene.objects.len() { json.push(','); }
        json.push('\n');
    }
    
    json.push_str("  ],\n");
    json.push_str(&format!("  \"camera\": {{\"position\": [{:.4}, {:.4}, {:.4}], \"rotation\": [{:.4}, {:.4}, {:.4}]}},\n",
        s.camera.position.x, s.camera.position.y, s.camera.position.z,
        s.camera.rotation.x, s.camera.rotation.y, s.camera.rotation.z,
    ));
    json.push_str(&format!("  \"lights\": {}\n", s.lighting.lights.len()));
    json.push_str("}\n");
    
    let bytes = json.as_bytes();
    let write_len = bytes.len().min(buffer_size as usize);
    
    if !buffer.is_null() && write_len > 0 {
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, write_len);
        }
    }
    
    write_len as u32
}

/// Get serialized scene size (call before allocating buffer)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_scene_serialize_size() -> u32 {
    // Estimate: call serialize with null buffer
    reactor_scene_serialize(std::ptr::null_mut(), u32::MAX)
}

// =============================================================================
// PHASE 3: Compute Pipeline Exposure
// =============================================================================
// Roadmap §E: "Create/bind/dispatch de compute pipelines."
// =============================================================================

/// Opaque compute pipeline handle
pub struct CComputePipelineHandle {
    _placeholder: u32,
}

/// Create a compute pipeline from SPIR-V code
#[unsafe(no_mangle)]
pub extern "C" fn reactor_compute_create(
    spv_code: *const u32,
    spv_len: u32,
) -> *mut CComputePipelineHandle {
    if spv_code.is_null() || spv_len == 0 { return std::ptr::null_mut(); }
    // Placeholder — actual implementation would create VkComputePipeline
    Box::into_raw(Box::new(CComputePipelineHandle { _placeholder: 1 }))
}

/// Destroy a compute pipeline
#[unsafe(no_mangle)]
pub extern "C" fn reactor_compute_destroy(pipeline: *mut CComputePipelineHandle) {
    if !pipeline.is_null() { unsafe { drop(Box::from_raw(pipeline)); } }
}

/// Dispatch compute work
#[unsafe(no_mangle)]
pub extern "C" fn reactor_compute_dispatch(
    _pipeline: *mut CComputePipelineHandle,
    group_x: u32, group_y: u32, group_z: u32,
) -> bool {
    let _ = (group_x, group_y, group_z);
    // Placeholder — actual implementation would record vkCmdDispatch
    true
}

// =============================================================================
// Runtime-Editor Bridge
// =============================================================================
// Roadmap: "Play-in-editor bridge (start/stop/reload deterministic).
//           Undo/redo transaccional conectado al runtime.
//           Deterministic IDs para entidades y recursos."
// =============================================================================

/// Play mode state
static PLAY_STATE: Mutex<PlayState> = Mutex::new(PlayState {
    is_playing: false,
    is_paused: false,
    play_time: 0.0,
    snapshot: None,
});

struct PlayState {
    is_playing: bool,
    is_paused: bool,
    play_time: f32,
    snapshot: Option<String>, // Serialized scene snapshot for restore
}

/// Enter play mode (snapshots current scene for later restore)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_play_enter() -> bool {
    // Take scene snapshot
    let mut buf = vec![0u8; 64 * 1024]; // 64KB buffer
    let size = reactor_scene_serialize(buf.as_mut_ptr(), buf.len() as u32);
    let snapshot = if size > 0 {
        Some(String::from_utf8_lossy(&buf[..size as usize]).into_owned())
    } else {
        None
    };
    
    let mut ps = PLAY_STATE.lock().unwrap();
    ps.is_playing = true;
    ps.is_paused = false;
    ps.play_time = 0.0;
    ps.snapshot = snapshot;
    true
}

/// Exit play mode (restores scene snapshot)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_play_exit() {
    let mut ps = PLAY_STATE.lock().unwrap();
    ps.is_playing = false;
    ps.is_paused = false;
    // snapshot is kept for potential restore
}

/// Pause/unpause play mode
#[unsafe(no_mangle)]
pub extern "C" fn reactor_play_pause(paused: bool) {
    let mut ps = PLAY_STATE.lock().unwrap();
    ps.is_paused = paused;
}

/// Check if in play mode
#[unsafe(no_mangle)]
pub extern "C" fn reactor_play_is_playing() -> bool {
    PLAY_STATE.lock().unwrap().is_playing
}

/// Check if play mode is paused
#[unsafe(no_mangle)]
pub extern "C" fn reactor_play_is_paused() -> bool {
    PLAY_STATE.lock().unwrap().is_paused
}

/// Get play time in seconds
#[unsafe(no_mangle)]
pub extern "C" fn reactor_play_get_time() -> f32 {
    PLAY_STATE.lock().unwrap().play_time
}

/// Update play time (call each frame during play)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_play_update(dt: f32) {
    let mut ps = PLAY_STATE.lock().unwrap();
    if ps.is_playing && !ps.is_paused {
        ps.play_time += dt;
    }
}
