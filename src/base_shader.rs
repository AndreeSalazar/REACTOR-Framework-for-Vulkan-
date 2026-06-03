// =============================================================================
// REACTOR · base_shader.rs — Cookbook central de SPIR-V embebido
// =============================================================================
// Hereda de `shaders/` (compilado a SPV por `build.rs`) y expone:
//
//   • `BaseShaderAsset`  — enum único con TODOS los binarios SPIR-V del repo,
//                          con metadatos (stage, family, descripción, paths).
//   • `BaseShaderPair`   — (vertex, fragment) pre-cargado en `Vec<u32>`.
//   • `DeferredKit`      — pareja gbuffer + helpers para deferred shading.
//   • `IblBakeKit`       — los 4 compute shaders de baking IBL (equirect →
//                          cube → irradiance / prefilter + BRDF LUT).
//   • `PostComputeKit`   — compute shaders de post-process (bloom, depth
//                          resolve MSAA, TAA resolve, GTAO).
//   • `BaseShaderCookbook` — agrupador top-level con todos los kits +
//                            material defaults + presets de post-process
//                            mutables por juego (`cinematic_aaa`, `mobile_low`,
//                            `performance`, `aaa_ultra`, `blender_live`,
//                            `xenofall_showcase`).
//
// Diseño: zero-cost al runtime — todo `include_bytes!`. Las APIs nuevas
// (family/description/from_name/iter_*) son helpers de ergonomía; las APIs
// previas (`ALL`, `name`, `stage`, `path`, `bytes`, `words`, `shader_manifest`,
// `apply_to_post_process`) se mantienen iguales para no romper consumidores.
// =============================================================================

use crate::graphics::post_process::{
    AAQualityPreset, AASettings, PostProcessEffect, PostProcessPipeline, PostProcessSettings,
};
use crate::graphics::shadows::ShadowConfig;

// =============================================================================
// Stage / Family — clasificación de cada SPIR-V
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl BaseShaderStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vertex => "vertex",
            Self::Fragment => "fragment",
            Self::Compute => "compute",
        }
    }
}

/// Agrupación lógica de cada shader según el subsistema gráfico al que
/// pertenece. Permite iterar por familia, generar manifests por sección
/// y razonar sobre qué hace cada SPIR-V sin tener que abrir el `.frag`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseShaderFamily {
    /// Forward shading clásico (mesh → luz simple por vertex color).
    CoreForward,
    /// Forward texturizado (sampler único).
    CoreTextured,
    /// PBR Blender Live (mini-AAA: PBR + IBL + CSM + sombras suaves).
    BlenderLivePbr,
    /// Cascaded Shadow Map depth-only pass.
    ShadowDepth,
    /// Pase de geometría deferred a G-Buffer de 4 attachments.
    Deferred,
    /// Fullscreen pass de post-process compositor.
    PostFullscreen,
    /// Compute helpers de post-process (bloom, TAA, GTAO, depth resolve).
    PostCompute,
    /// Compute kit de baking IBL (equirect → cube → irradiance / prefilter / BRDF LUT).
    IblBake,
}

impl BaseShaderFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CoreForward => "core.forward",
            Self::CoreTextured => "core.textured",
            Self::BlenderLivePbr => "live.pbr",
            Self::ShadowDepth => "shadow.depth",
            Self::Deferred => "deferred.gbuffer",
            Self::PostFullscreen => "post.fullscreen",
            Self::PostCompute => "post.compute",
            Self::IblBake => "ibl.bake",
        }
    }
}

