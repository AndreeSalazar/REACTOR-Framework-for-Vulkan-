//! # Shader Compiler — Multi-lenguaje (WGSL, GLSL, SPIR-V) + Reflection automática
//!
//! ## Flujo
//!
//! ```text
//! WGSL/GLSL  ─► naga::Module  ─► naga::valid::Validator
//!                      │                │
//!                      ▼                ▼
//!               ShaderReflection   naga::valid::ModuleInfo
//!                      │                │
//!                      ▼                ▼
//!              create_descriptor_set_layout()  +  spv::write_vec()
//!                                                   │
//!                                                   ▼
//!                                              SPIR-V (u32[])
//! ```
//!
//! ## HLSL input
//!
//! naga 0.19 **NO** soporta HLSL como input (solo output). Para HLSL input,
//! compilar externamente con `dxc` y cargar el `.spv` resultante.
//!
//! ## Ejemplo
//!
//! ```rust,ignore
//! let mut compiler = ShaderCompiler::new();
//! let compiled = compiler.compile_file("shaders/pbr.vert".as_ref(), ShaderStage::Vertex, "main")?;
//!
//! // Reflection automática → genera el descriptor set layout
//! let layout = compiled.create_descriptor_set_layout(&device)?;
//! ```

use std::path::Path;
use std::fs;
use std::collections::HashMap;
use std::ffi::CStr;

use ash::vk;
use naga::front;
use naga::back::spv;
use naga::valid::{Validator, ValidationFlags, Capabilities};

use crate::core::error::{ReactorError, ReactorResult, ErrorCode};
use crate::core::arc_handle::ArcDevice;

// ═══════════════════════════════════════════════════════════════════════════
// Tipos públicos
// ═══════════════════════════════════════════════════════════════════════════

/// Lenguaje de origen del shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
    SpirV,
}

impl ShaderLanguage {
    /// Detecta el lenguaje por la extensión del archivo.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wgsl" => Some(Self::Wgsl),
            "vert" | "frag" | "comp" | "geom" | "tesc" | "tese" | "glsl" => Some(Self::Glsl),
            "spv" => Some(Self::SpirV),
            _ => None,
        }
    }
}

/// Stage del shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl ShaderStage {
    pub fn to_vk(&self) -> vk::ShaderStageFlags {
        match self {
            Self::Vertex   => vk::ShaderStageFlags::VERTEX,
            Self::Fragment => vk::ShaderStageFlags::FRAGMENT,
            Self::Compute  => vk::ShaderStageFlags::COMPUTE,
        }
    }
    pub fn to_naga(&self) -> naga::ShaderStage {
        match self {
            Self::Vertex   => naga::ShaderStage::Vertex,
            Self::Fragment => naga::ShaderStage::Fragment,
            Self::Compute  => naga::ShaderStage::Compute,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Reflection
// ═══════════════════════════════════════════════════════════════════════════

/// Tipo de binding reflejado desde el shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingType {
    /// Uniform buffer (set=0, binding=N).
    UniformBuffer,
    /// Storage buffer (readonly o read-write).
    StorageBuffer { read_only: bool },
    /// Sampler (set=0, binding=N).
    Sampler,
    /// Sampled image (texture sin sampler).
    SampledImage,
    /// Combined image + sampler.
    CombinedImageSampler,
    /// Storage image (image load/store).
    StorageImage { read_only: bool },
    /// Input attachment (subpass).
    InputAttachment,
    /// Acceleration structure (ray tracing).
    AccelerationStructure,
}

impl BindingType {
    pub fn to_vk_descriptor_type(&self) -> vk::DescriptorType {
        match self {
            Self::UniformBuffer            => vk::DescriptorType::UNIFORM_BUFFER,
            Self::StorageBuffer { .. }     => vk::DescriptorType::STORAGE_BUFFER,
            Self::Sampler                  => vk::DescriptorType::SAMPLER,
            Self::SampledImage             => vk::DescriptorType::SAMPLED_IMAGE,
            Self::CombinedImageSampler     => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            Self::StorageImage { .. }      => vk::DescriptorType::STORAGE_IMAGE,
            Self::InputAttachment          => vk::DescriptorType::INPUT_ATTACHMENT,
            Self::AccelerationStructure    => vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
        }
    }
}

/// Un binding reflejado desde el SPIR-V / módulo naga.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectedBinding {
    pub name: String,
    pub group: u32,
    pub binding: u32,
    pub ty: BindingType,
    /// Stages donde se usa este binding (bitwise OR de stages).
    pub stages: vk::ShaderStageFlags,
    /// Tamaño del buffer (para uniform/storage), 0 para imágenes.
    pub size: u32,
    /// Cantidad de elementos si es array (1 si no es array).
    pub count: u32,
}

