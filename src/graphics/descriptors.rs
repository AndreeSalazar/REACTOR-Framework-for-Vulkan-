use ash::vk;
use crate::vulkan_context::VulkanContext;
use std::error::Error;

// ============================================================================
// Descriptor Set Layout
// ============================================================================

pub struct DescriptorSetLayout {
    pub handle: vk::DescriptorSetLayout,
    device: ash::Device,
}

#[derive(Clone)]
pub struct DescriptorBinding {
    pub binding: u32,
    pub descriptor_type: vk::DescriptorType,
    pub count: u32,
    pub stage_flags: vk::ShaderStageFlags,
}

impl DescriptorSetLayout {
    pub fn new(ctx: &VulkanContext, bindings: &[DescriptorBinding]) -> Result<Self, Box<dyn Error>> {
        let vk_bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings
            .iter()
            .map(|b| {
                vk::DescriptorSetLayoutBinding::default()
                    .binding(b.binding)
                    .descriptor_type(b.descriptor_type)
                    .descriptor_count(b.count)
                    .stage_flags(b.stage_flags)
            })
            .collect();

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&vk_bindings);

        let handle = unsafe { ctx.device.create_descriptor_set_layout(&layout_info, None)? };

        Ok(Self {
            handle,
            device: ctx.device.clone(),
        })
    }

    pub fn for_uniform_buffer(ctx: &VulkanContext, binding: u32, stages: vk::ShaderStageFlags) -> Result<Self, Box<dyn Error>> {
        Self::new(ctx, &[DescriptorBinding {
            binding,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            count: 1,
            stage_flags: stages,
        }])
    }

    pub fn for_texture(ctx: &VulkanContext, binding: u32) -> Result<Self, Box<dyn Error>> {
        Self::new(ctx, &[DescriptorBinding {
            binding,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
        }])
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_descriptor_set_layout(self.handle, None);
        }
    }
}

// ============================================================================
// Descriptor Pool
// ============================================================================

pub struct DescriptorPool {
    pub handle: vk::DescriptorPool,
    device: ash::Device,
}

pub struct PoolSize {
    pub descriptor_type: vk::DescriptorType,
    pub count: u32,
}

impl DescriptorPool {
    pub fn new(ctx: &VulkanContext, max_sets: u32, pool_sizes: &[PoolSize]) -> Result<Self, Box<dyn Error>> {
        let vk_sizes: Vec<vk::DescriptorPoolSize> = pool_sizes
            .iter()
            .map(|s| {
                vk::DescriptorPoolSize::default()
                    .ty(s.descriptor_type)
                    .descriptor_count(s.count)
            })
            .collect();

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&vk_sizes)
            .max_sets(max_sets)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);

        let handle = unsafe { ctx.device.create_descriptor_pool(&pool_info, None)? };

        Ok(Self {
            handle,
            device: ctx.device.clone(),
        })
    }

    pub fn standard(ctx: &VulkanContext, max_sets: u32) -> Result<Self, Box<dyn Error>> {
        Self::new(ctx, max_sets, &[
            PoolSize { descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, count: max_sets * 2 },
            PoolSize { descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, count: max_sets * 4 },
            PoolSize { descriptor_type: vk::DescriptorType::STORAGE_BUFFER, count: max_sets * 2 },
        ])
    }

    pub fn allocate(&self, ctx: &VulkanContext, layout: &DescriptorSetLayout) -> Result<DescriptorSet, Box<dyn Error>> {
        let layouts = [layout.handle];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.handle)
            .set_layouts(&layouts);

        let sets = unsafe { ctx.device.allocate_descriptor_sets(&alloc_info)? };

        Ok(DescriptorSet {
            handle: sets[0],
            device: ctx.device.clone(),
        })
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_descriptor_pool(self.handle, None);
        }
    }
}

// ============================================================================
// Descriptor Set
// ============================================================================

pub struct DescriptorSet {
    pub handle: vk::DescriptorSet,
    device: ash::Device,
}

impl DescriptorSet {
    pub fn update_buffer(&self, binding: u32, buffer: vk::Buffer, size: u64) {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(0)
            .range(size);

        let buffer_infos = [buffer_info];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.handle)
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_infos);

        unsafe {
            self.device.update_descriptor_sets(&[write], &[]);
        }
    }

    pub fn update_image(&self, binding: u32, image_view: vk::ImageView, sampler: vk::Sampler) {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .sampler(sampler);

        let image_infos = [image_info];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.handle)
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_infos);

        unsafe {
            self.device.update_descriptor_sets(&[write], &[]);
        }
    }

    pub fn update_storage_buffer(&self, binding: u32, buffer: vk::Buffer, size: u64) {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(0)
            .range(size);

        let buffer_infos = [buffer_info];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.handle)
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(&buffer_infos);

        unsafe {
            self.device.update_descriptor_sets(&[write], &[]);
        }
    }
}
