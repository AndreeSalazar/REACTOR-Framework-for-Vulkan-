use crate::resources::mesh::Mesh;
use crate::resources::material::Material;
use glam::Mat4;
use std::sync::Arc;

pub struct Model {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
    pub transform: Mat4,
}

impl Model {
    pub fn new(mesh: Arc<Mesh>, material: Arc<Material>) -> Self {
        Self {
            mesh,
            material,
            transform: Mat4::IDENTITY,
        }
    }

    pub fn with_transform(mut self, transform: Mat4) -> Self {
        self.transform = transform;
        self
    }

    pub fn translate(&mut self, translation: glam::Vec3) {
        self.transform = Mat4::from_translation(translation) * self.transform;
    }

    pub fn rotate(&mut self, axis: glam::Vec3, angle: f32) {
        self.transform = Mat4::from_axis_angle(axis, angle) * self.transform;
    }

    pub fn scale(&mut self, scale: glam::Vec3) {
        self.transform = Mat4::from_scale(scale) * self.transform;
    }

    pub fn set_position(&mut self, position: glam::Vec3) {
        let (scale, rotation, _) = self.transform.to_scale_rotation_translation();
        self.transform = Mat4::from_scale_rotation_translation(scale, rotation, position);
    }
}

pub struct ModelBatch {
    pub models: Vec<Model>,
}

impl ModelBatch {
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    pub fn add(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn clear(&mut self) {
        self.models.clear();
    }

    pub fn len(&self) -> usize {
        self.models.len()
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}
