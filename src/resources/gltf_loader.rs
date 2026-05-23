// =============================================================================
// GltfLoader — Cargador de modelos glTF 2.0 con soporte PBR
// =============================================================================
// Carga modelos glTF/GLB con:
// - Meshes con vertex attributes (position, normal, texcoord, tangent)
// - Materiales PBR (base color, metallic, roughness, normal, occlusion)
// - Texturas en formatos PNG/JPG/KTX2
// - Jerarquía de nodos con transforms
// - Animaciones (Fase 3.2)
// =============================================================================

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use glam::{Mat4, Vec2, Vec3, Vec4, Quat};
use gltf::buffer::Data as GltfBufferData;

use crate::core::error::{ReactorResult, ReactorError};
use crate::graphics::texture::{Texture, TextureFormat};
use crate::resources::mesh::Mesh;
use crate::resources::material::Material;
use crate::resources::vertex::Vertex;
use crate::resources::asset_id::AssetId;
use crate::resources::handle::Handle;

/// Resultado de cargar un modelo glTF completo
#[derive(Clone)]
pub struct GltfModel {
    /// Meshes cargados del modelo
    pub meshes: Vec<Handle<Mesh>>,
    /// Materiales asociados a los meshes
    pub materials: Vec<Handle<Material>>,
    /// Texturas usadas por los materiales
    pub textures: Vec<Handle<Texture>>,
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

/// Loader principal para assets glTF
pub struct GltfLoader {
    /// Cache de modelos ya cargados (AssetId -> GltfModel)
    loaded_models: HashMap<AssetId, GltfModel>,
    /// Cache de texturas por path
    texture_cache: HashMap<AssetId, Handle<Texture>>,
    /// Base path para resolver URIs relativas
    base_path: PathBuf,
}

impl GltfLoader {
    /// Crear nuevo loader con base path para assets
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            loaded_models: HashMap::new(),
            texture_cache: HashMap::new(),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Cargar un archivo .glb o .gltf desde disco (síncrono)
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> ReactorResult<GltfModel> {
        let path = path.as_ref();
        let content = std::fs::read(path)
            .map_err(|e| ReactorError::AssetLoad(format!("Failed to read {}: {}", path.display(), e)))?;
        
        let asset_id = AssetId::from_path_with_content(path, &content);
        
        // Check cache first
        if let Some(model) = self.loaded_models.get(&asset_id) {
            return Ok(model.clone());
        }

        // Parse glTF
        let (gltf, buffers, images) = gltf::import(path)
            .map_err(|e| ReactorError::AssetLoad(format!("gltf::import failed: {}", e)))?;
        
        // Load textures first (materials depend on them)
        let mut textures = Vec::new();
        for (idx, image) in images.iter().enumerate() {
            let texture = self.load_texture(image, &format!("{}#texture_{}", path.display(), idx))?;
            let tex_id = AssetId::from_path(format!("{}#{}", path.display(), idx));
            let handle = Handle::new(tex_id, texture);
            textures.push(handle.clone());
            self.texture_cache.insert(tex_id, handle);
        }

        // Load materials
        let mut materials = Vec::new();
        for (idx, mat) in gltf.materials().enumerate() {
            let material = self.load_material(&mat, &textures)?;
            let mat_id = AssetId::from_path(format!("{}#mat_{}", path.display(), idx));
            materials.push(Handle::new(mat_id, material));
        }

        // Load meshes
        let mut meshes = Vec::new();
        for (idx, mesh) in gltf.meshes().enumerate() {
            let mesh_data = self.load_mesh(&mesh, &buffers, &materials)?;
            let mesh_id = AssetId::from_path(format!("{}#mesh_{}", path.display(), idx));
            meshes.push(Handle::new(mesh_id, mesh_data));
        }

        // Build node hierarchy
        let root_node = self.build_node_hierarchy(&gltf, &meshes, &materials)?;

        // Load animations (stub for Fase 3.2)
        let animations = self.load_animations(&gltf)?;

        let model = GltfModel {
            meshes,
            materials,
            textures,
            root_node,
            animations,
            source_path: path.to_path_buf(),
        };

        // Cache the result
        self.loaded_models.insert(asset_id, model.clone());
        
        Ok(model)
    }

