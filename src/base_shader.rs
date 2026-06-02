use crate::graphics::post_process::{PostProcessEffect, PostProcessPipeline, PostProcessSettings};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseShaderAsset {
    CoreVert,
    CoreFrag,
    TextureVert,
    TextureFrag,
    BlenderLiveVert,
    BlenderLiveFrag,
    ShadowVert,
    ShadowFrag,
    PostProcessVert,
    PostProcessFrag,
    GBufferVert,
    GBufferFrag,
    BloomDownsample,
    BloomUpsample,
    DepthResolve,
    TaaResolve,
    Gtao,
    IblEquirectToCube,
    IblIrradiance,
    IblPrefilter,
    IblBrdfLut,
}

impl BaseShaderAsset {
    pub const ALL: [Self; 21] = [
        Self::CoreVert,
        Self::CoreFrag,
        Self::TextureVert,
        Self::TextureFrag,
        Self::BlenderLiveVert,
        Self::BlenderLiveFrag,
        Self::ShadowVert,
        Self::ShadowFrag,
        Self::PostProcessVert,
        Self::PostProcessFrag,
        Self::GBufferVert,
        Self::GBufferFrag,
        Self::BloomDownsample,
        Self::BloomUpsample,
        Self::DepthResolve,
        Self::TaaResolve,
        Self::Gtao,
        Self::IblEquirectToCube,
        Self::IblIrradiance,
        Self::IblPrefilter,
        Self::IblBrdfLut,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::CoreVert => "core.forward.vert",
            Self::CoreFrag => "core.forward.frag",
            Self::TextureVert => "core.textured.vert",
            Self::TextureFrag => "core.textured.frag",
            Self::BlenderLiveVert => "live.blender_pbr.vert",
            Self::BlenderLiveFrag => "live.blender_pbr.frag",
            Self::ShadowVert => "live.shadow.vert",
            Self::ShadowFrag => "live.shadow.frag",
            Self::PostProcessVert => "post.fullscreen.vert",
            Self::PostProcessFrag => "post.fullscreen.frag",
            Self::GBufferVert => "deferred.gbuffer.vert",
            Self::GBufferFrag => "deferred.gbuffer.frag",
            Self::BloomDownsample => "post.bloom_downsample.comp",
            Self::BloomUpsample => "post.bloom_upsample.comp",
            Self::DepthResolve => "post.depth_resolve.comp",
            Self::TaaResolve => "post.taa_resolve.comp",
            Self::Gtao => "post.gtao.comp",
            Self::IblEquirectToCube => "ibl.equirect_to_cube.comp",
            Self::IblIrradiance => "ibl.irradiance.comp",
            Self::IblPrefilter => "ibl.prefilter.comp",
            Self::IblBrdfLut => "ibl.brdf_lut.comp",
        }
    }

    pub fn stage(self) -> BaseShaderStage {
        match self {
            Self::CoreVert
            | Self::TextureVert
            | Self::BlenderLiveVert
            | Self::ShadowVert
            | Self::PostProcessVert
            | Self::GBufferVert => BaseShaderStage::Vertex,
            Self::CoreFrag
            | Self::TextureFrag
            | Self::BlenderLiveFrag
            | Self::ShadowFrag
            | Self::PostProcessFrag
            | Self::GBufferFrag => BaseShaderStage::Fragment,
            Self::BloomDownsample
            | Self::BloomUpsample
            | Self::DepthResolve
            | Self::TaaResolve
            | Self::Gtao
            | Self::IblEquirectToCube
            | Self::IblIrradiance
            | Self::IblPrefilter
            | Self::IblBrdfLut => BaseShaderStage::Compute,
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            Self::CoreVert => "shaders/vert.spv",
            Self::CoreFrag => "shaders/frag.spv",
            Self::TextureVert => "shaders/texture_vert.spv",
            Self::TextureFrag => "shaders/texture_frag.spv",
            Self::BlenderLiveVert => "shaders/blender_live_vert.spv",
            Self::BlenderLiveFrag => "shaders/blender_live_frag.spv",
            Self::ShadowVert => "shaders/shadow_vert.spv",
            Self::ShadowFrag => "shaders/shadow_frag.spv",
            Self::PostProcessVert => "shaders/post_process_vert.spv",
            Self::PostProcessFrag => "shaders/post_process_frag.spv",
            Self::GBufferVert => "shaders/deferred/gbuffer_vert.spv",
            Self::GBufferFrag => "shaders/deferred/gbuffer_frag.spv",
            Self::BloomDownsample => "shaders/post/bloom_downsample.spv",
            Self::BloomUpsample => "shaders/post/bloom_upsample.spv",
            Self::DepthResolve => "shaders/post/depth_resolve.spv",
            Self::TaaResolve => "shaders/post/taa_resolve.spv",
            Self::Gtao => "shaders/post/gtao.spv",
            Self::IblEquirectToCube => "shaders/ibl/equirect_to_cube.spv",
            Self::IblIrradiance => "shaders/ibl/irradiance.spv",
            Self::IblPrefilter => "shaders/ibl/prefilter.spv",
            Self::IblBrdfLut => "shaders/ibl/brdf_lut.spv",
        }
    }

    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::CoreVert => include_bytes!("../shaders/vert.spv"),
            Self::CoreFrag => include_bytes!("../shaders/frag.spv"),
            Self::TextureVert => include_bytes!("../shaders/texture_vert.spv"),
            Self::TextureFrag => include_bytes!("../shaders/texture_frag.spv"),
            Self::BlenderLiveVert => include_bytes!("../shaders/blender_live_vert.spv"),
            Self::BlenderLiveFrag => include_bytes!("../shaders/blender_live_frag.spv"),
            Self::ShadowVert => include_bytes!("../shaders/shadow_vert.spv"),
            Self::ShadowFrag => include_bytes!("../shaders/shadow_frag.spv"),
            Self::PostProcessVert => include_bytes!("../shaders/post_process_vert.spv"),
            Self::PostProcessFrag => include_bytes!("../shaders/post_process_frag.spv"),
            Self::GBufferVert => include_bytes!("../shaders/deferred/gbuffer_vert.spv"),
            Self::GBufferFrag => include_bytes!("../shaders/deferred/gbuffer_frag.spv"),
            Self::BloomDownsample => include_bytes!("../shaders/post/bloom_downsample.spv"),
            Self::BloomUpsample => include_bytes!("../shaders/post/bloom_upsample.spv"),
            Self::DepthResolve => include_bytes!("../shaders/post/depth_resolve.spv"),
            Self::TaaResolve => include_bytes!("../shaders/post/taa_resolve.spv"),
            Self::Gtao => include_bytes!("../shaders/post/gtao.spv"),
            Self::IblEquirectToCube => include_bytes!("../shaders/ibl/equirect_to_cube.spv"),
            Self::IblIrradiance => include_bytes!("../shaders/ibl/irradiance.spv"),
            Self::IblPrefilter => include_bytes!("../shaders/ibl/prefilter.spv"),
            Self::IblBrdfLut => include_bytes!("../shaders/ibl/brdf_lut.spv"),
        }
    }

    pub fn words(self) -> Vec<u32> {
        read_spv(self.bytes())
    }
}

