pub mod config;
pub mod feature;
pub mod gpu_data;
pub mod handle;
pub mod registry;

pub use config::{BindlessConfig, BindlessStats};
pub use feature::{bindless_feature_chain, check_bindless_support};
pub use gpu_data::{GpuMaterialData, GpuMeshData};
pub use handle::{BufferHandle, MaterialHandle, MeshHandle, SamplerHandle, TextureHandle};
pub use registry::BindlessRegistry;