    /// Carga asíncrona (para no bloquear el main thread)
    pub async fn load_async<P: AsRef<Path>>(&mut self, path: P) -> ReactorResult<GltfModel> {
        let path_buf = path.as_ref().to_path_buf();
        
        // Leer archivo en background
        let content = tokio::fs::read(&path_buf)
            .await
            .map_err(|e| ReactorError::AssetLoad(format!("Failed to read {}: {}", path_buf.display(), e)))?;
        
        let asset_id = AssetId::from_path_with_content(&path_buf, &content);
        
        // Check cache
        if let Some(model) = self.loaded_models.get(&asset_id) {
            return Ok(model.clone());
        }

        // Parse en background (gltf::import es síncrono pero rápido para modelos pequeños)
        let base_path = self.base_path.clone();
        let (gltf, buffers, images) = tokio::task::spawn_blocking(move || {
            gltf::import(&path_buf)
        })
        .await
        .map_err(|e| ReactorError::AssetLoad(format!("Spawn failed: {}", e)))?
        .map_err(|e| ReactorError::AssetLoad(format!("gltf::import failed: {}", e)))?;

        // Procesar en el main thread (Vulkan requiere thread affinity)
        self.process_gltf_data(&gltf, &buffers, &images, &path_buf, asset_id)
    }

