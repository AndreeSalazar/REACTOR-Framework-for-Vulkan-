use std::sync::Arc;
use glam::Mat4;
use crate::resources::mesh::Mesh;
use crate::resources::material::Material;

pub struct SceneObject {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
    pub transform: Mat4,
    pub visible: bool,
    pub name: Option<String>,
}

impl SceneObject {
    pub fn new(mesh: Arc<Mesh>, material: Arc<Material>, transform: Mat4) -> Self {
        Self {
            mesh,
            material,
            transform,
            visible: true,
            name: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub ambient_light: glam::Vec3,
    pub sun_direction: glam::Vec3,
    pub sun_color: glam::Vec3,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            ambient_light: glam::Vec3::splat(0.1),
            sun_direction: glam::Vec3::new(-0.5, -1.0, -0.5).normalize(),
            sun_color: glam::Vec3::ONE,
        }
    }

    pub fn add_object(&mut self, mesh: Arc<Mesh>, material: Arc<Material>, transform: Mat4) -> usize {
        let index = self.objects.len();
        self.objects.push(SceneObject::new(mesh, material, transform));
        index
    }

    pub fn add(&mut self, object: SceneObject) -> usize {
        let index = self.objects.len();
        self.objects.push(object);
        index
    }

    pub fn get(&self, index: usize) -> Option<&SceneObject> {
        self.objects.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut SceneObject> {
        self.objects.get_mut(index)
    }

    pub fn find_by_name(&self, name: &str) -> Option<usize> {
        self.objects.iter().position(|o| o.name.as_deref() == Some(name))
    }

    pub fn remove(&mut self, index: usize) -> Option<SceneObject> {
        if index < self.objects.len() {
            Some(self.objects.remove(index))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn visible_objects(&self) -> impl Iterator<Item = &SceneObject> {
        self.objects.iter().filter(|o| o.visible)
    }

    pub fn set_sun(&mut self, direction: glam::Vec3, color: glam::Vec3) {
        self.sun_direction = direction.normalize();
        self.sun_color = color;
    }

    pub fn set_ambient(&mut self, color: glam::Vec3) {
        self.ambient_light = color;
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
