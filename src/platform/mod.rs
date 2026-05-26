//! Platform abstraction layer
//! 
//! Handles OS-specific functionality, windowing, and input.

pub mod window;
pub mod input;
pub mod gamepad;
pub mod time;

pub use window::ReactorWindow;
pub use input::Input;
pub use gamepad::{Gamepad, GamepadButton, GamepadAxis};
pub use time::Time;
