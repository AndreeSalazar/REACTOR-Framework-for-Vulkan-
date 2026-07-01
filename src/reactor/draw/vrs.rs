use crate::reactor::Reactor;
use crate::core::VrsRate;
use ash::vk;

impl Reactor {
    pub(crate) fn apply_pixel_intelligent_vrs(
        &mut self,
        command_buffer: vk::CommandBuffer,
        visible_objects: usize,
    ) {
        let desired = self
            .pixel_intelligent
            .desired_rate(self.swapchain.extent, visible_objects);

        let Some(vrs) = self.context.fragment_shading_rate.as_ref() else {
            self.pixel_intelligent.current_rate = VrsRate::NATIVE;
            return;
        };

        let rate = vrs
            .capabilities
            .best_supported_rate(desired, self.msaa_samples);
        self.pixel_intelligent.current_rate = rate;

        unsafe {
            vrs.cmd_set_rate(command_buffer, rate);
        }
    }
}
