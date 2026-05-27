// =============================================================================
// AssetDatabase — Persistencia de metadata de assets con sled
// =============================================================================
// Almacena en disco:
// - Hash de contenido para detectar cambios (hot-reload)
// - Metadata de assets (tamaño, formato, dependencias)
// - Cache de assets procesados (texturas comprimidas, meshes optimizados)
//
// Usa sled (embedded KV store) para persistencia ligera sin dependencias externas.
// =============================================================================

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use sled::Db;

use crate::core::error::{ReactorError, ReactorResult};
use crate::resources::asset_id::AssetId;

/// Metadata persistente de un asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// Path original del archivo
    pub source_path: String,
    /// Hash del contenido (para detectar cambios)
    pub content_hash: u64,
    /// Timestamp de última modificación
    pub last_modified: u64,
    /// Tamaño del archivo en bytes
    pub file_size: u64,
    /// Tipo de asset (texture, mesh, model, etc.)
    pub asset_type: AssetType,
    /// Formato original (png, gltf, ktx2, etc.)
    pub source_format: String,
    /// Formato runtime optimizado (bc7, meshopt, ogg, etc.)
    pub runtime_format: Option<String>,
    /// Assets de los que depende (para reload en cascada)
    pub dependencies: Vec<AssetId>,
    /// Assets que dependen de este (para invalidar cache)
    pub dependents: Vec<AssetId>,
    /// Metadata específica del tipo (dimensiones, vertex count, etc.)
    pub extra: HashMap<String, serde_json::Value>,
}

/// Tipo de asset para clasificación y routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Mesh,
    Model,
    Material,
    Shader,
    Audio,
    Font,
    Config,
    Unknown,
}

impl AssetType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "tga" | "ktx2" | "dds" | "exr" | "hdr" => {
                AssetType::Texture
            }
            "obj" | "fbx" | "gltf" | "glb" => AssetType::Model,
            "vert" | "frag" | "comp" | "spv" | "wgsl" | "glsl" => AssetType::Shader,
            "wav" | "mp3" | "ogg" | "flac" | "xm" => AssetType::Audio,
            "ttf" | "otf" | "woff" | "woff2" => AssetType::Font,
            "json" | "ron" | "toml" | "yaml" | "yml" => AssetType::Config,
            "mat" | "material" => AssetType::Material,
            _ => AssetType::Unknown,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            AssetType::Texture => "ktx2",
            AssetType::Mesh => "mesh",
            AssetType::Model => "glb",
            AssetType::Material => "mat",
            AssetType::Shader => "spv",
            AssetType::Audio => "ogg",
            AssetType::Font => "ttf",
            AssetType::Config => "json",
            AssetType::Unknown => "bin",
        }
    }
}

/// Database principal para metadata de assets
pub struct AssetDatabase {
    db: Db,
    /// Cache en memoria para acceso rápido
    memory_cache: HashMap<AssetId, AssetMetadata>,
    /// Path base para resolver rutas relativas
    base_path: PathBuf,
}

impl AssetDatabase {
    /// Crear o abrir database en el path especificado
    pub fn open<P: AsRef<Path>>(path: P) -> ReactorResult<Self> {
        let db = sled::open(path.as_ref())
            .map_err(|e| ReactorError::asset_load(format!("Failed to open asset DB: {}", e)))?;

        Ok(Self {
            db,
            memory_cache: HashMap::new(),
            base_path: PathBuf::from("assets"),
        })
    }

    /// Crear database en memoria (para tests)
    pub fn in_memory() -> ReactorResult<Self> {
        let config = sled::Config::default().temporary(true);
        let db = config.open().map_err(|e| {
            ReactorError::asset_load(format!("Failed to create in-memory DB: {}", e))
        })?;

        Ok(Self {
            db,
            memory_cache: HashMap::new(),
            base_path: PathBuf::from("assets"),
        })
    }

