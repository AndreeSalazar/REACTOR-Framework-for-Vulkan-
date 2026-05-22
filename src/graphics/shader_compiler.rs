//! Shader Compiler - Multi-lenguaje (WGSL, GLSL, HLSL) via naga
use std::path::Path;
use std::fs;
use naga::front;
use naga::back::spv;
use naga::valid::{Validator, ValidationFlags, Capabilities};
use crate::core::error::{ReactorError, ReactorResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderLanguage { Wgsl, Glsl, Hlsl, SpirV }

impl ShaderLanguage {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wgsl" => Some(Self::Wgsl),
            "vert" | "frag" | "comp" | "glsl" => Some(Self::Glsl),
            "hlsl" | "fx" => Some(Self::Hlsl),
            "spv" => Some(Self::SpirV),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage { Vertex, Fragment, Compute }

#[derive(Debug, Clone)]
pub struct CompiledShader {
    pub spirv: Vec<u32>,
    pub stage: ShaderStage,
    pub entry_point: String,
}

pub struct ShaderCompiler { validator: Validator }

impl ShaderCompiler {
    pub fn new() -> Self {
        Self { validator: Validator::new(ValidationFlags::all(), Capabilities::all()) }
    }

    pub fn compile_file(&self, path: &Path, stage: ShaderStage, entry: &str) -> ReactorResult<CompiledShader> {
        let source = fs::read_to_string(path).map_err(ReactorError::Io)?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang = ShaderLanguage::from_extension(ext)
            .ok_or_else(|| ReactorError::Other(format!("Unsupported shader extension: {}", ext)))?;
        self.compile_source(&source, lang, stage, entry)
    }

    pub fn compile_source(&self, source: &str, lang: ShaderLanguage, stage: ShaderStage, entry: &str) -> ReactorResult<CompiledShader> {
        let module = match lang {
            ShaderLanguage::Wgsl => front::wgsl::parse_str(source)
                .map_err(|e| ReactorError::Other(format!("WGSL parse error: {:?}", e)))?,
            ShaderLanguage::Glsl => {
                let naga_stage = match stage {
                    ShaderStage::Vertex => naga::ShaderStage::Vertex,
                    ShaderStage::Fragment => naga::ShaderStage::Fragment,
                    ShaderStage::Compute => naga::ShaderStage::Compute,
                };
                let options = front::glsl::Options { stage: naga_stage, defines: Default::default() };
                let mut parser = front::glsl::Frontend::default();
                parser.parse(&options, source)
                    .map_err(|e| ReactorError::Other(format!("GLSL parse error: {:?}", e)))?
            }
            ShaderLanguage::Hlsl => front::hlsl::Frontend::new(Default::default())
                .parse(source).map_err(|e| ReactorError::Other(format!("HLSL parse error: {:?}", e)))?,
            ShaderLanguage::SpirV => return Err(ReactorError::Other("Use load_spirv() for .spv files".into())),
        };
        let info = self.validator.validate(&module)
            .map_err(|e| ReactorError::Other(format!("Shader validation error: {:?}", e)))?;
        let mut caps = naga::FastHashSet::default();
        caps.insert(spv::Capability::Shader);
        let options = spv::Options {
            lang_version: (1, 3),
            flags: spv::WriterFlags::DEBUG,
            capabilities: caps,
            bounds_check_policies: naga::proc::BoundsCheckPolicies::default(),
            ..Default::default()
        };
        let spirv = spv::write_vec(&module, &info, &options)
            .map_err(|e| ReactorError::Other(format!("SPIR-V generation error: {:?}", e)))?;
        Ok(CompiledShader { spirv, stage, entry_point: entry.to_string() })
    }
}
