//! PSO Hashing System
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use ash::vk;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PsoHash(pub u64);

impl PsoHash { pub fn as_u64(&self) -> u64 { self.0 } }

pub struct PsoHashBuilder { hasher: DefaultHasher }

impl PsoHashBuilder {
    pub fn new() -> Self { Self { hasher: DefaultHasher::new() } }

    pub fn hash_shader_spirv(&mut self, spirv: &[u32]) -> &mut Self {
        spirv.hash(&mut self.hasher); self
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

    pub fn hash_rasterization(&mut self, r: &vk::PipelineRasterizationStateCreateInfo) -> &mut Self {
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

    pub fn hash_render_pass_formats(&mut self, color: &[vk::Format], depth: Option<vk::Format>) -> &mut Self {
        color.len().hash(&mut self.hasher);
        for f in color { f.as_raw().hash(&mut self.hasher); }
        if let Some(df) = depth { df.as_raw().hash(&mut self.hasher); } else { 0u32.hash(&mut self.hasher); }
        self
    }

    pub fn finalize(&self) -> PsoHash { PsoHash(self.hasher.finish()) }
}

impl Default for PsoHashBuilder { fn default() -> Self { Self::new() } }
