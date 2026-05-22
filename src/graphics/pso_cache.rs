//! PSO Cache Manager - Persistencia en disco
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Write, BufReader, BufWriter};
use ash::vk;
use serde::{Serialize, Deserialize};
use crate::core::error::{ReactorError, ReactorResult, ErrorCode};
use crate::graphics::pso_hash::PsoHash;

const PSO_CACHE_MAGIC: u32 = 0x50534F43;
const PSO_CACHE_VERSION: u32 = 1;

#[derive(Clone)]
pub struct CachedPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

#[derive(Serialize, Deserialize)]
struct PsoCacheHeader { magic: u32, version: u32, entry_count: u32 }

#[derive(Serialize, Deserialize, Clone)]
pub struct SerializablePsoEntry {
    pub hash: u64,
    pub vertex_spirv: Vec<u32>,
    pub fragment_spirv: Vec<u32>,
    pub msaa_samples: u32,
    pub color_format: u32,
    pub depth_format: u32,
}

pub struct PsoCacheManager {
    cache_file: PathBuf,
}

impl PsoCacheManager {
    pub fn new(cache_dir: &Path) -> io::Result<Self> {
        fs::create_dir_all(cache_dir)?;
        Ok(Self { cache_file: cache_dir.join("pipeline_cache.bin") })
    }

    pub fn load(&self) -> io::Result<Vec<SerializablePsoEntry>> {
        if !self.cache_file.exists() { return Ok(Vec::new()); }
        let file = fs::File::open(&self.cache_file)?;
        let mut reader = BufReader::new(file);
        let header: PsoCacheHeader = bincode::deserialize_from(&mut reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if header.magic != PSO_CACHE_MAGIC || header.version != PSO_CACHE_VERSION {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid PSO cache"));
        }
        let mut entries = Vec::with_capacity(header.entry_count as usize);
        for _ in 0..header.entry_count {
            entries.push(bincode::deserialize_from(&mut reader)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?);
        }
        Ok(entries)
    }

    pub fn save(&self, entries: &[SerializablePsoEntry]) -> io::Result<()> {
        let file = fs::File::create(&self.cache_file)?;
        let mut writer = BufWriter::new(file);
        let header = PsoCacheHeader { magic: PSO_CACHE_MAGIC, version: PSO_CACHE_VERSION, entry_count: entries.len() as u32 };
        bincode::serialize_into(&mut writer, &header).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        for e in entries { bincode::serialize_into(&mut writer, e).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; }
        writer.flush()
    }

    pub fn clear(&self) -> io::Result<()> {
        if self.cache_file.exists() { fs::remove_file(&self.cache_file)?; }
        Ok(())
    }
}

pub struct PsoCache {
    manager: PsoCacheManager,
    pipelines: Arc<Mutex<HashMap<PsoHash, CachedPipeline>>>,
    device: Arc<ash::Device>,
}

impl PsoCache {
    pub fn new(device: Arc<ash::Device>, cache_dir: &Path) -> ReactorResult<Self> {
        let manager = PsoCacheManager::new(cache_dir)
            .map_err(|e| ReactorError::with_source(ErrorCode::IoError, "PSO cache init failed", e))?;
        Ok(Self {
            manager,
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            device,
        })
    }

    pub fn get_or_create<F>(&self, hash: &PsoHash, create: F) -> ReactorResult<CachedPipeline>
    where F: FnOnce() -> ReactorResult<CachedPipeline> {
        { let p = self.pipelines.lock().unwrap(); if let Some(c) = p.get(hash) { return Ok(c.clone()); } }
        let pipeline = create()?;
        self.pipelines.lock().unwrap().insert(*hash, pipeline.clone());
        Ok(pipeline)
    }

    pub fn clear(&self) -> ReactorResult<()> {
        self.pipelines.lock().unwrap().clear();
        self.manager.clear()
            .map_err(|e| ReactorError::with_source(ErrorCode::IoError, "PSO cache clear failed", e))
    }
}
