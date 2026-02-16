use crate::resources::mesh::Mesh;
use crate::resources::material::Material;
use crate::Vertex;
use glam::{Mat4, Vec2, Vec3};
use std::sync::Arc;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::error::Error;
use gltf;

// =============================================================================
// OBJ Loader — Basic Wavefront OBJ file loader
// =============================================================================

#[derive(Default)]
pub struct ObjData {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl ObjData {
    /// Load OBJ file from path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        
        let mut positions: Vec<Vec3> = Vec::new();
        let mut normals: Vec<Vec3> = Vec::new();
        let mut uvs: Vec<Vec2> = Vec::new();
        
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        
        // Map for deduplicating vertices: (pos_idx, uv_idx, norm_idx) -> vertex_index
        let mut vertex_map: std::collections::HashMap<(usize, usize, usize), u32> = std::collections::HashMap::new();
        
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "v" if parts.len() >= 4 => {
                    let x: f32 = parts[1].parse()?;
                    let y: f32 = parts[2].parse()?;
                    let z: f32 = parts[3].parse()?;
                    positions.push(Vec3::new(x, y, z));
                }
                "vn" if parts.len() >= 4 => {
                    let x: f32 = parts[1].parse()?;
                    let y: f32 = parts[2].parse()?;
                    let z: f32 = parts[3].parse()?;
                    normals.push(Vec3::new(x, y, z).normalize());
                }
                "vt" if parts.len() >= 3 => {
                    let u: f32 = parts[1].parse()?;
                    let v: f32 = parts[2].parse()?;
                    uvs.push(Vec2::new(u, 1.0 - v)); // Flip V for Vulkan
                }
                "f" if parts.len() >= 4 => {
                    // Parse face (triangulate if needed)
                    let mut face_indices: Vec<u32> = Vec::new();
                    
                    for i in 1..parts.len() {
                        let vertex_data = parts[i];
                        let indices_parts: Vec<&str> = vertex_data.split('/').collect();
                        
                        let pos_idx: usize = indices_parts[0].parse::<usize>()? - 1;
                        let uv_idx: usize = if indices_parts.len() > 1 && !indices_parts[1].is_empty() {
                            indices_parts[1].parse::<usize>()? - 1
                        } else {
                            0
                        };
                        let norm_idx: usize = if indices_parts.len() > 2 && !indices_parts[2].is_empty() {
                            indices_parts[2].parse::<usize>()? - 1
                        } else {
                            0
                        };
                        
                        let key = (pos_idx, uv_idx, norm_idx);
                        
                        let vertex_index = if let Some(&idx) = vertex_map.get(&key) {
                            idx
                        } else {
                            let pos = positions.get(pos_idx).copied().unwrap_or(Vec3::ZERO);
                            let uv = uvs.get(uv_idx).copied().unwrap_or(Vec2::ZERO);
                            let normal = normals.get(norm_idx).copied().unwrap_or(Vec3::Y);
                            
                            let vertex = Vertex::new(pos, normal, uv);
                            let idx = vertices.len() as u32;
                            vertices.push(vertex);
                            vertex_map.insert(key, idx);
                            idx
                        };
                        
                        face_indices.push(vertex_index);
                    }
                    
                    // Triangulate (fan triangulation for convex polygons)
                    for i in 1..face_indices.len() - 1 {
                        indices.push(face_indices[0]);
                        indices.push(face_indices[i]);
                        indices.push(face_indices[i + 1]);
                    }
                }
                _ => {}
            }
        }
        
        // Generate normals if none were provided
        if normals.is_empty() && !vertices.is_empty() {
            Self::generate_normals(&mut vertices, &indices);
        }
        
        Ok(Self {
            positions,
            normals,
            uvs,
            vertices,
            indices,
        })
    }
    
    /// Generate flat normals for vertices (stored in color field)
    fn generate_normals(vertices: &mut [Vertex], indices: &[u32]) {
        // Use a separate buffer for accumulating normals
        let mut normals: Vec<Vec3> = vec![Vec3::ZERO; vertices.len()];
        
        // Calculate face normals and accumulate
        for tri in indices.chunks(3) {
            if tri.len() < 3 { continue; }
            
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;
            
            let v0 = Vec3::from_array(vertices[i0].position);
            let v1 = Vec3::from_array(vertices[i1].position);
            let v2 = Vec3::from_array(vertices[i2].position);
            
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(edge2);
            
            normals[i0] += normal;
            normals[i1] += normal;
            normals[i2] += normal;
        }
        
        // Normalize and store in color field (used as normal by shaders)
        for (i, v) in vertices.iter_mut().enumerate() {
            let n = if normals[i].length_squared() > 0.0001 {
                normals[i].normalize()
            } else {
                Vec3::Y
            };
            v.color = n.to_array();
        }
    }
    
    /// Get vertex and index count
    pub fn vertex_count(&self) -> usize { self.vertices.len() }
    pub fn index_count(&self) -> usize { self.indices.len() }
    pub fn triangle_count(&self) -> usize { self.indices.len() / 3 }
}

// =============================================================================
// Model — A mesh with material and transform
// =============================================================================

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

// =============================================================================
// glTF 2.0 Loader — Standard 3D model format
// =============================================================================

#[derive(Default)]
pub struct GltfData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub name: String,
}

impl GltfData {
    /// Load glTF/GLB file from path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, Box<dyn Error>> {
        let (document, buffers, _images) = gltf::import(path.as_ref())?;
        let mut meshes = Vec::new();
        
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                
                let mut vertices = Vec::new();
                let indices;
                
                // Read positions
                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .map(|iter| iter.collect())
                    .unwrap_or_default();
                
                // Read normals
                let normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .map(|iter| iter.collect())
                    .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
                
                // Read texture coordinates
                let tex_coords: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|iter| iter.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
                
                // Build vertices
                for i in 0..positions.len() {
                    let pos = Vec3::from_array(positions[i]);
                    let normal = Vec3::from_array(normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]));
                    let uv = Vec2::from_array(tex_coords.get(i).copied().unwrap_or([0.0, 0.0]));
                    vertices.push(Vertex::new(pos, normal, uv));
                }
                
                // Read indices
                if let Some(indices_reader) = reader.read_indices() {
                    indices = indices_reader.into_u32().collect();
                } else {
                    // Generate indices if not present
                    indices = (0..vertices.len() as u32).collect();
                }
                
                meshes.push(GltfData {
                    vertices,
                    indices,
                    name: mesh.name().unwrap_or("unnamed").to_string(),
                });
            }
        }
        
        Ok(meshes)
    }
    
    /// Load first mesh from glTF file
    pub fn load_first<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let meshes = Self::load(path)?;
        meshes.into_iter().next().ok_or_else(|| "No meshes found in glTF file".into())
    }
    
    pub fn vertex_count(&self) -> usize { self.vertices.len() }
    pub fn index_count(&self) -> usize { self.indices.len() }
    pub fn triangle_count(&self) -> usize { self.indices.len() / 3 }
}
