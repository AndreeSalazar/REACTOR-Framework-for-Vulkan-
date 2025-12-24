use std::sync::Arc;
use glam::Mat4;
use crate::mesh::Mesh;
use crate::material::Material;

pub struct SceneObject {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
    pub transform: Mat4,
    pub visible: bool,
    pub name: Option<String>,
}

pub struct Scene {
    pub objects: Vec<SceneObject>,
}

impl Scene {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add_object(&mut self, mesh: Arc<Mesh>, material: Arc<Material>, transform: Mat4) -> usize {
        let idx = self.objects.len();
        self.objects.push(SceneObject { 
            mesh, 
            material, 
            transform,
            visible: true,
            name: None,
        });
        idx
    }

    pub fn add_named(&mut self, name: &str, mesh: Arc<Mesh>, material: Arc<Material>, transform: Mat4) -> usize {
        let idx = self.objects.len();
        self.objects.push(SceneObject { 
            mesh, 
            material, 
            transform,
            visible: true,
            name: Some(name.to_string()),
        });
        idx
    }

    pub fn visible_objects(&self) -> impl Iterator<Item = &SceneObject> {
        self.objects.iter().filter(|o| o.visible)
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }
}
