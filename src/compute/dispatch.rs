use ash::vk;
use crate::compute::pipeline::ComputePipeline;

pub struct ComputeDispatch;

impl ComputeDispatch {
    pub fn dispatch(
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        pipeline: &ComputePipeline,
        descriptor_sets: &[vk::DescriptorSet],
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unsafe {
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.pipeline);
            
            if !descriptor_sets.is_empty() {
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    pipeline.layout,
                    0,
                    descriptor_sets,
                    &[],
                );
            }

            device.cmd_dispatch(command_buffer, group_count_x, group_count_y, group_count_z);
        }
    }

    pub fn dispatch_indirect(
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        pipeline: &ComputePipeline,
        descriptor_sets: &[vk::DescriptorSet],
        indirect_buffer: vk::Buffer,
        offset: u64,
    ) {
        unsafe {
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.pipeline);
            
            if !descriptor_sets.is_empty() {
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    pipeline.layout,
                    0,
                    descriptor_sets,
                    &[],
                );
            }

            device.cmd_dispatch_indirect(command_buffer, indirect_buffer, offset);
        }
    }

    pub fn memory_barrier(device: &ash::Device, command_buffer: vk::CommandBuffer) {
        let barrier = vk::MemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ);

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[barrier],
                &[],
                &[],
            );
        }
    }

    pub fn buffer_barrier(
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        size: u64,
    ) {
        let barrier = vk::BufferMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .buffer(buffer)
            .offset(0)
            .size(size);

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[barrier],
                &[],
            );
        }
    }
}
