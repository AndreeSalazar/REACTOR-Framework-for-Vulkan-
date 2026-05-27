//! Indirect Draw System - GPU-Driven Rendering
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc};
use gpu_allocator::MemoryLocation;
use std::sync::Arc;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DrawIndexedIndirectCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub first_instance: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IndirectCommandWithMaterial {
    pub cmd: DrawIndexedIndirectCommand,
    pub material_index: u32,
    pub transform_index: u32,
}

pub struct IndirectDrawBuffer {
    device: Arc<ash::Device>,
    buffer: vk::Buffer,
    allocation: Allocation,
    capacity: u32,
    count: u32,
}

impl IndirectDrawBuffer {
    pub fn new(
        device: Arc<ash::Device>,
        allocator: &mut gpu_allocator::vulkan::Allocator,
        max_commands: u32,
    ) -> ReactorResult<Self> {
        let stride = std::mem::size_of::<IndirectCommandWithMaterial>() as vk::DeviceSize;
        let size = stride * max_commands as vk::DeviceSize;
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(
                vk::BufferUsageFlags::INDIRECT_BUFFER
                    | vk::BufferUsageFlags::STORAGE_BUFFER
                    | vk::BufferUsageFlags::TRANSFER_DST,
            )
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: "indirect_draw_buffer",
                requirements,
                location: MemoryLocation::CpuToGpu,
                linear: true,
                allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
            })
            .map_err(|e| {
                ReactorError::with_source(ErrorCode::OutOfMemory, "Indirect buffer alloc failed", e)
            })?;
        unsafe {
            device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;
        }
        Ok(Self {
            device,
            buffer,
            allocation,
            capacity: max_commands,
            count: 0,
        })
    }

    pub fn push(&mut self, cmd: IndirectCommandWithMaterial) -> ReactorResult<()> {
        if self.count >= self.capacity {
            return Err(ReactorError::new(
                ErrorCode::ResourceLimit,
                "Indirect buffer full",
            ));
        }
        let stride = std::mem::size_of::<IndirectCommandWithMaterial>();
        let offset = (self.count as usize) * stride;
        if let Some(mapped) = self.allocation.mapped_ptr() {
            unsafe {
                let ptr = mapped.as_ptr() as *mut u8;
                std::ptr::copy_nonoverlapping(
                    &cmd as *const _ as *const u8,
                    ptr.add(offset),
                    stride,
                );
            }
        }
        self.count += 1;
        Ok(())
    }

    pub fn record_draw(&self, cmd: vk::CommandBuffer) {
        let stride = std::mem::size_of::<IndirectCommandWithMaterial>() as u32;
        unsafe {
            self.device
                .cmd_draw_indexed_indirect(cmd, self.buffer, 0, self.count, stride);
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }
    pub fn count(&self) -> u32 {
        self.count
    }
    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }
}

impl Drop for IndirectDrawBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
        }
    }
}
