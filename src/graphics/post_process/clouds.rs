//! Volumetric clouds — ray-marched 3D cloud rendering (HZD/RDR2 style)
//!
//! Wraps `shaders/post/volumetric_clouds.comp`. The shader needs two 3D noise
//! textures (shape = Worley 64³, detail = value 16³) that are procedurally
//! generated on the CPU and uploaded via a staging buffer at init time.
//!
//! Public API:
//! - `VolumetricClouds::new(ctx, allocator, command_pool, queue, width, ...)`
//!   — creates the compute pipeline, descriptor resources, 3D noise textures
//!   and the output image array.
//! - `VolumetricClouds::dispatch(...)` — record a frame's cloud ray-march pass.

use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use crate::graphics::Image;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator};
use std::sync::{Arc, Mutex};

pub fn generate_value_noise_3d(size: u32, seed: u32) -> Vec<u8> {
    let n = size as usize;
    let mut rng = seed.wrapping_mul(2_654_435_761);
    let mut data = vec![0u8; n * n * n * 4];
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let i = ((z * n * n + y * n + x) * 4) as usize;
                let mut h = rng
                    .wrapping_add((x as u32).wrapping_mul(73))
                    .wrapping_add((y as u32).wrapping_mul(91))
                    .wrapping_add((z as u32).wrapping_mul(127));
                h = h.wrapping_mul(1664525).wrapping_add(1013904223);
                data[i] = ((h >> 24) & 0xFF) as u8;
                data[i + 1] = ((h >> 16) & 0xFF) as u8;
                data[i + 2] = ((h >> 8) & 0xFF) as u8;
                data[i + 3] = 255;
            }
        }
    }
    let mut smooth = vec![0u8; data.len()];
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let xi = (x + 1) % n;
                let yi = (y + 1) % n;
                let zi = (z + 1) % n;
                let mut acc = [0u32; 4];
                for dz in 0..2 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let sx = if dx == 0 { x } else { xi };
                            let sy = if dy == 0 { y } else { yi };
                            let sz = if dz == 0 { z } else { zi };
                            let idx = ((sz * n * n + sy * n + sx) * 4) as usize;
                            acc[0] += data[idx] as u32;
                            acc[1] += data[idx + 1] as u32;
                            acc[2] += data[idx + 2] as u32;
                            acc[3] += data[idx + 3] as u32;
                        }
                    }
                }
                let o = ((z * n * n + y * n + x) * 4) as usize;
                smooth[o] = (acc[0] / 8) as u8;
                smooth[o + 1] = (acc[1] / 8) as u8;
                smooth[o + 2] = (acc[2] / 8) as u8;
                smooth[o + 3] = (acc[3] / 8) as u8;
            }
        }
    }
    smooth
}

struct Noise3D {
    image: vk::Image,
    view: vk::ImageView,
    allocation: Allocation,
}

pub struct VolumetricClouds {
    pub pipeline: Option<crate::compute::ComputePipeline>,
    pub descriptor_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub output_images: Vec<Image>,
    pub time: f32,
    shape: Noise3D,
    detail: Noise3D,
    noise_sampler: vk::Sampler,
    device: ash::Device,
}

