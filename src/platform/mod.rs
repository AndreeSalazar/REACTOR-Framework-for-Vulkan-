//! Platform abstraction layer
//!
//! Handles OS-specific functionality, windowing, and input.

pub mod gamepad;
pub mod input;
pub mod time;
pub mod window;

pub use gamepad::{Gamepad, GamepadAxis, GamepadButton};
pub use input::Input;
pub use time::Time;
pub use window::ReactorWindow;
