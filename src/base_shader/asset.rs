use super::family::BaseShaderFamily;
use super::stage::BaseShaderStage;
use super::read_spv;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseShaderAsset {
    CoreVert, CoreFrag,
    TextureVert, TextureFrag,
    BlenderLiveVert, BlenderLiveFrag,
    ShadowVert, ShadowFrag,
    PostProcessVert, PostProcessFrag,
    DecalFrag,
    GBufferVert, GBufferFrag,
    BloomDownsample, BloomUpsample,
    DepthResolve, TaaResolve,
    Gtao, LightCull,
    IblEquirectToCube, IblIrradiance, IblPrefilter, IblBrdfLut,
}

impl BaseShaderAsset {
    pub const ALL: &'static [Self] = &[
        Self::CoreVert, Self::CoreFrag,
        Self::TextureVert, Self::TextureFrag,
        Self::BlenderLiveVert, Self::BlenderLiveFrag,
        Self::ShadowVert, Self::ShadowFrag,
        Self::PostProcessVert, Self::PostProcessFrag, Self::DecalFrag,
        Self::GBufferVert, Self::GBufferFrag,
        Self::BloomDownsample, Self::BloomUpsample,
        Self::DepthResolve, Self::TaaResolve,
        Self::Gtao, Self::LightCull,
        Self::IblEquirectToCube, Self::IblIrradiance, Self::IblPrefilter, Self::IblBrdfLut,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::CoreVert => "core.forward.vert", Self::CoreFrag => "core.forward.frag",
            Self::TextureVert => "core.textured.vert", Self::TextureFrag => "core.textured.frag",
            Self::BlenderLiveVert => "live.blender_pbr.vert", Self::BlenderLiveFrag => "live.blender_pbr.frag",
            Self::ShadowVert => "live.shadow.vert", Self::ShadowFrag => "live.shadow.frag",
            Self::PostProcessVert => "post.fullscreen.vert", Self::PostProcessFrag => "post.fullscreen.frag",
            Self::DecalFrag => "post.decal.frag",
            Self::GBufferVert => "deferred.gbuffer.vert", Self::GBufferFrag => "deferred.gbuffer.frag",
            Self::BloomDownsample => "post.bloom_downsample.comp", Self::BloomUpsample => "post.bloom_upsample.comp",
            Self::DepthResolve => "post.depth_resolve.comp", Self::TaaResolve => "post.taa_resolve.comp",
            Self::Gtao => "post.gtao.comp", Self::LightCull => "compute.light_cull.comp",
            Self::IblEquirectToCube => "ibl.equirect_to_cube.comp",
            Self::IblIrradiance => "ibl.irradiance.comp", Self::IblPrefilter => "ibl.prefilter.comp",
            Self::IblBrdfLut => "ibl.brdf_lut.comp",
        }
    }

    pub fn stage(self) -> BaseShaderStage {
        match self {
            Self::CoreVert | Self::TextureVert | Self::BlenderLiveVert | Self::ShadowVert
            | Self::PostProcessVert | Self::GBufferVert => BaseShaderStage::Vertex,
            Self::CoreFrag | Self::TextureFrag | Self::BlenderLiveFrag | Self::ShadowFrag
            | Self::PostProcessFrag | Self::DecalFrag | Self::GBufferFrag => BaseShaderStage::Fragment,
            Self::BloomDownsample | Self::BloomUpsample | Self::DepthResolve | Self::TaaResolve
            | Self::Gtao | Self::LightCull | Self::IblEquirectToCube | Self::IblIrradiance
            | Self::IblPrefilter | Self::IblBrdfLut => BaseShaderStage::Compute,
        }
    }

    pub fn family(self) -> BaseShaderFamily {
        match self {
            Self::CoreVert | Self::CoreFrag => BaseShaderFamily::CoreForward,
            Self::TextureVert | Self::TextureFrag => BaseShaderFamily::CoreTextured,
            Self::BlenderLiveVert | Self::BlenderLiveFrag => BaseShaderFamily::BlenderLivePbr,
            Self::ShadowVert | Self::ShadowFrag => BaseShaderFamily::ShadowDepth,
            Self::GBufferVert | Self::GBufferFrag => BaseShaderFamily::Deferred,
            Self::PostProcessVert | Self::PostProcessFrag | Self::DecalFrag => BaseShaderFamily::PostFullscreen,
            Self::BloomDownsample | Self::BloomUpsample | Self::DepthResolve | Self::TaaResolve
            | Self::Gtao | Self::LightCull => BaseShaderFamily::PostCompute,
            Self::IblEquirectToCube | Self::IblIrradiance | Self::IblPrefilter | Self::IblBrdfLut => BaseShaderFamily::IblBake,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::CoreVert => "Forward vert: mesh transform + normal + UV + vertex color",
            Self::CoreFrag => "Forward frag: vertex color + half-lambert simple",
            Self::TextureVert => "Textured vert: pasa UV a fragment",
            Self::TextureFrag => "Textured frag: sampler único diffuse",
            Self::BlenderLiveVert => "Blender Live vert: world pos + view dir para PBR",
            Self::BlenderLiveFrag => "Blender Live frag: PBR Cook-Torrance + IBL HD + CSM PCF + rim",
            Self::ShadowVert => "Shadow depth-only vert: transform a clip space del cascade",
            Self::ShadowFrag => "Shadow depth-only frag: vacío (sólo depth)",
            Self::PostProcessVert => "Fullscreen triangle vert para passes de post",
            Self::PostProcessFrag => "Compositor fullscreen: tone map, bloom, SSGI, SSR, fog, LUT, grain",
            Self::DecalFrag => "Decal projection: projects textures onto G-Buffer depth geometry",
            Self::GBufferVert => "G-Buffer vert: world pos + normal + UV + vertex color",
            Self::GBufferFrag => "G-Buffer frag: escribe 4 attachments (albedo/AO, normal/material, emissive, motion/depth/flags)",
            Self::BloomDownsample => "Bloom 13-tap Karis downsample con threshold mip 0",
            Self::BloomUpsample => "Bloom 9-tap upsample con tent filter",
            Self::DepthResolve => "Depth MSAA → single-sample R32F (mín sample)",
            Self::TaaResolve => "TAA resolve: reproyección + neighborhood clip + depth reject",
            Self::Gtao => "GTAO compute: 8 dirs × 4 steps con rotación temporal",
            Self::LightCull => "Clustered light culling: 16x16 tiles, 256 lights max per tile",
            Self::IblEquirectToCube => "IBL: equirectangular HDR 2D → cubemap radiance",
            Self::IblIrradiance => "IBL: irradiance cubemap difuso (Lambert)",
            Self::IblPrefilter => "IBL: prefilter especular GGX (5 mips, Karis 2014)",
            Self::IblBrdfLut => "IBL: BRDF integration LUT 2D (scale + bias Fresnel)",
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            Self::CoreVert => "shaders/vert.spv", Self::CoreFrag => "shaders/frag.spv",
            Self::TextureVert => "shaders/texture_vert.spv", Self::TextureFrag => "shaders/texture_frag.spv",
            Self::BlenderLiveVert => "shaders/blender_live_vert.spv", Self::BlenderLiveFrag => "shaders/blender_live_frag.spv",
            Self::ShadowVert => "shaders/shadow_vert.spv", Self::ShadowFrag => "shaders/shadow_frag.spv",
            Self::PostProcessVert => "shaders/post_process_vert.spv", Self::PostProcessFrag => "shaders/post_process_frag.spv",
            Self::DecalFrag => "shaders/post/decal.spv",
            Self::GBufferVert => "shaders/deferred/gbuffer_vert.spv", Self::GBufferFrag => "shaders/deferred/gbuffer_frag.spv",
            Self::BloomDownsample => "shaders/post/bloom_downsample.spv", Self::BloomUpsample => "shaders/post/bloom_upsample.spv",
            Self::DepthResolve => "shaders/post/depth_resolve.spv", Self::TaaResolve => "shaders/post/taa_resolve.spv",
            Self::Gtao => "shaders/post/gtao.spv", Self::LightCull => "shaders/compute/light_cull.spv",
            Self::IblEquirectToCube => "shaders/ibl/equirect_to_cube.spv", Self::IblIrradiance => "shaders/ibl/irradiance.spv",
            Self::IblPrefilter => "shaders/ibl/prefilter.spv", Self::IblBrdfLut => "shaders/ibl/brdf_lut.spv",
        }
    }

    pub fn source_path(self) -> Option<&'static str> {
        match self {
            Self::CoreVert => Some("shaders/core/shader.vert"), Self::CoreFrag => Some("shaders/core/shader.frag"),
            Self::TextureVert => Some("shaders/core/texture.vert"), Self::TextureFrag => Some("shaders/core/texture.frag"),
            Self::BlenderLiveVert => Some("shaders/live/blender_live.vert"), Self::BlenderLiveFrag => Some("shaders/live/blender_live.frag"),
            Self::ShadowVert => Some("shaders/live/shadow.vert"), Self::ShadowFrag => Some("shaders/live/shadow.frag"),
            Self::PostProcessVert => Some("shaders/post/post_process.vert"), Self::PostProcessFrag => Some("shaders/post/post_process.frag"),
            Self::DecalFrag => Some("shaders/post/decal.frag"),
            Self::GBufferVert => Some("shaders/deferred/gbuffer.vert"), Self::GBufferFrag => Some("shaders/deferred/gbuffer.frag"),
            Self::BloomDownsample => Some("shaders/post/bloom_downsample.comp"), Self::BloomUpsample => Some("shaders/post/bloom_upsample.comp"),
            Self::DepthResolve => Some("shaders/post/depth_resolve.comp"), Self::TaaResolve => Some("shaders/post/taa_resolve.comp"),
            Self::Gtao => Some("shaders/post/gtao.comp"), Self::LightCull => Some("shaders/compute/light_cull.comp"),
            Self::IblEquirectToCube => Some("shaders/ibl/equirect_to_cube.comp"),
            Self::IblIrradiance => Some("shaders/ibl/irradiance.comp"), Self::IblPrefilter => Some("shaders/ibl/prefilter.comp"),
            Self::IblBrdfLut => Some("shaders/ibl/brdf_lut.comp"),
        }
    }

    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::CoreVert => include_bytes!("../../shaders/vert.spv"), Self::CoreFrag => include_bytes!("../../shaders/frag.spv"),
            Self::TextureVert => include_bytes!("../../shaders/texture_vert.spv"), Self::TextureFrag => include_bytes!("../../shaders/texture_frag.spv"),
            Self::BlenderLiveVert => include_bytes!("../../shaders/blender_live_vert.spv"), Self::BlenderLiveFrag => include_bytes!("../../shaders/blender_live_frag.spv"),
            Self::ShadowVert => include_bytes!("../../shaders/shadow_vert.spv"), Self::ShadowFrag => include_bytes!("../../shaders/shadow_frag.spv"),
            Self::PostProcessVert => include_bytes!("../../shaders/post_process_vert.spv"), Self::PostProcessFrag => include_bytes!("../../shaders/post_process_frag.spv"),
            Self::DecalFrag => include_bytes!("../../shaders/post/decal.spv"),
            Self::GBufferVert => include_bytes!("../../shaders/deferred/gbuffer_vert.spv"), Self::GBufferFrag => include_bytes!("../../shaders/deferred/gbuffer_frag.spv"),
            Self::BloomDownsample => include_bytes!("../../shaders/post/bloom_downsample.spv"), Self::BloomUpsample => include_bytes!("../../shaders/post/bloom_upsample.spv"),
            Self::DepthResolve => include_bytes!("../../shaders/post/depth_resolve.spv"), Self::TaaResolve => include_bytes!("../../shaders/post/taa_resolve.spv"),
            Self::Gtao => include_bytes!("../../shaders/post/gtao.spv"), Self::LightCull => include_bytes!("../../shaders/compute/light_cull.spv"),
            Self::IblEquirectToCube => include_bytes!("../../shaders/ibl/equirect_to_cube.spv"),
            Self::IblIrradiance => include_bytes!("../../shaders/ibl/irradiance.spv"), Self::IblPrefilter => include_bytes!("../../shaders/ibl/prefilter.spv"),
            Self::IblBrdfLut => include_bytes!("../../shaders/ibl/brdf_lut.spv"),
        }
    }

    pub fn words(self) -> Vec<u32> { read_spv(self.bytes()) }
    pub fn byte_len(self) -> usize { self.bytes().len() }

    pub fn from_name(name: &str) -> Option<Self> { Self::ALL.iter().copied().find(|a| a.name() == name) }
    pub fn from_path(path: &str) -> Option<Self> {
        let normalised = path.replace('\\', "/");
        Self::ALL.iter().copied().find(|a| a.path() == normalised.as_str())
    }
    pub fn iter_stage(stage: BaseShaderStage) -> impl Iterator<Item = Self> {
        Self::ALL.iter().copied().filter(move |a| a.stage() == stage)
    }
    pub fn iter_family(family: BaseShaderFamily) -> impl Iterator<Item = Self> {
        Self::ALL.iter().copied().filter(move |a| a.family() == family)
    }
}
