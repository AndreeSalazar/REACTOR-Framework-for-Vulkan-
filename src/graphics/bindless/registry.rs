use ash::vk;

use crate::core::arc_handle::ArcDevice;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};

use super::config::BindlessConfig;
use super::config::BindlessStats;
use super::handle::{BufferHandle, MaterialHandle, MeshHandle, SamplerHandle, TextureHandle};

pub struct BindlessRegistry {
    device: ArcDevice,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,

    free_texture_slots: Vec<u32>,
    free_buffer_slots: Vec<u32>,
    free_mesh_slots: Vec<u32>,
    free_material_slots: Vec<u32>,

    config: BindlessConfig,
}

impl BindlessRegistry {
    pub fn new(device: ArcDevice, config: BindlessConfig) -> ReactorResult<Self> {
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(config.max_textures)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(config.max_samplers)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(config.max_buffers)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(config.max_meshes)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(4)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(config.max_materials)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS | vk::ShaderStageFlags::COMPUTE),
        ];

        let binding_flags = [
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
            vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
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

        let push_constant_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: 128,
        }];
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&set_layout))
            .push_constant_ranges(&push_constant_ranges);
        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

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
            self.device.update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        Ok(TextureHandle(slot))
    }

    pub fn unregister_texture(&mut self, handle: TextureHandle) {
        if handle.is_valid() {
            self.free_texture_slots.push(handle.0);
        }
    }

    pub fn register_sampler(
        &mut self,
        sampler: vk::Sampler,
        slot: u32,
    ) -> ReactorResult<SamplerHandle> {
        if slot >= self.config.max_samplers {
            return Err(ReactorError::new(ErrorCode::ResourceLimit, "Sampler slot out of range"));
        }
        let image_info = vk::DescriptorImageInfo::default().sampler(sampler);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(1)
            .dst_array_element(slot)
            .descriptor_type(vk::DescriptorType::SAMPLER)
            .image_info(std::slice::from_ref(&image_info));
        unsafe {
            self.device.update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        Ok(SamplerHandle(slot))
    }

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
            self.device.update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        Ok(BufferHandle(slot))
    }

    pub fn unregister_buffer(&mut self, handle: BufferHandle) {
        if handle.is_valid() {
            self.free_buffer_slots.push(handle.0);
        }
    }

    pub fn register_mesh_at(
        &mut self,
        slot: u32,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
    ) -> ReactorResult<MeshHandle> {
        if slot >= self.config.max_meshes {
            return Err(ReactorError::new(ErrorCode::ResourceLimit, "Mesh slot out of range"));
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
            self.device.update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        self.free_mesh_slots.retain(|&s| s != slot);
        Ok(MeshHandle(slot))
    }

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

    pub fn register_material_at(
        &mut self,
        slot: u32,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
    ) -> ReactorResult<MaterialHandle> {
        if slot >= self.config.max_materials {
            return Err(ReactorError::new(ErrorCode::ResourceLimit, "Material slot out of range"));
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
            self.device.update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
        self.free_material_slots.retain(|&s| s != slot);
        Ok(MaterialHandle(slot))
    }

    pub fn allocate_material_slot(&mut self) -> ReactorResult<MaterialHandle> {
        let slot = self.free_material_slots.pop().ok_or_else(|| {
            ReactorError::new(ErrorCode::ResourceLimit, "Bindless material slots exhausted")
        })?;
        Ok(MaterialHandle(slot))
    }

    pub fn free_material_slot(&mut self, handle: MaterialHandle) {
        if handle.is_valid() {
            self.free_material_slots.push(handle.0);
        }
    }

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
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_descriptor_set_layout(self.set_layout, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
