//! # Mesh Shader Support (VK_EXT_mesh_shader) — Opcional
//!
//! Soporte opcional para mesh shaders cuando la extensión `VK_EXT_mesh_shader`
//! está disponible en el hardware. Los mesh shaders reemplazan el pipeline
//! vertex/geometry tradicional con un modelo más flexible y eficiente.
//!
//! ## Arquitectura
//!
//! ```text
//! Traditional:  Vertex Shader → (Optional Geometry) → Rasterization
//! Mesh Shader:  Task Shader → Mesh Shader → Rasterization
//! ```
//!
//! ## Uso
//!
//! ```rust,ignore
//! // Verificar soporte
//! let supported = check_mesh_shader_support(&instance, physical_device)?;
//! if supported {
//!     let features = mesh_shader_feature_chain();
//!     // Habilitar en device creation
//! }
//!
//! // Crear pipeline con mesh shader
//! let pipeline = MeshShaderPipeline::new(
//!     &device,
//!     task_shader_spirv,   // Opcional
//!     mesh_shader_spirv,
//!     fragment_spirv,
//!     &bindless_layout,
//! )?;
//! ```

use ash::vk;
use std::ffi::CStr;
use crate::core::error::{ReactorError, ReactorResult, ErrorCode};
use crate::core::arc_handle::ArcDevice;

// ═══════════════════════════════════════════════════════════════════════════
// Soporte de extensión
// ═══════════════════════════════════════════════════════════════════════════

/// Verifica si `VK_EXT_mesh_shader` está disponible en el dispositivo.
pub fn check_mesh_shader_support(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> ReactorResult<bool> {
    use std::ffi::CStr;
    let props = unsafe { instance.enumerate_device_extension_properties(physical_device)? };
    let ext_name = CStr::from_bytes_with_nul(b"VK_EXT_mesh_shader\0").unwrap();
    Ok(props.iter().any(|p| {
        let name = unsafe { CStr::from_ptr(p.extension_name.as_ptr()) };
        name == ext_name
    }))
}

/// Feature chain para habilitar mesh shaders en device creation.
pub fn mesh_shader_feature_chain() -> vk::PhysicalDeviceMeshShaderFeaturesEXT<'static> {
    vk::PhysicalDeviceMeshShaderFeaturesEXT::default()
        .task_shader(true)
        .mesh_shader(true)
}

/// Propiedades de mesh shaders del dispositivo (límites).
#[derive(Debug, Clone, Copy)]
pub struct MeshShaderProperties {
    pub max_task_work_group_total_count: u32,
    pub max_task_work_group_count: [u32; 3],
    pub max_task_work_group_invocations: u32,
    pub max_task_work_group_size: [u32; 3],
    pub max_task_payload_size: u32,
    pub max_mesh_work_group_total_count: u32,
    pub max_mesh_work_group_count: [u32; 3],
    pub max_mesh_work_group_invocations: u32,
    pub max_mesh_work_group_size: [u32; 3],
    pub max_mesh_output_vertices: u32,
    pub max_mesh_output_primitives: u32,
    pub max_mesh_multiview_view_count: u32,
}

