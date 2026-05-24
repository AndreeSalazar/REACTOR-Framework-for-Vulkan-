//! Fábricas de recursos: meshes, texturas y materiales.
//!
//! Estas funciones son la "API corta" del runtime para crear assets vinculados
//! al `VulkanContext` y al allocator que el `Reactor` ya tiene listos.

use super::Reactor;
use crate::core::error::ReactorResult;
use crate::resources::material::Material;
use crate::resources::mesh::Mesh;
use crate::resources::texture::Texture;
use crate::resources::vertex::Vertex;

impl Reactor {
    /// Crea un mesh GPU a partir de vértices e índices.
    pub fn create_mesh(&self, vertices: &[Vertex], indices: &[u32]) -> ReactorResult<Mesh> {
        Mesh::new(&self.context, &self.allocator, vertices, indices)
    }

    /// Carga una textura desde fichero (PNG/JPG/BMP/HDR/…).
    pub fn load_texture(&self, path: &str) -> ReactorResult<Texture> {
        Texture::from_file(&self.context, self.allocator.clone(), path, true)
    }

    /// Carga una textura desde bytes embebidos.
    pub fn load_texture_bytes(&self, bytes: &[u8]) -> ReactorResult<Texture> {
        Texture::from_bytes(&self.context, self.allocator.clone(), bytes, true)
    }

    /// Crea una textura sólida 1×1 del color RGBA dado.
    pub fn create_solid_texture(
        &self,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> ReactorResult<Texture> {
        Texture::solid_color(&self.context, self.allocator.clone(), r, g, b, a)
    }

    /// Crea un material sin texturas usando *Dynamic Rendering*.
    pub fn create_material(
        &self,
        vert_code: &[u32],
        frag_code: &[u32],
    ) -> ReactorResult<Material> {
        Material::new_with_msaa(
            &self.context,
            None, // Dynamic Rendering
            vert_code,
            frag_code,
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            self.msaa_samples,
            self.swapchain.format,
            Some(self.depth_format),
        )
    }

    /// Crea un material con textura difusa usando *Dynamic Rendering*.
    pub fn create_textured_material(
        &self,
        vert_code: &[u32],
        frag_code: &[u32],
        texture: &Texture,
    ) -> ReactorResult<Material> {
        Material::with_texture(
            &self.context,
            None, // Dynamic Rendering
            vert_code,
            frag_code,
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            texture,
            self.msaa_samples,
            self.swapchain.format,
            Some(self.depth_format),
        )
    }
}