// =============================================================================
// BaseShaderAsset — enum con todos los binarios SPIR-V del repo
// =============================================================================

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
    /// Lista canónica de todos los assets — tamaño calculado por el compilador.
    /// Para añadir un shader nuevo basta con: (1) añadir variante al enum,
    /// (2) añadirla aquí, (3) rellenar los `match` de name/stage/path/bytes/...
    /// El check de exhaustividad del compilador hace el resto.
    pub const ALL: &'static [Self] = &[
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

    /// Identificador legible "namespace.dot.case" — útil para logs/manifests.
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

    pub fn family(self) -> BaseShaderFamily {
        match self {
            Self::CoreVert | Self::CoreFrag => BaseShaderFamily::CoreForward,
            Self::TextureVert | Self::TextureFrag => BaseShaderFamily::CoreTextured,
            Self::BlenderLiveVert | Self::BlenderLiveFrag => BaseShaderFamily::BlenderLivePbr,
            Self::ShadowVert | Self::ShadowFrag => BaseShaderFamily::ShadowDepth,
            Self::GBufferVert | Self::GBufferFrag => BaseShaderFamily::Deferred,
            Self::PostProcessVert | Self::PostProcessFrag => BaseShaderFamily::PostFullscreen,
            Self::BloomDownsample
            | Self::BloomUpsample
            | Self::DepthResolve
            | Self::TaaResolve
            | Self::Gtao => BaseShaderFamily::PostCompute,
            Self::IblEquirectToCube
            | Self::IblIrradiance
            | Self::IblPrefilter
            | Self::IblBrdfLut => BaseShaderFamily::IblBake,
        }
    }

    /// Descripción humana de una línea — útil para tooling/editor/inspector.
    pub fn description(self) -> &'static str {
        match self {
            Self::CoreVert => "Forward vert: mesh transform + normal + UV + vertex color",
            Self::CoreFrag => "Forward frag: vertex color + half-lambert simple",
            Self::TextureVert => "Textured vert: pasa UV a fragment",
            Self::TextureFrag => "Textured frag: sampler único diffuse",
            Self::BlenderLiveVert => "Blender Live vert: world pos + view dir para PBR",
            Self::BlenderLiveFrag => {
                "Blender Live frag: PBR Cook-Torrance + IBL HD + CSM PCF + rim"
            }
            Self::ShadowVert => "Shadow depth-only vert: transform a clip space del cascade",
            Self::ShadowFrag => "Shadow depth-only frag: vacío (sólo depth)",
            Self::PostProcessVert => "Fullscreen triangle vert para passes de post",
            Self::PostProcessFrag => {
                "Compositor fullscreen: tone map, bloom, SSGI, SSR, fog, LUT, grain"
            }
            Self::GBufferVert => "G-Buffer vert: world pos + normal + UV + vertex color",
            Self::GBufferFrag => {
                "G-Buffer frag: escribe 4 attachments (albedo/AO, normal/material, emissive, motion/depth/flags)"
            }
            Self::BloomDownsample => "Bloom 13-tap Karis downsample con threshold mip 0",
            Self::BloomUpsample => "Bloom 9-tap upsample con tent filter",
            Self::DepthResolve => "Depth MSAA → single-sample R32F (mín sample)",
            Self::TaaResolve => "TAA resolve: reproyección + neighborhood clip + depth reject",
            Self::Gtao => "GTAO compute: 8 dirs × 4 steps con rotación temporal",
            Self::IblEquirectToCube => "IBL: equirectangular HDR 2D → cubemap radiance",
            Self::IblIrradiance => "IBL: irradiance cubemap difuso (Lambert)",
            Self::IblPrefilter => "IBL: prefilter especular GGX (5 mips, Karis 2014)",
            Self::IblBrdfLut => "IBL: BRDF integration LUT 2D (scale + bias Fresnel)",
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

    /// Ruta al `.glsl` / `.frag` / `.vert` / `.comp` fuente (útil para
    /// inspector, hot-reload o tooling). Devuelve `None` para los shaders
    /// legacy cuyo fuente vive en la raíz de `shaders/` sin sub-carpeta.
    pub fn source_path(self) -> Option<&'static str> {
        match self {
            Self::CoreVert => Some("shaders/core/shader.vert"),
            Self::CoreFrag => Some("shaders/core/shader.frag"),
            Self::TextureVert => Some("shaders/core/texture.vert"),
            Self::TextureFrag => Some("shaders/core/texture.frag"),
            Self::BlenderLiveVert => Some("shaders/live/blender_live.vert"),
            Self::BlenderLiveFrag => Some("shaders/live/blender_live.frag"),
            Self::ShadowVert => Some("shaders/live/shadow.vert"),
            Self::ShadowFrag => Some("shaders/live/shadow.frag"),
            Self::PostProcessVert => Some("shaders/post/post_process.vert"),
            Self::PostProcessFrag => Some("shaders/post/post_process.frag"),
            Self::GBufferVert => Some("shaders/deferred/gbuffer.vert"),
            Self::GBufferFrag => Some("shaders/deferred/gbuffer.frag"),
            Self::BloomDownsample => Some("shaders/post/bloom_downsample.comp"),
            Self::BloomUpsample => Some("shaders/post/bloom_upsample.comp"),
            Self::DepthResolve => Some("shaders/post/depth_resolve.comp"),
            Self::TaaResolve => Some("shaders/post/taa_resolve.comp"),
            Self::Gtao => Some("shaders/post/gtao.comp"),
            Self::IblEquirectToCube => Some("shaders/ibl/equirect_to_cube.comp"),
            Self::IblIrradiance => Some("shaders/ibl/irradiance.comp"),
            Self::IblPrefilter => Some("shaders/ibl/prefilter.comp"),
            Self::IblBrdfLut => Some("shaders/ibl/brdf_lut.comp"),
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

    /// Tamaño en bytes del SPIR-V embebido (para presupuesto de binario).
    pub fn byte_len(self) -> usize {
        self.bytes().len()
    }

    /// Lookup inverso por `name()` ("core.forward.vert", ...). `None` si no existe.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|a| a.name() == name)
    }

    /// Lookup inverso por `path()` ("shaders/post/gtao.spv", ...). `None` si no existe.
    pub fn from_path(path: &str) -> Option<Self> {
        let normalised = path.replace('\\', "/");
        Self::ALL
            .iter()
            .copied()
            .find(|a| a.path() == normalised.as_str())
    }

    /// Iterador filtrado por stage.
    pub fn iter_stage(stage: BaseShaderStage) -> impl Iterator<Item = Self> {
        Self::ALL.iter().copied().filter(move |a| a.stage() == stage)
    }

    /// Iterador filtrado por family.
    pub fn iter_family(family: BaseShaderFamily) -> impl Iterator<Item = Self> {
        Self::ALL
            .iter()
            .copied()
            .filter(move |a| a.family() == family)
    }
}

