use crate::core::VulkanContext;
use crate::graphics::{Buffer, Image};
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use std::sync::{Arc, Mutex};

use super::types::{PostProcessPreset, PostProcessSettings};

/// Post-processing pipeline manager — owns all GPU resources for post-process effects.
///
/// Each effect (bloom, TAA, fog, lens flare, GTAO, light cull, depth resolve,
/// auto-exposure) has its own file with `impl PostProcessPipeline { ... }` blocks.
/// This file contains the struct definition, constructors, init/recreate, and Drop.
pub struct PostProcessPipeline {
    pub settings: PostProcessSettings,
    pub enabled: bool,

    pub pipeline: Option<vk::Pipeline>,
    pub layout: Option<vk::PipelineLayout>,
    pub descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub descriptor_pool: Option<vk::DescriptorPool>,
    pub descriptor_sets: Vec<vk::DescriptorSet>,

    pub offscreen_images: Vec<Image>,
    pub sampler: Option<vk::Sampler>,

    // ── Depth Resolve ──
    pub depth_resolve_pipeline: Option<crate::compute::ComputePipeline>,
    pub depth_resolve_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub depth_resolve_descriptor_pool: Option<vk::DescriptorPool>,
    pub depth_resolve_sets: Vec<vk::DescriptorSet>,
    pub depth_resolved_images: Vec<Image>,
    pub depth_resolved_initialized: Vec<bool>,

    // ── Bloom Mip-Chain ──
    pub bloom_downsample_pipeline: Option<crate::compute::ComputePipeline>,
    pub bloom_upsample_pipeline: Option<crate::compute::ComputePipeline>,
    pub bloom_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub bloom_descriptor_pool: Option<vk::DescriptorPool>,
    pub bloom_images: Vec<Image>,
    pub bloom_mip_views_sampled: Vec<Vec<vk::ImageView>>,
    pub bloom_mip_views_storage: Vec<Vec<vk::ImageView>>,
    pub bloom_downsample_sets: Vec<Vec<vk::DescriptorSet>>,
    pub bloom_upsample_sets: Vec<Vec<vk::DescriptorSet>>,

    // ── Auto-Exposure ──
    pub auto_exposure_pipeline: Option<crate::compute::ComputePipeline>,
    pub exposure_buffers: Vec<Buffer>,
    pub last_time: f32,
    pub delta_time: f32,

    // ── TAA ──
    pub taa_pipeline: Option<crate::compute::ComputePipeline>,
    pub taa_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub taa_descriptor_pool: Option<vk::DescriptorPool>,
    pub taa_descriptor_sets: Vec<vk::DescriptorSet>,

    // ── Volumetric Fog ──
    pub fog_pipeline: Option<crate::compute::ComputePipeline>,
    pub fog_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub fog_descriptor_pool: Option<vk::DescriptorPool>,
    pub fog_descriptor_sets: Vec<vk::DescriptorSet>,
    pub fog_output_images: Vec<Image>,

    // ── Lens Flare ──
    pub lens_flare_pipeline: Option<crate::compute::ComputePipeline>,
    pub lens_flare_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub lens_flare_descriptor_pool: Option<vk::DescriptorPool>,
    pub lens_flare_descriptor_sets: Vec<vk::DescriptorSet>,
    pub lens_flare_output_images: Vec<Image>,

    // ── GTAO ──
    pub gtao_pipeline: Option<crate::compute::ComputePipeline>,
    pub gtao_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub gtao_descriptor_pool: Option<vk::DescriptorPool>,
    pub gtao_descriptor_sets: Vec<vk::DescriptorSet>,
    pub gtao_ao_images: Vec<Image>,
    pub gtao_initialized: Vec<bool>,

    // ── Light Culling ──
    pub light_cull_pipeline: Option<crate::compute::ComputePipeline>,
    pub light_cull_descriptor_layout: Option<vk::DescriptorSetLayout>,
    pub light_cull_descriptor_pool: Option<vk::DescriptorPool>,
    pub light_cull_descriptor_sets: Vec<vk::DescriptorSet>,
    pub light_cull_tile_buffer: Option<Buffer>,
    pub light_cull_index_buffer: Option<Buffer>,
    pub light_cull_atomic_buffer: Option<Buffer>,
    pub light_cull_light_buffer: Option<Buffer>,

    pub lut_texture: Option<crate::resources::texture::Texture>,
    pub device: Option<crate::core::arc_handle::ArcDevice>,
}

impl PostProcessPipeline {
    pub fn new() -> Self {
        Self {
            settings: PostProcessSettings::default(),
            enabled: true,
            pipeline: None, layout: None, descriptor_layout: None, descriptor_pool: None,
            descriptor_sets: Vec::new(), offscreen_images: Vec::new(), sampler: None,
            depth_resolve_pipeline: None, depth_resolve_descriptor_layout: None,
            depth_resolve_descriptor_pool: None, depth_resolve_sets: Vec::new(),
            depth_resolved_images: Vec::new(), depth_resolved_initialized: Vec::new(),
            bloom_downsample_pipeline: None, bloom_upsample_pipeline: None,
            bloom_descriptor_layout: None, bloom_descriptor_pool: None,
            bloom_images: Vec::new(), bloom_mip_views_sampled: Vec::new(),
            bloom_mip_views_storage: Vec::new(), bloom_downsample_sets: Vec::new(),
            bloom_upsample_sets: Vec::new(),
            auto_exposure_pipeline: None, exposure_buffers: Vec::new(),
            last_time: 0.0, delta_time: 0.0166,
            taa_pipeline: None, taa_descriptor_layout: None, taa_descriptor_pool: None,
            taa_descriptor_sets: Vec::new(),
            fog_pipeline: None, fog_descriptor_layout: None, fog_descriptor_pool: None,
            fog_descriptor_sets: Vec::new(), fog_output_images: Vec::new(),
            lens_flare_pipeline: None, lens_flare_descriptor_layout: None,
            lens_flare_descriptor_pool: None, lens_flare_descriptor_sets: Vec::new(),
            lens_flare_output_images: Vec::new(),
            gtao_pipeline: None, gtao_descriptor_layout: None, gtao_descriptor_pool: None,
            gtao_descriptor_sets: Vec::new(), gtao_ao_images: Vec::new(),
            gtao_initialized: Vec::new(),
            light_cull_pipeline: None, light_cull_descriptor_layout: None,
            light_cull_descriptor_pool: None, light_cull_descriptor_sets: Vec::new(),
            light_cull_tile_buffer: None, light_cull_index_buffer: None,
            light_cull_atomic_buffer: None, light_cull_light_buffer: None,
            lut_texture: None, device: None,
        }
    }

