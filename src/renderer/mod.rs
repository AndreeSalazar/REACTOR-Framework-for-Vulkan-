//! High-level rendering systems
//! 
//! Abstract rendering pipelines (forward, deferred, etc).

pub mod forward;
// pub mod deferred;  // Fase 4
// pub mod shadows;   // Fase 4
// pub mod postprocess; // Fase 4

pub use forward::ForwardRenderer;
