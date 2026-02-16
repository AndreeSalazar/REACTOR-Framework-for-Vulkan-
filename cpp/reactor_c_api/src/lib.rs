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
use reactor::systems::camera::Camera;
use reactor::systems::lighting::LightingSystem;
use reactor::systems::physics::PhysicsWorld;
use reactor::systems::frustum::CullingSystem;
use reactor::graphics::debug_renderer::DebugRenderer;

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

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              🚀 REACTOR Framework for C++                    ║");
        println!("║  Version: {}                                          ║", "0.4.1");
        println!("║  Title: {:52} ║", s.title);
        println!("║  Resolution: {}x{:<44} ║", s.width, s.height);
        println!("║  Ray Tracing: {:<47} ║",
            if reactor.ray_tracing.is_some() { "✅ Enabled" } else { "❌ Not available" });
        println!("╚══════════════════════════════════════════════════════════════╝");

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
