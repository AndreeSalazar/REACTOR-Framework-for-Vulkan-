//! Platform abstraction layer
//! 
//! Handles OS-specific functionality, windowing, and input.

pub mod window;
pub mod input;
pub mod time;

pub use window::ReactorWindow;
pub use input::Input;
pub use time::Time;
