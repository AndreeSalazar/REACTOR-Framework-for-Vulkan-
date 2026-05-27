//! Utility functions and helpers
//!
//! General-purpose utilities for the REACTOR framework.

pub mod cpu_detector;
pub mod gpu_detector;
pub mod hash;
pub mod math;
pub mod resolution_detector;

pub use cpu_detector::{CPUDetector, CPUInfo};
pub use gpu_detector::{GPUDetector, GPUInfo};
pub use resolution_detector::ResolutionDetector;

// Re-export glam for convenience
pub use glam;
