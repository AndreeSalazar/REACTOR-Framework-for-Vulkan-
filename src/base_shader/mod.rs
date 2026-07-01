pub mod asset;
pub mod cookbook;
pub mod family;
pub mod pair;
pub mod stage;

pub use asset::BaseShaderAsset;
pub use cookbook::BaseShaderCookbook;
pub use family::BaseShaderFamily;
pub use pair::{BaseMaterialDefaults, BaseShaderPair, DeferredKit, IblBakeKit, PostComputeKit};
pub use stage::BaseShaderStage;

pub fn read_spv(bytes: &[u8]) -> Vec<u32> {
    ash::util::read_spv(&mut std::io::Cursor::new(bytes))
        .expect("Embedded SPIR-V is invalid; rebuild shaders with `cargo check`")
}
