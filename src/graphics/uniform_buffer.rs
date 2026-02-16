use ash::vk;
use gpu_allocator::vulkan::Allocator;
use gpu_allocator::MemoryLocation;
use crate::vulkan_context::VulkanContext;
use crate::graphics::buffer::Buffer;
use std::error::Error;
use std::sync::{Arc, Mutex};
use bytemuck::{Pod, Zeroable};

/// Global scene data passed to shaders
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlobalUniformData {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_projection: [[f32; 4]; 4],
    pub camera_position: [f32; 4],
    pub time: f32,
    pub delta_time: f32,
    pub screen_width: f32,
    pub screen_height: f32,
}

impl Default for GlobalUniformData {
    fn default() -> Self {
        Self {
            view: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            projection: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            view_projection: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            camera_position: [0.0, 0.0, 0.0, 1.0],
            time: 0.0,
            delta_time: 0.0,
            screen_width: 1920.0,
            screen_height: 1080.0,
        }
    }
}

impl GlobalUniformData {
    pub fn from_matrices(view: glam::Mat4, projection: glam::Mat4, camera_pos: glam::Vec3) -> Self {
        let vp = projection * view;
        Self {
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            view_projection: vp.to_cols_array_2d(),
            camera_position: [camera_pos.x, camera_pos.y, camera_pos.z, 1.0],
            ..Default::default()
        }
    }

    pub fn set_time(&mut self, time: f32, delta: f32) {
        self.time = time;
        self.delta_time = delta;
    }

    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }
}

/// Light data for shaders
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct LightData {
    pub position: [f32; 4],      // w = range for point/spot
    pub direction: [f32; 4],     // w = spot angle
    pub color: [f32; 4],         // w = intensity
    pub light_type: u32,         // 0 = directional, 1 = point, 2 = spot
    pub cast_shadows: u32,
    pub _padding: [u32; 2],
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            position: [0.0, 10.0, 0.0, 50.0],
            direction: [0.0, -1.0, 0.0, 45.0],
            color: [1.0, 1.0, 1.0, 1.0],
            light_type: 0,
            cast_shadows: 0,
            _padding: [0, 0],
        }
    }
}

pub const MAX_LIGHTS: usize = 16;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct LightUniformData {
    pub ambient_color: [f32; 4],
    pub light_count: u32,
    pub _padding: [u32; 3],
    pub lights: [LightData; MAX_LIGHTS],
}

impl Default for LightUniformData {
    fn default() -> Self {
        Self {
            ambient_color: [0.1, 0.1, 0.1, 1.0],
            light_count: 0,
            _padding: [0, 0, 0],
            lights: [LightData::default(); MAX_LIGHTS],
        }
    }
}

/// Material properties for PBR
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MaterialUniformData {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
    pub emissive_strength: f32,
    pub emissive_color: [f32; 4],
    pub use_textures: u32,  // Bitflags: 1=albedo, 2=normal, 4=metallic, 8=roughness
    pub _padding: [u32; 3],
}

impl Default for MaterialUniformData {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            ao: 1.0,
            emissive_strength: 0.0,
            emissive_color: [0.0, 0.0, 0.0, 1.0],
            use_textures: 0,
            _padding: [0, 0, 0],
        }
    }
}

/// Uniform buffer wrapper with double/triple buffering support
pub struct UniformBuffer<T: Pod + Zeroable> {
    buffers: Vec<Buffer>,
    current_frame: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Pod + Zeroable> UniformBuffer<T> {
    pub fn new(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        frame_count: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let size = std::mem::size_of::<T>() as u64;
        let mut buffers = Vec::with_capacity(frame_count);

        for _ in 0..frame_count {
            let buffer = Buffer::new(
                ctx,
                allocator.clone(),
                size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                MemoryLocation::CpuToGpu,
            )?;
            buffers.push(buffer);
        }

        Ok(Self {
            buffers,
            current_frame: 0,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn update(&mut self, data: &T) {
        if let Some(ptr) = self.buffers[self.current_frame].map::<T>() {
            unsafe {
                std::ptr::copy_nonoverlapping(data, ptr, 1);
            }
        }
    }

    pub fn advance_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.buffers.len();
    }

    pub fn current_buffer(&self) -> &Buffer {
        &self.buffers[self.current_frame]
    }

    pub fn buffer_handle(&self) -> vk::Buffer {
        self.buffers[self.current_frame].handle
    }

    pub fn size(&self) -> u64 {
        std::mem::size_of::<T>() as u64
    }
}
