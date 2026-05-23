// =============================================================================
// GltfLoader — Cargador de modelos glTF 2.0 con soporte PBR
// =============================================================================
// Carga modelos glTF/GLB y extrae datos CPU (vertices, indices, texturas raw).
// La conversión a recursos Vulkan (Mesh, Texture, Material) ocurre en un paso
// separado para desacoplar I/O de GPU.
//
// Fase 3: Asset Pipeline — CPU-side data extraction.
// =============================================================================

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use glam::{Mat4, Vec2, Vec3, Vec4, Quat};
use gltf::buffer::Data as GltfBufferData;
use gltf::image::Data as GltfImageData;

use crate::core::error::{ReactorResult, ReactorError};
use crate::resources::vertex::Vertex;
use crate::resources::asset_id::AssetId;

// =============================================================================
// CPU-side data types (no Vulkan dependency)
// =============================================================================

/// Datos de mesh extraídos de glTF (CPU-side, sin GPU upload)
#[derive(Clone, Debug)]
pub struct GltfMeshData {
    /// Vértices del mesh
    pub vertices: Vec<Vertex>,
    /// Índices del mesh
    pub indices: Vec<u32>,
    /// Nombre del mesh en el archivo glTF
    pub name: String,
    /// Índice del material asociado (si existe)
    pub material_index: Option<usize>,
}

/// Datos de textura extraídos de glTF (CPU-side, RGBA raw bytes)
#[derive(Clone, Debug)]
pub struct GltfTextureData {
    /// Datos RGBA8 de la imagen
    pub pixels: Vec<u8>,
    /// Ancho en píxeles
    pub width: u32,
    /// Alto en píxeles
    pub height: u32,
    /// Nombre/referencia de la textura
    pub name: String,
}

/// Alpha mode del material glTF
#[derive(Clone, Debug, PartialEq)]
pub enum GltfAlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}

/// Datos de material PBR extraídos de glTF (CPU-side)
#[derive(Clone, Debug)]
pub struct GltfMaterialData {
    /// Color base RGBA
    pub base_color: [f32; 4],
    /// Factor metálico [0.0 - 1.0]
    pub metallic: f32,
    /// Factor de rugosidad [0.0 - 1.0]
    pub roughness: f32,
    /// Índice de textura base color (en GltfModel.textures)
    pub base_color_texture_index: Option<usize>,
    /// Índice de textura normal map
    pub normal_texture_index: Option<usize>,
    /// Índice de textura metallic-roughness
    pub metallic_roughness_texture_index: Option<usize>,
    /// Índice de textura occlusion
    pub occlusion_texture_index: Option<usize>,
    /// Índice de textura emissive
    pub emissive_texture_index: Option<usize>,
    /// Factor emissive RGB
    pub emissive_factor: [f32; 3],
    /// Modo alpha
    pub alpha_mode: GltfAlphaMode,
    /// Doble cara
    pub double_sided: bool,
    /// Nombre del material
    pub name: String,
}

/// Resultado de cargar un modelo glTF completo (CPU-side)
#[derive(Clone, Debug)]
pub struct GltfModel {
    /// Meshes extraídos del modelo
    pub meshes: Vec<GltfMeshData>,
    /// Materiales asociados a los meshes
    pub materials: Vec<GltfMaterialData>,
    /// Texturas usadas por los materiales (RGBA raw)
    pub textures: Vec<GltfTextureData>,
    /// Nodo raíz de la jerarquía del modelo
    pub root_node: GltfNode,
    /// Animaciones (Fase 3.2)
    pub animations: Vec<GltfAnimation>,
    /// Metadata del archivo original
    pub source_path: PathBuf,
}

/// Nodo en la jerarquía de un modelo glTF
#[derive(Clone, Debug)]
pub struct GltfNode {
    pub name: String,
    pub transform: Mat4,
    pub mesh_index: Option<usize>,
    pub material_index: Option<usize>,
    pub children: Vec<GltfNode>,
}

/// Animación glTF (Fase 3.2)
#[derive(Clone, Debug)]
pub struct GltfAnimation {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
    pub samplers: Vec<AnimationSampler>,
}

