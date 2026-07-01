use crate::reactor::Reactor;
use ash::vk;

impl Reactor {
    pub unsafe fn bind_reactor_system_descriptors(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        bind_ibl: bool,
        has_shadow_set: bool,
    ) {
        if bind_ibl {
            if let Some(ref ibl) = self.ibl_textures {
                self.context.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    1,
                    &[ibl.descriptor_set],
                    &[],
                );
            }
        }

        if has_shadow_set && !self.shadow_descriptor_sets.is_empty() {
            self.context.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                2,
                &[self.shadow_descriptor_sets[self.current_frame]],
                &[],
            );
        }
    }
}
