use std::ffi::CStr;

use ash::vk;

use crate::core::arc_handle::ArcDevice;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};

use super::types::CompiledShader;

impl CompiledShader {
    pub fn create_shader_module(&self, device: &ArcDevice) -> ReactorResult<vk::ShaderModule> {
        let create_info = vk::ShaderModuleCreateInfo::default().code(&self.spirv);
        unsafe {
            device
                .create_shader_module(&create_info, None)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanShaderCompilation,
                        "create_shader_module failed",
                        e,
                    )
                })
        }
    }

    pub fn stage_create_info<'a>(
        &self,
        module: vk::ShaderModule,
        name: &'a CStr,
    ) -> vk::PipelineShaderStageCreateInfo<'a> {
        vk::PipelineShaderStageCreateInfo::default()
            .stage(self.stage.to_vk())
            .module(module)
            .name(name)
    }

    pub fn create_descriptor_set_layouts(
        &self,
        device: &ArcDevice,
    ) -> ReactorResult<Vec<vk::DescriptorSetLayout>> {
        self.reflection.create_descriptor_set_layouts(device)
    }
}
