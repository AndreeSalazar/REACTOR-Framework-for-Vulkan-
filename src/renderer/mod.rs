//! High-level rendering systems
//!
//! Abstract rendering pipelines (forward, deferred, etc).

pub mod bindless_forward;
pub mod forward;
// pub mod deferred;  // Fase 4
// pub mod shadows;   // Fase 4
// pub mod postprocess; // Fase 4

pub use bindless_forward::{
    BindlessForwardConfig, BindlessForwardRenderer, FrameStats, RenderObject,
};
pub use forward::ForwardRenderer;