/// Consulta las propiedades de mesh shaders del dispositivo.
pub fn query_mesh_shader_properties(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> MeshShaderProperties {
    let mut props = vk::PhysicalDeviceMeshShaderPropertiesEXT::default();
    let mut props2 = vk::PhysicalDeviceProperties2::default().push_next(&mut props);
    unsafe {
        instance.get_physical_device_properties2(physical_device, &mut props2);
    }

    MeshShaderProperties {
        max_task_work_group_total_count: props.max_task_work_group_total_count,
        max_task_work_group_count: [
            props.max_task_work_group_count[0],
            props.max_task_work_group_count[1],
            props.max_task_work_group_count[2],
        ],
        max_task_work_group_invocations: props.max_task_work_group_invocations,
        max_task_work_group_size: [
            props.max_task_work_group_size[0],
            props.max_task_work_group_size[1],
            props.max_task_work_group_size[2],
        ],
        max_task_payload_size: props.max_task_payload_size,
        max_mesh_work_group_total_count: props.max_mesh_work_group_total_count,
        max_mesh_work_group_count: [
            props.max_mesh_work_group_count[0],
            props.max_mesh_work_group_count[1],
            props.max_mesh_work_group_count[2],
        ],
        max_mesh_work_group_invocations: props.max_mesh_work_group_invocations,
        max_mesh_work_group_size: [
            props.max_mesh_work_group_size[0],
            props.max_mesh_work_group_size[1],
            props.max_mesh_work_group_size[2],
        ],
        max_mesh_output_vertices: props.max_mesh_output_vertices,
        max_mesh_output_primitives: props.max_mesh_output_primitives,
        max_mesh_multiview_view_count: props.max_mesh_multiview_view_count,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Mesh Shader Pipeline
// ═══════════════════════════════════════════════════════════════════════════

/// Pipeline que usa mesh shaders (task → mesh → fragment).
pub struct MeshShaderPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    task_module: Option<vk::ShaderModule>,
    mesh_module: vk::ShaderModule,
    fragment_module: vk::ShaderModule,
    device: ArcDevice,
}

impl MeshShaderPipeline {
    /// Crea un pipeline con mesh shader.
    ///
    /// # Arguments
    ///
    /// * `device` - ArcDevice wrapper
    /// * `task_spirv` - SPIR-V del task shader (opcional, puede ser None)
    /// * `mesh_spirv` - SPIR-V del mesh shader (requerido)
    /// * `fragment_spirv` - SPIR-V del fragment shader
    /// * `set_layouts` - Descriptor set layouts
    /// * `push_constant_size` - Tamaño de push constants (0 si no se usan)
    /// * `render_format` - Formato del color attachment
    /// * `depth_format` - Formato del depth attachment (FORMAT_UNDEFINED si no hay)
    pub fn new(
        device: ArcDevice,
        task_spirv: Option<&[u32]>,
        mesh_spirv: &[u32],
        fragment_spirv: &[u32],
        set_layouts: &[vk::DescriptorSetLayout],
        push_constant_size: u32,
        render_format: vk::Format,
        depth_format: vk::Format,
    ) -> ReactorResult<Self> {
        // ── Shader modules ─────────────────────────────────────────────
        let task_module = if let Some(spirv) = task_spirv {
            let info = vk::ShaderModuleCreateInfo::default().code(spirv);
            Some(unsafe {
                device.create_shader_module(&info, None)
                    .map_err(|e| ReactorError::with_source(
                        ErrorCode::VulkanShaderCompilation,
                        "Failed to create task shader module",
                        e,
                    ))?
            })
        } else {
            None
        };

        let mesh_info = vk::ShaderModuleCreateInfo::default().code(mesh_spirv);
        let mesh_module = unsafe {
            device.create_shader_module(&mesh_info, None)
                .map_err(|e| ReactorError::with_source(
                    ErrorCode::VulkanShaderCompilation,
                    "Failed to create mesh shader module",
                    e,
                ))?
        };

        let frag_info = vk::ShaderModuleCreateInfo::default().code(fragment_spirv);
        let fragment_module = unsafe {
            device.create_shader_module(&frag_info, None)
                .map_err(|e| ReactorError::with_source(
                    ErrorCode::VulkanShaderCompilation,
                    "Failed to create fragment shader module",
                    e,
                ))?
        };

        // ── Pipeline layout ────────────────────────────────────────────
        let push_constant_ranges = if push_constant_size > 0 {
            vec![vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::TASK_EXT
                    | vk::ShaderStageFlags::MESH_EXT
                    | vk::ShaderStageFlags::FRAGMENT,
                offset: 0,
                size: push_constant_size,
            }]
        } else {
            vec![]
        };

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let layout = unsafe {
            device.create_pipeline_layout(&layout_info, None)
                .map_err(|e| ReactorError::with_source(
                    ErrorCode::VulkanPipelineCreation,
                    "Failed to create mesh shader pipeline layout",
                    e,
                ))?
        };

        // ── Shader stages ──────────────────────────────────────────────
        let mut stages = Vec::new();

        if let Some(task_mod) = task_module {
            stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::TASK_EXT)
                    .module(task_mod)
                    .name(CStr::from_bytes_with_nul(b"main\0").unwrap()),
            );
        }

        stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::MESH_EXT)
                .module(mesh_module)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap()),
        );

        stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap()),
        );

        // ── Pipeline state (sin vertex input, sin input assembly) ──────
        // Mesh shaders NO usan vertex input ni input assembly tradicional

        // Rasterization
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0);

        // Multisample (sin MSAA por defecto)
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Depth stencil
        let depth_stencil = if depth_format != vk::Format::UNDEFINED {
            vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
        } else {
            vk::PipelineDepthStencilStateCreateInfo::default()
        };

        // Color blend
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA);
        let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&[color_blend_attachment]);

        // Dynamic state
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&dynamic_states);

        // ── Pipeline creation (usa pNext para mesh shader state) ──────
        // Nota: Mesh shader pipeline NO tiene vertex_input_state ni input_assembly_state
        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(layout);

        // Nota: No necesitamos vk::PipelineRenderingCreateInfo porque usamos
        // dynamic rendering que se configura en el command buffer con beginRendering

        let pipelines = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, e)| ReactorError::with_source(
                    ErrorCode::VulkanPipelineCreation,
                    "Failed to create mesh shader pipeline",
                    e,
                ))?
        };

        Ok(Self {
            pipeline: pipelines[0],
            layout,
            task_module,
            mesh_module,
            fragment_module,
            device,
        })
    }

    /// Graba un draw call de mesh shader en el command buffer.
    ///
    /// # Arguments
    /// * `cmd` - Command buffer
    /// * `group_count_x` - Número de workgroups en X
    /// * `group_count_y` - Número de workgroups en Y (1 si no se usa)
    /// * `group_count_z` - Número de workgroups en Z (1 si no se usa)
    pub fn draw_mesh_tasks(
        &self,
        device: &ash::Device,
        cmd: vk::CommandBuffer,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unsafe {
            device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
            device.cmd_draw_mesh_tasks_ext(cmd, group_count_x, group_count_y, group_count_z);
        }
    }

    /// Draw mesh tasks indirect (GPU-driven).
    pub fn draw_mesh_tasks_indirect(
        &self,
        device: &ash::Device,
        cmd: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
            device.cmd_draw_mesh_tasks_indirect_ext(cmd, buffer, offset, draw_count, stride);
        }
    }

    /// Draw mesh tasks indirect count (con count buffer).
    pub fn draw_mesh_tasks_indirect_count(
        &self,
        device: &ash::Device,
        cmd: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        count_buffer: vk::Buffer,
        count_buffer_offset: vk::DeviceSize,
        max_draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
            device.cmd_draw_mesh_tasks_indirect_count_ext(
                cmd,
                buffer,
                offset,
                count_buffer,
                count_buffer_offset,
                max_draw_count,
                stride,
            );
        }
    }

    #[inline]
    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    #[inline]
    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Drop for MeshShaderPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            if let Some(task_mod) = self.task_module {
                self.device.destroy_shader_module(task_mod, None);
            }
            self.device.destroy_shader_module(self.mesh_module, None);
            self.device.destroy_shader_module(self.fragment_module, None);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Meshlet Utilities
