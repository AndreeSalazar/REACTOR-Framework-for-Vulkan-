use std::collections::HashMap;

use ash::vk;

use crate::core::arc_handle::ArcDevice;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};

use super::types::{BindingType, ReflectedBinding, ReflectedEntryPoint, ReflectedPushConstant, ShaderReflection, ShaderStage};

impl ShaderReflection {
    pub fn from_naga(
        module: &naga::Module,
        info: &naga::valid::ModuleInfo,
        stage: ShaderStage,
    ) -> Self {
        let mut bindings = Vec::new();
        let mut push_constants = Vec::new();
        let vk_stage = stage.to_vk();

        for (_, global) in module.global_variables.iter() {
            let is_relevant = matches!(
                global.space,
                naga::AddressSpace::Uniform
                    | naga::AddressSpace::Storage { .. }
                    | naga::AddressSpace::Handle
                    | naga::AddressSpace::PushConstant
            );
            if !is_relevant {
                continue;
            }

            let name = global.name.clone().unwrap_or_else(|| "unnamed".to_string());

            match global.space {
                naga::AddressSpace::PushConstant => {
                    let size = Self::type_size(module, global.ty);
                    push_constants.push(ReflectedPushConstant {
                        name,
                        stages: vk_stage,
                        size: size as u32,
                    });
                }
                naga::AddressSpace::Uniform => {
                    if let Some(ref br) = global.binding {
                        let size = Self::type_size(module, global.ty);
                        bindings.push(ReflectedBinding {
                            name,
                            group: br.group,
                            binding: br.binding,
                            ty: BindingType::UniformBuffer,
                            stages: vk_stage,
                            size: size as u32,
                            count: 1,
                        });
                    }
                }
                naga::AddressSpace::Storage { access } => {
                    let read_only = !access.contains(naga::StorageAccess::STORE);
                    if let Some(ref br) = global.binding {
                        let size = Self::type_size(module, global.ty);
                        bindings.push(ReflectedBinding {
                            name,
                            group: br.group,
                            binding: br.binding,
                            ty: BindingType::StorageBuffer { read_only },
                            stages: vk_stage,
                            size: size as u32,
                            count: 1,
                        });
                    }
                }
                naga::AddressSpace::Handle => {
                    if let Some(ref br) = global.binding {
                        let ty = Self::handle_type(module, global.ty);
                        bindings.push(ReflectedBinding {
                            name,
                            group: br.group,
                            binding: br.binding,
                            ty,
                            stages: vk_stage,
                            size: 0,
                            count: 1,
                        });
                    }
                }
                _ => {}
            }
        }

        let entry_points = module
            .entry_points
            .iter()
            .map(|ep| {
                let stage = match ep.stage {
                    naga::ShaderStage::Vertex => ShaderStage::Vertex,
                    naga::ShaderStage::Fragment => ShaderStage::Fragment,
                    naga::ShaderStage::Compute => ShaderStage::Compute,
                };
                let workgroup_size = if ep.stage == naga::ShaderStage::Compute {
                    Some(ep.workgroup_size)
                } else {
                    None
                };
                ReflectedEntryPoint {
                    name: ep.name.clone(),
                    stage,
                    workgroup_size,
                }
            })
            .collect();

        Self { entry_points, bindings, push_constants }
    }

    fn type_size(module: &naga::Module, ty: naga::Handle<naga::Type>) -> usize {
        let t = &module.types[ty];
        match t.inner {
            naga::TypeInner::Scalar(s) => s.width as usize,
            naga::TypeInner::Vector { size, scalar } => size as usize * scalar.width as usize,
            naga::TypeInner::Matrix { columns, rows, scalar } => {
                columns as usize * rows as usize * scalar.width as usize
            }
            naga::TypeInner::Struct { span, .. } => span as usize,
            naga::TypeInner::Array { base, size, stride } => {
                let count = match size {
                    naga::ArraySize::Constant(n) => n.get() as usize,
                    _ => 1,
                };
                count * stride as usize
            }
            _ => 0,
        }
    }

    fn handle_type(module: &naga::Module, ty: naga::Handle<naga::Type>) -> BindingType {
        let t = &module.types[ty];
        match t.inner {
            naga::TypeInner::Sampler { comparison: _ } => BindingType::Sampler,
            naga::TypeInner::Image { class, .. } => match class {
                naga::ImageClass::Sampled { .. } => BindingType::SampledImage,
                naga::ImageClass::Depth { .. } => BindingType::SampledImage,
                naga::ImageClass::Storage { access, .. } => BindingType::StorageImage {
                    read_only: !access.contains(naga::StorageAccess::STORE),
                },
            },
            _ => BindingType::SampledImage,
        }
    }

    pub fn merge_stages(&mut self, other: &ShaderReflection) {
        for other_b in &other.bindings {
            if let Some(mine) = self
                .bindings
                .iter_mut()
                .find(|b| b.group == other_b.group && b.binding == other_b.binding)
            {
                mine.stages |= other_b.stages;
            } else {
                self.bindings.push(other_b.clone());
            }
        }
        for other_pc in &other.push_constants {
            if let Some(mine) = self
                .push_constants
                .iter_mut()
                .find(|pc| pc.name == other_pc.name)
            {
                mine.stages |= other_pc.stages;
                mine.size = mine.size.max(other_pc.size);
            } else {
                self.push_constants.push(other_pc.clone());
            }
        }
    }

    pub fn create_descriptor_set_layouts(
        &self,
        device: &ArcDevice,
    ) -> ReactorResult<Vec<vk::DescriptorSetLayout>> {
        let mut groups: HashMap<u32, Vec<&ReflectedBinding>> = HashMap::new();
        for b in &self.bindings {
            groups.entry(b.group).or_default().push(b);
        }

        let mut layouts = Vec::new();
        let max_group = groups.keys().copied().max().unwrap_or(0);

        for g in 0..=max_group {
            let bindings_in_group = groups.get(&g).cloned().unwrap_or_default();

            let vk_bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings_in_group
                .iter()
                .map(|b| {
                    vk::DescriptorSetLayoutBinding::default()
                        .binding(b.binding)
                        .descriptor_type(b.ty.to_vk_descriptor_type())
                        .descriptor_count(b.count)
                        .stage_flags(b.stages)
                })
                .collect();

            let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&vk_bindings)
                .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);

            let layout = unsafe {
                device
                    .create_descriptor_set_layout(&layout_info, None)
                    .map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanDescriptorSet,
                            format!("Failed to create descriptor set layout for group {}", g),
                            e,
                        )
                    })?
            };
            layouts.push(layout);
        }

        Ok(layouts)
    }

    pub fn create_pipeline_layout(
        &self,
        device: &ArcDevice,
        set_layouts: &[vk::DescriptorSetLayout],
    ) -> ReactorResult<vk::PipelineLayout> {
        let push_constant_ranges: Vec<vk::PushConstantRange> = self
            .push_constants
            .iter()
            .map(|pc| vk::PushConstantRange {
                stage_flags: pc.stages,
                offset: 0,
                size: pc.size,
            })
            .collect();

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        unsafe {
            device
                .create_pipeline_layout(&layout_info, None)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanPipelineCreation,
                        "Failed to create pipeline layout from reflection",
                        e,
                    )
                })
        }
    }
}