    /// Procesar datos glTF ya parseados (separa I/O de procesamiento Vulkan)
    fn process_gltf_data(
        &mut self,
        gltf: &gltf::Document,
        buffers: &[GltfBufferData],
        images: &[gltf::Image],
        path: &Path,
        asset_id: AssetId,
    ) -> ReactorResult<GltfModel> {
        // Load textures
        let mut textures = Vec::new();
        for (idx, image) in images.iter().enumerate() {
            let texture = self.load_texture(image, &format!("{}#texture_{}", path.display(), idx))?;
            let tex_id = AssetId::from_path(format!("{}#{}", path.display(), idx));
            let handle = Handle::new(tex_id, texture);
            textures.push(handle.clone());
            self.texture_cache.insert(tex_id, handle);
        }

        // Load materials
        let mut materials = Vec::new();
        for (idx, mat) in gltf.materials().enumerate() {
            let material = self.load_material(&mat, &textures)?;
            let mat_id = AssetId::from_path(format!("{}#mat_{}", path.display(), idx));
            materials.push(Handle::new(mat_id, material));
        }

        // Load meshes
        let mut meshes = Vec::new();
        for (idx, mesh) in gltf.meshes().enumerate() {
            let mesh_data = self.load_mesh(&mesh, buffers, &materials)?;
            let mesh_id = AssetId::from_path(format!("{}#mesh_{}", path.display(), idx));
            meshes.push(Handle::new(mesh_id, mesh_data));
        }

        // Build node hierarchy
        let root_node = self.build_node_hierarchy(gltf, &meshes, &materials)?;
        let animations = self.load_animations(gltf)?;

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

    /// Cargar textura desde glTF image
    fn load_texture(&self, image: &gltf::Image, name: &str) -> ReactorResult<Texture> {
        use image::ImageFormat;
        
        match &image.source {
            gltf::image::Source::View { view, mime_type } => {
                let buffer = &view.buffer().data();
                let start = view.offset();
                let end = start + view.length();
                let data = &buffer[start..end];
                
                let fmt = match mime_type.as_str() {
                    "image/png" => ImageFormat::Png,
                    "image/jpeg" | "image/jpg" => ImageFormat::Jpeg,
                    _ => return Err(ReactorError::AssetLoad(format!("Unsupported texture format: {}", mime_type))),
                };
                
                let img = image::load_from_memory_with_format(data, fmt)
                    .map_err(|e| ReactorError::AssetLoad(format!("Failed to decode texture {}: {}", name, e)))?;
                
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                
                Ok(Texture::from_rgba8(w, h, &rgba))
            }
            gltf::image::Source::Uri { uri, .. } => {
                // Resolver path relativo
                let full_path = if Path::new(uri).is_absolute() {
                    PathBuf::from(uri)
                } else {
                    self.base_path.join(uri)
                };
                
                // Intentar cargar como KTX2 primero (formato optimizado)
                if full_path.extension().map_or(false, |e| e == "ktx2") {
                    return self.load_ktx2_texture(&full_path, name);
                }
                
                // Fallback a image crate
                let img = image::open(&full_path)
                    .map_err(|e| ReactorError::AssetLoad(format!("Failed to open texture {}: {}", full_path.display(), e)))?;
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                
                Ok(Texture::from_rgba8(w, h, &rgba))
            }
        }
    }

    /// Cargar textura en formato KTX2 (Basis Universal compressed)
    fn load_ktx2_texture(&self, path: &Path, name: &str) -> ReactorResult<Texture> {
        use ktx2::Ktx2;
        
        let data = std::fs::read(path)
            .map_err(|e| ReactorError::AssetLoad(format!("Failed to read KTX2 {}: {}", path.display(), e)))?;
        
        let ktx = Ktx2::new(&data)
            .map_err(|e| ReactorError::AssetLoad(format!("Invalid KTX2 {}: {}", name, e)))?;
        
        // Obtener primer nivel de mip y formato
        let level_data = ktx.levels().next()
            .ok_or_else(|| ReactorError::AssetLoad(format!("KTX2 {} has no levels", name)))?;
        
        let (width, height) = (ktx.width(), ktx.height());
        
        // Determinar formato Vulkan desde KTX2
        let format = self.ktx_format_to_vulkan(ktx.format())?;
        
        // Crear textura con datos comprimidos (Fase 4 optimizará upload directo a GPU)
        Texture::from_compressed_data(
            width,
            height,
            format,
            level_data.as_ref(),
            ktx.mip_levels() as u32,
        )
    }

    /// Mapear formato KTX2 a TextureFormat
    fn ktx_format_to_vulkan(&self, ktx_fmt: ktx2::Format) -> ReactorResult<TextureFormat> {
        use ktx2::Format as KtxFmt;
        use crate::graphics::texture::TextureFormat as TexFmt;
        
        match ktx_fmt {
            KtxFmt::R8G8B8A8_UNORM => Ok(TexFmt::Rgba8Unorm),
            KtxFmt::R8G8B8A8_SRGB => Ok(TexFmt::Rgba8Srgb),
            KtxFmt::BC1_RGB_UNORM_BLOCK => Ok(TexFmt::Bc1RgbaUnorm),
            KtxFmt::BC1_RGB_SRGB_BLOCK => Ok(TexFmt::Bc1RgbaSrgb),
            KtxFmt::BC3_UNORM_BLOCK => Ok(TexFmt::Bc3RgbaUnorm),
            KtxFmt::BC3_SRGB_BLOCK => Ok(TexFmt::Bc3RgbaSrgb),
            KtxFmt::BC7_UNORM_BLOCK => Ok(TexFmt::Bc7RgbaUnorm),
            KtxFmt::BC7_SRGB_BLOCK => Ok(TexFmt::Bc7RgbaSrgb),
            _ => Err(ReactorError::AssetLoad(format!("Unsupported KTX2 format: {:?}", ktx_fmt))),
        }
    }

    /// Crear material PBR desde glTF material
    fn load_material(&self, mat: &gltf::Material, textures: &[Handle<Texture>]) -> ReactorResult<Material> {
        use gltf::material::AlphaMode;
        
        let pbr = mat.pbr_metallic_roughness();
        
        let base_color = pbr.base_color_factor();
        let metallic = pbr.metallic_factor();
        let roughness = pbr.roughness_factor();
        
        let mut material = Material::new()
            .with_base_color(Vec4::new(base_color[0], base_color[1], base_color[2], base_color[3]))
            .with_metallic(metallic)
            .with_roughness(roughness);
        
        // Attach textures if available
        if let Some(tex_info) = pbr.base_color_texture() {
            if let Some(handle) = textures.get(tex_info.texture().index()) {
                material = material.with_base_color_texture(handle.clone());
            }
        }
        
        if let Some(tex_info) = pbr.metallic_roughness_texture() {
            if let Some(handle) = textures.get(tex_info.texture().index()) {
                material = material.with_metallic_roughness_texture(handle.clone());
            }
        }
        
        if let Some(tex_info) = mat.normal_texture() {
            if let Some(handle) = textures.get(tex_info.texture().index()) {
                material = material.with_normal_texture(handle.clone());
            }
        }
        
        if let Some(tex_info) = mat.occlusion_texture() {
            if let Some(handle) = textures.get(tex_info.texture().index()) {
                material = material.with_occlusion_texture(handle.clone());
            }
        }
        
        if let Some(tex_info) = mat.emissive_texture() {
            if let Some(handle) = textures.get(tex_info.texture().index()) {
                material = material.with_emissive_texture(handle.clone());
            }
        }
        
        // Alpha mode
        match mat.alpha_mode() {
            AlphaMode::Mask => {
                let cutoff = mat.alpha_cutoff().unwrap_or(0.5);
                material = material.with_alpha_test(cutoff);
            }
            AlphaMode::Blend => {
                material = material.with_transparent(true);
            }
            AlphaMode::Opaque => {}
        }
        
        // Double sided
        if mat.double_sided() {
            material = material.with_double_sided(true);
        }
        
        Ok(material)
    }

    /// Cargar mesh desde glTF primitive
    fn load_mesh(
        &self,
        mesh: &gltf::Mesh,
        buffers: &[GltfBufferData],
        materials: &[Handle<Material>],
    ) -> ReactorResult<Mesh> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            
            // Positions (required)
            let positions: Vec<[f32; 3]> = reader.read_positions()
                .ok_or_else(|| ReactorError::AssetLoad("Mesh missing positions".to_string()))?
                .map(|p| p.into())
                .collect();
            
            // Normals (optional but recommended for PBR)
            let normals: Vec<[f32; 3]> = reader.read_normals()
                .map(|n| n.map(|n| n.into()).collect())
                .unwrap_or_else(|| vec![[0.0, 0.0, 1.0]; positions.len()]);
            
            // TexCoords (optional)
            let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0)
                .map(|uv| uv.map(|uv| uv.into()).collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
            
            // Tangents (optional, needed for normal mapping)
            let tangents: Vec<[f32; 4]> = reader.read_tangents()
                .map(|t| t.map(|t| t.into()).collect())
                .unwrap_or_else(|| {
                    // Generar tangents dummy si no existen
                    positions.iter().map(|_| [1.0, 0.0, 0.0, 1.0]).collect()
                });
            
            // Build vertices
            for i in 0..positions.len() {
                vertices.push(Vertex {
                    position: positions[i].into(),
                    normal: normals[i].into(),
                    tex_coord: uvs[i].into(),
                    tangent: tangents[i].into(),
                });
            }
            
            // Indices
            if let Some(indices_reader) = reader.read_indices() {
                match indices_reader {
                    gltf::accessor::Indices::U16(iter) => {
                        indices.extend(iter.map(|i| i as u32));
                    }
                    gltf::accessor::Indices::U32(iter) => {
                        indices.extend(iter);
                    }
                }
            }
        }
        
