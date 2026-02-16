use glam::Vec4;

/// Configuration for creating a REACTOR application
#[derive(Clone, Debug)]
pub struct ReactorConfig {
    /// Window title
    pub title: String,
    /// Requested window width
    pub width: u32,
    /// Requested window height
    pub height: u32,
    /// Enable VSync (FIFO present mode)
    pub vsync: bool,
    /// Enable MSAA anti-aliasing (auto-selects best sample count)
    pub msaa: bool,
    /// Enable Vulkan validation layers (debug only)
    pub validation_layers: bool,
    /// Enable ray tracing if supported
    pub ray_tracing: bool,
    /// Start maximized
    pub maximized: bool,
    /// Allow window resizing
    pub resizable: bool,
    /// Target frames per second (0 = unlimited)
    pub target_fps: u32,
    /// Clear color (RGBA)
    pub clear_color: Vec4,
    /// Fixed physics timestep (Hz, 0 = disabled)
    pub physics_hz: u32,
}

impl Default for ReactorConfig {
    fn default() -> Self {
        Self {
            title: "REACTOR Application".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
            msaa: true,
            validation_layers: cfg!(debug_assertions),
            ray_tracing: true,
            maximized: false,
            resizable: true,
            target_fps: 0,
            clear_color: Vec4::new(0.1, 0.1, 0.1, 1.0),
            physics_hz: 60,
        }
    }
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

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    pub fn with_msaa(mut self, msaa: bool) -> Self {
        self.msaa = msaa;
        self
    }

    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    pub fn with_clear_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.clear_color = Vec4::new(r, g, b, 1.0);
        self
    }

    pub fn with_physics_hz(mut self, hz: u32) -> Self {
        self.physics_hz = hz;
        self
    }

    /// Preset: Game (1920x1080, maximized, MSAA, 60fps physics)
    pub fn game(title: &str) -> Self {
        Self::new(title)
            .with_size(1920, 1080)
            .with_maximized(true)
    }

    /// Preset: Prototype (800x600, windowed, quick iteration)
    pub fn prototype(title: &str) -> Self {
        Self::new(title)
            .with_size(800, 600)
    }

    /// Preset: VR (high resolution, no vsync for low latency)
    pub fn vr(title: &str) -> Self {
        Self::new(title)
            .with_size(2160, 2160)
            .with_vsync(false)
    }
}
