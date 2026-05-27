//! # Bindless Descriptor System (VK_EXT_descriptor_indexing)
//!
//! Sistema bindless completo estilo UE5: todos los recursos del juego
//! (texturas, samplers, meshes, materiales) se referencian por índices u32
//! dentro de arrays globales de descriptors.
//!
//! ## Layout de bindings
//!
//! | Binding | Tipo                  | Slot máx. | Descripción                          |
//! |---------|-----------------------|-----------|--------------------------------------|
//! | 0       | `SAMPLED_IMAGE`       | 8192      | Texturas globales (albedo, normal…)  |
//! | 1       | `SAMPLER`             | 16        | Samplers reutilizables               |
//! | 2       | `STORAGE_BUFFER`      | 4096      | Buffers genéricos (meshlets, etc.)   |
//! | 3       | `STORAGE_BUFFER`      | 4096      | Array de `MeshData` (bindless mesh)  |
//! | 4       | `STORAGE_BUFFER`      | 4096      | Array de `MaterialData` (bindless)   |
//!
//! ## Handles
//!
//! `TextureHandle`, `BufferHandle`, `SamplerHandle`, `MeshHandle` y `MaterialHandle`
//! son newtypes sobre `u32` con valor sentinela `u32::MAX` (`INVALID`).
//! Se pueden copiar libremente, pasar por push constants, escribir a disco, etc.

use crate::core::arc_handle::ArcDevice;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use ash::vk;

// ═══════════════════════════════════════════════════════════════════════════
// Handles (newtypes u32)
// ═══════════════════════════════════════════════════════════════════════════

macro_rules! define_handle {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[repr(transparent)]
        pub struct $name(pub u32);

        impl $name {
            /// Sentinel value for invalid/uninitialized handles.
            pub const INVALID: Self = Self(u32::MAX);

            /// Raw index into the bindless array.
            #[inline(always)]
            pub fn index(&self) -> u32 {
                self.0
            }

            /// True if this handle is not `INVALID`.
            #[inline(always)]
            pub fn is_valid(&self) -> bool {
                self.0 != u32::MAX
            }

            /// Build from a raw index.
            #[inline(always)]
            pub fn from_index(i: u32) -> Self {
                Self(i)
            }
        }

        impl From<u32> for $name {
            #[inline(always)]
            fn from(v: u32) -> Self {
                Self(v)
            }
        }

        impl From<$name> for u32 {
            #[inline(always)]
            fn from(h: $name) -> Self {
                h.0
            }
        }
    };
}

define_handle!(
    TextureHandle,
    "Índice al array bindless de texturas (binding 0)."
);
define_handle!(
    SamplerHandle,
    "Índice al array bindless de samplers (binding 1)."
);
define_handle!(
    BufferHandle,
    "Índice al array bindless de buffers genéricos (binding 2)."
);
define_handle!(
    MeshHandle,
    "Índice al array bindless de `MeshData` (binding 3)."
);
define_handle!(
    MaterialHandle,
    "Índice al array bindless de `MaterialData` (binding 4)."
);

// ═══════════════════════════════════════════════════════════════════════════
// Datos que viajan al GPU (repr C, alineados a 16 bytes)
// ═══════════════════════════════════════════════════════════════════════════

/// Datos de un mesh accesibles desde shader vía `meshes[index]`.
///
/// ```glsl
/// layout(set = 0, binding = 3) readonly buffer MeshBuffer { MeshData meshes[]; };
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GpuMeshData {
    /// Offset (en vértices) dentro del vertex buffer global.
    pub vertex_offset: u32,
    /// Offset (en índices) dentro del index buffer global.
    pub index_offset: u32,
    /// Cantidad de índices a dibujar.
    pub index_count: u32,
    /// Cantidad de vértices (para bounds / draw no-indexed).
    pub vertex_count: u32,
    /// AABB min (para culling GPU).
    pub aabb_min: [f32; 3],
    /// AABB max (para culling GPU).
    pub aabb_max: [f32; 3],
    /// Padding para alinear a 16 bytes (2 floats).
    pub _pad0: f32,
    pub _pad1: f32,
}

