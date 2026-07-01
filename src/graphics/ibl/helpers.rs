use crate::core::error::ReactorResult;
use crate::core::VulkanContext;
use crate::graphics::ibl::verr;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};

pub fn create_cubemap_sampler(ctx: &VulkanContext, max_lod: f32) -> ReactorResult<vk::Sampler> {
    let info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .anisotropy_enable(false).max_anisotropy(1.0)
        .compare_enable(false).min_lod(0.0).max_lod(max_lod)
        .border_color(vk::BorderColor::FLOAT_OPAQUE_BLACK);
    unsafe { ctx.ash_device().create_sampler(&info, None).map_err(verr) }
}

pub fn create_2d_sampler(ctx: &VulkanContext) -> ReactorResult<vk::Sampler> {
    let info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .min_lod(0.0).max_lod(1.0);
    unsafe { ctx.ash_device().create_sampler(&info, None).map_err(verr) }
}

pub fn combined_image_sampler_b(b: u32) -> vk::DescriptorSetLayoutBinding<'static> {
    vk::DescriptorSetLayoutBinding::default().binding(b)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE)
}

pub fn storage_image_b(b: u32) -> vk::DescriptorSetLayoutBinding<'static> {
    vk::DescriptorSetLayoutBinding::default().binding(b)
        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
        .descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE)
}

pub fn create_bake_descriptor_pool(ctx: &VulkanContext, max_sets: u32) -> ReactorResult<vk::DescriptorPool> {
    let sizes = [
        vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(max_sets * 2),
        vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(max_sets * 2),
    ];
    let info = vk::DescriptorPoolCreateInfo::default().max_sets(max_sets)
        .pool_sizes(&sizes).flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
    unsafe { ctx.ash_device().create_descriptor_pool(&info, None).map_err(verr) }
}

pub fn create_final_descriptor_layout(ctx: &VulkanContext) -> ReactorResult<vk::DescriptorSetLayout> {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default().binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default().binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default().binding(2)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default().binding(3)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT),
    ];
    let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings)
        .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
    unsafe { ctx.ash_device().create_descriptor_set_layout(&info, None).map_err(verr) }
}

pub fn create_final_descriptor_pool(ctx: &VulkanContext) -> ReactorResult<vk::DescriptorPool> {
    let sizes = [
        vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(3),
        vk::DescriptorPoolSize::default().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1),
    ];
    let info = vk::DescriptorPoolCreateInfo::default().max_sets(1).pool_sizes(&sizes)
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET | vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
    unsafe { ctx.ash_device().create_descriptor_pool(&info, None).map_err(verr) }
}

pub fn allocate_set(
    ctx: &VulkanContext, pool: vk::DescriptorPool, layout: vk::DescriptorSetLayout,
) -> ReactorResult<vk::DescriptorSet> {
    let layouts = [layout];
    let info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(pool).set_layouts(&layouts);
    let sets = unsafe { ctx.ash_device().allocate_descriptor_sets(&info).map_err(verr)? };
    Ok(sets[0])
}

pub fn update_set_combined(
    ctx: &VulkanContext, set: vk::DescriptorSet, binding: u32,
    view: vk::ImageView, sampler: vk::Sampler, layout: vk::ImageLayout,
) {
    let img = [vk::DescriptorImageInfo::default().image_layout(layout).image_view(view).sampler(sampler)];
    let w = vk::WriteDescriptorSet::default().dst_set(set).dst_binding(binding)
        .dst_array_element(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&img);
    unsafe { ctx.ash_device().update_descriptor_sets(&[w], &[]); }
}

pub fn update_set_storage_image(
    ctx: &VulkanContext, set: vk::DescriptorSet, binding: u32, view: vk::ImageView,
) {
    let img = [vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::GENERAL).image_view(view).sampler(vk::Sampler::null())];
    let w = vk::WriteDescriptorSet::default().dst_set(set).dst_binding(binding)
        .dst_array_element(0).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(&img);
    unsafe { ctx.ash_device().update_descriptor_sets(&[w], &[]); }
}

