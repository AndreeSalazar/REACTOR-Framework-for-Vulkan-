use crate::base_shader::asset::BaseShaderAsset;
use crate::base_shader::family::BaseShaderFamily;
use crate::base_shader::pair::{BaseMaterialDefaults, BaseShaderPair, DeferredKit, IblBakeKit, PostComputeKit};
use crate::base_shader::stage::BaseShaderStage;
use crate::compute::pipeline::ComputePipeline;
use crate::core::context::VulkanContext;
use crate::core::error::ReactorResult;
use crate::graphics::pipeline::{Pipeline, PipelineConfig};
use crate::graphics::post_process::{
    AAQualityPreset, AASettings, PostProcessEffect, PostProcessPipeline, PostProcessSettings,
};
use crate::graphics::shadows::ShadowConfig;
use ash::vk;

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
    pub light_cull: Vec<u32>,
    pub ibl_equirect_to_cube: Vec<u32>,
    pub ibl_irradiance: Vec<u32>,
    pub ibl_prefilter: Vec<u32>,
    pub ibl_brdf_lut: Vec<u32>,
    pub deferred: DeferredKit,
    pub post_compute: PostComputeKit,
    pub ibl_bake: IblBakeKit,
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
            textured: BaseShaderPair::new(BaseShaderAsset::TextureVert, BaseShaderAsset::TextureFrag),
            blender_live_pbr: BaseShaderPair::new(BaseShaderAsset::BlenderLiveVert, BaseShaderAsset::BlenderLiveFrag),
            gbuffer: BaseShaderPair::new(BaseShaderAsset::GBufferVert, BaseShaderAsset::GBufferFrag),
            shadow_depth: BaseShaderPair::new(BaseShaderAsset::ShadowVert, BaseShaderAsset::ShadowFrag),
            post_process: BaseShaderPair::new(BaseShaderAsset::PostProcessVert, BaseShaderAsset::PostProcessFrag),
            bloom_downsample: BaseShaderAsset::BloomDownsample.words(),
            bloom_upsample: BaseShaderAsset::BloomUpsample.words(),
            depth_resolve: BaseShaderAsset::DepthResolve.words(),
            taa_resolve: BaseShaderAsset::TaaResolve.words(),
            gtao: BaseShaderAsset::Gtao.words(),
            light_cull: BaseShaderAsset::LightCull.words(),
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
    pub fn blender_live() -> Self {
        let mut c = Self::cinematic_aaa();
        c.post_settings.exposure = 1.04; c.post_settings.bloom_threshold = 0.7; c.post_settings.bloom_intensity = 0.5;
        c.post_settings.ssgi_intensity = 0.28; c.post_settings.ssgi_radius = 10.0; c.post_settings.ssr_strength = 0.38;
        c.post_settings.fog_density = 0.12; c.post_settings.fog_scatter = 0.42; c.post_settings.flare_intensity = 0.35;
        c.post_settings.highlight_recovery = 0.74; c.material.metallic = 0.0; c.material.roughness = 0.48;
        c
    }

    pub fn cinematic_aaa() -> Self {
        let mut c = Self::default();
        c.post_settings = PostProcessSettings::cinematic();
        c.post_settings.exposure = 1.08; c.post_settings.bloom_threshold = 0.62; c.post_settings.bloom_intensity = 0.78;
        c.post_settings.ssgi_intensity = 0.34; c.post_settings.ssgi_radius = 14.0; c.post_settings.ssr_strength = 0.46;
        c.post_settings.fog_density = 0.24; c.post_settings.fog_scatter = 0.58; c.post_settings.flare_intensity = 0.52;
        c.post_settings.highlight_recovery = 0.82;
        for e in [PostProcessEffect::Bloom, PostProcessEffect::SSGI, PostProcessEffect::SSR,
            PostProcessEffect::VolumetricFog, PostProcessEffect::LutColorGrading, PostProcessEffect::ToneMapping,
            PostProcessEffect::AnamorphicFlares, PostProcessEffect::ContactShadows, PostProcessEffect::SSSDiffusion,
            PostProcessEffect::DepthOfField, PostProcessEffect::AutoExposure, PostProcessEffect::TAA,
            PostProcessEffect::MotionBlur, PostProcessEffect::GTAO] { c.post_settings.enable_effect(e); }
        c.material.roughness = 0.42; c.aa_settings = AASettings::cinematic(); c.shadow_config = ShadowConfig::high_quality();
        c
    }

    pub fn xenofall_showcase() -> Self {
        let mut c = Self::cinematic_aaa();
        c.material.color = glam::Vec4::new(0.34, 0.36, 0.35, 1.0); c.material.metallic = 0.05; c.material.roughness = 0.55;
        c
    }

    pub fn performance() -> Self {
        let mut c = Self::default();
        for e in [PostProcessEffect::SSGI, PostProcessEffect::SSR, PostProcessEffect::VolumetricFog,
            PostProcessEffect::PathTracedLighting, PostProcessEffect::AnamorphicFlares,
            PostProcessEffect::ChromaticAberration, PostProcessEffect::FilmGrain] {
            c.post_settings.disable_effect(e);
        }
        c.post_settings.bloom_intensity = 0.25; c.post_settings.bloom_threshold = 1.1;
        c.aa_settings.quality = AAQualityPreset::Low; c.shadow_config = ShadowConfig::low_quality();
        c
    }

    pub fn mobile_low() -> Self {
        let mut c = Self::performance();
        for e in [PostProcessEffect::Bloom, PostProcessEffect::LutColorGrading, PostProcessEffect::Vignette,
            PostProcessEffect::FXAA, PostProcessEffect::SMAA, PostProcessEffect::TAA] {
            c.post_settings.disable_effect(e);
        }
        c.post_settings.enable_effect(PostProcessEffect::ToneMapping);
        c.post_settings.exposure = 1.0; c.aa_settings.quality = AAQualityPreset::Off;
        c.shadow_config = ShadowConfig { resolution: 512, cascade_count: 1, cascade_splits: vec![1.0], pcf_samples: 1, soft_shadows: false, ..ShadowConfig::default() };
        c
    }

    pub fn aaa_ultra() -> Self {
        let mut c = Self::cinematic_aaa();
        c.post_settings.exposure = 1.15; c.post_settings.bloom_intensity = 0.95;
        c.post_settings.ssgi_intensity = 0.42; c.post_settings.ssr_strength = 0.6;
        c.post_settings.fog_density = 0.32; c.post_settings.highlight_recovery = 0.9;
        c.aa_settings = AASettings::cinematic(); c.shadow_config = ShadowConfig::high_quality();
        c
    }

    pub fn apply_to_post_process(&self, post_process: &mut PostProcessPipeline) {
        post_process.enabled = self.post_enabled; post_process.settings = self.post_settings;
    }
    pub fn apply_to_shadow_config(&self, target: &mut ShadowConfig) { *target = self.shadow_config.clone(); }
    pub fn apply_to_aa_settings(&self, target: &mut AASettings) { *target = self.aa_settings; }

    pub fn shader_manifest(&self) -> Vec<(&'static str, BaseShaderStage, &'static str)> {
        BaseShaderAsset::ALL.iter().map(|a| (a.name(), a.stage(), a.path())).collect()
    }
    pub fn shader_manifest_full(&self) -> Vec<(&'static str, BaseShaderStage, BaseShaderFamily, &'static str, &'static str)> {
        BaseShaderAsset::ALL.iter().map(|a| (a.name(), a.stage(), a.family(), a.path(), a.description())).collect()
    }
    pub fn total_embedded_bytes(&self) -> usize { BaseShaderAsset::ALL.iter().map(|a| a.byte_len()).sum() }
    pub fn shader_count_per_stage(&self) -> (usize, usize, usize) {
        let (mut v, mut f, mut c) = (0, 0, 0);
        for a in BaseShaderAsset::ALL { match a.stage() { BaseShaderStage::Vertex => v += 1, BaseShaderStage::Fragment => f += 1, BaseShaderStage::Compute => c += 1 } }
        (v, f, c)
    }
    pub fn summary(&self) -> String {
        let (v, fc, c) = self.shader_count_per_stage();
        let total_kb = self.total_embedded_bytes() as f32 / 1024.0;
        format!(
            "BaseShaderCookbook — {} shaders ({} vert, {} frag, {} compute), {:.1} KiB SPIR-V embebido\n\
             material: color={:?} metallic={:.2} roughness={:.2}\n\
             post   : enabled={} exposure={:.2} bloom={:.2}\n\
             AA     : {:?} edge_w={:.2} subpix={}\n\
             shadows: res={} cascades={} pcf_samples={} soft={}",
            BaseShaderAsset::ALL.len(), v, fc, c, total_kb,
            self.material.color.to_array(), self.material.metallic, self.material.roughness,
            self.post_enabled, self.post_settings.exposure, self.post_settings.bloom_intensity,
            self.aa_settings.quality, self.aa_settings.edge_width, self.aa_settings.subpixel_aa,
            self.shadow_config.resolution, self.shadow_config.cascade_count,
            self.shadow_config.pcf_samples, self.shadow_config.soft_shadows,
        )
    }

    pub fn create_compute_pipeline(&self, ctx: &VulkanContext, asset: BaseShaderAsset)
        -> ReactorResult<(ComputePipeline, Vec<vk::DescriptorSetLayout>)> {
        let mut compiler = crate::graphics::shader_compiler::ShaderCompiler::new();
        let compiled = compiler.load_spirv_words(&asset.words(), crate::graphics::shader_compiler::ShaderStage::Compute, "main")?;
        let layouts = compiled.reflection.create_descriptor_set_layouts(&ctx.device)?;
        let push = compiled.reflection.push_constants.first().map(|pc| pc.size);
        Ok((ComputePipeline::new(ctx, &asset.words(), &layouts, push)?, layouts))
    }

    pub fn create_graphics_pipeline(&self, ctx: &VulkanContext, render_pass: Option<vk::RenderPass>,
        vert_asset: BaseShaderAsset, frag_asset: BaseShaderAsset, width: u32, height: u32,
        config: &PipelineConfig, color_formats: &[vk::Format], depth_format: Option<vk::Format>,
    ) -> ReactorResult<(Pipeline, Vec<vk::DescriptorSetLayout>)> {
        let mut compiler = crate::graphics::shader_compiler::ShaderCompiler::new();
        let vert = compiler.load_spirv_words(&vert_asset.words(), crate::graphics::shader_compiler::ShaderStage::Vertex, "main")?;
        let frag = compiler.load_spirv_words(&frag_asset.words(), crate::graphics::shader_compiler::ShaderStage::Fragment, "main")?;
        let mut mr = vert.reflection.clone();
        mr.merge_stages(&frag.reflection);
        let layouts = mr.create_descriptor_set_layouts(&ctx.device)?;
        let pipeline = Pipeline::with_config_and_cache_multi_color(&ctx.device, render_pass,
            &vert_asset.words(), &frag_asset.words(), width, height, config, &layouts,
            color_formats, depth_format, vk::PipelineCache::null())?;
        Ok((pipeline, layouts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::post_process::AAQualityPreset;

    #[test]
    fn all_assets_have_unique_names_and_paths() {
        let mut names: Vec<_> = BaseShaderAsset::ALL.iter().map(|a| a.name()).collect();
        names.sort(); names.dedup();
        assert_eq!(names.len(), BaseShaderAsset::ALL.len());
        let mut paths: Vec<_> = BaseShaderAsset::ALL.iter().map(|a| a.path()).collect();
        paths.sort(); paths.dedup();
        assert_eq!(paths.len(), BaseShaderAsset::ALL.len());
    }

    #[test]
    fn from_name_roundtrips_for_all_assets() {
        for a in BaseShaderAsset::ALL { assert_eq!(BaseShaderAsset::from_name(a.name()), Some(*a)); assert_eq!(BaseShaderAsset::from_path(a.path()), Some(*a)); }
    }

    #[test]
    fn all_spirv_blobs_are_non_empty_and_word_aligned() {
        for a in BaseShaderAsset::ALL { let b = a.bytes(); assert!(!b.is_empty(), "{} vacío", a.name()); assert!(b.len() % 4 == 0, "{} no alineado", a.name()); }
    }

    #[test]
    fn cookbook_summary_runs() { let s = BaseShaderCookbook::cinematic_aaa().summary(); assert!(s.contains("shaders")); assert!(s.contains("post")); }

    #[test]
    fn presets_compile_and_set_expected_state() {
        let p = BaseShaderCookbook::performance(); assert_eq!(p.aa_settings.quality, AAQualityPreset::Low); assert!(!p.post_settings.is_effect_enabled(PostProcessEffect::SSGI));
        let m = BaseShaderCookbook::mobile_low(); assert_eq!(m.aa_settings.quality, AAQualityPreset::Off); assert_eq!(m.shadow_config.resolution, 512);
        let u = BaseShaderCookbook::aaa_ultra(); assert!(u.post_settings.is_effect_enabled(PostProcessEffect::SSR)); assert_eq!(u.shadow_config.resolution, 4096);
    }
}
