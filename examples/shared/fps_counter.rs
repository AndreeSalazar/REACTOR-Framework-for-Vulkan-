//! Smoothed FPS counter + title formatter.

use reactor_vulkan::app::ReactorContext;

pub struct FpsCounter {
    smoothed: f32,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self { smoothed: 0.0 }
    }
}

impl FpsCounter {
    pub fn update(&mut self, ctx: &ReactorContext) -> f32 {
        let instant = ctx.fps();
        if self.smoothed == 0.0 {
            self.smoothed = instant;
        } else {
            self.smoothed = self.smoothed * 0.9 + instant * 0.1;
        }
        self.smoothed
    }

    pub fn format_title(&mut self, ctx: &ReactorContext, prefix: &str) -> String {
        format!("{} | FPS: {:.0}", prefix, self.update(ctx))
    }
}