pub fn update_set_uniform_buffer(
    ctx: &VulkanContext, set: vk::DescriptorSet, binding: u32, buffer: vk::Buffer, size: u64,
) {
    let bi = [vk::DescriptorBufferInfo::default().buffer(buffer).offset(0).range(size)];
    let w = vk::WriteDescriptorSet::default().dst_set(set).dst_binding(binding)
        .dst_array_element(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(&bi);
    unsafe { ctx.ash_device().update_descriptor_sets(&[w], &[]); }
}

pub fn create_uniform_buffer(
    ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>, max_mip: f32,
) -> ReactorResult<(vk::Buffer, Allocation)> {
    let device = ctx.ash_device();
    let size = std::mem::size_of::<f32>() as u64;
    let info = vk::BufferCreateInfo::default().size(size)
        .usage(vk::BufferUsageFlags::UNIFORM_BUFFER).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buf = unsafe { device.create_buffer(&info, None).map_err(verr)? };
    let req = unsafe { device.get_buffer_memory_requirements(buf) };
    let mut alloc = allocator.lock().unwrap()
        .allocate(&AllocationCreateDesc {
            name: "ibl_params_ubo", requirements: req,
            location: MemoryLocation::CpuToGpu, linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        }).map_err(verr)?;
    unsafe { device.bind_buffer_memory(buf, alloc.memory(), alloc.offset()).map_err(verr)? };
    alloc.mapped_slice_mut().expect("ubo no mapeado")[..4].copy_from_slice(&max_mip.to_le_bytes());
    Ok((buf, alloc))
}

pub fn create_one_shot_command_pool(ctx: &VulkanContext) -> ReactorResult<vk::CommandPool> {
    let info = vk::CommandPoolCreateInfo::default().queue_family_index(ctx.queue_family_index)
        .flags(vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    unsafe { ctx.ash_device().create_command_pool(&info, None).map_err(verr) }
}

pub fn begin_one_shot(ctx: &VulkanContext, pool: vk::CommandPool) -> ReactorResult<vk::CommandBuffer> {
    let alloc = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1);
    let cb = unsafe { ctx.ash_device().allocate_command_buffers(&alloc).map_err(verr)?[0] };
    let begin = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { ctx.ash_device().begin_command_buffer(cb, &begin).map_err(verr)? };
    Ok(cb)
}

pub fn end_and_submit(ctx: &VulkanContext, _pool: vk::CommandPool, cb: vk::CommandBuffer) -> ReactorResult<()> {
    let device = ctx.ash_device();
    unsafe { device.end_command_buffer(cb).map_err(verr)?; }
    let cbs = [cb];
    let submit = vk::SubmitInfo::default().command_buffers(&cbs);
    unsafe {
        device.queue_submit(ctx.graphics_queue, &[submit], vk::Fence::null()).map_err(verr)?;
        device.queue_wait_idle(ctx.graphics_queue).map_err(verr)?;
    }
    Ok(())
}

pub fn transition_2d(
    ctx: &VulkanContext, cmd: vk::CommandBuffer, image: vk::Image, mip_levels: u32,
    old_l: vk::ImageLayout, new_l: vk::ImageLayout,
    src_a: vk::AccessFlags, dst_a: vk::AccessFlags,
    src_s: vk::PipelineStageFlags, dst_s: vk::PipelineStageFlags,
) {
    let b = vk::ImageMemoryBarrier::default()
        .old_layout(old_l).new_layout(new_l)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0).level_count(mip_levels)
            .base_array_layer(0).layer_count(1))
        .src_access_mask(src_a).dst_access_mask(dst_a);
    unsafe {
        ctx.ash_device().cmd_pipeline_barrier(cmd, src_s, dst_s, vk::DependencyFlags::empty(), &[], &[], &[b]);
    }
}

pub fn transition_cube(
    ctx: &VulkanContext, cmd: vk::CommandBuffer, image: vk::Image,
    mip_levels: u32, layer_count: u32,
    old_l: vk::ImageLayout, new_l: vk::ImageLayout,
    src_a: vk::AccessFlags, dst_a: vk::AccessFlags,
    src_s: vk::PipelineStageFlags, dst_s: vk::PipelineStageFlags,
) {
    let b = vk::ImageMemoryBarrier::default()
        .old_layout(old_l).new_layout(new_l)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0).level_count(mip_levels)
            .base_array_layer(0).layer_count(layer_count))
        .src_access_mask(src_a).dst_access_mask(dst_a);
    unsafe {
        ctx.ash_device().cmd_pipeline_barrier(cmd, src_s, dst_s, vk::DependencyFlags::empty(), &[], &[], &[b]);
    }
}
