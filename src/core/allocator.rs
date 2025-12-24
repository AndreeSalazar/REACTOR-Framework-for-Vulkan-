use gpu_allocator::vulkan::*;
use std::sync::{Arc, Mutex};
use std::error::Error;

use crate::core::context::VulkanContext;

pub struct MemoryAllocator {
    pub allocator: Arc<Mutex<Allocator>>,
}

impl MemoryAllocator {
    pub fn new(ctx: &VulkanContext) -> Result<Self, Box<dyn Error>> {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: ctx.instance.clone(),
            device: ctx.device.clone(),
            physical_device: ctx.physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })?;

        Ok(Self {
            allocator: Arc::new(Mutex::new(allocator)),
        })
    }

    pub fn get(&self) -> Arc<Mutex<Allocator>> {
        self.allocator.clone()
    }
}
