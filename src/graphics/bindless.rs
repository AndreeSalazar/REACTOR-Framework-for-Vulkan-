//! Bindless Descriptor System (VK_EXT_descriptor_indexing)
use std::sync::Arc;
use ash::vk;
use crate::core::error::{ReactorError, ReactorResult, ErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerHandle(pub u32);

impl TextureHandle {
    pub const INVALID: Self = Self(u32::MAX);
    pub fn index(&self) -> u32 { self.0 }
    pub fn is_valid(&self) -> bool { self.0 != u32::MAX }
}

impl BufferHandle {
    pub const INVALID: Self = Self(u32::MAX);
    pub fn index(&self) -> u32 { self.0 }
}

impl SamplerHandle {
    pub const INVALID: Self = Self(u32::MAX);
    pub fn index(&self) -> u32 { self.0 }
}

#[derive(Debug, Clone, Copy)]
pub struct BindlessConfig {
    pub max_textures: u32,
    pub max_buffers: u32,
    pub max_samplers: u32,
}

impl Default for BindlessConfig {
    fn default() -> Self {
        Self {
            max_textures: 8192,
            max_buffers: 4096,
            max_samplers: 16,
        }
    }
}

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

pub struct BindlessRegistry {
    device: Arc<ash::Device>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    free_texture_slots: Vec<u32>,
    free_buffer_slots: Vec<u32>,
    config: BindlessConfig,
}

impl BindlessRegistry {
    pub fn new(device: Arc<ash::Device>, config: BindlessConfig) -> ReactorResult<Self> {
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
        ];

        let binding_flags = [
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
        ];

        let mut flags_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::default()
            .binding_flags(&binding_flags);

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .bindings(&bindings)
            .push_next(&mut flags_info);

        let set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::SAMPLED_IMAGE).descriptor_count(config.max_textures),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::SAMPLER).descriptor_count(config.max_samplers),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(config.max_buffers),
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .max_sets(1)
            .pool_sizes(&pool_sizes);

        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let variable_counts = [config.max_textures, config.max_samplers, config.max_buffers];
        let mut variable_info = vk::DescriptorSetVariableDescriptorCountAllocateInfo::default()
            .descriptor_counts(&variable_counts);

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&set_layout))
            .push_next(&mut variable_info);

        let sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };
        let descriptor_set = sets[0];

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&set_layout));
        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

        let free_texture_slots: Vec<u32> = (0..config.max_textures).rev().collect();
        let free_buffer_slots: Vec<u32> = (0..config.max_buffers).rev().collect();

        Ok(Self {
            device, descriptor_pool, descriptor_set, set_layout, pipeline_layout,
            free_texture_slots, free_buffer_slots, config,
        })
    }

    pub fn register_texture(&mut self, image_view: vk::ImageView) -> ReactorResult<TextureHandle> {
        let slot = self.free_texture_slots.pop()
            .ok_or_else(|| ReactorError::new(ErrorCode::ResourceLimit, "Bindless texture slots exhausted"))?;
        let image_info = vk::DescriptorImageInfo::default()
            .image_view(image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(0).dst_array_element(slot)
            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
            .image_info(std::slice::from_ref(&image_info));
        unsafe { self.device.update_descriptor_sets(std::slice::from_ref(&write), &[]); }
        Ok(TextureHandle(slot))
    }

    pub fn unregister_texture(&mut self, handle: TextureHandle) {
        if handle.is_valid() { self.free_texture_slots.push(handle.0); }
    }

    pub fn register_buffer(&mut self, buffer: vk::Buffer, offset: vk::DeviceSize, range: vk::DeviceSize) -> ReactorResult<BufferHandle> {
        let slot = self.free_buffer_slots.pop()
            .ok_or_else(|| ReactorError::new(ErrorCode::ResourceLimit, "Bindless buffer slots exhausted"))?;
        let buffer_info = vk::DescriptorBufferInfo::default().buffer(buffer).offset(offset).range(range);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set).dst_binding(2).dst_array_element(slot)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));
        unsafe { self.device.update_descriptor_sets(std::slice::from_ref(&write), &[]); }
        Ok(BufferHandle(slot))
    }

    pub fn unregister_buffer(&mut self, handle: BufferHandle) { self.free_buffer_slots.push(handle.0); }
    pub fn descriptor_set(&self) -> vk::DescriptorSet { self.descriptor_set }
    pub fn set_layout(&self) -> vk::DescriptorSetLayout { self.set_layout }
    pub fn pipeline_layout(&self) -> vk::PipelineLayout { self.pipeline_layout }
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