#[derive(Clone, Debug)]
pub struct AnimationChannel {
    pub node_index: usize,
    pub sampler_index: usize,
    pub path: AnimationPath,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationPath {
    Translation,
    Rotation,
    Scale,
    Weights,
}

#[derive(Clone, Debug)]
pub struct AnimationSampler {
    pub input: Vec<f32>,  // tiempos
    pub output: Vec<f32>, // valores
    pub interpolation: AnimationInterpolation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationInterpolation {
    Linear,
    Step,
    CubicSpline,
}

// =============================================================================
// GltfLoader — Parser principal
// =============================================================================

/// Loader principal para assets glTF (extrae datos CPU sin tocar Vulkan)
pub struct GltfLoader {
    /// Cache de modelos ya cargados (AssetId -> GltfModel)
    loaded_models: HashMap<AssetId, GltfModel>,
    /// Base path para resolver URIs relativas
    base_path: PathBuf,
}

impl GltfLoader {
    /// Crear nuevo loader con base path para assets
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            loaded_models: HashMap::new(),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Cargar un archivo .glb o .gltf desde disco (síncrono)
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> ReactorResult<GltfModel> {
        let path = path.as_ref();
        let content = std::fs::read(path)
            .map_err(|e| ReactorError::asset_load(format!("Failed to read {}: {}", path.display(), e)))?;
        
        let asset_id = AssetId::from_path_with_content(path, &content);
        
        // Check cache first
        if let Some(model) = self.loaded_models.get(&asset_id) {
            return Ok(model.clone());
        }

        // Parse glTF (gltf::import handles both .gltf and .glb)
        let (gltf, buffers, images) = gltf::import(path)
            .map_err(|e| ReactorError::asset_load(format!("gltf::import failed for {}: {}", path.display(), e)))?;
        
        let model = self.process_gltf_data(&gltf, &buffers, &images, path, asset_id)?;
        
        // Cache the result
        self.loaded_models.insert(asset_id, model.clone());
        
        Ok(model)
    }

    /// Carga asíncrona (para no bloquear el main thread)
    pub async fn load_async<P: AsRef<Path> + Send + 'static>(&mut self, path: P) -> ReactorResult<GltfModel> {
        let path_buf = path.as_ref().to_path_buf();
        
        // Leer archivo en background
        let content = tokio::fs::read(&path_buf)
            .await
            .map_err(|e| ReactorError::asset_load(format!("Failed to read {}: {}", path_buf.display(), e)))?;
        
        let asset_id = AssetId::from_path_with_content(&path_buf, &content);
        
        // Check cache
        if let Some(model) = self.loaded_models.get(&asset_id) {
            return Ok(model.clone());
        }

        // Parse en background (gltf::import es síncrono pero rápido para modelos pequeños)
        let (gltf, buffers, images) = tokio::task::spawn_blocking(move || {
            gltf::import(&path_buf)
        })
        .await
        .map_err(|e| ReactorError::asset_load(format!("Spawn failed: {}", e)))?
        .map_err(|e| ReactorError::asset_load(format!("gltf::import failed: {}", e)))?;

        // Process (CPU only, no Vulkan)
        let source = path.as_ref().to_path_buf();
        self.process_gltf_data(&gltf, &buffers, &images, &source, asset_id)
    }

    /// Procesar datos glTF ya parseados
    fn process_gltf_data(
        &mut self,
        gltf: &gltf::Document,
        buffers: &[GltfBufferData],
        images: &[GltfImageData],
        path: &Path,
        asset_id: AssetId,
    ) -> ReactorResult<GltfModel> {
        // Extract textures (CPU-side RGBA data)
        let mut textures = Vec::new();
        for (idx, image) in images.iter().enumerate() {
            let texture_data = self.extract_texture(image, &format!("{}#texture_{}", path.display(), idx))?;
            textures.push(texture_data);
        }

        // Extract materials (CPU-side PBR properties)
        let mut materials = Vec::new();
        for mat in gltf.materials() {
            let material_data = Self::extract_material(&mat);
            materials.push(material_data);
        }

        // Extract meshes (CPU-side vertices + indices)
        let mut meshes = Vec::new();
        for mesh in gltf.meshes() {
            let mesh_data = self.extract_mesh(&mesh, buffers)?;
            meshes.push(mesh_data);
        }

        // Build node hierarchy
        let root_node = self.build_node_hierarchy(gltf)?;

        // Extract animations (stub para Fase 3.2)
        let animations = self.extract_animations(gltf);

        let model = GltfModel {
            meshes,
            materials,
            textures,
            root_node,
            animations,
            source_path: path.to_path_buf(),
        };

        self.loaded_models.insert(asset_id, model.clone());
        Ok(model)
    }

