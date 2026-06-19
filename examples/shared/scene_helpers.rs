//! Scene-construction helpers.

use reactor_vulkan::app::ReactorContext;
use reactor_vulkan::{Material, Mesh, SceneObject, Texture, Vertex};
use std::sync::Arc;

#[derive(Debug)]
pub enum SceneAddError {
    #[allow(dead_code)]
    Mesh(String),
    #[allow(dead_code)]
    Material(String),
}

#[allow(dead_code)]
pub fn add_object(
    ctx: &mut ReactorContext,
    vertices: &[Vertex],
    indices: &[u32],
    vert_code: &[u32],
    frag_code: &[u32],
    transform: glam::Mat4,
) -> Result<usize, SceneAddError> {
    let mesh: Arc<Mesh> = Arc::new(
        ctx.create_mesh(vertices, indices)
            .map_err(|e| SceneAddError::Mesh(e.to_string()))?,
    );
    let material: Arc<Material> = Arc::new(
        ctx.create_material(vert_code, frag_code)
            .map_err(|e| SceneAddError::Material(e.to_string()))?,
    );
    let object = SceneObject {
        mesh,
        material,
        transform,
        visible: true,
        name: None,
        color: glam::Vec4::ONE,
        metallic: 0.0,
        roughness: 0.5,
        emission: glam::Vec4::ZERO,
        anisotropy: 0.0,
    };
    Ok(ctx.scene.add(object))
}

#[allow(dead_code)]
pub fn add_object_with_texture(
    ctx: &mut ReactorContext,
    vertices: &[Vertex],
    indices: &[u32],
    vert_code: &[u32],
    frag_code: &[u32],
    texture: &Texture,
    transform: glam::Mat4,
) -> Result<usize, SceneAddError> {
    let mesh: Arc<Mesh> = Arc::new(
        ctx.create_mesh(vertices, indices)
            .map_err(|e| SceneAddError::Mesh(e.to_string()))?,
    );
    let material: Arc<Material> = Arc::new(
        ctx.create_textured_material(vert_code, frag_code, texture)
            .map_err(|e| SceneAddError::Material(e.to_string()))?,
    );
    let object = SceneObject {
        mesh,
        material,
        transform,
        visible: true,
        name: None,
        color: glam::Vec4::ONE,
        metallic: 0.0,
        roughness: 0.5,
        emission: glam::Vec4::ZERO,
        anisotropy: 0.0,
    };
    Ok(ctx.scene.add(object))
}
