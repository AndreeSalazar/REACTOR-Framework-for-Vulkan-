use ash::vk;
use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use std::mem;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn new(position: Vec3, color: Vec3, uv: Vec2) -> Self {
        Self {
            position: position.to_array(),
            color: color.to_array(),
            uv: uv.to_array(),
        }
    }

    pub fn with_normal(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position: position.to_array(),
            color: normal.to_array(), // Using color slot for normal temporarily
            uv: uv.to_array(),
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            // Position
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            // Color/Normal
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(mem::size_of::<[f32; 3]>() as u32),
            // UV
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset((mem::size_of::<[f32; 3]>() * 2) as u32),
        ]
    }
}

// Extended vertex with tangent for normal mapping
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct VertexPBR {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

impl VertexPBR {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2, tangent: glam::Vec4) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
            uv: uv.to_array(),
            tangent: tangent.to_array(),
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(mem::size_of::<[f32; 3]>() as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset((mem::size_of::<[f32; 3]>() * 2) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset((mem::size_of::<[f32; 3]>() * 2 + mem::size_of::<[f32; 2]>()) as u32),
        ]
    }
}

// Instance data for instanced rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
}

impl InstanceData {
    pub fn new(transform: glam::Mat4) -> Self {
        Self {
            model_matrix: transform.to_cols_array_2d(),
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(1)
            .stride(mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::INSTANCE)
    }

    pub fn attribute_descriptions(start_location: u32) -> [vk::VertexInputAttributeDescription; 4] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(1)
                .location(start_location)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(1)
                .location(start_location + 1)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(mem::size_of::<[f32; 4]>() as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(1)
                .location(start_location + 2)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset((mem::size_of::<[f32; 4]>() * 2) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(1)
                .location(start_location + 3)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset((mem::size_of::<[f32; 4]>() * 3) as u32),
        ]
    }
}