impl VolumetricClouds {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        width: u32,
        height: u32,
        image_count: u32,
        linear_sampler: vk::Sampler,
    ) -> ReactorResult<Self> {
        let device = ctx.ash_device().clone();

        let shape = create_3d_noise_image(
            ctx,
            &device,
            allocator.clone(),
            command_pool,
            queue,
            64,
            0xDEAD_BEEF,
        )?;
        let detail = create_3d_noise_image(
            ctx,
            &device,
            allocator.clone(),
            command_pool,
            queue,
            16,
            0xC0FF_EE42,
        )?;

        let noise_sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS);
        let noise_sampler = unsafe { device.create_sampler(&noise_sampler_info, None) }
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanImageCreation,
                    "Clouds: create sampler",
                    e,
                )
            })?;

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanDescriptorSet,
                    "Clouds: create descriptor layout",
                    e,
                )
            })?;

        let spv = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../../../shaders/post/volumetric_clouds.spv"
        )))
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanShaderCompilation,
                "Clouds: load volumetric_clouds.spv",
                e,
            )
        })?;
        let pipeline = crate::compute::ComputePipeline::new(
            ctx,
            &spv,
            &[descriptor_layout],
            Some(120),
        )?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(image_count),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(image_count * 3),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(image_count)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None) }
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanDescriptorSet,
                    "Clouds: create descriptor pool",
                    e,
                )
            })?;

        let layouts = vec![descriptor_layout; image_count as usize];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info) }.map_err(
            |e| {
                ReactorError::with_source(
                    ErrorCode::VulkanDescriptorSet,
                    "Clouds: allocate descriptor sets",
                    e,
                )
            },
        )?;

        let mut output_images = Vec::with_capacity(image_count as usize);
        for _ in 0..image_count {
            let img = Image::new(
                ctx,
                allocator.clone(),
                width,
                height,
                vk::Format::R16G16B16A16_SFLOAT,
                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
                vk::ImageAspectFlags::COLOR,
                1,
            )?;
            output_images.push(img);
        }

        let shape_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(shape.view)
            .sampler(linear_sampler);
        let detail_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(detail.view)
            .sampler(linear_sampler);

        for i in 0..image_count as usize {
            let set = descriptor_sets[i];

            let output_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(output_images[i].view);

            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .image_info(std::slice::from_ref(&output_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(2)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&shape_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(3)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&detail_info)),
            ];
            unsafe {
                device.update_descriptor_sets(&writes, &[]);
            }
        }

        Ok(Self {
            pipeline: Some(pipeline),
            descriptor_layout,
            descriptor_pool,
            descriptor_sets,
            output_images,
            time: 0.0,
            shape,
            detail,
            noise_sampler,
            device,
        })
    }

    pub fn advance_time(&mut self, dt: f32) {
        self.time += dt;
    }

    pub fn dispatch(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        width: u32,
        height: u32,
        inv_view_proj: glam::Mat4,
        camera_pos: glam::Vec3,
        sun_direction: glam::Vec3,
        sun_color: glam::Vec3,
        depth_view: vk::ImageView,
        sampler: vk::Sampler,
    ) {
        let Some(pipeline) = self.pipeline.as_ref() else {
            return;
        };
        let Some(set) = self.descriptor_sets.get(image_index) else {
            return;
        };
        if image_index >= self.output_images.len() {
            return;
        }

        let depth_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(depth_view)
            .sampler(sampler);
        let depth_write = vk::WriteDescriptorSet::default()
            .dst_set(*set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&depth_info));
        unsafe {
            device.update_descriptor_sets(&[depth_write], &[]);
        }

        let to_general = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(self.output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_general],
            );
        }

        let mut push_bytes = [0u8; 120];
        let mut o = 0usize;
        for col in inv_view_proj.to_cols_array() {
            push_bytes[o..o + 4].copy_from_slice(&col.to_ne_bytes());
            o += 4;
        }
        push_bytes[o..o + 4].copy_from_slice(&camera_pos.x.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&camera_pos.y.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&camera_pos.z.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_direction.x.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_direction.y.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_direction.z.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_color.x.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_color.y.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&sun_color.z.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&1.0f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&2000.0f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&6000.0f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0.5f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&0.6f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&self.time.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&15.0f32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&64u32.to_ne_bytes());
        o += 4;
        push_bytes[o..o + 4].copy_from_slice(&6u32.to_ne_bytes());

        pipeline.bind(command_buffer, device);
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout,
                0,
                &[*set],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                &push_bytes,
            );
            let gx = (width + 7) / 8;
            let gy = (height + 7) / 8;
            device.cmd_dispatch(command_buffer, gx, gy, 1);
        }

        let to_read = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(self.output_images[image_index].handle)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_read],
            );
        }
    }
}

