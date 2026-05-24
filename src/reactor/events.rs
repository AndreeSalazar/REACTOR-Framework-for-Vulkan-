//! Eventos de ventana + queries sobre el dispositivo.

use super::Reactor;
use ash::vk;
use winit::event::WindowEvent;

impl Reactor {
    /// Propaga eventos de la ventana al subsistema de input y marca resize
    /// cuando corresponde.
    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.input.process_event(event);
        if let WindowEvent::Resized(_) = event {
            self.resized = true;
        }
    }

    /// Devuelve el máximo MSAA soportado por la GPU (color ∩ depth), preferencia
    /// 8x → 4x → 2x → 1x.
    pub fn get_max_msaa_samples(&self) -> vk::SampleCountFlags {
        let props = unsafe {
            self.context
                .instance
                .get_physical_device_properties(self.context.physical_device)
        };
        let counts = props.limits.framebuffer_color_sample_counts
            & props.limits.framebuffer_depth_sample_counts;

        if counts.contains(vk::SampleCountFlags::TYPE_8) {
            vk::SampleCountFlags::TYPE_8
        } else if counts.contains(vk::SampleCountFlags::TYPE_4) {
            vk::SampleCountFlags::TYPE_4
        } else if counts.contains(vk::SampleCountFlags::TYPE_2) {
            vk::SampleCountFlags::TYPE_2
        } else {
            vk::SampleCountFlags::TYPE_1
        }
    }
}