    /// Extraer textura como datos RGBA raw desde gltf::image::Data
    fn extract_texture(&self, image: &GltfImageData, name: &str) -> ReactorResult<GltfTextureData> {
        // gltf::image::Data ya contiene los pixels decodificados
        let pixels = match image.format {
            gltf::image::Format::R8G8B8A8 => {
                // Ya es RGBA8, copiar directamente
                image.pixels.clone()
            }
            gltf::image::Format::R8G8B8 => {
                // RGB8 -> RGBA8 (añadir canal alpha = 255)
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
                // Grayscale -> RGBA
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
                // RG -> RGBA (useful for metallic-roughness)
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
                // Para formatos de 16-bit, intentar truncar a 8-bit
                // En la práctica, la mayoría de modelos usan 8-bit
                return Err(ReactorError::asset_load(
                    format!("Unsupported texture format {:?} in {}", image.format, name)
                ));
            }
        };

        Ok(GltfTextureData {
            pixels,
            width: image.width,
            height: image.height,
            name: name.to_string(),
        })
    }

    /// Extraer material PBR como datos CPU
    fn extract_material(mat: &gltf::Material) -> GltfMaterialData {
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
            metallic_roughness_texture_index: pbr.metallic_roughness_texture().map(|t| t.texture().index()),
            occlusion_texture_index: mat.occlusion_texture().map(|t| t.texture().index()),
            emissive_texture_index: mat.emissive_texture().map(|t| t.texture().index()),
            emissive_factor: emissive,
            alpha_mode,
            double_sided: mat.double_sided(),
            name: mat.name().unwrap_or("unnamed").to_string(),
        }
    }

    /// Extraer mesh como vértices + índices CPU
    fn extract_mesh(
        &self,
        mesh: &gltf::Mesh,
        buffers: &[GltfBufferData],
    ) -> ReactorResult<GltfMeshData> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut material_index = None;

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            
            // Positions (required)
            let positions: Vec<[f32; 3]> = reader.read_positions()
                .ok_or_else(|| ReactorError::asset_load("Mesh missing positions"))?
                .collect();
            
            // Normals (optional but recommended for PBR)
            let normals: Vec<[f32; 3]> = reader.read_normals()
                .map(|n| n.collect())
                .unwrap_or_else(|| vec![[0.0, 0.0, 1.0]; positions.len()]);
            
