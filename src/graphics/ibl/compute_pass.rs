use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use crate::graphics::ibl::verr;
use ash::util::read_spv;
use ash::vk;
use std::io::Cursor;

pub(crate) struct ComputePass {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    pub(crate) layout_set: vk::DescriptorSetLayout,
    device: ash::Device,
}

impl ComputePass {
    pub fn new(
        ctx: &VulkanContext, spv: &[u8],
        bindings: &[vk::DescriptorSetLayoutBinding], push_size: u32,
    ) -> ReactorResult<Self> {
        let device = ctx.ash_device();
        let code = read_spv(&mut Cursor::new(spv))
            .map_err(|e| ReactorError::with_source(ErrorCode::VulkanImageCreation, "spv inválido", e))?;
        let sm = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&code), None).map_err(verr)? };
        let layout_set = unsafe {
            device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(bindings)
                    .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL),
                None,
            ).map_err(verr)?
        };
        let layouts = [layout_set];
        let push_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::COMPUTE, offset: 0, size: push_size,
        }];
        let layout = unsafe {
            device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts).push_constant_ranges(&push_ranges),
                None,
            ).map_err(verr)?
        };
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE).module(sm)
            .name(unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0") });
        let pipelines = unsafe {
            device.create_compute_pipelines(
                vk::PipelineCache::null(), &[vk::ComputePipelineCreateInfo::default().stage(stage).layout(layout)], None,
            ).map_err(|(_, e)| ReactorError::from(e))?
        };
        unsafe { device.destroy_shader_module(sm, None); }
        Ok(Self { pipeline: pipelines[0], layout, layout_set, device: device.clone() })
    }

    pub fn dispatch<T: Copy>(
        &self, ctx: &VulkanContext, cmd: vk::CommandBuffer,
        sets: &[vk::DescriptorSet], pc: &T, gx: u32, gy: u32, gz: u32,
    ) {
        let device = ctx.ash_device();
        unsafe {
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.pipeline);
            device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE, self.layout, 0, sets, &[]);
            let bytes = std::slice::from_raw_parts(pc as *const T as *const u8, std::mem::size_of::<T>());
            device.cmd_push_constants(cmd, self.layout, vk::ShaderStageFlags::COMPUTE, 0, bytes);
            device.cmd_dispatch(cmd, gx, gy, gz);
            let mb = vk::MemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ);
            device.cmd_pipeline_barrier(
                cmd, vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[mb], &[], &[]
            );
        }
    }
}

impl Drop for ComputePass {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_descriptor_set_layout(self.layout_set, None);
        }
    }
}

// ── Push-constant structs ────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct EquirectPC {
    pub(crate) face_size: i32,
    pub(crate) num_faces: i32,
    pub(crate) _pad: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct IrradiancePC {
    pub(crate) face_size: i32,
    pub(crate) num_faces: i32,
    pub(crate) _pad: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct PrefilterPC {
    pub(crate) mip_size: i32,
    pub(crate) num_faces: i32,
    pub(crate) roughness: f32,
    pub(crate) src_face_size: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BrdfLutPC {
    pub(crate) size: i32,
    pub(crate) _pad: i32,
    pub(crate) _pad2: [f32; 2],
}
