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
#[derive(Clone, Copy, Default)]
pub struct CVec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
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

#[unsafe(no_mangle)]
pub extern "C" fn reactor_version() -> *const c_char {
    c"REACTOR v0.4.1".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_engine_name() -> *const c_char {
    c"REACTOR Framework for Vulkan".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_version_major() -> u32 { 0 }

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_version_minor() -> u32 { 4 }

#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_version_patch() -> u32 { 1 }

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
            if let Some(ref reactor) = s.reactor {
                unsafe {
                    let _ = reactor.context.device.device_wait_idle();
                }
            }
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

/// Create a mesh from vertex data
/// vertices: array of CVertex, vertex_count: number of vertices
/// indices: array of u32, index_count: number of indices
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_mesh(
    vertices: *const CVertex,
    vertex_count: u32,
    indices: *const u32,
    index_count: u32,
) -> *mut MeshHandle {
    // Note: Mesh creation requires VulkanContext
    // This is a placeholder - actual mesh creation happens through reactor_run() callbacks
    if vertices.is_null() || indices.is_null() { return std::ptr::null_mut(); }
    
    // We can't create GPU resources without the Vulkan context
    // Return null - user should create meshes in on_init callback
    std::ptr::null_mut()
}

/// Destroy a mesh handle
#[unsafe(no_mangle)]
pub extern "C" fn reactor_destroy_mesh(mesh: *mut MeshHandle) {
    if !mesh.is_null() {
        unsafe { drop(Box::from_raw(mesh)); }
    }
}

// =============================================================================
// Material Creation API
// =============================================================================

/// Create a simple colored material (placeholder - requires shaders)
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_material_simple(_r: f32, _g: f32, _b: f32) -> *mut MaterialHandle {
    // Material creation requires SPIR-V shaders
    // This is a placeholder - use reactor_create_material with shader code
    std::ptr::null_mut()
}

/// Create a basic material from shader code
/// Note: This is a placeholder - actual material creation requires Vulkan context
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_material(
    _vert_spv: *const u32,
    _vert_len: u32,
    _frag_spv: *const u32,
    _frag_len: u32,
) -> *mut MaterialHandle {
    // Material creation requires Vulkan context - placeholder
    std::ptr::null_mut()
}

/// Create a textured material from shader code and texture
/// Note: This is a placeholder - actual material creation requires Vulkan context
#[unsafe(no_mangle)]
pub extern "C" fn reactor_create_textured_material(
    _vert_spv: *const u32,
    _vert_len: u32,
    _frag_spv: *const u32,
    _frag_len: u32,
    _texture: *const TextureHandle,
) -> *mut MaterialHandle {
    // Material creation requires Vulkan context - placeholder
    std::ptr::null_mut()
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

/// Get VRAM in MB
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_vram_mb() -> u32 {
    0 // Placeholder - would query Vulkan
}

/// Get current MSAA samples
#[unsafe(no_mangle)]
pub extern "C" fn reactor_get_msaa_samples() -> u32 {
    4 // Default
}

/// Check if ray tracing is supported
#[unsafe(no_mangle)]
pub extern "C" fn reactor_is_raytracing_supported() -> bool {
    false // Placeholder
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
