//! # PSO Cache Manager — Persistencia en disco + Vulkan PipelineCache nativo
//!
//! ## Capas de cacheo
//!
//! 1. **In-memory HashMap** (`HashMap<PsoHash, CachedPipeline>`) — acceso O(1) por frame.
//! 2. **Vulkan PipelineCache nativo** (`vk::PipelineCache`) — el driver optimiza
//!    creación de pipelines con estado interno.
//! 3. **Disco** (`.reactor/pipeline_cache.bin`) — sobrevive a reinicios.
//!
//! ## Hot-reload
//!
//! Cuando un shader cambia en disco (se detecta por `spirv_hash`), el PSO
//! se invalida automáticamente y se recompila en el siguiente `get_or_create`.

use ash::vk;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::core::arc_handle::ArcDevice;
use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::graphics::pso_hash::PsoHash;

// ═══════════════════════════════════════════════════════════════════════════
// Constantes del formato de archivo
// ═══════════════════════════════════════════════════════════════════════════

/// Magic bytes: "PSOC" (Pipeline State Object Cache).
const PSO_CACHE_MAGIC: u32 = 0x50534F43;
/// Versión del formato. Bump cuando cambie la estructura.
const PSO_CACHE_VERSION: u32 = 2;

// ═══════════════════════════════════════════════════════════════════════════
// Tipos serializables
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize)]
struct PsoCacheHeader {
    magic: u32,
    version: u32,
    entry_count: u32,
    /// Hash del device (vendor + device ID) para invalidar cache entre GPUs.
    device_hash: u64,
}

/// Entrada serializable de un PSO en disco.
#[derive(Serialize, Deserialize, Clone)]
pub struct SerializablePsoEntry {
    /// Hash completo del PSO (shader + state).
    pub hash: u64,
    /// SPIR-V del vertex shader.
    pub vertex_spirv: Vec<u32>,
    /// SPIR-V del fragment shader.
    pub fragment_spirv: Vec<u32>,
    /// Hash del vertex SPIR-V (para invalidación rápida).
    pub vertex_spirv_hash: u64,
    /// Hash del fragment SPIR-V (para invalidación rápida).
    pub fragment_spirv_hash: u64,
    /// Samples de MSAA.
    pub msaa_samples: u32,
    /// Formato de color attachment.
    pub color_format: u32,
    /// Formato de depth attachment (0 = UNDEFINED).
    pub depth_format: u32,
    /// Pipeline state flags (cull mode, polygon mode, depth test, etc).
    pub state_flags: u32,
}

// ═══════════════════════════════════════════════════════════════════════════
// CachedPipeline
// ═══════════════════════════════════════════════════════════════════════════

/// Pipeline cacheado (Vulkan handles).
#[derive(Clone)]
pub struct CachedPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

// ═══════════════════════════════════════════════════════════════════════════
// PsoCacheManager (I/O a disco)
// ═══════════════════════════════════════════════════════════════════════════

pub struct PsoCacheManager {
    cache_file: PathBuf,
    device_hash: u64,
}

impl PsoCacheManager {
    pub fn new(cache_dir: &Path, device_hash: u64) -> io::Result<Self> {
        fs::create_dir_all(cache_dir)?;
        Ok(Self {
            cache_file: cache_dir.join("pipeline_cache.bin"),
            device_hash,
        })
    }