// =============================================================================
// BaseShaderPair — pareja (vertex, fragment) lista para crear pipelines
// =============================================================================

#[derive(Clone, Debug)]
pub struct BaseShaderPair {
    pub vertex: Vec<u32>,
    pub fragment: Vec<u32>,
}

impl BaseShaderPair {
    pub fn new(vertex: BaseShaderAsset, fragment: BaseShaderAsset) -> Self {
        debug_assert_eq!(vertex.stage(), BaseShaderStage::Vertex, "esperado vertex");
        debug_assert_eq!(
            fragment.stage(),
            BaseShaderStage::Fragment,
            "esperado fragment"
        );
        Self {
            vertex: vertex.words(),
            fragment: fragment.words(),
        }
    }
}

// =============================================================================
// Kits agrupados — sub-cookbooks por subsistema
// =============================================================================

/// Pipeline de geometría deferred — escribe G-Buffer de 4 attachments.
#[derive(Clone, Debug)]
pub struct DeferredKit {
    pub gbuffer: BaseShaderPair,
}

impl Default for DeferredKit {
    fn default() -> Self {
        Self {
            gbuffer: BaseShaderPair::new(BaseShaderAsset::GBufferVert, BaseShaderAsset::GBufferFrag),
        }
    }
}

/// Compute kit de post-process — bloom, depth resolve MSAA, TAA, GTAO.
#[derive(Clone, Debug)]
pub struct PostComputeKit {
    pub bloom_downsample: Vec<u32>,
    pub bloom_upsample: Vec<u32>,
    pub depth_resolve: Vec<u32>,
    pub taa_resolve: Vec<u32>,
    pub gtao: Vec<u32>,
}

impl Default for PostComputeKit {
    fn default() -> Self {
        Self {
            bloom_downsample: BaseShaderAsset::BloomDownsample.words(),
            bloom_upsample: BaseShaderAsset::BloomUpsample.words(),
            depth_resolve: BaseShaderAsset::DepthResolve.words(),
            taa_resolve: BaseShaderAsset::TaaResolve.words(),
            gtao: BaseShaderAsset::Gtao.words(),
        }
    }
}

