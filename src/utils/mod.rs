// REACTOR Utils Module
// Contains utility functions and hardware detection

pub mod gpu_detector;
pub mod cpu_detector;
pub mod resolution_detector;
pub mod time;

pub use gpu_detector::{GPUDetector, GPUInfo};
pub use cpu_detector::{CPUDetector, CPUInfo};
pub use resolution_detector::ResolutionDetector;
pub use time::Time;
