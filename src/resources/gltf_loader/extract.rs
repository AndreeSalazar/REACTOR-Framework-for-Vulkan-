use gltf::buffer::Data as GltfBufferData;
use gltf::image::Data as GltfImageData;
use glam::{Mat4, Vec2, Vec3};
use crate::core::error::{ReactorError, ReactorResult};
use crate::resources::gltf_loader::types::*;
use crate::resources::vertex::Vertex;

pub(super) fn extract_texture(image: &GltfImageData, name: &str) -> ReactorResult<GltfTextureData> {
    let pixels = match image.format {
        gltf::image::Format::R8G8B8A8 => {
            image.pixels.clone()
        }
        gltf::image::Format::R8G8B8 => {
            let mut rgba = Vec::with_capacity(image.pixels.len() / 3 * 4);
            for chunk in image.pixels.chunks(3) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(chunk[2]);
                rgba.push(255);
            }
            rgba
        }
        gltf::image::Format::R8 => {
            let mut rgba = Vec::with_capacity(image.pixels.len() * 4);
            for &p in &image.pixels {
                rgba.push(p);
                rgba.push(p);
                rgba.push(p);
                rgba.push(255);
            }
            rgba
        }
        gltf::image::Format::R8G8 => {
            let mut rgba = Vec::with_capacity(image.pixels.len() / 2 * 4);
            for chunk in image.pixels.chunks(2) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(0);
                rgba.push(255);
            }
            rgba
        }
        _ => {
            return Err(ReactorError::asset_load(format!(
                "Unsupported texture format {:?} in {}",
                image.format, name
            )));
        }
    };

    Ok(GltfTextureData {
        pixels,
        width: image.width,
        height: image.height,
        name: name.to_string(),
    })
}

pub(super) fn extract_material(mat: &gltf::Material) -> GltfMaterialData {
    let pbr = mat.pbr_metallic_roughness();
    let base_color = pbr.base_color_factor();
    let emissive = mat.emissive_factor();

    let alpha_mode = match mat.alpha_mode() {
        gltf::material::AlphaMode::Opaque => GltfAlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => GltfAlphaMode::Mask {
            cutoff: mat.alpha_cutoff().unwrap_or(0.5),
        },
        gltf::material::AlphaMode::Blend => GltfAlphaMode::Blend,
    };

    GltfMaterialData {
        base_color,
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        base_color_texture_index: pbr.base_color_texture().map(|t| t.texture().index()),
        normal_texture_index: mat.normal_texture().map(|t| t.texture().index()),
        metallic_roughness_texture_index: pbr
            .metallic_roughness_texture()
            .map(|t| t.texture().index()),
        occlusion_texture_index: mat.occlusion_texture().map(|t| t.texture().index()),
        emissive_texture_index: mat.emissive_texture().map(|t| t.texture().index()),
        emissive_factor: emissive,
        alpha_mode,
        double_sided: mat.double_sided(),
        name: mat.name().unwrap_or("unnamed").to_string(),
    }
}

pub(super) fn extract_mesh(
    mesh: &gltf::Mesh,
    buffers: &[GltfBufferData],
) -> ReactorResult<GltfMeshData> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut material_index = None;

    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let positions: Vec<[f32; 3]> = reader
            .read_positions()
            .ok_or_else(|| ReactorError::asset_load("Mesh missing positions"))?
            .collect();

        let normals: Vec<[f32; 3]> = reader
            .read_normals()
            .map(|n| n.collect())
            .unwrap_or_else(|| vec![[0.0, 0.0, 1.0]; positions.len()]);

        let uvs: Vec<[f32; 2]> = reader
            .read_tex_coords(0)
            .map(|uv| uv.into_f32().collect())
            .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

        let base_vertex = vertices.len() as u32;
        for i in 0..positions.len() {
            vertices.push(Vertex::with_normal(
                Vec3::from(positions[i]),
                Vec3::from(normals[i]),
                Vec2::from(uvs[i]),
            ));
        }

        if let Some(idx_reader) = reader.read_indices() {
            indices.extend(idx_reader.into_u32().map(|i| i + base_vertex));
        } else {
            indices.extend(base_vertex..(base_vertex + positions.len() as u32));
        }

        if material_index.is_none() {
            material_index = primitive.material().index();
        }
    }

    if vertices.is_empty() {
        return Err(ReactorError::asset_load("Mesh has no vertices"));
    }

    Ok(GltfMeshData {
        vertices,
        indices,
        name: mesh.name().unwrap_or("unnamed").to_string(),
        material_index,
    })
}

pub(super) fn build_node_hierarchy(gltf: &gltf::Document) -> ReactorResult<GltfNode> {
    let scene = gltf
        .default_scene()
        .or_else(|| gltf.scenes().next())
        .ok_or_else(|| ReactorError::asset_load("glTF has no scenes"))?;

    let children: Vec<GltfNode> = scene
        .nodes()
        .map(|node| build_node(&node))
        .collect::<Result<_, _>>()?;

    Ok(GltfNode {
        name: "root".to_string(),
        transform: Mat4::IDENTITY,
        mesh_index: None,
        material_index: None,
        children,
    })
}

fn build_node(node: &gltf::Node) -> ReactorResult<GltfNode> {
    let transform = Mat4::from_cols_array_2d(&node.transform().matrix());

    let mesh_index = node.mesh().map(|m| m.index());
    let material_index = node
        .mesh()
        .and_then(|m| m.primitives().next())
        .and_then(|p| p.material().index());

    let children: Vec<GltfNode> = node
        .children()
        .map(|child| build_node(&child))
        .collect::<Result<_, _>>()?;

    Ok(GltfNode {
        name: node.name().unwrap_or("unnamed").to_string(),
        transform,
        mesh_index,
        material_index,
        children,
    })
}

pub(super) fn extract_animations(gltf: &gltf::Document) -> Vec<GltfAnimation> {
    let mut animations = Vec::new();

    for anim in gltf.animations() {
        let name = anim.name().unwrap_or("unnamed").to_string();

        animations.push(GltfAnimation {
            name,
            duration: 0.0,
            channels: Vec::new(),
            samplers: Vec::new(),
        });
    }

    animations
}