/// Compute kit de baking IBL — los 4 stages para construir un environment HD.
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

// =============================================================================
// BaseMaterialDefaults — valores PBR base por preset
// =============================================================================

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

// =============================================================================
// BaseShaderCookbook — agrupador top-level con todos los kits + presets
// =============================================================================

#[derive(Clone, Debug)]
pub struct BaseShaderCookbook {
    // Pipelines clásicos (vertex + fragment).
    pub forward: BaseShaderPair,
    pub textured: BaseShaderPair,
    pub blender_live_pbr: BaseShaderPair,
    pub gbuffer: BaseShaderPair,
    pub shadow_depth: BaseShaderPair,
    pub post_process: BaseShaderPair,

    // SPIR-V de compute sueltos (mantenidos por backward compat — los kits
    // agrupados de abajo son la API recomendada para nuevos consumidores).
    pub bloom_downsample: Vec<u32>,
    pub bloom_upsample: Vec<u32>,
    pub depth_resolve: Vec<u32>,
    pub taa_resolve: Vec<u32>,
    pub gtao: Vec<u32>,
    pub ibl_equirect_to_cube: Vec<u32>,
    pub ibl_irradiance: Vec<u32>,
    pub ibl_prefilter: Vec<u32>,
    pub ibl_brdf_lut: Vec<u32>,

    // Kits agrupados (API nueva, recomendada).
    pub deferred: DeferredKit,
    pub post_compute: PostComputeKit,
    pub ibl_bake: IblBakeKit,

    // Defaults mutables aplicables a runtime.
    pub material: BaseMaterialDefaults,
    pub post_settings: PostProcessSettings,
    pub post_enabled: bool,
    pub aa_settings: AASettings,
    pub shadow_config: ShadowConfig,
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
            deferred: DeferredKit::default(),
            post_compute: PostComputeKit::default(),
            ibl_bake: IblBakeKit::default(),
            material: BaseMaterialDefaults::default(),
            post_settings: PostProcessSettings::default(),
            post_enabled: true,
            aa_settings: AASettings::default(),
            shadow_config: ShadowConfig::default(),
        }
    }
}

impl BaseShaderCookbook {
    // ── Presets de calidad ───────────────────────────────────────────────────

    /// Preset Blender Live — material preview + estudio 3-point + IBL HD.
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

