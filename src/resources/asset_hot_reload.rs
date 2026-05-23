// =============================================================================
// AssetHotReload — Sistema de recarga automática de assets en caliente
// =============================================================================
// Usa `notify` para watch de filesystem y recarga assets cuando cambian.
// - Detecta cambios en texturas, modelos, shaders
// - Notifica via eventos al engine para re-bind de recursos
// - Soporta debounce para evitar reloads múltiples por save atómico
// =============================================================================

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::core::error::{ReactorResult, ReactorError};
use crate::resources::asset_id::AssetId;
use crate::resources::handle::Handle;
use crate::resources::asset_database::{AssetType, AssetMetadata};

/// Evento emitido cuando un asset es recargado
#[derive(Debug, Clone)]
pub enum AssetReloadEvent {
    /// Asset fue recargado exitosamente
    Reloaded {
        id: AssetId,
        path: PathBuf,
        timestamp: Instant,
    },
    /// Falló al recargar el asset (pero el anterior sigue válido)
    ReloadFailed {
        id: AssetId,
        path: PathBuf,
        error: String,
        timestamp: Instant,
    },
    /// Asset nuevo detectado (primera carga)
    AssetDiscovered {
        id: AssetId,
        path: PathBuf,
        asset_type: AssetType,
    },
    /// Asset eliminado del filesystem
    AssetRemoved {
        id: AssetId,
        path: PathBuf,
        timestamp: Instant,
    },
}

/// Configuración del watcher de hot-reload
#[derive(Clone, Debug)]
pub struct HotReloadConfig {
    /// Directorios a watch (recursivo)
    pub watch_dirs: Vec<PathBuf>,
    /// Extensions a monitorizar (empty = todas)
    pub extensions: HashSet<String>,
    /// Tiempo de debounce para evitar múltiples eventos por save atómico
    pub debounce_duration: Duration,
    /// Ignorar paths que contengan estos substrings
    pub ignore_patterns: Vec<String>,
    /// Habilitar reload automático o solo notificar
    pub auto_reload: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            watch_dirs: vec![PathBuf::from("assets")],
            extensions: HashSet::from([
                "png".into(), "jpg".into(), "jpeg".into(), "bmp".into(), "tga".into(),
                "gltf".into(), "glb".into(), "obj".into(), "fbx".into(),
                "vert".into(), "frag".into(), "comp".into(), "spv".into(), "wgsl".into(),
                "wav".into(), "mp3".into(), "ogg".into(),
                "ktx2".into(), "dds".into(),
            ]),
            debounce_duration: Duration::from_millis(250),
            ignore_patterns: vec![".git".into(), "target".into(), "tmp".into(), ".tmp".into()],
            auto_reload: true,
        }
    }
}

/// Estado interno para tracking de assets con hot-reload
#[derive(Clone)]
struct TrackedAsset {
    path: PathBuf,
    asset_type: AssetType,
    last_mtime: u64,
    last_hash: u64,
    reload_count: u32,
}

/// Shared state between manager and watcher callback
struct SharedState {
    tracked_assets: HashMap<AssetId, TrackedAsset>,
    path_to_id: HashMap<PathBuf, AssetId>,
}

/// Manager principal para hot-reload de assets
pub struct AssetHotReloadManager {
    config: HotReloadConfig,
    /// Shared state (accessed by both manager and watcher thread)
    shared: Arc<Mutex<SharedState>>,
    /// Channel para emitir eventos al engine
    event_tx: UnboundedSender<AssetReloadEvent>,
    /// Watcher de filesystem (notify)
    watcher: Option<RecommendedWatcher>,
    /// Shutdown signal
    shutdown: Arc<Mutex<bool>>,
    /// AssetDatabase para metadata persistente
    asset_db: Option<Arc<Mutex<crate::resources::asset_database::AssetDatabase>>>,
}

impl AssetHotReloadManager {
    /// Crear nuevo manager con config y channel de eventos
    pub fn new(
        config: HotReloadConfig,
        event_tx: UnboundedSender<AssetReloadEvent>,
    ) -> ReactorResult<Self> {
        let shared = Arc::new(Mutex::new(SharedState {
            tracked_assets: HashMap::new(),
            path_to_id: HashMap::new(),
        }));

        let mut manager = Self {
            config: config.clone(),
            shared,
            event_tx,
            watcher: None,
            shutdown: Arc::new(Mutex::new(false)),
            asset_db: None,
        };

        // Inicializar watcher
        manager.start_watching()?;

        Ok(manager)
    }