    /// Carga todas las entradas desde disco.
    pub fn load(&self) -> io::Result<Vec<SerializablePsoEntry>> {
        if !self.cache_file.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(&self.cache_file)?;
        let mut reader = BufReader::new(file);
        let header: PsoCacheHeader = bincode::deserialize_from(&mut reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Validar magic + version
        if header.magic != PSO_CACHE_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid PSO cache magic",
            ));
        }
        if header.version != PSO_CACHE_VERSION {
            // Versión incompatible → descartar cache silenciosamente
            return Ok(Vec::new());
        }
        // Si cambió la GPU, descartar
        if header.device_hash != self.device_hash {
            return Ok(Vec::new());
        }

        let mut entries = Vec::with_capacity(header.entry_count as usize);
        for _ in 0..header.entry_count {
            entries.push(
                bincode::deserialize_from(&mut reader)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            );
        }
        Ok(entries)
    }

    /// Guarda todas las entradas a disco.
    pub fn save(&self, entries: &[SerializablePsoEntry]) -> io::Result<()> {
        let file = fs::File::create(&self.cache_file)?;
        let mut writer = BufWriter::new(file);
        let header = PsoCacheHeader {
            magic: PSO_CACHE_MAGIC,
            version: PSO_CACHE_VERSION,
            entry_count: entries.len() as u32,
            device_hash: self.device_hash,
        };
        bincode::serialize_into(&mut writer, &header)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        for e in entries {
            bincode::serialize_into(&mut writer, e)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
        writer.flush()
    }

    pub fn clear(&self) -> io::Result<()> {
        if self.cache_file.exists() {
            fs::remove_file(&self.cache_file)?;
        }
        Ok(())
    }

    pub fn cache_file(&self) -> &Path {
        &self.cache_file
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PsoCache (wrapper con Vulkan PipelineCache + in-memory + disco)
// ═══════════════════════════════════════════════════════════════════════════

pub struct PsoCache {
    manager: PsoCacheManager,
    pipelines: Arc<Mutex<HashMap<PsoHash, CachedPipeline>>>,
    /// Vulkan PipelineCache nativo — el driver lo usa para acelerar
    /// la creación de pipelines con estado interno reutilizable.
    vk_pipeline_cache: vk::PipelineCache,
    device: ArcDevice,
}

impl PsoCache {
    pub fn new(device: ArcDevice, cache_dir: &Path, device_hash: u64) -> ReactorResult<Self> {
        let manager = PsoCacheManager::new(cache_dir, device_hash).map_err(|e| {
            ReactorError::with_source(ErrorCode::IoError, "PSO cache init failed", e)
        })?;

        // ── Load Vulkan native pipeline cache from disk (warm-up) ──
        // The driver uses this blob to skip recompilation of previously seen
        // pipeline states, eliminating first-frame stutter.
        let vkcache_path = manager.cache_file().with_extension("vkcache");
        let initial_data = if vkcache_path.exists() {
            match fs::read(&vkcache_path) {
                Ok(data) => {
                    log::info!(
                        "⚡ Pipeline cache loaded from disk ({} KB)",
                        data.len() / 1024
                    );
                    data
                }
                Err(e) => {
                    log::warn!("⚠ Failed to read pipeline cache from disk: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        let cache_info = if initial_data.is_empty() {
            vk::PipelineCacheCreateInfo::default()
        } else {
            vk::PipelineCacheCreateInfo::default()
                .initial_data(&initial_data)
        };

        let vk_pipeline_cache = unsafe {
            device
                .create_pipeline_cache(&cache_info, None)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanPipelineCreation,
                        "Failed to create Vulkan PipelineCache",
                        e,
                    )
                })?
        };

        if !initial_data.is_empty() {
            log::info!("⚡ Pipeline warm-up: driver cache pre-seeded");
        }

        Ok(Self {
            manager,
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            vk_pipeline_cache,
            device,
        })
    }

    /// Retorna el `VkPipelineCache` para usar en `vkCreateGraphicsPipelines`.
    #[inline]
    pub fn vk_pipeline_cache(&self) -> vk::PipelineCache {
        self.vk_pipeline_cache
    }

    /// Obtiene un PSO cacheado o lo crea usando el closure.
    pub fn get_or_create<F>(&self, hash: &PsoHash, create: F) -> ReactorResult<CachedPipeline>
    where
        F: FnOnce() -> ReactorResult<CachedPipeline>,
    {
        // Fast path: ya está en memoria
        {
            let p = self.pipelines.lock().unwrap();
            if let Some(c) = p.get(hash) {
                return Ok(c.clone());
            }
        }
        // Slow path: crear + insertar
        let pipeline = create()?;
        self.pipelines
            .lock()
            .unwrap()
            .insert(*hash, pipeline.clone());
        Ok(pipeline)
    }

    /// Cantidad de PSOs actualmente en memoria.
    pub fn len(&self) -> usize {
        self.pipelines.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.pipelines.lock().unwrap().is_empty()
    }

    /// Guarda el Vulkan PipelineCache nativo a disco (llamar al shutdown).
    pub fn save_vk_cache(&self) -> ReactorResult<()> {
        let data = unsafe {
            self.device
                .get_pipeline_cache_data(self.vk_pipeline_cache)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::IoError,
                        "Failed to get Vulkan pipeline cache data",
                        e,
                    )
                })?
        };
        let path = self.manager.cache_file().with_extension("vkcache");
        fs::write(&path, &data).map_err(|e| {
            ReactorError::with_source(
                ErrorCode::IoError,
                "Failed to write Vulkan pipeline cache",
                e,
            )
        })?;
        Ok(())
    }

    /// Limpia cache en memoria + disco.
    pub fn clear(&self) -> ReactorResult<()> {
        self.pipelines.lock().unwrap().clear();
        self.manager
            .clear()
            .map_err(|e| ReactorError::with_source(ErrorCode::IoError, "PSO cache clear failed", e))
    }

    /// Persiste una lista de entradas serializables a disco.
    pub fn save_entries(&self, entries: &[SerializablePsoEntry]) -> ReactorResult<()> {
        self.manager
            .save(entries)
            .map_err(|e| ReactorError::with_source(ErrorCode::IoError, "PSO cache save failed", e))
    }

    /// Carga entradas desde disco.
    pub fn load_entries(&self) -> ReactorResult<Vec<SerializablePsoEntry>> {
        self.manager
            .load()
            .map_err(|e| ReactorError::with_source(ErrorCode::IoError, "PSO cache load failed", e))
    }
}

impl Drop for PsoCache {
    fn drop(&mut self) {
        // Intentar guardar el Vulkan PipelineCache antes de destruirlo
        let _ = self.save_vk_cache();
        unsafe {
            self.device
                .destroy_pipeline_cache(self.vk_pipeline_cache, None);
        }
    }
}