    /// Preset cinemático AAA — todos los efectos screen-space encendidos,
    /// CSM 4 cascadas 2K, AA Ultra (SMAA+TAA), exposure 1.08.
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
        for effect in [
            PostProcessEffect::Bloom,
            PostProcessEffect::SSGI,
            PostProcessEffect::SSR,
            PostProcessEffect::VolumetricFog,
            PostProcessEffect::LutColorGrading,
            PostProcessEffect::ToneMapping,
            PostProcessEffect::AnamorphicFlares,
        ] {
            cookbook.post_settings.enable_effect(effect);
        }
        cookbook.material.roughness = 0.42;
        cookbook.aa_settings = AASettings::cinematic();
        cookbook.shadow_config = ShadowConfig::high_quality();
        cookbook
    }

    /// Preset showcase específico para Xenofall (tono apagado).
    pub fn xenofall_showcase() -> Self {
        let mut cookbook = Self::cinematic_aaa();
        cookbook.material.color = glam::Vec4::new(0.34, 0.36, 0.35, 1.0);
        cookbook.material.metallic = 0.05;
        cookbook.material.roughness = 0.55;
        cookbook
    }

    /// Preset performance — apaga SSGI/SSR/Fog para target 144 fps.
    /// AA Low (FXAA), CSM 2 cascadas 1K, bloom suave.
    pub fn performance() -> Self {
        let mut cookbook = Self::default();
        for effect in [
            PostProcessEffect::SSGI,
            PostProcessEffect::SSR,
            PostProcessEffect::VolumetricFog,
            PostProcessEffect::PathTracedLighting,
            PostProcessEffect::AnamorphicFlares,
            PostProcessEffect::ChromaticAberration,
            PostProcessEffect::FilmGrain,
        ] {
            cookbook.post_settings.disable_effect(effect);
        }
        cookbook.post_settings.bloom_intensity = 0.25;
        cookbook.post_settings.bloom_threshold = 1.1;
        cookbook.aa_settings.quality = AAQualityPreset::Low;
        cookbook.shadow_config = ShadowConfig::low_quality();
        cookbook
    }

    /// Preset mobile/lowend — apaga TODO post excepto tonemap + vignette suave.
    /// AA Off, sombras 512 sin PCF.
    pub fn mobile_low() -> Self {
        let mut cookbook = Self::performance();
        for effect in [
            PostProcessEffect::Bloom,
            PostProcessEffect::LutColorGrading,
            PostProcessEffect::Vignette,
            PostProcessEffect::FXAA,
            PostProcessEffect::SMAA,
            PostProcessEffect::TAA,
        ] {
            cookbook.post_settings.disable_effect(effect);
        }
        cookbook.post_settings.enable_effect(PostProcessEffect::ToneMapping);
        cookbook.post_settings.exposure = 1.0;
        cookbook.aa_settings.quality = AAQualityPreset::Off;
        cookbook.shadow_config = ShadowConfig {
            resolution: 512,
            cascade_count: 1,
            cascade_splits: vec![1.0],
            pcf_samples: 1,
            soft_shadows: false,
            ..ShadowConfig::default()
        };
        cookbook
    }

    /// Preset AAA Ultra — extiende cinematic_aaa con shadows 4K, AA cinematic
    /// y exposure HDR. Pensado para captures/screenshots/cinemáticas.
    pub fn aaa_ultra() -> Self {
        let mut cookbook = Self::cinematic_aaa();
        cookbook.post_settings.exposure = 1.15;
        cookbook.post_settings.bloom_intensity = 0.95;
        cookbook.post_settings.ssgi_intensity = 0.42;
        cookbook.post_settings.ssr_strength = 0.6;
        cookbook.post_settings.fog_density = 0.32;
        cookbook.post_settings.highlight_recovery = 0.9;
        cookbook.aa_settings = AASettings::cinematic();
        cookbook.shadow_config = ShadowConfig::high_quality();
        cookbook
    }

    // ── Aplicación a subsistemas runtime ─────────────────────────────────────

    pub fn apply_to_post_process(&self, post_process: &mut PostProcessPipeline) {
        post_process.enabled = self.post_enabled;
        post_process.settings = self.post_settings;
    }

    /// Sustituye la configuración de sombras en sitio.
    pub fn apply_to_shadow_config(&self, target: &mut ShadowConfig) {
        *target = self.shadow_config.clone();
    }

    /// Sustituye la configuración de AA en sitio.
    pub fn apply_to_aa_settings(&self, target: &mut AASettings) {
        *target = self.aa_settings;
    }

    // ── Manifests / introspección ────────────────────────────────────────────

    /// Manifest (nombre, stage, path) de todos los SPIR-V — útil para
    /// imprimir al arrancar o exponer en una pestaña del editor.
    pub fn shader_manifest(&self) -> Vec<(&'static str, BaseShaderStage, &'static str)> {
        BaseShaderAsset::ALL
            .iter()
            .map(|asset| (asset.name(), asset.stage(), asset.path()))
            .collect()
    }

    /// Manifest extendido con family y descripción.
    pub fn shader_manifest_full(
        &self,
    ) -> Vec<(
        &'static str,
        BaseShaderStage,
        BaseShaderFamily,
        &'static str,
        &'static str,
    )> {
        BaseShaderAsset::ALL
            .iter()
            .map(|asset| {
                (
                    asset.name(),
                    asset.stage(),
                    asset.family(),
                    asset.path(),
                    asset.description(),
                )
            })
            .collect()
    }

    /// Total de bytes SPIR-V embebidos en el binario.
    pub fn total_embedded_bytes(&self) -> usize {
        BaseShaderAsset::ALL.iter().map(|a| a.byte_len()).sum()
    }

    /// Conteo de shaders por stage — (vertex, fragment, compute).
    pub fn shader_count_per_stage(&self) -> (usize, usize, usize) {
        let mut v = 0;
        let mut f = 0;
        let mut c = 0;
        for asset in BaseShaderAsset::ALL {
            match asset.stage() {
                BaseShaderStage::Vertex => v += 1,
                BaseShaderStage::Fragment => f += 1,
                BaseShaderStage::Compute => c += 1,
            }
        }
        (v, f, c)
    }

    /// Resumen multilinea legible — pensado para `info!()` al arrancar
    /// el editor o para dumpear desde la consola pause.
    pub fn summary(&self) -> String {
        let (v, f, c) = self.shader_count_per_stage();
        let total_kb = self.total_embedded_bytes() as f32 / 1024.0;
        let mut out = String::new();
        out.push_str(&format!(
            "BaseShaderCookbook — {} shaders ({} vert, {} frag, {} compute), {:.1} KiB SPIR-V embebido\n",
            BaseShaderAsset::ALL.len(),
            v,
            f,
            c,
            total_kb
        ));
        out.push_str(&format!(
            "  material: color={:?} metallic={:.2} roughness={:.2}\n",
            self.material.color.to_array(),
            self.material.metallic,
            self.material.roughness
        ));
        out.push_str(&format!(
            "  post   : enabled={} exposure={:.2} bloom={:.2}\n",
            self.post_enabled, self.post_settings.exposure, self.post_settings.bloom_intensity
        ));
        out.push_str(&format!(
            "  AA     : {:?} edge_w={:.2} subpix={}\n",
            self.aa_settings.quality, self.aa_settings.edge_width, self.aa_settings.subpixel_aa
        ));
        out.push_str(&format!(
            "  shadows: res={} cascades={} pcf_samples={} soft={}\n",
            self.shadow_config.resolution,
            self.shadow_config.cascade_count,
            self.shadow_config.pcf_samples,
            self.shadow_config.soft_shadows
        ));
        out
    }
}

