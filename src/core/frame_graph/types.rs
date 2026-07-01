use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PassId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceType {
    Texture,
    Buffer,
    DepthBuffer,
    RenderTarget,
    Swapchain,
}

#[derive(Clone, Debug)]
pub struct ResourceDesc {
    pub id: ResourceId,
    pub name: String,
    pub resource_type: ResourceType,
    pub width: u32,
    pub height: u32,
    pub format: ResourceFormat,
    pub persistent: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceFormat {
    RGBA8,
    RGBA16F,
    RGBA32F,
    R8,
    R16F,
    R32F,
    Depth32F,
    Depth24Stencil8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
    ReadWrite,
}

#[derive(Clone, Debug)]
pub struct PassDependency {
    pub resource: ResourceId,
    pub access: AccessType,
}

#[derive(Clone, Debug)]
pub struct PassDesc {
    pub id: PassId,
    pub name: String,
    pub reads: Vec<ResourceId>,
    pub writes: Vec<ResourceId>,
    pub enabled: bool,
    pub order: i32,
}

#[derive(Clone, Debug)]
pub struct Barrier {
    pub resource: ResourceId,
    pub from_pass: Option<PassId>,
    pub to_pass: PassId,
    pub from_access: AccessType,
    pub to_access: AccessType,
}

#[derive(Clone, Debug, Default)]
pub struct FrameGraphStats {
    pub total_passes: u32,
    pub enabled_passes: u32,
    pub total_resources: u32,
    pub transient_resources: u32,
    pub barriers_generated: u32,
}