/// Datos de un material accesibles desde shader vía `materials[index]`.
///
/// ```glsl
/// layout(set = 0, binding = 4) readonly buffer MaterialBuffer { MaterialData materials[]; };
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuMaterialData {
    /// Albedo base (RGB) + alpha.
    pub albedo: [f32; 4],
    /// Índices de texturas (INVALID si no hay).
    pub albedo_texture: u32,
    pub normal_texture: u32,
    pub metallic_roughness_texture: u32,
    pub ao_texture: u32,
    /// Emissive (RGB) + intensidad.
    pub emissive: [f32; 4],
    /// Metallic, roughness, AO, alpha cuando no hay texturas.
    pub metallic_roughness_ao_alpha: [f32; 4],
    /// Flags (bit 0 = double_sided, bit 1 = alpha_test, …).
    pub flags: u32,
    /// Sampler a usar para las texturas del material.
    pub sampler_index: u32,
    /// Padding a 16 bytes.
    pub _pad0: u32,
    pub _pad1: u32,
}

impl Default for GpuMaterialData {
    fn default() -> Self {
        Self {
            albedo: [1.0, 1.0, 1.0, 1.0],
            albedo_texture: TextureHandle::INVALID.0,
            normal_texture: TextureHandle::INVALID.0,
            metallic_roughness_texture: TextureHandle::INVALID.0,
            ao_texture: TextureHandle::INVALID.0,
            emissive: [0.0; 4],
            metallic_roughness_ao_alpha: [0.0, 0.5, 1.0, 1.0],
            flags: 0,
            sampler_index: 0,
            _pad0: 0,
            _pad1: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Configuración
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
pub struct BindlessConfig {
    pub max_textures: u32,
    pub max_buffers: u32,
    pub max_samplers: u32,
    pub max_meshes: u32,
    pub max_materials: u32,
}

impl Default for BindlessConfig {
    fn default() -> Self {
        Self {
            max_textures: 8192,
            max_buffers: 4096,
            max_samplers: 16,
            max_meshes: 4096,
            max_materials: 4096,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Feature chain y soporte
// ═══════════════════════════════════════════════════════════════════════════

pub fn check_bindless_support(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> ReactorResult<bool> {
    use std::ffi::CStr;
    let props = unsafe { instance.enumerate_device_extension_properties(physical_device)? };
    let ext_name = CStr::from_bytes_with_nul(b"VK_EXT_descriptor_indexing\0").unwrap();
    Ok(props.iter().any(|p| {
        let name = unsafe { CStr::from_ptr(p.extension_name.as_ptr()) };
        name == ext_name
    }))
}

pub fn bindless_feature_chain() -> vk::PhysicalDeviceDescriptorIndexingFeatures<'static> {
    vk::PhysicalDeviceDescriptorIndexingFeatures::default()
        .shader_sampled_image_array_non_uniform_indexing(true)
        .shader_storage_buffer_array_non_uniform_indexing(true)
        .shader_storage_image_array_non_uniform_indexing(true)
        .descriptor_binding_variable_descriptor_count(true)
        .runtime_descriptor_array(true)
        .descriptor_binding_partially_bound(true)
        .descriptor_binding_update_unused_while_pending(true)
}

// ═══════════════════════════════════════════════════════════════════════════
// BindlessRegistry
// ═══════════════════════════════════════════════════════════════════════════

pub struct BindlessRegistry {
    device: ArcDevice,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,

    // Free-slot stacks (LIFO)
    free_texture_slots: Vec<u32>,
    free_buffer_slots: Vec<u32>,
    free_mesh_slots: Vec<u32>,
    free_material_slots: Vec<u32>,
    // Nota: los samplers se registran manualmente (pocos, se reutilizan)
    config: BindlessConfig,
}

impl BindlessRegistry {
    pub fn new(device: ArcDevice, config: BindlessConfig) -> ReactorResult<Self> {
        // ── Descriptor set layout ──────────────────────────────────────
        let bindings = [
            // 0 — Sampled images (texturas)
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(config.max_textures)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
            // 1 — Samplers
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(config.max_samplers)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
            // 2 — Storage buffers genéricos
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(config.max_buffers)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
            // 3 — MeshData array
            vk::DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(config.max_meshes)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
            // 4 — MaterialData array
            vk::DescriptorSetLayoutBinding::default()
                .binding(4)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(config.max_materials)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
        ];

        let binding_flags = [
            // Texturas
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
            // Samplers
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            // Buffers genéricos
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
            // Meshes
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
            // Materials
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
        ];

        let mut flags_info =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .bindings(&bindings)
            .push_next(&mut flags_info);

        let set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        // ── Descriptor pool ────────────────────────────────────────────
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(config.max_textures),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::SAMPLER)
                .descriptor_count(config.max_samplers),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(config.max_buffers + config.max_meshes + config.max_materials),
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .max_sets(1)
            .pool_sizes(&pool_sizes);

        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        // ── Allocate descriptor set ────────────────────────────────────
        let variable_counts = [
            config.max_textures,
            config.max_buffers,
            config.max_meshes,
            config.max_materials,
        ];
        let mut variable_info = vk::DescriptorSetVariableDescriptorCountAllocateInfo::default()
            .descriptor_counts(&variable_counts);

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&set_layout))
            .push_next(&mut variable_info);

        let sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };
        let descriptor_set = sets[0];

        // ── Pipeline layout (con push constant para transform + índices) ──
        let push_constant_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: 128, // Mat4 (64) + índices mesh/material/transform (16) + padding
        }];
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&set_layout))
            .push_constant_ranges(&push_constant_ranges);
        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

        // ── Free-slot stacks ───────────────────────────────────────────
        let free_texture_slots: Vec<u32> = (0..config.max_textures).rev().collect();
        let free_buffer_slots: Vec<u32> = (0..config.max_buffers).rev().collect();
        let free_mesh_slots: Vec<u32> = (0..config.max_meshes).rev().collect();
        let free_material_slots: Vec<u32> = (0..config.max_materials).rev().collect();

        Ok(Self {
            device,
            descriptor_pool,
            descriptor_set,
            set_layout,
            pipeline_layout,
            free_texture_slots,
            free_buffer_slots,
            free_mesh_slots,
            free_material_slots,
            config,
        })
    }

    // ── Texturas ───────────────────────────────────────────────────────
    pub fn register_texture(&mut self, image_view: vk::ImageView) -> ReactorResult<TextureHandle> {
        let slot = self.free_texture_slots.pop().ok_or_else(|| {
            ReactorError::new(ErrorCode::ResourceLimit, "Bindless texture slots exhausted")
        })?;
        let image_info = vk::DescriptorImageInfo::default()
            .image_view(image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(0)
            .dst_array_element(slot)
            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
            .image_info(std::slice::from_ref(&image_info));
        unsafe {
            self.device
                .update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        Ok(TextureHandle(slot))
    }

    pub fn unregister_texture(&mut self, handle: TextureHandle) {
        if handle.is_valid() {
            self.free_texture_slots.push(handle.0);
        }
    }

    // ── Samplers ───────────────────────────────────────────────────────
    pub fn register_sampler(
        &mut self,
        sampler: vk::Sampler,
        slot: u32,
    ) -> ReactorResult<SamplerHandle> {
        if slot >= self.config.max_samplers {
            return Err(ReactorError::new(
                ErrorCode::ResourceLimit,
                "Sampler slot out of range",
            ));
        }
        let image_info = vk::DescriptorImageInfo::default().sampler(sampler);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(1)
            .dst_array_element(slot)
            .descriptor_type(vk::DescriptorType::SAMPLER)
            .image_info(std::slice::from_ref(&image_info));
        unsafe {
            self.device
                .update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        Ok(SamplerHandle(slot))
    }

    // ── Buffers genéricos ──────────────────────────────────────────────
    pub fn register_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
    ) -> ReactorResult<BufferHandle> {
        let slot = self.free_buffer_slots.pop().ok_or_else(|| {
            ReactorError::new(ErrorCode::ResourceLimit, "Bindless buffer slots exhausted")
        })?;
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(offset)
            .range(range);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(2)
            .dst_array_element(slot)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));
        unsafe {
            self.device
                .update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        Ok(BufferHandle(slot))
    }

    pub fn unregister_buffer(&mut self, handle: BufferHandle) {
        if handle.is_valid() {
            self.free_buffer_slots.push(handle.0);
        }
    }

    // ── Meshes (bindless MeshData) ─────────────────────────────────────
    /// Registra un mesh en el slot específico de un storage buffer que contiene
    /// un array de `GpuMeshData`. El usuario debe mantener vivo ese buffer
    /// (generalmente uno global compartido).
    pub fn register_mesh_at(
        &mut self,
        slot: u32,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
    ) -> ReactorResult<MeshHandle> {
        if slot >= self.config.max_meshes {
            return Err(ReactorError::new(
                ErrorCode::ResourceLimit,
                "Mesh slot out of range",
            ));
        }
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(offset)
            .range(range);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(3)
            .dst_array_element(slot)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));
        unsafe {
            self.device
                .update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        // Reservamos el slot del pool para poder liberarlo después
        self.free_mesh_slots.retain(|&s| s != slot);
        Ok(MeshHandle(slot))
    }

    /// Reserva el siguiente slot libre (sin tocar el descriptor).
    /// Útil cuando usas un solo storage buffer con array dinámico.
    pub fn allocate_mesh_slot(&mut self) -> ReactorResult<MeshHandle> {
        let slot = self.free_mesh_slots.pop().ok_or_else(|| {
            ReactorError::new(ErrorCode::ResourceLimit, "Bindless mesh slots exhausted")
        })?;
        Ok(MeshHandle(slot))
    }

    pub fn free_mesh_slot(&mut self, handle: MeshHandle) {
        if handle.is_valid() {
            self.free_mesh_slots.push(handle.0);
        }
    }

    // ── Materials (bindless MaterialData) ──────────────────────────────
    pub fn register_material_at(
        &mut self,
        slot: u32,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
    ) -> ReactorResult<MaterialHandle> {
        if slot >= self.config.max_materials {
            return Err(ReactorError::new(
                ErrorCode::ResourceLimit,
                "Material slot out of range",
            ));
        }
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(offset)
            .range(range);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(4)
            .dst_array_element(slot)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));
        unsafe {
            self.device
                .update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        self.free_material_slots.retain(|&s| s != slot);
        Ok(MaterialHandle(slot))
    }

    pub fn allocate_material_slot(&mut self) -> ReactorResult<MaterialHandle> {
        let slot = self.free_material_slots.pop().ok_or_else(|| {
            ReactorError::new(
                ErrorCode::ResourceLimit,
                "Bindless material slots exhausted",
            )
        })?;
        Ok(MaterialHandle(slot))
    }

    pub fn free_material_slot(&mut self, handle: MaterialHandle) {
        if handle.is_valid() {
            self.free_material_slots.push(handle.0);
        }
    }

    // ── Accessors ──────────────────────────────────────────────────────
    #[inline]
    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }
    #[inline]
    pub fn set_layout(&self) -> vk::DescriptorSetLayout {
        self.set_layout
    }
    #[inline]
    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
    #[inline]
    pub fn config(&self) -> &BindlessConfig {
        &self.config
    }

    /// Stats rápidos (útiles para el HUD del profiler).
    pub fn stats(&self) -> BindlessStats {
        BindlessStats {
            textures_used: self.config.max_textures - self.free_texture_slots.len() as u32,
            textures_max: self.config.max_textures,
            buffers_used: self.config.max_buffers - self.free_buffer_slots.len() as u32,
            buffers_max: self.config.max_buffers,
            meshes_used: self.config.max_meshes - self.free_mesh_slots.len() as u32,
            meshes_max: self.config.max_meshes,
            materials_used: self.config.max_materials - self.free_material_slots.len() as u32,
            materials_max: self.config.max_materials,
        }
    }
}

impl Drop for BindlessRegistry {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.set_layout, None);
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BindlessStats {
    pub textures_used: u32,
    pub textures_max: u32,
    pub buffers_used: u32,
    pub buffers_max: u32,
    pub meshes_used: u32,
    pub meshes_max: u32,
    pub materials_used: u32,
    pub materials_max: u32,
}