// =============================================================================
// Helpers
// =============================================================================

pub fn read_spv(bytes: &[u8]) -> Vec<u32> {
    ash::util::read_spv(&mut std::io::Cursor::new(bytes))
        .expect("Embedded SPIR-V is invalid; rebuild shaders with `cargo check`")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_assets_have_unique_names_and_paths() {
        let mut names: Vec<&'static str> = BaseShaderAsset::ALL.iter().map(|a| a.name()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), BaseShaderAsset::ALL.len());

        let mut paths: Vec<&'static str> = BaseShaderAsset::ALL.iter().map(|a| a.path()).collect();
        paths.sort();
        paths.dedup();
        assert_eq!(paths.len(), BaseShaderAsset::ALL.len());
    }

    #[test]
    fn from_name_roundtrips_for_all_assets() {
        for asset in BaseShaderAsset::ALL {
            assert_eq!(BaseShaderAsset::from_name(asset.name()), Some(*asset));
            assert_eq!(BaseShaderAsset::from_path(asset.path()), Some(*asset));
        }
    }

    #[test]
    fn all_spirv_blobs_are_non_empty_and_word_aligned() {
        for asset in BaseShaderAsset::ALL {
            let bytes = asset.bytes();
            assert!(!bytes.is_empty(), "{} está vacío", asset.name());
            assert!(
                bytes.len() % 4 == 0,
                "{} no está alineado a 4 bytes",
                asset.name()
            );
        }
    }

    #[test]
    fn cookbook_summary_runs() {
        let cookbook = BaseShaderCookbook::cinematic_aaa();
        let summary = cookbook.summary();
        assert!(summary.contains("shaders"));
        assert!(summary.contains("post"));
    }

    #[test]
    fn presets_compile_and_set_expected_state() {
        let perf = BaseShaderCookbook::performance();
        assert_eq!(perf.aa_settings.quality, AAQualityPreset::Low);
        assert!(!perf.post_settings.is_effect_enabled(PostProcessEffect::SSGI));

        let mobile = BaseShaderCookbook::mobile_low();
        assert_eq!(mobile.aa_settings.quality, AAQualityPreset::Off);
        assert_eq!(mobile.shadow_config.resolution, 512);

        let ultra = BaseShaderCookbook::aaa_ultra();
        assert!(ultra.post_settings.is_effect_enabled(PostProcessEffect::SSR));
        assert_eq!(ultra.shadow_config.resolution, 4096);
    }
}