// ═══════════════════════════════════════════════════════════════════════════

/// Un meshlet (grupo de vértices + primitivas para mesh shader).
///
/// Los meshlets son la unidad de trabajo de los mesh shaders.
/// Típicamente 64 vértices + 124 triángulos (límite de NVIDIA).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Meshlet {
    /// Offset al primer vértice en el vertex buffer.
    pub vertex_offset: u32,
    /// Cantidad de vértices en este meshlet.
    pub vertex_count: u32,
    /// Offset al primer índice en el index buffer.
    pub index_offset: u32,
    /// Cantidad de índices (triángulos * 3).
    pub index_count: u32,
    /// AABB del meshlet (para culling).
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    /// Cono de normales (para backface culling).
    pub cone_apex: [f32; 3],
    pub cone_axis: [f32; 3],
    pub cone_cutoff: f32,
    /// Padding para alinear a 16 bytes.
    pub _pad: [f32; 3],
}

/// Builder para crear meshlets desde un mesh tradicional.
///
/// ```rust,ignore
/// let meshlets = MeshletBuilder::build(&vertices, &indices, max_vertices=64, max_triangles=124);
/// ```
pub struct MeshletBuilder;

impl MeshletBuilder {
    /// Construye meshlets desde vértices e índices.
    ///
    /// Esta es una implementación simplificada. Para producción, usa
    /// `meshopt` crate que tiene algoritmos optimizados (meshopt_buildMeshlets).
    pub fn build(
        vertices: &[[f32; 3]],
        indices: &[u32],
        max_vertices: u32,
        max_triangles: u32,
    ) -> Vec<Meshlet> {
        let triangle_count = indices.len() / 3;
        let mut meshlets = Vec::new();

        // Simplificación: dividir en chunks de max_triangles
        let triangles_per_meshlet = max_triangles as usize;
        let mut current_triangle = 0;

        while current_triangle < triangle_count {
            let end_triangle = (current_triangle + triangles_per_meshlet).min(triangle_count);
            let index_start = current_triangle * 3;
            let index_end = end_triangle * 3;

            // Calcular AABB
            let mut aabb_min = [f32::MAX; 3];
            let mut aabb_max = [f32::MIN; 3];

            for i in (index_start..index_end).step_by(3) {
                for j in 0..3 {
                    let idx = indices[i + j] as usize;
                    if idx < vertices.len() {
                        let v = vertices[idx];
                        for k in 0..3 {
                            aabb_min[k] = aabb_min[k].min(v[k]);
                            aabb_max[k] = aabb_max[k].max(v[k]);
                        }
                    }
                }
            }

            // Contar vértices únicos
            let unique_vertices: std::collections::HashSet<u32> =
                indices[index_start..index_end].iter().copied().collect();

            if unique_vertices.len() <= max_vertices as usize {
                meshlets.push(Meshlet {
                    vertex_offset: 0, // El caller debe ajustar
                    vertex_count: unique_vertices.len() as u32,
                    index_offset: index_start as u32,
                    index_count: (index_end - index_start) as u32,
                    aabb_min,
                    aabb_max,
                    cone_apex: [0.0; 3],
                    cone_axis: [0.0, 0.0, 1.0],
                    cone_cutoff: 0.0,
                    _pad: [0.0; 3],
                });
            }

            current_triangle = end_triangle;
        }

        meshlets
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meshlet_builder() {
        let vertices: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 1, 3, 2];

        let meshlets = MeshletBuilder::build(&vertices, &indices, 64, 124);
        assert_eq!(meshlets.len(), 1);
        assert_eq!(meshlets[0].index_count, 6);
        assert_eq!(meshlets[0].vertex_count, 4);
    }
}
