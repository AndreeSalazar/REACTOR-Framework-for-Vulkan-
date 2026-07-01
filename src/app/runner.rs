use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::app::config::RendererMode;
use crate::app::context::ReactorContext;
use crate::app::ReactorApp;
use crate::platform::time::Time;
use crate::reactor::Reactor;
use crate::resources::{
    AssetDatabase, AssetHotReloadManager, AssetLoaderQueue, AssetManager, GltfLoader,
};

struct AppRunner<A: ReactorApp> {
    app: A,
    context: Option<ReactorContext>,
}

impl<A: ReactorApp> ApplicationHandler for AppRunner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.is_some() { return; }
        let config = self.app.config();
        let window_attributes = Window::default_attributes()
            .with_title(&config.title)
            .with_inner_size(LogicalSize::new(config.width, config.height));
        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => { eprintln!("Failed to create window: {}", e); event_loop.exit(); return; }
        };
        let reactor = match Reactor::init(&window, config.msaa_samples, config.renderer == RendererMode::RayTracing, config.vsync) {
            Ok(r) => r,
            Err(e) => { eprintln!("Failed to initialize Reactor: {}", e); event_loop.exit(); return; }
        };
        crate::systems::console::init();
        crate::systems::console::ReactorBanner::print_init(
            &config.title,
            &format!("{}×{}", window.inner_size().width, window.inner_size().height),
            &format!("{:?}", reactor.msaa_samples),
            reactor.ray_tracing.is_some(),
            &format!("{}", crate::systems::console::gpu_name_short(&reactor.context)),
        );
        let aspect = window.inner_size().width as f32 / window.inner_size().height.max(1) as f32;
        let asset_manager = AssetManager::new();
        let gltf_loader = GltfLoader::new("assets");
        let asset_db = AssetDatabase::open(".reactor/assets.db")
            .unwrap_or_else(|_| AssetDatabase::in_memory().unwrap());
        let asset_loader_queue = AssetLoaderQueue::new().unwrap_or_else(|_| {
            AssetLoaderQueue::with_config(crate::resources::LoaderQueueConfig {
                num_workers: 2, ..Default::default()
            }).unwrap()
        });
        let (hot_reload_tx, hot_reload_rx) = tokio::sync::mpsc::unbounded_channel();
        let asset_hot_reload = AssetHotReloadManager::new(crate::resources::HotReloadConfig::default(), hot_reload_tx).ok();
        let hot_reload_rx = if asset_hot_reload.is_some() { Some(hot_reload_rx) } else { None };
        let mut ctx = ReactorContext {
            reactor, window,
            time: Time::new(), config: config.clone(),
            camera: crate::scene::camera::Camera::perspective(60.0, aspect, 0.1, 1000.0),
            scene: crate::systems::scene::Scene::new(),
            lighting: crate::systems::lighting::LightingSystem::new(),
            physics: crate::systems::physics::PhysicsWorld::new(),
            culling: crate::systems::frustum::CullingSystem::new(),
            debug: crate::graphics::debug_renderer::DebugRenderer::new(),
            asset_manager, gltf_loader, asset_db, asset_hot_reload, asset_loader_queue,
            audio: crate::systems::audio::AudioSystem::new(),
            event_bus: crate::systems::event_bus::EventBus::new(),
            hot_reload_rx,
            blob_shadow_mesh: None, blob_shadow_material: None,
            fixed_accumulator: 0.0,
        };
        self.app.init(&mut ctx);
        self.context = Some(ctx);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(ctx) = &mut self.context else { return };
        ctx.reactor.handle_event(&event);
        if self.app.on_event(ctx, &event) { return; }
        match event {
            WindowEvent::CloseRequested => { event_loop.exit(); }
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                self.app.on_resize(ctx, size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                if let Some(ref mut rx) = ctx.hot_reload_rx {
                    while let Ok(event) = rx.try_recv() { ctx.event_bus.emit(event); }
                }
                ctx.time.update();
                let dt = ctx.time.delta();
                if ctx.config.physics_hz > 0 {
                    let fixed_dt = 1.0 / ctx.config.physics_hz as f32;
                    ctx.fixed_accumulator += dt;
                    while ctx.fixed_accumulator >= fixed_dt {
                        self.app.fixed_update(ctx, fixed_dt);
                        ctx.fixed_accumulator -= fixed_dt;
                    }
                }
                self.app.update(ctx);
                self.app.render(ctx);
                ctx.reactor.input.begin_frame();
                if ctx.reactor.device_lost || ctx.reactor.exit_requested { event_loop.exit(); return; }
                ctx.window.request_redraw();
            }
            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ctx) = &mut self.context {
            unsafe { let _ = ctx.reactor.context.device.device_wait_idle(); }
            self.app.on_exit(ctx);
            unsafe { let _ = ctx.reactor.context.device.device_wait_idle(); }
        }
        self.context.take();
    }
}

pub fn run<A: ReactorApp + 'static>(app: A) {
    let _ = env_logger::try_init();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all().build().expect("Failed to create Tokio runtime");
    let _guard = rt.enter();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut runner = AppRunner { app, context: None };
    event_loop.run_app(&mut runner).expect("Event loop error");
}
