use crate::core::error::ReactorResult;
use crate::resources::gltf_loader::types::*;

impl GltfModel {
    pub fn upload_first_mesh(
        &self,
        ctx: &crate::core::VulkanContext,
        allocator: &std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>,
    ) -> ReactorResult<crate::resources::mesh::Mesh> {
        let mesh_data = self
            .meshes
            .first()
            .ok_or_else(|| crate::core::error::ReactorError::asset_load("Model has no meshes"))?;

        crate::resources::mesh::Mesh::new(ctx, allocator, &mesh_data.vertices, &mesh_data.indices)
    }

    pub fn upload_all_meshes(
        &self,
        ctx: &crate::core::VulkanContext,
        allocator: &std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>,
    ) -> ReactorResult<Vec<crate::resources::mesh::Mesh>> {
        self.meshes
            .iter()
            .map(|mesh_data| {
                crate::resources::mesh::Mesh::new(
                    ctx,
                    allocator,
                    &mesh_data.vertices,
                    &mesh_data.indices,
                )
            })
            .collect()
    }

    pub fn upload_first_texture(
        &self,
        ctx: &crate::core::VulkanContext,
        allocator: std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>,
        generate_mipmaps: bool,
    ) -> ReactorResult<Option<crate::resources::texture::Texture>> {
        if let Some(tex_data) = self.textures.first() {
            let texture = crate::resources::texture::Texture::from_rgba(
                ctx,
                allocator,
                &tex_data.pixels,
                tex_data.width,
                tex_data.height,
                generate_mipmaps,
            )?;
            Ok(Some(texture))
        } else {
            Ok(None)
        }
    }

    pub fn bounds(&self) -> Option<(glam::Vec3, glam::Vec3)> {
        if self.meshes.is_empty() {
            return None;
        }

        let mut min = glam::Vec3::splat(f32::INFINITY);
        let mut max = glam::Vec3::splat(f32::NEG_INFINITY);
        let mut found = false;

        fn walk(
            node: &GltfNode,
            model: &GltfModel,
            parent: glam::Mat4,
            min: &mut glam::Vec3,
            max: &mut glam::Vec3,
            found: &mut bool,
        ) {
            let world = parent * node.transform;
            if let Some(mesh_idx) = node.mesh_index {
                if let Some(mesh) = model.meshes.get(mesh_idx) {
                    for v in &mesh.vertices {
                        let p = world.transform_point3(glam::Vec3::from(v.position));
                        *min = min.min(p);
                        *max = max.max(p);
                        *found = true;
                    }
                }
            }
            for child in &node.children {
                walk(child, model, world, min, max, found);
            }
        }

        walk(
            &self.root_node,
            self,
            glam::Mat4::IDENTITY,
            &mut min,
            &mut max,
            &mut found,
        );

        if found {
            Some((min, max))
        } else {
            None
        }
    }

    pub fn height(&self) -> f32 {
        self.bounds().map(|(mn, mx)| mx.y - mn.y).unwrap_or(0.0)
    }

    pub fn center(&self) -> glam::Vec3 {
        self.bounds()
            .map(|(mn, mx)| (mn + mx) * 0.5)
            .unwrap_or(glam::Vec3::ZERO)
    }
}
