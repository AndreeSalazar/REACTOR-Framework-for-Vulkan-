//! Shader Compiler - Multi-lenguaje (WGSL, GLSL) via naga
//!
//! **Nota**: naga 0.19 NO soporta HLSL como input, solo como output.
//! Para HLSL input, se recomienda compilar externamente con `dxc` y cargar el SPIR-V.

use std::path::Path;
use std::fs;
use naga::front;
use naga::back::spv;
use naga::valid::{Validator, ValidationFlags, Capabilities};
use crate::core::error::{ReactorError, ReactorResult, ErrorCode};

/// Lenguaje de origen del shader
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
    SpirV,
}

impl ShaderLanguage {
    /// Detecta el lenguaje por extension del archivo
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wgsl" => Some(Self::Wgsl),
            "vert" | "frag" | "comp" | "geom" | "tesc" | "tese" | "glsl" => Some(Self::Glsl),
            "spv" => Some(Self::SpirV),
            _ => None,
        }
    }
}

/// Stage del shader
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

/// Modulo de shader ya compilado + metadatos
#[derive(Debug, Clone)]
pub struct CompiledShader {
    pub spirv: Vec<u32>,
    pub stage: ShaderStage,
    pub entry_point: String,
}

/// Compilador de shaders
pub struct ShaderCompiler {
    validator: Validator,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        Self {
            validator: Validator::new(ValidationFlags::all(), Capabilities::all()),
        }
    }

    /// Compila un archivo de shader a SPIR-V
    pub fn compile_file(
        &self,
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

    /// Compila codigo fuente a SPIR-V
    pub fn compile_source(
        &self,
        source: &str,
        lang: ShaderLanguage,
        stage: ShaderStage,
        entry_point: &str,
    ) -> ReactorResult<CompiledShader> {
        // 1. Parsear a modulo naga
        let module = match lang {
            ShaderLanguage::Wgsl => front::wgsl::parse_str(source).map_err(|e| {
                ReactorError::new(
                    ErrorCode::ShaderCompilation,
                    format!("WGSL parse error: {:?}", e),
                )
            })?,
            ShaderLanguage::Glsl => {
                let naga_stage = match stage {
                    ShaderStage::Vertex => naga::ShaderStage::Vertex,
                    ShaderStage::Fragment => naga::ShaderStage::Fragment,
                    ShaderStage::Compute => naga::ShaderStage::Compute,
                };
                let options = front::glsl::Options {
                    stage: naga_stage,
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

        // 3. Generar SPIR-V
        let mut caps = naga::FastHashSet::default();
        caps.insert(spv::Capability::Shader);

        let options = spv::Options {
            lang_version: (1, 3), // Vulkan 1.3
            flags: spv::WriterFlags::DEBUG,
            capabilities: caps,
            bounds_check_policies: naga::proc::BoundsCheckPolicies::default(),
            ..Default::default()
        };

        let spirv = spv::write_vec(&module, &info, &options).map_err(|e| {
            ReactorError::new(
                ErrorCode::ShaderCompilation,
                format!("SPIR-V generation error: {:?}", e),
            )
        })?;

        Ok(CompiledShader {
            spirv,
            stage,
            entry_point: entry_point.to_string(),
        })
    }
}

impl Default for ShaderCompiler {
    fn default() -> Self {
        Self::new()
    }
}
