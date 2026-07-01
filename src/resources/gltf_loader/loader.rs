use std::collections::HashMap;
use std::path::{Path, PathBuf};
use gltf::buffer::Data as GltfBufferData;
use gltf::image::Data as GltfImageData;
use crate::core::error::{ReactorError, ReactorResult};
use crate::resources::asset_id::AssetId;
use crate::resources::gltf_loader::types::*;
use crate::resources::gltf_loader::extract;

#[derive(Clone)]
pub struct GltfLoader {
    loaded_models: HashMap<AssetId, GltfModel>,
    base_path: PathBuf,
}

impl GltfLoader {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            loaded_models: HashMap::new(),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> ReactorResult<GltfModel> {
        let path = path.as_ref();
        let content = std::fs::read(path).map_err(|e| {
            ReactorError::asset_load(format!("Failed to read {}: {}", path.display(), e))
        })?;

        let asset_id = AssetId::from_path_with_content(path, &content);

        if let Some(model) = self.loaded_models.get(&asset_id) {
            return Ok(model.clone());
        }

        let (gltf, buffers, images) = gltf::import(path).map_err(|e| {
            ReactorError::asset_load(format!("gltf::import failed for {}: {}", path.display(), e))
        })?;

        let model = self.process_gltf_data(&gltf, &buffers, &images, path, asset_id)?;

        self.loaded_models.insert(asset_id, model.clone());

        Ok(model)
    }

    pub async fn load_async<P: AsRef<Path> + Send + 'static>(
        &mut self,
        path: P,
    ) -> ReactorResult<GltfModel> {
        let path_buf = path.as_ref().to_path_buf();

        let content = tokio::fs::read(&path_buf).await.map_err(|e| {
            ReactorError::asset_load(format!("Failed to read {}: {}", path_buf.display(), e))
        })?;

        let asset_id = AssetId::from_path_with_content(&path_buf, &content);

        if let Some(model) = self.loaded_models.get(&asset_id) {
            return Ok(model.clone());
        }

        let (gltf, buffers, images) = tokio::task::spawn_blocking(move || gltf::import(&path_buf))
            .await
            .map_err(|e| ReactorError::asset_load(format!("Spawn failed: {}", e)))?
            .map_err(|e| ReactorError::asset_load(format!("gltf::import failed: {}", e)))?;

        let source = path.as_ref().to_path_buf();
        self.process_gltf_data(&gltf, &buffers, &images, &source, asset_id)
    }

    pub fn get_cached(&self, id: AssetId) -> Option<&GltfModel> {
        self.loaded_models.get(&id)
    }

    pub fn clear_cache(&mut self) {
        self.loaded_models.clear();
    }

    pub fn cache_stats(&self) -> GltfCacheStats {
        GltfCacheStats { models_cached: self.loaded_models.len() }
    }

    fn process_gltf_data(
        &mut self,
        gltf: &gltf::Document,
        buffers: &[GltfBufferData],
        images: &[GltfImageData],
        path: &Path,
        asset_id: AssetId,
    ) -> ReactorResult<GltfModel> {
        let mut textures = Vec::new();
        for (idx, image) in images.iter().enumerate() {
            let texture_data =
                extract::extract_texture(image, &format!("{}#texture_{}", path.display(), idx))?;
            textures.push(texture_data);
        }

        let mut materials = Vec::new();
        for mat in gltf.materials() {
            let material_data = extract::extract_material(&mat);
            materials.push(material_data);
        }

        let mut meshes = Vec::new();
        for mesh in gltf.meshes() {
            let mesh_data = extract::extract_mesh(&mesh, buffers)?;
            meshes.push(mesh_data);
        }

        let root_node = extract::build_node_hierarchy(gltf)?;

        let animations = extract::extract_animations(gltf);

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
}