impl Drop for VolumetricClouds {
    fn drop(&mut self) {
        unsafe {
            self.output_images.clear();
            self.device.destroy_image_view(self.shape.view, None);
            self.device.destroy_image(self.shape.image, None);
            self.device.destroy_image_view(self.detail.view, None);
            self.device.destroy_image(self.detail.image, None);
            self.device.destroy_sampler(self.noise_sampler, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
    }
}

fn create_3d_noise_image(
    ctx: &VulkanContext,
    device: &ash::Device,
    allocator: Arc<Mutex<Allocator>>,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    size: u32,
    seed: u32,
) -> ReactorResult<Noise3D> {
    use gpu_allocator::MemoryLocation;
    let data = generate_value_noise_3d(size, seed);
    let extent = vk::Extent3D {
        width: size,
        height: size,
        depth: size,
    };
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_3D)
        .extent(extent)
        .mip_levels(1)
        .array_layers(1)
        .format(vk::Format::R8G8B8A8_UNORM)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1);
    let image = unsafe { device.create_image(&image_info, None) }.map_err(|e| {
        ReactorError::with_source(
            ErrorCode::VulkanImageCreation,
            "Clouds: create 3D noise image",
            e,
        )
    })?;
    let req = unsafe { device.get_image_memory_requirements(image) };
    let allocation = allocator
        .lock()
        .unwrap()
        .allocate(&gpu_allocator::vulkan::AllocationCreateDesc {
            name: "clouds_noise_3d",
            requirements: req,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
        })
        .map_err(|e| {
            ReactorError::with_source(
                ErrorCode::VulkanMemoryAllocation,
                "Clouds: allocate 3D noise memory",
                e,
            )
        })?;
    unsafe {
        device
            .bind_image_memory(image, allocation.memory(), allocation.offset())
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanMemoryAllocation,
                    "Clouds: bind 3D noise memory",
                    e,
                )
            })?
    };

    let staging_size = data.len() as u64;
    let staging_buffer = crate::graphics::Buffer::new(
        ctx,
        allocator.clone(),
        staging_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    staging_buffer.write_slice(&data);

    let cmd_alloc = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let cmd = unsafe { device.allocate_command_buffers(&cmd_alloc) }.map_err(|e| {
        ReactorError::with_source(
            ErrorCode::VulkanCommandBuffer,
            "Clouds: allocate cmd buffer",
            e,
        )
    })?[0];
    let begin = vk::CommandBufferBeginInfo::default()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe {
        device
            .begin_command_buffer(cmd, &begin)
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanCommandBuffer,
                    "Clouds: begin cmd buffer",
                    e,
                )
            })?;
    }

    let to_transfer = vk::ImageMemoryBarrier::default()
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[to_transfer],
        );
    }

    let copy = vk::BufferImageCopy::default()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(extent);
    unsafe {
        device.cmd_copy_buffer_to_image(
            cmd,
            staging_buffer.handle,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[copy],
        );
    }

    let to_shader = vk::ImageMemoryBarrier::default()
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::SHADER_READ)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[to_shader],
        );
    }
    unsafe {
        device
            .end_command_buffer(cmd)
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanCommandBuffer,
                    "Clouds: end cmd buffer",
                    e,
                )
            })?;
    }
    let cbs = [cmd];
    let submit = vk::SubmitInfo::default().command_buffers(&cbs);
    unsafe {
        device
            .queue_submit(queue, &[submit], vk::Fence::null())
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanSynchronization,
                    "Clouds: submit",
                    e,
                )
            })?;
        device
            .queue_wait_idle(queue)
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanSynchronization,
                    "Clouds: wait idle",
                    e,
                )
            })?;
    }
    unsafe {
        device.free_command_buffers(command_pool, &[cmd]);
    }
    let mut staging_buffer = staging_buffer;
    staging_buffer.destroy();

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_3D)
        .format(vk::Format::R8G8B8A8_UNORM)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { device.create_image_view(&view_info, None) }.map_err(|e| {
        ReactorError::with_source(
            ErrorCode::VulkanImageCreation,
            "Clouds: create 3D noise view",
            e,
        )
    })?;

    Ok(Noise3D {
        image,
        view,
        allocation,
    })
}
