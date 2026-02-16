use winit::window::Window;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use std::sync::Arc;

use crate::platform::config::ReactorConfig;

/// Platform window abstraction wrapping winit
pub struct ReactorWindow {
    pub(crate) inner: Arc<Window>,
}

impl ReactorWindow {
    /// Create a new window from config and event loop
    pub fn new(event_loop: &ActiveEventLoop, config: &ReactorConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut attributes = Window::default_attributes()
            .with_title(&config.title)
            .with_inner_size(LogicalSize::new(config.width as f32, config.height as f32))
            .with_resizable(config.resizable);

        if config.maximized {
            attributes = attributes.with_maximized(true);
        }

        let window = event_loop.create_window(attributes)?;

        Ok(Self {
            inner: Arc::new(window),
        })
    }

    /// Get the underlying winit window
    pub fn winit_window(&self) -> &Window {
        &self.inner
    }

    /// Get window Arc (for sharing)
    pub fn arc(&self) -> Arc<Window> {
        self.inner.clone()
    }

    /// Get current window size in pixels
    pub fn size(&self) -> (u32, u32) {
        let size = self.inner.inner_size();
        (size.width, size.height)
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        let (w, h) = self.size();
        w as f32 / h as f32
    }

    /// Set window title
    pub fn set_title(&self, title: &str) {
        self.inner.set_title(title);
    }

    /// Request a redraw
    pub fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    /// Get DPI scale factor
    pub fn scale_factor(&self) -> f64 {
        self.inner.scale_factor()
    }
}
