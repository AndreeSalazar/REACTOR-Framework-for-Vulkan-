use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use crate::vulkan_context::VulkanContext;
use std::error::Error;

pub struct Buffer {
    pub handle: vk::Buffer,
    pub allocation: Allocation,
    pub size: u64,
}

impl Buffer {
    pub fn new(
        ctx: &VulkanContext,
        allocator: &mut Allocator,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
    ) -> Result<Self, Box<dyn Error>> {
        let create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let handle = unsafe { ctx.device.create_buffer(&create_info, None)? };

        let requirements = unsafe { ctx.device.get_buffer_memory_requirements(handle) };

        let allocation = allocator.allocate(&AllocationCreateDesc {
            name: "buffer",
            requirements,
            location,
            linear: true, // Buffers are linear
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            ctx.device.bind_buffer_memory(handle, allocation.memory(), allocation.offset())?;
        }

        Ok(Self {
            handle,
            allocation,
            size,
        })
    }

    pub fn destroy(&mut self, ctx: &VulkanContext, allocator: &mut Allocator) {
        if self.handle != vk::Buffer::null() {
            unsafe {
                ctx.device.destroy_buffer(self.handle, None);
            }
            if let Err(e) = allocator.free(std::mem::take(&mut self.allocation)) {
                eprintln!("Failed to free buffer memory: {:?}", e);
            }
            self.handle = vk::Buffer::null();
        }
    }
}