#[derive(Clone, Debug)]
pub struct BaseShaderPair {
    pub vertex: Vec<u32>,
    pub fragment: Vec<u32>,
}

impl BaseShaderPair {
    pub fn new(vertex: BaseShaderAsset, fragment: BaseShaderAsset) -> Self {
        Self {
            vertex: vertex.words(),
            fragment: fragment.words(),
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
    fn default() -> Self {
        Self {
            color: glam::Vec4::ONE,
            metallic: 0.0,
            roughness: 0.5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BaseShaderCookbook {
    pub forward: BaseShaderPair,
    pub textured: BaseShaderPair,
    pub blender_live_pbr: BaseShaderPair,
    pub gbuffer: BaseShaderPair,
    pub shadow_depth: BaseShaderPair,
    pub post_process: BaseShaderPair,
    pub bloom_downsample: Vec<u32>,
    pub bloom_upsample: Vec<u32>,
    pub depth_resolve: Vec<u32>,
    pub taa_resolve: Vec<u32>,
    pub gtao: Vec<u32>,
    pub ibl_equirect_to_cube: Vec<u32>,
    pub ibl_irradiance: Vec<u32>,
    pub ibl_prefilter: Vec<u32>,
    pub ibl_brdf_lut: Vec<u32>,
    pub material: BaseMaterialDefaults,
    pub post_settings: PostProcessSettings,
    pub post_enabled: bool,
}

impl Default for BaseShaderCookbook {
    fn default() -> Self {
        Self {
            forward: BaseShaderPair::new(BaseShaderAsset::CoreVert, BaseShaderAsset::CoreFrag),
            textured: BaseShaderPair::new(
                BaseShaderAsset::TextureVert,
                BaseShaderAsset::TextureFrag,
            ),
            blender_live_pbr: BaseShaderPair::new(
                BaseShaderAsset::BlenderLiveVert,
                BaseShaderAsset::BlenderLiveFrag,
            ),
            gbuffer: BaseShaderPair::new(
                BaseShaderAsset::GBufferVert,
                BaseShaderAsset::GBufferFrag,
            ),
            shadow_depth: BaseShaderPair::new(
                BaseShaderAsset::ShadowVert,
                BaseShaderAsset::ShadowFrag,
            ),
            post_process: BaseShaderPair::new(
                BaseShaderAsset::PostProcessVert,
                BaseShaderAsset::PostProcessFrag,
            ),
            bloom_downsample: BaseShaderAsset::BloomDownsample.words(),
            bloom_upsample: BaseShaderAsset::BloomUpsample.words(),
            depth_resolve: BaseShaderAsset::DepthResolve.words(),
            taa_resolve: BaseShaderAsset::TaaResolve.words(),
            gtao: BaseShaderAsset::Gtao.words(),
            ibl_equirect_to_cube: BaseShaderAsset::IblEquirectToCube.words(),
            ibl_irradiance: BaseShaderAsset::IblIrradiance.words(),
            ibl_prefilter: BaseShaderAsset::IblPrefilter.words(),
            ibl_brdf_lut: BaseShaderAsset::IblBrdfLut.words(),
            material: BaseMaterialDefaults::default(),
            post_settings: PostProcessSettings::default(),
            post_enabled: true,
        }
    }
}

impl BaseShaderCookbook {
    pub fn blender_live() -> Self {
        let mut cookbook = Self::cinematic_aaa();
        cookbook.post_settings.exposure = 1.04;
        cookbook.post_settings.bloom_threshold = 0.7;
        cookbook.post_settings.bloom_intensity = 0.5;
        cookbook.post_settings.ssgi_intensity = 0.28;
        cookbook.post_settings.ssgi_radius = 10.0;
        cookbook.post_settings.ssr_strength = 0.38;
        cookbook.post_settings.fog_density = 0.12;
        cookbook.post_settings.fog_scatter = 0.42;
        cookbook.post_settings.flare_intensity = 0.35;
        cookbook.post_settings.highlight_recovery = 0.74;
        cookbook.material.metallic = 0.0;
        cookbook.material.roughness = 0.48;
        cookbook
    }

    pub fn cinematic_aaa() -> Self {
        let mut cookbook = Self::default();
        cookbook.post_settings = PostProcessSettings::cinematic();
        cookbook.post_settings.exposure = 1.08;
        cookbook.post_settings.bloom_threshold = 0.62;
        cookbook.post_settings.bloom_intensity = 0.78;
        cookbook.post_settings.ssgi_intensity = 0.34;
        cookbook.post_settings.ssgi_radius = 14.0;
        cookbook.post_settings.ssr_strength = 0.46;
        cookbook.post_settings.fog_density = 0.24;
        cookbook.post_settings.fog_scatter = 0.58;
        cookbook.post_settings.flare_intensity = 0.52;
        cookbook.post_settings.highlight_recovery = 0.82;
        cookbook
            .post_settings
            .enable_effect(PostProcessEffect::Bloom);
        cookbook
            .post_settings
            .enable_effect(PostProcessEffect::SSGI);
        cookbook.post_settings.enable_effect(PostProcessEffect::SSR);
        cookbook
            .post_settings
            .enable_effect(PostProcessEffect::VolumetricFog);
        cookbook
            .post_settings
            .enable_effect(PostProcessEffect::LutColorGrading);
        cookbook
            .post_settings
            .enable_effect(PostProcessEffect::ToneMapping);
        cookbook
            .post_settings
            .enable_effect(PostProcessEffect::AnamorphicFlares);
        cookbook.material.roughness = 0.42;
        cookbook
    }

    pub fn xenofall_showcase() -> Self {
        let mut cookbook = Self::cinematic_aaa();
        cookbook.material.color = glam::Vec4::new(0.34, 0.36, 0.35, 1.0);
        cookbook.material.metallic = 0.05;
        cookbook.material.roughness = 0.55;
        cookbook
    }

    pub fn apply_to_post_process(&self, post_process: &mut PostProcessPipeline) {
        post_process.enabled = self.post_enabled;
        post_process.settings = self.post_settings;
    }

    pub fn shader_manifest(&self) -> Vec<(&'static str, BaseShaderStage, &'static str)> {
        BaseShaderAsset::ALL
            .iter()
            .map(|asset| (asset.name(), asset.stage(), asset.path()))
            .collect()
    }
}

pub fn read_spv(bytes: &[u8]) -> Vec<u32> {
    ash::util::read_spv(&mut std::io::Cursor::new(bytes))
        .expect("Embedded SPIR-V is invalid; rebuild shaders with `cargo check`")
}
