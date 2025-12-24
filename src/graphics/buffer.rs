use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use crate::core::context::VulkanContext;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct Buffer {
    pub handle: vk::Buffer,
    pub allocation: Option<Allocation>,
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
            linear: true,
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

    pub fn new_staging(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        size: u64,
    ) -> Result<Self, Box<dyn Error>> {
        Self::new(ctx, allocator, size, vk::BufferUsageFlags::TRANSFER_SRC, MemoryLocation::CpuToGpu)
    }

    pub fn new_vertex(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        size: u64,
    ) -> Result<Self, Box<dyn Error>> {
        Self::new(
            ctx,
            allocator,
            size,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )
    }

    pub fn new_index(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        size: u64,
    ) -> Result<Self, Box<dyn Error>> {
        Self::new(
            ctx,
            allocator,
            size,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )
    }

    pub fn new_uniform(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        size: u64,
    ) -> Result<Self, Box<dyn Error>> {
        Self::new(
            ctx,
            allocator,
            size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
        )
    }

    pub fn new_storage(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        size: u64,
    ) -> Result<Self, Box<dyn Error>> {
        Self::new(
            ctx,
            allocator,
            size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )
    }

    pub fn map<T>(&self) -> Option<*mut T> {
        self.allocation.as_ref()
            .and_then(|a| a.mapped_ptr())
            .map(|p| p.as_ptr() as *mut T)
    }

    pub fn write<T: Copy>(&self, data: &[T]) {
        if let Some(ptr) = self.map::<T>() {
            unsafe {
                ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
            }
        }
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
