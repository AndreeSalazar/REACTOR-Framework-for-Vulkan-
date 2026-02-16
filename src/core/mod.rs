// REACTOR Core Module
// Contains fundamental Vulkan abstractions and context management

pub mod context;
pub mod device;
pub mod surface;
pub mod allocator;
pub mod command;
pub mod frame_graph;
pub mod importance_map;
pub mod error;

pub use context::VulkanContext;
pub use device::DeviceInfo;
pub use allocator::MemoryAllocator;
pub use command::CommandManager;
pub use frame_graph::{FrameGraph, PassId, ResourceId, ResourceType, ResourceFormat, PassDesc, Barrier, FrameGraphStats};
pub use importance_map::{ImportanceMap, ImportanceMapConfig, ImportanceTileData, ImportanceMapStats, ImportanceType};
pub use error::{ReactorError, ReactorResult, ErrorCode, set_last_error, get_last_error_code, get_last_error_message, clear_last_error, has_error};
