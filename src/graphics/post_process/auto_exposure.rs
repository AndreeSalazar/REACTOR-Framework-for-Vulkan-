use ash::vk;
use super::PostProcessPipeline;

impl PostProcessPipeline {
    pub fn dispatch_auto_exposure(&self, device: &ash::Device, command_buffer: vk::CommandBuffer, image_index: usize, dt: f32) {
        use super::types::AutoExposureParams;
        let Some(pipeline) = self.auto_exposure_pipeline.as_ref() else { return; };

        pipeline.bind(command_buffer, device);
        unsafe {
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.layout, 0, &[self.descriptor_sets[image_index]], &[]);
        }

        let params = AutoExposureParams {
            dt, speed: 1.5, target_luminance: 0.18, max_exposure: 3.5, min_exposure: 0.2,
        };
        let push_bytes = bytemuck::bytes_of(&params);
        unsafe {
            device.cmd_push_constants(command_buffer, pipeline.layout, vk::ShaderStageFlags::COMPUTE, 0, push_bytes);
            device.cmd_dispatch(command_buffer, 1, 1, 1);
        }

        let buffer_barrier = vk::BufferMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(self.exposure_buffers[image_index].handle).offset(0).size(4);
        unsafe {
            device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[buffer_barrier], &[]);
        }
    }
}
