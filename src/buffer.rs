use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use crate::vulkan_context::VulkanContext;
use std::error::Error;

use std::sync::{Arc, Mutex};

pub struct Buffer {
    pub handle: vk::Buffer,
    pub allocation: Option<Allocation>, // Option to allow taking in Drop
    pub size: u64,
    device: ash::Device,
    allocator: Arc<Mutex<Allocator>>,
}

impl Buffer {
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
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

        let allocation = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
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
            allocation: Some(allocation),
            size,
            device: ctx.device.clone(),
            allocator,
        })
    }

    pub fn destroy(&mut self) {
        if self.handle != vk::Buffer::null() {
            unsafe {
                self.device.destroy_buffer(self.handle, None);
            }
            if let Some(allocation) = self.allocation.take() {
                if let Err(e) = self.allocator.lock().unwrap().free(allocation) {
                    eprintln!("Failed to free buffer memory: {:?}", e);
                }
            }
            self.handle = vk::Buffer::null();
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.destroy();
    }
}

