//! `Reactor::init_decals` — Screen-Space MRT Decal projection pipeline
//!
//! Creates the descriptor layout, dual-color-attachment pipeline, and unit cube
//! mesh used to project decals in screen space (deferred-style).

use super::super::Reactor;
use crate::core::error::ReactorResult;
use ash::vk;

impl Reactor {
    /// Inicializa el pipeline de proyección de decals en espacio de pantalla (MRT).
    pub fn init_decals(&mut self) -> ReactorResult<()> {
        let device = self.context.ash_device();

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let decal_descriptor_layout =
            unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let vert_words = crate::base_shader::BaseShaderAsset::ShadowVert.words();
        let frag_words = crate::base_shader::BaseShaderAsset::DecalFrag.words();

        let config = crate::graphics::pipeline::PipelineConfig {
            cull_mode: vk::CullModeFlags::NONE,
            depth_write: false,
            depth_test: true,
            blend_enable: true,
            ..Default::default()
        };

        let color_formats = [
            vk::Format::R8G8B8A8_UNORM,
            vk::Format::R16G16B16A16_SFLOAT,
        ];

        let decal_pipeline = crate::graphics::pipeline::Pipeline::with_config_multi_color(
            &self.context.device,
            None,
            &vert_words,
            &frag_words,
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            &config,
            &[decal_descriptor_layout],
            &color_formats,
            Some(self.depth_format),
        )?;

        self.decal_descriptor_layout = Some(decal_descriptor_layout);
        self.decal_pipeline = Some(decal_pipeline);

        let (vertices, indices) = crate::resources::primitives::Primitives::cube();
        let decal_cube_mesh = self.create_mesh(&vertices, &indices)?;
        self.decal_cube_mesh = Some(decal_cube_mesh);

        log::info!("✅ Screen-Space MRT Decals pipeline initialized successfully");

        Ok(())
    }
}
