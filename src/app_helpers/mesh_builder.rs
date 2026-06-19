//! Fluent builder for "create mesh + material + add to scene" patterns.

use crate::app::ReactorContext;
use crate::{Material, SceneObject, Vertex};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("mesh creation failed: {0}")]
    Mesh(String),
    #[error("material creation failed: {0}")]
    Material(String),
}

pub struct MeshBuilder<'a> {
    ctx: &'a mut ReactorContext,
    vertices: Option<&'a [Vertex]>,
    indices: Option<&'a [u32]>,
    vert_code: Option<Vec<u32>>,
    frag_code: Option<Vec<u32>>,
    texture: Option<&'a crate::Texture>,
    transform: glam::Mat4,
    name: Option<String>,
}

impl<'a> MeshBuilder<'a> {
    pub fn new(ctx: &'a mut ReactorContext) -> Self {
        Self {
            ctx,
            vertices: None,
            indices: None,
            vert_code: None,
            frag_code: None,
            texture: None,
            transform: glam::Mat4::IDENTITY,
            name: None,
        }
    }

    pub fn vertices(mut self, v: &'a [Vertex]) -> Self {
        self.vertices = Some(v);
        self
    }

    pub fn indices(mut self, i: &'a [u32]) -> Self {
        self.indices = Some(i);
        self
    }

    pub fn shader(mut self, vert: &'a [u32], frag: &'a [u32]) -> Self {
        self.vert_code = Some(vert.to_vec());
        self.frag_code = Some(frag.to_vec());
        self
    }

    pub fn transform(mut self, t: glam::Mat4) -> Self {
        self.transform = t;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn use_cookbook_forward_material(mut self) -> Self {
        let cookbook = self.ctx.base_shader_cookbook();
        self.vert_code = Some(cookbook.forward.vertex.clone());
        self.frag_code = Some(cookbook.forward.fragment.clone());
        self
    }

    pub fn use_cookbook_textured_material(mut self) -> Self {
        let cookbook = self.ctx.base_shader_cookbook();
        self.vert_code = Some(cookbook.textured.vertex.clone());
        self.frag_code = Some(cookbook.textured.fragment.clone());
        self
    }

    pub fn with_texture(mut self, tex: &'a crate::Texture) -> Self {
        self.texture = Some(tex);
        self
    }

    pub fn spawn(self) -> Result<usize, SpawnError> {
        let vertices = self
            .vertices
            .ok_or_else(|| SpawnError::Mesh("vertices not set".into()))?;
        let indices = self
            .indices
            .ok_or_else(|| SpawnError::Mesh("indices not set".into()))?;

        let mesh = Arc::new(
            self.ctx
                .create_mesh(vertices, indices)
                .map_err(|e| SpawnError::Mesh(e.to_string()))?,
        );

        let vert = self
            .vert_code
            .as_ref()
            .ok_or_else(|| SpawnError::Material("shader.vert not set".into()))?;
        let frag = self
            .frag_code
            .as_ref()
            .ok_or_else(|| SpawnError::Material("shader.frag not set".into()))?;

        let material: Arc<Material> = if let Some(tex) = self.texture {
            Arc::new(
                self.ctx
                    .create_textured_material(vert, frag, tex)
                    .map_err(|e| SpawnError::Material(e.to_string()))?,
            )
        } else {
            Arc::new(
                self.ctx
                    .create_material(vert, frag)
                    .map_err(|e| SpawnError::Material(e.to_string()))?,
            )
        };

        let object = SceneObject {
            mesh,
            material,
            transform: self.transform,
            visible: true,
            name: self.name,
            color: glam::Vec4::ONE,
            metallic: 0.0,
            roughness: 0.5,
            emission: glam::Vec4::ZERO,
            anisotropy: 0.0,
        };
        Ok(self.ctx.scene.add(object))
    }
}
