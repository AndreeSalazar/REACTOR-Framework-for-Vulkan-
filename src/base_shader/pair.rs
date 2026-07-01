use super::asset::BaseShaderAsset;
use super::stage::BaseShaderStage;

#[derive(Clone, Debug)]
pub struct BaseShaderPair {
    pub vertex: Vec<u32>,
    pub fragment: Vec<u32>,
}

impl BaseShaderPair {
    pub fn new(vertex: BaseShaderAsset, fragment: BaseShaderAsset) -> Self {
        debug_assert_eq!(vertex.stage(), BaseShaderStage::Vertex);
        debug_assert_eq!(fragment.stage(), BaseShaderStage::Fragment);
        Self { vertex: vertex.words(), fragment: fragment.words() }
    }
}

#[derive(Clone, Debug)]
pub struct DeferredKit {
    pub gbuffer: BaseShaderPair,
}

impl Default for DeferredKit {
    fn default() -> Self {
        Self { gbuffer: BaseShaderPair::new(BaseShaderAsset::GBufferVert, BaseShaderAsset::GBufferFrag) }
    }
}

#[derive(Clone, Debug)]
pub struct PostComputeKit {
    pub bloom_downsample: Vec<u32>,
    pub bloom_upsample: Vec<u32>,
    pub depth_resolve: Vec<u32>,
    pub taa_resolve: Vec<u32>,
    pub gtao: Vec<u32>,
    pub light_cull: Vec<u32>,
}

impl Default for PostComputeKit {
    fn default() -> Self {
        Self {
            bloom_downsample: BaseShaderAsset::BloomDownsample.words(),
            bloom_upsample: BaseShaderAsset::BloomUpsample.words(),
            depth_resolve: BaseShaderAsset::DepthResolve.words(),
            taa_resolve: BaseShaderAsset::TaaResolve.words(),
            gtao: BaseShaderAsset::Gtao.words(),
            light_cull: BaseShaderAsset::LightCull.words(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IblBakeKit {
    pub equirect_to_cube: Vec<u32>,
    pub irradiance: Vec<u32>,
    pub prefilter: Vec<u32>,
    pub brdf_lut: Vec<u32>,
}

impl Default for IblBakeKit {
    fn default() -> Self {
        Self {
            equirect_to_cube: BaseShaderAsset::IblEquirectToCube.words(),
            irradiance: BaseShaderAsset::IblIrradiance.words(),
            prefilter: BaseShaderAsset::IblPrefilter.words(),
            brdf_lut: BaseShaderAsset::IblBrdfLut.words(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BaseMaterialDefaults {
    pub color: glam::Vec4,
    pub metallic: f32,
    pub roughness: f32,
}

impl Default for BaseMaterialDefaults {
    fn default() -> Self { Self { color: glam::Vec4::ONE, metallic: 0.0, roughness: 0.5 } }
}
