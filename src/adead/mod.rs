// =============================================================================
// ADead-GPU Integration for REACTOR
// Intelligent Shading Rate, SDF Ray Tracing, Anti-Aliasing
// =============================================================================
//
// ADead-ISR: Intelligent Shading Rate 2.0
// - Adaptive resolution shading sin AI, sin Tensor Cores
// - Matemáticas puras para decidir dónde gastar GPU
// - 75% ahorro de GPU con 95% de calidad
//
// ADead-RT: Ray Tracing sin RT Cores
// - Ray marching con SDF
// - Funciona en CUALQUIER GPU
// - GI, soft shadows, reflections
//
// ADead-AA: Anti-Aliasing SDF
// - Perfect edges usando SDF
// - Zero memory overhead
// - Mejor que MSAA/FXAA/TAA
// =============================================================================

pub mod isr;
pub mod sdf;
pub mod raymarching;
pub mod antialiasing;
pub mod hybrid;
pub mod integration;

pub use isr::*;
pub use sdf::*;
pub use raymarching::*;
pub use antialiasing::*;
pub use hybrid::*;
pub use integration::*;