    /// Establecer base path para assets
    pub fn with_base_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.base_path = path.as_ref().to_path_buf();
        self
    }

    /// Registrar metadata de un asset
    pub fn register_asset(&mut self, id: AssetId, meta: AssetMetadata) -> ReactorResult<()> {
        // Serializar metadata
        let serialized = serde_json::to_vec(&meta).map_err(|e| {
            ReactorError::asset_load(format!("Failed to serialize metadata: {}", e))
        })?;

        // Guardar en sled con key = AssetId como string hex
        let key = format!("meta:{:016x}", id.as_u64());
        self.db
            .insert(key.as_bytes(), serialized.as_slice())
            .map_err(|e| ReactorError::asset_load(format!("Failed to write to DB: {}", e)))?;

        // Flush para garantizar persistencia
        self.db.flush()?;

        // Actualizar cache en memoria
        self.memory_cache.insert(id, meta.clone());

        // Registrar dependencias inversas
        for dep_id in &meta.dependencies {
            self.add_dependent(*dep_id, id)?;
        }

        Ok(())
    }

    /// Obtener metadata de un asset por AssetId
    pub fn get_metadata(&mut self, id: AssetId) -> ReactorResult<Option<AssetMetadata>> {
        // Check memory cache first
        if let Some(meta) = self.memory_cache.get(&id) {
            return Ok(Some(meta.clone()));
        }

        // Load from disk
        let key = format!("meta:{:016x}", id.as_u64());
        if let Some(data) = self
            .db
            .get(key.as_bytes())
            .map_err(|e| ReactorError::asset_load(format!("Failed to read from DB: {}", e)))?
        {
            let meta: AssetMetadata = serde_json::from_slice(&data).map_err(|e| {
                ReactorError::asset_load(format!("Failed to deserialize metadata: {}", e))
            })?;

            // Cache in memory
            self.memory_cache.insert(id, meta.clone());
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }

    /// Obtener metadata por path de archivo
    pub fn get_by_path<P: AsRef<Path>>(&mut self, path: P) -> ReactorResult<Option<AssetMetadata>> {
        let id = AssetId::from_path(path.as_ref());
        self.get_metadata(id)
    }

    /// Verificar si un asset ha cambiado desde la última vez registrado
    pub fn has_changed<P: AsRef<Path>>(&self, path: P) -> ReactorResult<bool> {
        let path = path.as_ref();
        let current_meta = std::fs::metadata(path).map_err(|e| {
            ReactorError::asset_load(format!("Failed to stat {}: {}", path.display(), e))
        })?;

        let current_mtime = current_meta
            .modified()
            .map_err(|e| ReactorError::asset_load(format!("Failed to get mtime: {}", e)))?
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ReactorError::asset_load("Invalid timestamp"))?
            .as_secs();

        let current_size = current_meta.len();
        let current_hash = self.compute_content_hash(path)?;

        // Check against stored metadata
        let id = AssetId::from_path(path);
        if let Some(stored) = self.memory_cache.get(&id) {
            return Ok(stored.content_hash != current_hash || stored.last_modified != current_mtime);
        }

        // If not in cache, assume it's new/changed
        Ok(true)
    }

    /// Calcular hash de contenido de un archivo
    pub fn compute_content_hash<P: AsRef<Path>>(&self, path: P) -> ReactorResult<u64> {
        use xxhash_rust::xxh3::xxh3_64;

        let content = std::fs::read(path.as_ref()).map_err(|e| {
            ReactorError::asset_load(format!("Failed to read {}: {}", path.as_ref().display(), e))
        })?;

        Ok(xxh3_64(&content))
    }

    /// Añadir dependencia entre assets
    pub fn add_dependency(&mut self, asset_id: AssetId, depends_on: AssetId) -> ReactorResult<()> {
        if let Some(mut meta) = self.get_metadata(asset_id)? {
            if !meta.dependencies.contains(&depends_on) {
                meta.dependencies.push(depends_on);
                self.register_asset(asset_id, meta)?;
            }
        }
        Ok(())
    }

    /// Añadir dependiente inverso (para invalidación en cascada)
    fn add_dependent(&mut self, asset_id: AssetId, dependent: AssetId) -> ReactorResult<()> {
        if let Some(mut meta) = self.get_metadata(asset_id)? {
            if !meta.dependents.contains(&dependent) {
                meta.dependents.push(dependent);
                self.register_asset(asset_id, meta)?;
            }
        }
        Ok(())
    }

    /// Invalidar asset y todos sus dependientes (para hot-reload en cascada)
    pub fn invalidate_with_dependents(&mut self, id: AssetId) -> ReactorResult<Vec<AssetId>> {
        let mut invalidated = Vec::new();

        if let Some(meta) = self.get_metadata(id)? {
            // Invalidar dependientes recursivamente
            for dep_id in &meta.dependents {
                invalidated.extend(self.invalidate_with_dependents(*dep_id)?);
            }

            // Remover de cache
            self.memory_cache.remove(&id);

            // Remover de DB
            let key = format!("meta:{:016x}", id.as_u64());
            self.db.remove(key.as_bytes())?;

            invalidated.push(id);
        }

        Ok(invalidated)
    }

    /// Listar todos los assets registrados
    pub fn list_assets(&self) -> ReactorResult<Vec<(AssetId, AssetMetadata)>> {
        let mut assets = Vec::new();

        for item in self.db.iter() {
            let (key, value) = item?;
            let key_str = std::str::from_utf8(&key).unwrap_or("");

            if key_str.starts_with("meta:") {
                if let Ok(id_str) = key_str
                    .strip_prefix("meta:")
                    .ok_or_else(|| ReactorError::asset_load("Invalid metadata key format"))
                {
                    if let Ok(id_val) = u64::from_str_radix(id_str, 16) {
                        let id = AssetId::from(id_val);
                        if let Ok(meta) = serde_json::from_slice::<AssetMetadata>(&value) {
                            assets.push((id, meta));
                        }
                    }
                }
            }
        }

        Ok(assets)
    }

    /// Obtener estadísticas de la database
    pub fn stats(&self) -> AssetDbStats {
        AssetDbStats {
            cached_in_memory: self.memory_cache.len(),
            total_entries: self.db.len(),
            size_on_disk: self.db.size_on_disk().unwrap_or(0) as usize,
        }
    }

    /// Compactar la database (reclaim space from deleted entries)
    pub fn compact(&self) -> ReactorResult<bool> {
        self.db.flush()?;
        Ok(true)
    }

    /// Exportar metadata a JSON (para debugging o backup)
    pub fn export_json(&self) -> ReactorResult<String> {
        let mut assets = Vec::new();

        for item in self.db.iter() {
            let (_, value) = item?;
            if let Ok(meta) = serde_json::from_slice::<AssetMetadata>(&value) {
                assets.push(meta);
            }
        }

        serde_json::to_string_pretty(&assets)
            .map_err(|e| ReactorError::asset_load(format!("Failed to serialize: {}", e)))
    }

    /// Importar metadata desde JSON
    pub fn import_json(&mut self, json: &str) -> ReactorResult<usize> {
        let assets: Vec<AssetMetadata> = serde_json::from_str(json)
            .map_err(|e| ReactorError::asset_load(format!("Failed to parse JSON: {}", e)))?;

        let mut count = 0;
        for meta in assets {
            let id = AssetId::from_path(&meta.source_path);
            self.register_asset(id, meta)?;
            count += 1;
        }

        Ok(count)
    }
}

