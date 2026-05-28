//! Core framework modules
//!
//! This module contains the foundational systems that power REACTOR.

// Vulkan abstractions
pub mod allocator;
pub mod arc_handle;
pub mod command;
pub mod context;
pub mod debug_utils;
pub mod device;
pub mod memory_budget;
pub mod surface;
pub mod vrs;

// Deterministic render graph
pub mod frame_graph;

// Universal importance system (LOD, streaming, AI priority)
pub mod importance_map;

// Error handling (ReactorError, ReactorResult, ErrorCode)
pub mod error;

// --- Core subsystems ----------------------------------------------------------

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
pub use arc_handle::{ArcDevice, ArcInstance, ArcSurface};
pub use command::CommandManager;
pub use context::VulkanContext;
pub use debug_utils::DebugNamer;
pub use device::DeviceInfo;
pub use error::{ErrorCode, ReactorError, ReactorResult};
pub use frame_graph::{
    Barrier, FrameGraph, FrameGraphStats, PassDesc, PassId, ResourceFormat, ResourceId,
    ResourceType,
};
pub use importance_map::{
    ImportanceMap, ImportanceMapConfig, ImportanceMapStats, ImportanceTileData, ImportanceType,
};
pub use memory_budget::{GpuMemoryBudget, HeapBudget};
pub use vrs::{
    PixelIntelligent, PixelIntelligentProfile, VrsCapabilities, VrsContext, VrsRate,
    VrsSupportedRate,
};

// Profiler
pub use profiler::{begin_frame, get_frame_id, CpuTimer, PerfCounter};

// Logging
pub use logging::{init_logger, init_logger_with, LogLevel};

// Jobs
pub use jobs::{init_job_system, join, par_iter, par_iter_mut, parallel_for, scope};

// Allocators
pub use linear_allocator::{BumpArena, LinearAllocator};
