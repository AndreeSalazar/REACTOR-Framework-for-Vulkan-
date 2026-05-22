//! Platform abstraction layer
//! 
//! Handles OS-specific functionality, windowing, and input.

pub mod config;
pub mod window;
pub mod input;
pub mod time;

pub use window::Window;
pub use input::Input;
pub use time::Time;
pub use config::{ReactorConfig, RendererMode};
