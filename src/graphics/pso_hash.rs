//! # PSO Hashing System
//!
//! Hash determinístico de (shader SPIR-V + render state) → `PsoHash(u64)`.
//! Dos pipelines con el mismo hash son binariamente idénticos y se pueden
//! reutilizar sin recompilar.

use ash::vk;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Hash único de un Pipeline State Object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PsoHash(pub u64);

impl PsoHash {
    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Crea desde dos spirv_hash (vertex + fragment) + state bits.
    pub fn from_shaders_and_state(
        vert_spirv_hash: u64,
        frag_spirv_hash: u64,
        state_bits: u64,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        vert_spirv_hash.hash(&mut hasher);
        frag_spirv_hash.hash(&mut hasher);
        state_bits.hash(&mut hasher);
        Self(hasher.finish())
    }
}

/// Builder incremental para calcular el hash de un PSO.
pub struct PsoHashBuilder {
    hasher: DefaultHasher,
}

impl PsoHashBuilder {
    pub fn new() -> Self {
        Self { hasher: DefaultHasher::new() }
    }

    /// Hash del SPIR-V de un shader (usar `CompiledShader::spirv_hash`).
    pub fn hash_shader_spirv(&mut self, spirv: &[u32]) -> &mut Self {
        spirv.hash(&mut self.hasher);
        self
    }

    /// Hash directamente desde el `spirv_hash` pre-calculado (más rápido).
    pub fn hash_shader_spirv_hash(&mut self, hash: u64) -> &mut Self {
        hash.hash(&mut self.hasher);
        self
    }

    pub fn hash_vertex_input(&mut self, vi: &vk::PipelineVertexInputStateCreateInfo) -> &mut Self {
        let bc = vi.vertex_binding_description_count as usize;
        let bindings = unsafe { std::slice::from_raw_parts(vi.p_vertex_binding_descriptions, bc) };
        for b in bindings {
            b.binding.hash(&mut self.hasher);
            b.stride.hash(&mut self.hasher);
            b.input_rate.as_raw().hash(&mut self.hasher);
        }
        let ac = vi.vertex_attribute_description_count as usize;
        let attrs = unsafe { std::slice::from_raw_parts(vi.p_vertex_attribute_descriptions, ac) };
        for a in attrs {
            a.location.hash(&mut self.hasher);
            a.binding.hash(&mut self.hasher);
            a.format.as_raw().hash(&mut self.hasher);
            a.offset.hash(&mut self.hasher);
        }
        self
    }

    pub fn hash_rasterization(
        &mut self,
        r: &vk::PipelineRasterizationStateCreateInfo,
    ) -> &mut Self {
        r.depth_clamp_enable.hash(&mut self.hasher);
        r.rasterizer_discard_enable.hash(&mut self.hasher);
        r.polygon_mode.as_raw().hash(&mut self.hasher);
        r.cull_mode.as_raw().hash(&mut self.hasher);
        r.front_face.as_raw().hash(&mut self.hasher);
        r.depth_bias_enable.hash(&mut self.hasher);
        r.line_width.to_bits().hash(&mut self.hasher);
        self
    }

    pub fn hash_multisample(&mut self, m: &vk::PipelineMultisampleStateCreateInfo) -> &mut Self {
        m.rasterization_samples.as_raw().hash(&mut self.hasher);
        m.sample_shading_enable.hash(&mut self.hasher);
        m.min_sample_shading.to_bits().hash(&mut self.hasher);
        self
    }

    pub fn hash_depth_stencil(&mut self, d: &vk::PipelineDepthStencilStateCreateInfo) -> &mut Self {
        d.depth_test_enable.hash(&mut self.hasher);
        d.depth_write_enable.hash(&mut self.hasher);
        d.depth_compare_op.as_raw().hash(&mut self.hasher);
        d.depth_bounds_test_enable.hash(&mut self.hasher);
        d.stencil_test_enable.hash(&mut self.hasher);
        self
    }

    pub fn hash_color_blend(&mut self, cb: &vk::PipelineColorBlendStateCreateInfo) -> &mut Self {
        cb.logic_op_enable.hash(&mut self.hasher);
        let ac = cb.attachment_count as usize;
        let attachments = unsafe { std::slice::from_raw_parts(cb.p_attachments, ac) };
        for a in attachments {
            a.blend_enable.hash(&mut self.hasher);
            a.src_color_blend_factor.as_raw().hash(&mut self.hasher);
            a.dst_color_blend_factor.as_raw().hash(&mut self.hasher);
            a.color_blend_op.as_raw().hash(&mut self.hasher);
            a.src_alpha_blend_factor.as_raw().hash(&mut self.hasher);
            a.dst_alpha_blend_factor.as_raw().hash(&mut self.hasher);
            a.alpha_blend_op.as_raw().hash(&mut self.hasher);
            a.color_write_mask.as_raw().hash(&mut self.hasher);
        }
        self
    }

    pub fn hash_render_pass_formats(
        &mut self,
        color: &[vk::Format],
        depth: Option<vk::Format>,
    ) -> &mut Self {
        color.len().hash(&mut self.hasher);
        for f in color {
            f.as_raw().hash(&mut self.hasher);
        }
        if let Some(df) = depth {
            df.as_raw().hash(&mut self.hasher);
        } else {
            0u32.hash(&mut self.hasher);
        }
        self
    }

    /// Hash de un `PipelineConfig` simplificado (cull mode, polygon mode, etc).
    pub fn hash_pipeline_config_bits(&mut self, bits: u64) -> &mut Self {
        bits.hash(&mut self.hasher);
        self
    }

    pub fn finalize(&self) -> PsoHash {
        PsoHash(self.hasher.finish())
    }
}

impl Default for PsoHashBuilder {
    fn default() -> Self {
        Self::new()
    }
}