        if vertices.is_empty() {
            return Err(ReactorError::AssetLoad("Mesh has no vertices".to_string()));
        }
        
        Mesh::from_vertices_and_indices(&vertices, &indices)
    }

    /// Construir jerarquía de nodos recursivamente
    fn build_node_hierarchy(
        &self,
        gltf: &gltf::Document,
        meshes: &[Handle<Mesh>],
        materials: &[Handle<Material>],
    ) -> ReactorResult<GltfNode> {
        let scene = gltf.default_scene()
            .ok_or_else(|| ReactorError::AssetLoad("glTF has no default scene".to_string()))?;
        
        let children: Vec<GltfNode> = scene.nodes()
            .map(|node| self.build_node(&node, gltf, meshes, materials))
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
    fn build_node(
        &self,
        node: &gltf::Node,
        gltf: &gltf::Document,
        meshes: &[Handle<Mesh>],
        materials: &[Handle<Material>],
    ) -> ReactorResult<GltfNode> {
        let transform = Mat4::from_cols_array_2d(&node.transform().matrix());
        
        let mesh_index = node.mesh().map(|m| m.index());
        let material_index = node.mesh()
            .and_then(|m| m.primitives().next())
            .and_then(|p| p.material().map(|mat| mat.index()));
        
        let children: Vec<GltfNode> = node.children()
            .map(|child| self.build_node(&child, gltf, meshes, materials))
            .collect::<Result<_, _>>()?;
        
        Ok(GltfNode {
            name: node.name().unwrap_or("unnamed").to_string(),
            transform,
            mesh_index,
            material_index,
            children,
        })
    }

    /// Cargar animaciones (stub para Fase 3.2)
    fn load_animations(&self, gltf: &gltf::Document) -> ReactorResult<Vec<GltfAnimation>> {
        let mut animations = Vec::new();
        
        for anim in gltf.animations() {
            let name = anim.name().unwrap_or("unnamed").to_string();
            
            // Calcular duración máxima
            let duration = anim.samplers()
                .filter_map(|s| s.input().read().ok())
                .flat_map(|times| times.max().copied())
                .fold(0.0f32, f32::max);
            
            // Stub: guardar metadata pero no los datos de animación aún
            animations.push(GltfAnimation {
                name,
                duration,
                channels: Vec::new(), // TODO: Fase 3.2
                samplers: Vec::new(),
            });
        }
        
        Ok(animations)
    }

    /// Obtener modelo del cache por AssetId
    pub fn get_cached(&self, id: AssetId) -> Option<&GltfModel> {
        self.loaded_models.get(&id)
    }

    /// Limpiar cache de modelos
    pub fn clear_cache(&mut self) {
        self.loaded_models.clear();
        self.texture_cache.clear();
    }

    /// Obtener estadísticas del cache
    pub fn cache_stats(&self) -> GltfCacheStats {
        GltfCacheStats {
            models_cached: self.loaded_models.len(),
            textures_cached: self.texture_cache.len(),
        }
    }
}

/// Estadísticas del cache de GltfLoader
#[derive(Clone, Debug)]
pub struct GltfCacheStats {
    pub models_cached: usize,
    pub textures_cached: usize,
}

// =============================================================================
// Helpers para XENOFALL y otros juegos
// =============================================================================

/// Cargar modelo glTF y devolver solo el primer mesh (conveniencia para juegos simples)
pub fn load_gltf_simple<P: AsRef<Path>>(path: P) -> ReactorResult<(Handle<Mesh>, Handle<Material>)> {
    let mut loader = GltfLoader::new(".");
    let model = loader.load(path)?;
    
    let mesh = model.meshes.first()
        .cloned()
        .ok_or_else(|| ReactorError::AssetLoad("Model has no meshes".to_string()))?;
    
    let material = model.materials.first()
        .cloned()
        .unwrap_or_else(|| Handle::new(AssetId::INVALID, Material::new()));
    
    Ok((mesh, material))
}
