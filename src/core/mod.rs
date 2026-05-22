// =============================================================================
// REACTOR Core Module
// =============================================================================
// Fundamental engine primitives: Vulkan context, memory, concurrency,
// profiling, and error handling. All other crates/modules depend on this.
// =============================================================================

// Vulkan abstractions
pub mod allocator;
pub mod command;
pub mod context;
pub mod device;
pub mod surface;

// Deterministic render graph
pub mod frame_graph;

// Universal importance system (LOD, streaming, AI priority)
pub mod importance_map;

// Error handling (ReactorError, ReactorResult, ErrorCode)
pub mod error;

// --- New UE5-style core subsystems ------------------------------------------

/// Hierarchical profiler (tracing-backed, Tracy-ready).
/// Use `profile_scope!("name")` to instrument any scope.
pub mod profiler;

/// Structured logging (tracing-subscriber, env-filter via `REACTOR_LOG`).
pub mod logging;

/// Parallel job system (rayon-backed, UE5 TaskGraph-style).
/// Use `jobs::parallel_for`, `jobs::join`, `jobs::scope` for concurrency.
pub mod jobs;

/// Frame-scoped linear allocator (bump allocator, zero-fragmentation).
/// Use `LinearAllocator` or `BumpArena` for per-frame scratch memory.
pub mod linear_allocator;

// =============================================================================
// Re-exports
// =============================================================================

pub use allocator::MemoryAllocator;
pub use command::CommandManager;
pub use context::VulkanContext;
pub use device::DeviceInfo;
pub use error::{
    clear_last_error, get_last_error_code, get_last_error_message, has_error, set_last_error,
    ErrorCode, ReactorError, ReactorResult,
};
pub use frame_graph::{
    Barrier, FrameGraph, FrameGraphStats, PassDesc, PassId, ResourceFormat, ResourceId,
    ResourceType,
};
pub use importance_map::{
    ImportanceMap, ImportanceMapConfig, ImportanceMapStats, ImportanceTileData, ImportanceType,
};

// Profiler
pub use profiler::{begin_frame, get_frame_id, CpuTimer, PerfCounter};

// Logging
pub use logging::{init_logger, init_logger_with, LogLevel};

// Jobs
pub use jobs::{init_job_system, join, par_iter, par_iter_mut, parallel_for, scope};

// Allocators
pub use linear_allocator::{BumpArena, LinearAllocator};
