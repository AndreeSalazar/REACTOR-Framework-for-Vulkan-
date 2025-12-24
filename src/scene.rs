use std::sync::Arc;
use glam::Mat4;
use crate::mesh::Mesh;
use crate::material::Material;

pub struct SceneObject {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
    pub transform: Mat4,
}

pub struct Scene {
    pub objects: Vec<SceneObject>,
}

impl Scene {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add_object(&mut self, mesh: Arc<Mesh>, material: Arc<Material>, transform: Mat4) {
        self.objects.push(SceneObject { mesh, material, transform });
    }
}