            // TexCoords (optional) — use into_f32() to normalize
            let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0)
                .map(|uv| uv.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
            
            // Build vertices using Vertex::with_normal (color slot stores normal)
            let base_vertex = vertices.len() as u32;
            for i in 0..positions.len() {
                vertices.push(Vertex::with_normal(
                    Vec3::from(positions[i]),
                    Vec3::from(normals[i]),
                    Vec2::from(uvs[i]),
                ));
            }
            
            // Indices — use into_u32() for uniform access
            if let Some(idx_reader) = reader.read_indices() {
                indices.extend(idx_reader.into_u32().map(|i| i + base_vertex));
            } else {
                // Sin índices: generar secuencia lineal
                indices.extend(base_vertex..(base_vertex + positions.len() as u32));
            }

            // Guardar material de la primera primitiva
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

    /// Construir jerarquía de nodos recursivamente
    fn build_node_hierarchy(&self, gltf: &gltf::Document) -> ReactorResult<GltfNode> {
        let scene = gltf.default_scene()
            .or_else(|| gltf.scenes().next())
            .ok_or_else(|| ReactorError::asset_load("glTF has no scenes"))?;
        
        let children: Vec<GltfNode> = scene.nodes()
            .map(|node| self.build_node(&node))
            .collect::<Result<_, _>>()?;
        
        Ok(GltfNode {
            name: "root".to_string(),
            transform: Mat4::IDENTITY,
            mesh_index: None,
            material_index: None,
            children,
        })
    }

    /// Construir nodo individual
    fn build_node(&self, node: &gltf::Node) -> ReactorResult<GltfNode> {
        let transform = Mat4::from_cols_array_2d(&node.transform().matrix());
        
        let mesh_index = node.mesh().map(|m| m.index());
        let material_index = node.mesh()
            .and_then(|m| m.primitives().next())
            .and_then(|p| p.material().index());
        
        let children: Vec<GltfNode> = node.children()
            .map(|child| self.build_node(&child))
            .collect::<Result<_, _>>()?;
        
        Ok(GltfNode {
            name: node.name().unwrap_or("unnamed").to_string(),
            transform,
            mesh_index,
            material_index,
            children,
        })
    }

    /// Extraer animaciones (stub para Fase 3.2)
    fn extract_animations(&self, gltf: &gltf::Document) -> Vec<GltfAnimation> {
        let mut animations = Vec::new();
        
        for anim in gltf.animations() {
            let name = anim.name().unwrap_or("unnamed").to_string();
            
            // Stub: guardar metadata pero no los datos de animación aún
            animations.push(GltfAnimation {
                name,
                duration: 0.0, // TODO: Fase 3.2 — calcular duración real
                channels: Vec::new(),
                samplers: Vec::new(),
            });
        }
        
        animations
    }

    /// Obtener modelo del cache por AssetId
    pub fn get_cached(&self, id: AssetId) -> Option<&GltfModel> {
        self.loaded_models.get(&id)
    }

    /// Limpiar cache de modelos
    pub fn clear_cache(&mut self) {
        self.loaded_models.clear();
    }

    /// Obtener estadísticas del cache
    pub fn cache_stats(&self) -> GltfCacheStats {
        GltfCacheStats {
            models_cached: self.loaded_models.len(),
        }
    }
}

/// Estadísticas del cache de GltfLoader
#[derive(Clone, Debug)]
pub struct GltfCacheStats {
    pub models_cached: usize,
}

// =============================================================================
// GPU Upload helpers (requieren VulkanContext)
// =============================================================================

impl GltfModel {
    /// Subir el primer mesh a la GPU como Mesh Vulkan
    ///
    /// Helper de conveniencia para juegos simples que solo necesitan
    /// el primer mesh del modelo.
    pub fn upload_first_mesh(
        &self,
        ctx: &crate::core::VulkanContext,
        allocator: &std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>,
    ) -> ReactorResult<crate::resources::mesh::Mesh> {
        let mesh_data = self.meshes.first()
            .ok_or_else(|| ReactorError::asset_load("Model has no meshes"))?;
        
        crate::resources::mesh::Mesh::new(ctx, allocator, &mesh_data.vertices, &mesh_data.indices)
    }

    /// Subir todos los meshes a la GPU
    pub fn upload_all_meshes(
        &self,
        ctx: &crate::core::VulkanContext,
        allocator: &std::sync::Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>,
    ) -> ReactorResult<Vec<crate::resources::mesh::Mesh>> {
        self.meshes.iter()
            .map(|mesh_data| {
                crate::resources::mesh::Mesh::new(ctx, allocator, &mesh_data.vertices, &mesh_data.indices)
            })
            .collect()
    }

    /// Subir la primera textura a la GPU
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
}

// =============================================================================
// Helpers para XENOFALL y otros juegos
// =============================================================================

/// Cargar modelo glTF y devolver datos CPU (conveniencia para juegos simples)
pub fn load_gltf_simple<P: AsRef<Path>>(path: P) -> ReactorResult<GltfModel> {
    let base_path = path.as_ref().parent().unwrap_or(Path::new("."));
    let mut loader = GltfLoader::new(base_path);
    loader.load(path)
}
