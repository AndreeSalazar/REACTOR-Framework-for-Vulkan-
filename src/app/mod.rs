pub mod config;
pub mod context;
pub mod pause_config;
pub mod quick;
pub mod runner;

pub use config::{ReactorConfig, RendererMode};
pub use context::{AssetPipelineStats, GltfBounds, GltfSpawn, ModelSpawnInfo, ReactorContext};
pub use quick::{call_init, call_update, quick, quick_with};
pub use runner::run;

use winit::event::WindowEvent;

/// Trait that users implement to create a REACTOR application.
pub trait ReactorApp {
    fn config(&self) -> ReactorConfig { ReactorConfig::default() }
    fn init(&mut self, ctx: &mut ReactorContext);
    fn update(&mut self, ctx: &mut ReactorContext);
    fn render(&mut self, ctx: &mut ReactorContext) { ctx.render_scene(); }
    fn fixed_update(&mut self, _ctx: &mut ReactorContext, _fixed_dt: f32) {}
    fn on_resize(&mut self, _ctx: &mut ReactorContext, _width: u32, _height: u32) {}
    fn on_exit(&mut self, _ctx: &mut ReactorContext) {}
    fn on_event(&mut self, _ctx: &mut ReactorContext, _event: &WindowEvent) -> bool { false }
}