/// Estadísticas de la AssetDatabase
#[derive(Clone, Debug)]
pub struct AssetDbStats {
    pub cached_in_memory: usize,
    pub total_entries: usize,
    pub size_on_disk: usize,
}

// =============================================================================
// Helpers para crear metadata
// =============================================================================

impl AssetMetadata {
    /// Crear metadata básica desde un path de archivo
    pub fn from_path<P: AsRef<Path>>(path: P) -> ReactorResult<Self> {
        let path = path.as_ref();
        let meta = std::fs::metadata(path).map_err(|e| {
            ReactorError::asset_load(format!("Failed to stat {}: {}", path.display(), e))
        })?;

        let content = std::fs::read(path).map_err(|e| {
            ReactorError::asset_load(format!("Failed to read {}: {}", path.display(), e))
        })?;

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        Ok(Self {
            source_path: path.to_string_lossy().to_string(),
            content_hash: xxhash_rust::xxh3::xxh3_64(&content),
            last_modified: meta
                .modified()
                .unwrap_or(UNIX_EPOCH)
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            file_size: meta.len(),
            asset_type: AssetType::from_extension(&ext),
            source_format: ext,
            runtime_format: None,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            extra: HashMap::new(),
        })
    }

    /// Añadir metadata extra
    pub fn with_extra(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }

    /// Establecer formato runtime optimizado
    pub fn with_runtime_format(mut self, format: impl Into<String>) -> Self {
        self.runtime_format = Some(format.into());
        self
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("png"), AssetType::Texture);
        assert_eq!(AssetType::from_extension("GLTF"), AssetType::Model);
        assert_eq!(AssetType::from_extension("unknown"), AssetType::Unknown);
    }

    #[test]
    fn test_database_basic() -> ReactorResult<()> {
        let mut db = AssetDatabase::in_memory()?;

        let id = AssetId::from_path("test.png");
        let meta = AssetMetadata {
            source_path: "test.png".into(),
            content_hash: 12345,
            last_modified: 0,
            file_size: 1024,
            asset_type: AssetType::Texture,
            source_format: "png".into(),
            runtime_format: Some("ktx2".into()),
            dependencies: Vec::new(),
            dependents: Vec::new(),
            extra: HashMap::new(),
        };

        db.register_asset(id, meta.clone())?;

        let retrieved = db.get_metadata(id)?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content_hash, 12345);

        Ok(())
    }
}
