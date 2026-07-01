use std::fs;
use std::path::Path;

use naga::back::spv;
use naga::front;
use naga::valid::{Capabilities, ValidationFlags, Validator};

use crate::core::error::{ErrorCode, ReactorError, ReactorResult};

use super::types::{CompiledShader, ShaderLanguage, ShaderReflection, ShaderStage};

pub struct ShaderCompiler {
    validator: Validator,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        Self {
            validator: Validator::new(ValidationFlags::all(), Capabilities::all()),
        }
    }

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
            return Err(ReactorError::new(
                ErrorCode::InvalidFormat,
                "SPIR-V file size not multiple of 4",
            ));
        }
        let spirv: Vec<u32> = bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        self.load_spirv_words(&spirv, stage, entry_point)
    }

    pub fn load_spirv_words(
        &mut self,
        spirv: &[u32],
        stage: ShaderStage,
        entry_point: &str,
    ) -> ReactorResult<CompiledShader> {
        let module = naga::front::spv::Frontend::new(
            spirv.iter().copied(),
            &naga::front::spv::Options {
                adjust_coordinate_space: false,
                strict_capabilities: false,
                block_ctx_dump_prefix: None,
            },
        )
        .parse()
        .map_err(|e| {
            ReactorError::new(
                ErrorCode::ShaderCompilation,
                format!("SPIR-V parse error: {:?}", e),
            )
        })?;

        let info = self.validator.validate(&module).map_err(|e| {
            ReactorError::new(
                ErrorCode::ShaderCompilation,
                format!("SPIR-V validation error: {:?}", e),
            )
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

    pub fn compile_source(
        &mut self,
        source: &str,
        lang: ShaderLanguage,
        stage: ShaderStage,
        entry_point: &str,
    ) -> ReactorResult<CompiledShader> {
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

        let info = self.validator.validate(&module).map_err(|e| {
            ReactorError::new(
                ErrorCode::ShaderCompilation,
                format!("Shader validation error: {:?}", e),
            )
        })?;

        let reflection = ShaderReflection::from_naga(&module, &info, stage);

        let mut caps = naga::FastHashSet::default();
        caps.insert(spv::Capability::Shader);

        let options = spv::Options {
            lang_version: (1, 3),
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
    fn default() -> Self {
        Self::new()
    }
}
