#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum RendererMode {
    #[default]
    Forward,
    Deferred,
    RayTracing,
}

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
        Self { title: title.to_string(), ..Default::default() }
    }
    pub fn with_size(mut self, width: u32, height: u32) -> Self { self.width = width; self.height = height; self }
    pub fn with_vsync(mut self, vsync: bool) -> Self { self.vsync = vsync; self }
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self { self.fullscreen = fullscreen; self }
    pub fn with_resizable(mut self, resizable: bool) -> Self { self.resizable = resizable; self }
    pub fn with_maximized(mut self, maximized: bool) -> Self { self.maximized = maximized; self }
    pub fn with_msaa(mut self, samples: u32) -> Self { self.msaa_samples = samples; self }
    pub fn with_renderer(mut self, renderer: RendererMode) -> Self { self.renderer = renderer; self }
    pub fn with_physics_hz(mut self, hz: u32) -> Self { self.physics_hz = hz; self }
    pub fn with_scene(mut self, scene: &str) -> Self { self.scene = Some(scene.to_string()); self }
}

impl Default for ReactorConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            width: 1280,
            height: 720,
            vsync: true,
            fullscreen: false,
            resizable: true,
            maximized: false,
            msaa_samples: 1,
            renderer: RendererMode::default(),
            physics_hz: 0,
            scene: None,
        }
    }
}
