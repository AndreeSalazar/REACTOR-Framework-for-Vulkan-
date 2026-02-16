// =============================================================================
// Asset Manager — Centralized asset loading with caching and deduplication
// =============================================================================

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::error::Error;

use crate::resources::texture::Texture;
use crate::resources::mesh::Mesh;
use crate::resources::model::{ObjData, GltfData};
use crate::Vertex;

/// Asset handle for type-safe asset references
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssetHandle(pub u64);

impl AssetHandle {
    pub fn invalid() -> Self {
        Self(0)
    }
    
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// Asset loading state
#[derive(Clone, Debug, PartialEq)]
pub enum AssetState {
    NotLoaded,
    Loading,
    Loaded,
    Failed(String),
}

/// Asset metadata
#[derive(Clone, Debug)]
pub struct AssetMeta {
    pub handle: AssetHandle,
    pub path: PathBuf,
    pub state: AssetState,
    pub ref_count: u32,
}

/// Cached texture entry
struct TextureEntry {
    texture: Texture,
    meta: AssetMeta,
}

/// Cached mesh entry
struct MeshEntry {
    mesh: Arc<Mesh>,
    meta: AssetMeta,
}

/// Asset Manager for centralized asset loading and caching
pub struct AssetManager {
    next_handle: u64,
    texture_cache: HashMap<PathBuf, TextureEntry>,
    mesh_cache: HashMap<PathBuf, MeshEntry>,
    handle_to_path: HashMap<AssetHandle, PathBuf>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            next_handle: 1,
            texture_cache: HashMap::new(),
            mesh_cache: HashMap::new(),
            handle_to_path: HashMap::new(),
        }
    }

    fn next_handle(&mut self) -> AssetHandle {
        let handle = AssetHandle(self.next_handle);
        self.next_handle += 1;
        handle
    }

    /// Get texture from cache or return None
    pub fn get_texture(&self, path: &Path) -> Option<&Texture> {
        self.texture_cache.get(path).map(|e| &e.texture)
    }

    /// Get mesh from cache or return None
    pub fn get_mesh(&self, path: &Path) -> Option<Arc<Mesh>> {
        self.mesh_cache.get(path).map(|e| e.mesh.clone())
    }

    /// Check if texture is cached
    pub fn has_texture(&self, path: &Path) -> bool {
        self.texture_cache.contains_key(path)
    }

    /// Check if mesh is cached
    pub fn has_mesh(&self, path: &Path) -> bool {
        self.mesh_cache.contains_key(path)
    }

    /// Cache a texture
    pub fn cache_texture(&mut self, path: &Path, texture: Texture) -> AssetHandle {
        let handle = self.next_handle();
        let path_buf = path.to_path_buf();
        
        self.texture_cache.insert(path_buf.clone(), TextureEntry {
            texture,
            meta: AssetMeta {
                handle,
                path: path_buf.clone(),
                state: AssetState::Loaded,
                ref_count: 1,
            },
        });
        
        self.handle_to_path.insert(handle, path_buf);
        handle
    }

    /// Cache a mesh
    pub fn cache_mesh(&mut self, path: &Path, mesh: Arc<Mesh>) -> AssetHandle {
        let handle = self.next_handle();
        let path_buf = path.to_path_buf();
        
        self.mesh_cache.insert(path_buf.clone(), MeshEntry {
            mesh,
            meta: AssetMeta {
                handle,
                path: path_buf.clone(),
                state: AssetState::Loaded,
                ref_count: 1,
            },
        });
        
        self.handle_to_path.insert(handle, path_buf);
        handle
    }

    /// Get asset state by handle
    pub fn get_state(&self, handle: AssetHandle) -> AssetState {
        if let Some(path) = self.handle_to_path.get(&handle) {
            if let Some(entry) = self.texture_cache.get(path) {
                return entry.meta.state.clone();
            }
            if let Some(entry) = self.mesh_cache.get(path) {
                return entry.meta.state.clone();
            }
        }
        AssetState::NotLoaded
    }

    /// Increment reference count
    pub fn add_ref(&mut self, handle: AssetHandle) {
        if let Some(path) = self.handle_to_path.get(&handle).cloned() {
            if let Some(entry) = self.texture_cache.get_mut(&path) {
                entry.meta.ref_count += 1;
            }
            if let Some(entry) = self.mesh_cache.get_mut(&path) {
                entry.meta.ref_count += 1;
            }
        }
    }

    /// Decrement reference count and optionally unload
    pub fn release(&mut self, handle: AssetHandle) {
        if let Some(path) = self.handle_to_path.get(&handle).cloned() {
            let mut should_remove_texture = false;
            let mut should_remove_mesh = false;
            
            if let Some(entry) = self.texture_cache.get_mut(&path) {
                entry.meta.ref_count = entry.meta.ref_count.saturating_sub(1);
                if entry.meta.ref_count == 0 {
                    should_remove_texture = true;
                }
            }
            if let Some(entry) = self.mesh_cache.get_mut(&path) {
                entry.meta.ref_count = entry.meta.ref_count.saturating_sub(1);
                if entry.meta.ref_count == 0 {
                    should_remove_mesh = true;
                }
            }
            
            if should_remove_texture {
                self.texture_cache.remove(&path);
                self.handle_to_path.remove(&handle);
            }
            if should_remove_mesh {
                self.mesh_cache.remove(&path);
                self.handle_to_path.remove(&handle);
            }
        }
    }

    /// Clear all cached assets
    pub fn clear(&mut self) {
        self.texture_cache.clear();
        self.mesh_cache.clear();
        self.handle_to_path.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> AssetStats {
        AssetStats {
            texture_count: self.texture_cache.len(),
            mesh_count: self.mesh_cache.len(),
            total_handles: self.handle_to_path.len(),
        }
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Asset cache statistics
#[derive(Clone, Debug)]
pub struct AssetStats {
    pub texture_count: usize,
    pub mesh_count: usize,
    pub total_handles: usize,
}

// =============================================================================
// Asset Loading Helpers
// =============================================================================

/// Load OBJ model and extract vertices/indices
pub fn load_obj_mesh(path: &Path) -> Result<(Vec<Vertex>, Vec<u32>), Box<dyn Error>> {
    let obj = ObjData::load(path)?;
    Ok((obj.vertices, obj.indices))
}

/// Load glTF model and extract first mesh vertices/indices
pub fn load_gltf_mesh(path: &Path) -> Result<(Vec<Vertex>, Vec<u32>), Box<dyn Error>> {
    let gltf = GltfData::load_first(path)?;
    Ok((gltf.vertices, gltf.indices))
}

/// Load any supported model format based on extension
pub fn load_model_auto(path: &Path) -> Result<(Vec<Vertex>, Vec<u32>), Box<dyn Error>> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    
    match ext.as_str() {
        "obj" => load_obj_mesh(path),
        "gltf" | "glb" => load_gltf_mesh(path),
        _ => Err(format!("Unsupported model format: {}", ext).into()),
    }
}