    /// Vincular con AssetDatabase para metadata persistente
    pub fn with_asset_db(mut self, db: Arc<Mutex<crate::resources::asset_database::AssetDatabase>>) -> Self {
        self.asset_db = Some(db);
        self
    }

    /// Iniciar filesystem watcher en background thread
    fn start_watching(&mut self) -> ReactorResult<()> {
        let tx = self.event_tx.clone();
        let shutdown = self.shutdown.clone();
        let config = self.config.clone();
        let shared = Arc::clone(&self.shared);
        
        // Callback para eventos de filesystem
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if *shutdown.lock().unwrap() {
                return;
            }
            
            match res {
                Ok(event) => Self::handle_filesystem_event(&event, &config, &tx, &shared),
                Err(e) => eprintln!("[AssetHotReload] Watcher error: {}", e),
            }
        }).map_err(|e| ReactorError::asset_load(format!("Failed to create watcher: {}", e)))?;

        // Registrar directorios a watch
        for dir in &self.config.watch_dirs {
            if dir.exists() {
                watcher.watch(dir, RecursiveMode::Recursive)
                    .map_err(|e| ReactorError::asset_load(
                        format!("Failed to watch directory {}: {}", dir.display(), e)
                    ))?;
                println!("[AssetHotReload] Watching: {}", dir.display());
            }
        }

        self.watcher = Some(watcher);
        Ok(())
    }

    /// Procesar evento de filesystem con debounce y filtrado
    fn handle_filesystem_event(
        event: &Event,
        config: &HotReloadConfig,
        tx: &UnboundedSender<AssetReloadEvent>,
        shared: &Arc<Mutex<SharedState>>,
    ) {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                for path in &event.paths {
                    Self::process_create_or_modify(path, config, tx, shared);
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    Self::process_remove(path, config, tx, shared);
                }
            }
            _ => {} // Ignorar otros eventos
        }
    }

    fn process_create_or_modify(
        path: &PathBuf,
        config: &HotReloadConfig,
        tx: &UnboundedSender<AssetReloadEvent>,
        shared: &Arc<Mutex<SharedState>>,
    ) {
        // Filtrar por extensión
        if !config.extensions.is_empty() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !config.extensions.contains(ext) {
                    return;
                }
            } else {
                return;
            }
        }

        // Filtrar por ignore patterns
        if config.ignore_patterns.iter().any(|pat| {
            path.to_string_lossy().contains(pat.as_str())
        }) {
            return;
        }

        let asset_id = AssetId::from_path(path);
        let asset_type = path.extension()
            .and_then(|e| e.to_str())
            .map_or(AssetType::Unknown, AssetType::from_extension);

        // Verificar si ya está trackeado
        let is_new = {
            let state = shared.lock().unwrap();
            !state.tracked_assets.contains_key(&asset_id)
        };

        let event = if is_new {
            AssetReloadEvent::AssetDiscovered {
                id: asset_id,
                path: path.clone(),
                asset_type,
            }
        } else {
            AssetReloadEvent::Reloaded {
                id: asset_id,
                path: path.clone(),
                timestamp: Instant::now(),
            }
        };

        let _ = tx.send(event);
    }

    fn process_remove(
        path: &PathBuf,
        config: &HotReloadConfig,
        tx: &UnboundedSender<AssetReloadEvent>,
        shared: &Arc<Mutex<SharedState>>,
    ) {
        if !config.extensions.is_empty() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !config.extensions.contains(ext) {
                    return;
                }
            } else {
                return;
            }
        }

        let asset_id = AssetId::from_path(path);
        
        // Remover del tracking
        {
            let mut state = shared.lock().unwrap();
            state.tracked_assets.remove(&asset_id);
            state.path_to_id.remove(path);
        }

        let _ = tx.send(AssetReloadEvent::AssetRemoved {
            id: asset_id,
            path: path.clone(),
            timestamp: Instant::now(),
        });
    }

    /// Registrar un asset para tracking de hot-reload
    pub fn track_asset<P: AsRef<Path>>(
        &self,
        id: AssetId,
        path: P,
        asset_type: AssetType,
    ) -> ReactorResult<()> {
        let path = path.as_ref().to_path_buf();
        let (mtime, hash) = Self::get_file_info(&path)?;
        
        let tracked = TrackedAsset {
            path: path.clone(),
            asset_type,
            last_mtime: mtime,
            last_hash: hash,
            reload_count: 0,
        };
        
        let mut state = self.shared.lock()
            .map_err(|_| ReactorError::internal("Mutex poison"))?;
        
        state.tracked_assets.insert(id, tracked);
        state.path_to_id.insert(path, id);
        
        Ok(())
    }

    /// Detener tracking de un asset
    pub fn untrack_asset(&self, id: AssetId) {
        if let Ok(mut state) = self.shared.lock() {
            if let Some(tracked_asset) = state.tracked_assets.remove(&id) {
                state.path_to_id.remove(&tracked_asset.path);
            }
        }
    }

    /// Verificar si un asset ha cambiado desde la última vez
    pub fn has_changed(&self, id: AssetId) -> ReactorResult<bool> {
        let state = self.shared.lock()
            .map_err(|_| ReactorError::internal("Mutex poison"))?;
        
        if let Some(tracked_asset) = state.tracked_assets.get(&id) {
            let (current_mtime, current_hash) = Self::get_file_info(&tracked_asset.path)?;
            Ok(current_mtime != tracked_asset.last_mtime || current_hash != tracked_asset.last_hash)
        } else {
            Ok(false)
        }
    }

    /// Forzar recarga de un asset específico (llamado por el engine al recibir evento)
    pub async fn reload_asset<T, F, Fut>(
        &self,
        id: AssetId,
        loader: F,
    ) -> ReactorResult<Handle<T>>
    where
        F: FnOnce(&Path) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ReactorResult<T>> + Send + 'static,
        T: Send + Sync + 'static,
    {
        let path = {
            let state = self.shared.lock()
                .map_err(|_| ReactorError::internal("Mutex poison"))?;
            state.tracked_assets.get(&id)
                .map(|t| t.path.clone())
                .ok_or_else(|| ReactorError::asset_load("Asset not tracked"))?
        };

        // Ejecutar loader
        let new_asset = loader(&path).await?;
        
        // Actualizar metadata
        let (new_mtime, new_hash) = Self::get_file_info(&path)?;
        {
            let mut state = self.shared.lock()
                .map_err(|_| ReactorError::internal("Mutex poison"))?;
            if let Some(entry) = state.tracked_assets.get_mut(&id) {
                entry.last_mtime = new_mtime;
                entry.last_hash = new_hash;
                entry.reload_count += 1;
            }
        }

        Ok(Handle::new(id, new_asset))
    }

    /// Obtener receiver para escuchar eventos de reload
    pub fn event_sender(&self) -> UnboundedSender<AssetReloadEvent> {
        self.event_tx.clone()
    }

    /// Obtener estadísticas de tracking
    pub fn stats(&self) -> HotReloadStats {
        let state = self.shared.lock().unwrap();
        HotReloadStats {
            tracked_count: state.tracked_assets.len(),
            total_reloads: state.tracked_assets.values().map(|t| t.reload_count).sum(),
        }
    }

    /// Shutdown limpio del watcher
    pub fn shutdown(&mut self) {
        *self.shutdown.lock().unwrap() = true;
        self.watcher.take();
    }

    /// Helper: obtener mtime y hash de archivo
    fn get_file_info(path: &Path) -> ReactorResult<(u64, u64)> {
        use xxhash_rust::xxh3::xxh3_64;
        
        let metadata = std::fs::metadata(path)
            .map_err(|e| ReactorError::asset_load(
                format!("Failed to stat {}: {}", path.display(), e)
            ))?;
        
        let mtime = metadata.modified()
            .map_err(|e| ReactorError::asset_load(
                format!("Failed to get mtime for {}: {}", path.display(), e)
            ))?
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|e| ReactorError::asset_load(
                format!("Invalid timestamp for {}: {}", path.display(), e)
            ))?;
        
        let content = std::fs::read(path)
            .map_err(|e| ReactorError::asset_load(
                format!("Failed to read {}: {}", path.display(), e)
            ))?;
        let hash = xxh3_64(&content);
        
        Ok((mtime, hash))
    }
}

impl Drop for AssetHotReloadManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Estadísticas del hot-reload manager
#[derive(Clone, Debug, Default)]
pub struct HotReloadStats {
    pub tracked_count: usize,
    pub total_reloads: u32,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;

    #[test]
    fn test_asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("png"), AssetType::Texture);
        assert_eq!(AssetType::from_extension("GLTF"), AssetType::Model);
        assert_eq!(AssetType::from_extension("unknown"), AssetType::Unknown);
    }
}