/// Push constant reflejado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectedPushConstant {
    pub name: String,
    pub stages: vk::ShaderStageFlags,
    pub size: u32,
}

/// Entry point reflejado.
#[derive(Debug, Clone)]
pub struct ReflectedEntryPoint {
    pub name: String,
    pub stage: ShaderStage,
    pub workgroup_size: Option<[u32; 3]>,
}

/// Información completa de reflection de un shader compilado.
#[derive(Debug, Clone, Default)]
pub struct ShaderReflection {
    pub entry_points: Vec<ReflectedEntryPoint>,
    pub bindings: Vec<ReflectedBinding>,
    pub push_constants: Vec<ReflectedPushConstant>,
}

impl ShaderReflection {
    /// Construye la reflection desde un módulo naga + info de validación.
    pub fn from_naga(module: &naga::Module, info: &naga::valid::ModuleInfo, stage: ShaderStage) -> Self {
        let mut bindings = Vec::new();
        let mut push_constants = Vec::new();
        let vk_stage = stage.to_vk();

        // ── Recorrer global variables → bindings ───────────────────────
        for (_, global) in module.global_variables.iter() {
            // Filtrar solo los address spaces que nos interesan para reflection
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

        // ── Entry points ───────────────────────────────────────────────
        let entry_points = module.entry_points.iter().map(|ep| {
            let stage = match ep.stage {
                naga::ShaderStage::Vertex   => ShaderStage::Vertex,
                naga::ShaderStage::Fragment => ShaderStage::Fragment,
                naga::ShaderStage::Compute  => ShaderStage::Compute,
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
        }).collect();

        Self { entry_points, bindings, push_constants }
    }

    /// Tamaño aproximado de un tipo (para push constants / uniforms).
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

    /// Determina el tipo de binding para una variable Handle.
    fn handle_type(module: &naga::Module, ty: naga::Handle<naga::Type>) -> BindingType {
        let t = &module.types[ty];
        match t.inner {
            naga::TypeInner::Sampler { comparison: _ } => {
                BindingType::Sampler
            }
            naga::TypeInner::Image { class, .. } => {
                match class {
                    naga::ImageClass::Sampled { .. } => BindingType::SampledImage,
                    naga::ImageClass::Depth { .. }   => BindingType::SampledImage,
                    naga::ImageClass::Storage { access, .. } => {
                        BindingType::StorageImage {
                            read_only: !access.contains(naga::StorageAccess::STORE),
                        }
                    }
                }
            }
            _ => BindingType::SampledImage, // fallback
        }
    }

    // ── Merge stages de bindings con el mismo (group, binding) ────────
    /// Cuando compilas vertex + fragment por separado, los bindings pueden
    /// estar duplicados. Este método fusiona las `stages` para producir un
    /// layout unificado.
    pub fn merge_stages(&mut self, other: &ShaderReflection) {
        for other_b in &other.bindings {
            if let Some(mine) = self.bindings.iter_mut().find(|b| {
                b.group == other_b.group && b.binding == other_b.binding
            }) {
                mine.stages |= other_b.stages;
            } else {
                self.bindings.push(other_b.clone());
            }
        }
        for other_pc in &other.push_constants {
            if let Some(mine) = self.push_constants.iter_mut().find(|pc| pc.name == other_pc.name) {
                mine.stages |= other_pc.stages;
                mine.size = mine.size.max(other_pc.size);
            } else {
                self.push_constants.push(other_pc.clone());
            }
        }
    }

    // ── Generación de descriptor set layout ────────────────────────────
    /// Crea un `VkDescriptorSetLayout` automáticamente desde la reflection.
    ///
    /// Agrupa bindings por `group` y genera un layout por cada grupo.
    /// Típicamente solo hay un grupo (group=0), así que devuelve un Vec con 1 elemento.
    pub fn create_descriptor_set_layouts(
        &self,
        device: &ArcDevice,
    ) -> ReactorResult<Vec<vk::DescriptorSetLayout>> {
        // Agrupar bindings por group
        let mut groups: HashMap<u32, Vec<&ReflectedBinding>> = HashMap::new();
        for b in &self.bindings {
            groups.entry(b.group).or_default().push(b);
        }

        let mut layouts = Vec::new();
        let max_group = groups.keys().copied().max().unwrap_or(0);

        for g in 0..=max_group {
            let bindings_in_group = groups.get(&g).cloned().unwrap_or_default();

            let vk_bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings_in_group.iter().map(|b| {
                vk::DescriptorSetLayoutBinding::default()
                    .binding(b.binding)
                    .descriptor_type(b.ty.to_vk_descriptor_type())
                    .descriptor_count(b.count)
                    .stage_flags(b.stages)
            }).collect();

            let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&vk_bindings);

            let layout = unsafe {
                device.create_descriptor_set_layout(&layout_info, None)
                    .map_err(|e| ReactorError::with_source(
                        ErrorCode::VulkanDescriptorSet,
                        format!("Failed to create descriptor set layout for group {}", g),
                        e,
                    ))?
            };
            layouts.push(layout);
        }

        Ok(layouts)
    }

    /// Crea un `VkPipelineLayout` con los descriptor set layouts + push constants.
    pub fn create_pipeline_layout(
        &self,
        device: &ArcDevice,
        set_layouts: &[vk::DescriptorSetLayout],
    ) -> ReactorResult<vk::PipelineLayout> {
        let push_constant_ranges: Vec<vk::PushConstantRange> = self.push_constants.iter().map(|pc| {
            vk::PushConstantRange {
                stage_flags: pc.stages,
                offset: 0,
                size: pc.size,
            }
        }).collect();

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        unsafe {
            device.create_pipeline_layout(&layout_info, None)
                .map_err(|e| ReactorError::with_source(
                    ErrorCode::VulkanPipelineCreation,
                    "Failed to create pipeline layout from reflection",
                    e,
                ))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CompiledShader
// ═══════════════════════════════════════════════════════════════════════════

/// Shader ya compilado + metadatos de reflection.
#[derive(Debug, Clone)]
pub struct CompiledShader {
    /// SPIR-V words listos para `vkCreateShaderModule`.
    pub spirv: Vec<u32>,
    /// Stage del shader.
    pub stage: ShaderStage,
    /// Entry point principal (típicamente "main").
    pub entry_point: String,
    /// Información de reflection (bindings, push constants, …).
    pub reflection: ShaderReflection,
    /// Hash del SPIR-V (para PSO cache).
    pub spirv_hash: u64,
}

impl CompiledShader {
    /// Crea un `VkShaderModule` desde el SPIR-V.
    pub fn create_shader_module(&self, device: &ArcDevice) -> ReactorResult<vk::ShaderModule> {
        let create_info = vk::ShaderModuleCreateInfo::default().code(&self.spirv);
        unsafe {
            device.create_shader_module(&create_info, None)
                .map_err(|e| ReactorError::with_source(
                    ErrorCode::VulkanShaderCompilation,
                    "create_shader_module failed",
                    e,
                ))
        }
    }

    /// Crea un `VkPipelineShaderStageCreateInfo` listo para usar en un pipeline.
    ///
    /// **Importante**: El `name` retornado es un `CStr` estático; si tu entry point
    /// no es "main", guarda el `CString` en el caller y pasa la referencia.
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

    /// Helper: crea descriptor set layouts desde la reflection.
    pub fn create_descriptor_set_layouts(&self, device: &ArcDevice) -> ReactorResult<Vec<vk::DescriptorSetLayout>> {
        self.reflection.create_descriptor_set_layouts(device)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ShaderCompiler
// ═══════════════════════════════════════════════════════════════════════════

pub struct ShaderCompiler {
    validator: Validator,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        Self {
            validator: Validator::new(ValidationFlags::all(), Capabilities::all()),
        }
    }

    /// Compila un archivo de shader a SPIR-V + reflection.
    pub fn compile_file(
        &mut self,
        path: &Path,
        stage: ShaderStage,
        entry_point: &str,
    ) -> ReactorResult<CompiledShader> {
        let source = fs::read_to_string(path).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::IoError,
                format!("Failed to read shader file: {}", path.display()),
                e,
            )
        })?;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang = ShaderLanguage::from_extension(ext).ok_or_else(|| {
            ReactorError::new(
                ErrorCode::InvalidArgument,
                format!("Unsupported shader extension: {}", ext),
            )
        })?;

        self.compile_source(&source, lang, stage, entry_point)
    }

    /// Carga un SPIR-V precompilado y le hace reflection.
    pub fn load_spirv(
        &mut self,
        path: &Path,
        stage: ShaderStage,
        entry_point: &str,
    ) -> ReactorResult<CompiledShader> {
        let bytes = fs::read(path).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::IoError,
                format!("Failed to read SPIR-V file: {}", path.display()),
                e,
            )
        })?;
        if bytes.len() % 4 != 0 {
            return Err(ReactorError::new(ErrorCode::InvalidFormat, "SPIR-V file size not multiple of 4"));
        }
        let spirv: Vec<u32> = bytes.chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        self.load_spirv_words(&spirv, stage, entry_point)
    }

    /// Carga desde words SPIR-V (útil si ya lo tienes en memoria).
    pub fn load_spirv_words(
        &mut self,
        spirv: &[u32],
        stage: ShaderStage,
        entry_point: &str,
    ) -> ReactorResult<CompiledShader> {
        // Parsear SPIR-V con naga para extraer reflection
        let module = naga::front::spv::Frontend::new(
            spirv.iter().copied(),
            &naga::front::spv::Options {
                adjust_coordinate_space: false,
                strict_capabilities: false,
                block_ctx_dump_prefix: None,
            },
        ).parse().map_err(|e| {
            ReactorError::new(ErrorCode::ShaderCompilation, format!("SPIR-V parse error: {:?}", e))
        })?;

        let info = self.validator.validate(&module).map_err(|e| {
            ReactorError::new(ErrorCode::ShaderCompilation, format!("SPIR-V validation error: {:?}", e))
        })?;

        let reflection = ShaderReflection::from_naga(&module, &info, stage);

        let spirv_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            spirv.hash(&mut h);
            h.finish()
        };

        Ok(CompiledShader {
            spirv: spirv.to_vec(),
            stage,
            entry_point: entry_point.to_string(),
            reflection,
            spirv_hash,
        })
    }

    /// Compila código fuente a SPIR-V + reflection.
    pub fn compile_source(
        &mut self,
        source: &str,
        lang: ShaderLanguage,
        stage: ShaderStage,
        entry_point: &str,
    ) -> ReactorResult<CompiledShader> {
        // 1. Parsear a módulo naga
        let module = match lang {
            ShaderLanguage::Wgsl => front::wgsl::parse_str(source).map_err(|e| {
                ReactorError::new(
                    ErrorCode::ShaderCompilation,
                    format!("WGSL parse error: {:?}", e),
                )
            })?,
            ShaderLanguage::Glsl => {
                let options = front::glsl::Options {
                    stage: stage.to_naga(),
                    defines: Default::default(),
                };
                let mut parser = front::glsl::Frontend::default();
                parser.parse(&options, source).map_err(|e| {
                    ReactorError::new(
                        ErrorCode::ShaderCompilation,
                        format!("GLSL parse error: {:?}", e),
                    )
                })?
            }
            ShaderLanguage::SpirV => {
                return Err(ReactorError::new(
                    ErrorCode::InvalidArgument,
                    "Use load_spirv() for .spv files",
                ));
            }
        };

        // 2. Validar
        let info = self.validator.validate(&module).map_err(|e| {
            ReactorError::new(
                ErrorCode::ShaderCompilation,
                format!("Shader validation error: {:?}", e),
            )
        })?;

        // 3. Reflection
        let reflection = ShaderReflection::from_naga(&module, &info, stage);

        // 4. Generar SPIR-V
        let mut caps = naga::FastHashSet::default();
        caps.insert(spv::Capability::Shader);

        let options = spv::Options {
            lang_version: (1, 3), // Vulkan 1.3
            flags: spv::WriterFlags::DEBUG,
            capabilities: Some(caps),
            bounds_check_policies: naga::proc::BoundsCheckPolicies::default(),
            ..Default::default()
        };

        let spirv = spv::write_vec(&module, &info, &options, None).map_err(|e| {
            ReactorError::new(
                ErrorCode::ShaderCompilation,
                format!("SPIR-V generation error: {:?}", e),
            )
        })?;

        let spirv_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            spirv.hash(&mut h);
            h.finish()
        };

        Ok(CompiledShader {
            spirv,
            stage,
            entry_point: entry_point.to_string(),
            reflection,
            spirv_hash,
        })
    }
}

impl Default for ShaderCompiler {
    fn default() -> Self { Self::new() }
}