    pub fn with_preset(preset: PostProcessPreset) -> Self {
        Self {
            settings: match preset {
                PostProcessPreset::None => PostProcessSettings::default(),
                PostProcessPreset::Cinematic => PostProcessSettings::cinematic(),
                PostProcessPreset::Vibrant => PostProcessSettings::vibrant(),
                PostProcessPreset::Retro => PostProcessSettings::retro(),
            },
            enabled: true,
            pipeline: None, layout: None, descriptor_layout: None, descriptor_pool: None,
            descriptor_sets: Vec::new(), offscreen_images: Vec::new(), sampler: None,
            depth_resolve_pipeline: None, depth_resolve_descriptor_layout: None,
            depth_resolve_descriptor_pool: None, depth_resolve_sets: Vec::new(),
            depth_resolved_images: Vec::new(), depth_resolved_initialized: Vec::new(),
            bloom_downsample_pipeline: None, bloom_upsample_pipeline: None,
            bloom_descriptor_layout: None, bloom_descriptor_pool: None,
            bloom_images: Vec::new(), bloom_mip_views_sampled: Vec::new(),
            bloom_mip_views_storage: Vec::new(), bloom_downsample_sets: Vec::new(),
            bloom_upsample_sets: Vec::new(),
            auto_exposure_pipeline: None, exposure_buffers: Vec::new(),
            last_time: 0.0, delta_time: 0.0166,
            taa_pipeline: None, taa_descriptor_layout: None, taa_descriptor_pool: None,
            taa_descriptor_sets: Vec::new(),
            fog_pipeline: None, fog_descriptor_layout: None, fog_descriptor_pool: None,
            fog_descriptor_sets: Vec::new(), fog_output_images: Vec::new(),
            lens_flare_pipeline: None, lens_flare_descriptor_layout: None,
            lens_flare_descriptor_pool: None, lens_flare_descriptor_sets: Vec::new(),
            lens_flare_output_images: Vec::new(),
            gtao_pipeline: None, gtao_descriptor_layout: None, gtao_descriptor_pool: None,
            gtao_descriptor_sets: Vec::new(), gtao_ao_images: Vec::new(),
            gtao_initialized: Vec::new(),
            light_cull_pipeline: None, light_cull_descriptor_layout: None,
            light_cull_descriptor_pool: None, light_cull_descriptor_sets: Vec::new(),
            light_cull_tile_buffer: None, light_cull_index_buffer: None,
            light_cull_atomic_buffer: None, light_cull_light_buffer: None,
            lut_texture: None, device: None,
        }
    }

    pub fn update_time(&mut self, time: f32) {
        let raw_dt = time - self.last_time;
        self.delta_time = if raw_dt > 0.0 && raw_dt < 2.0 { raw_dt } else { 0.0166 };
        self.last_time = time;
        self.settings.time = time;
    }
}

impl Drop for PostProcessPipeline {
    fn drop(&mut self) {
        self.exposure_buffers.clear();
        self.auto_exposure_pipeline = None;
        if let Some(device) = self.device.clone() {
            self.destroy_depth_resolve_resources(&device);
            self.destroy_bloom_resources(&device);
            self.destroy_taa_resources(&device);
            self.destroy_fog_resources(&device);
            self.destroy_lens_flare_resources(&device);
            self.destroy_gtao_resources(&device);
            self.destroy_light_cull_resources(&device);
            unsafe {
                if let Some(sampler) = self.sampler.take() { device.destroy_sampler(sampler, None); }
                if let Some(pool) = self.descriptor_pool.take() { device.destroy_descriptor_pool(pool, None); }
                if let Some(layout) = self.descriptor_layout.take() { device.destroy_descriptor_set_layout(layout, None); }
                if let Some(layout) = self.layout.take() { device.destroy_pipeline_layout(layout, None); }
                if let Some(pipe) = self.pipeline.take() { device.destroy_pipeline(pipe, None); }
                for views in &self.bloom_mip_views_sampled { for &view in views { device.destroy_image_view(view, None); } }
                for views in &self.bloom_mip_views_storage { for &view in views { device.destroy_image_view(view, None); } }
                if let Some(pool) = self.bloom_descriptor_pool.take() { device.destroy_descriptor_pool(pool, None); }
                if let Some(layout) = self.bloom_descriptor_layout.take() { device.destroy_descriptor_set_layout(layout, None); }
            }
        }
    }
}

impl Default for PostProcessPipeline {
    fn default() -> Self { Self::new() }
}
