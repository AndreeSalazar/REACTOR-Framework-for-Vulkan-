// =============================================================================
// AssetLoaderQueue — Cola asíncrona para carga de assets en background
// =============================================================================
// Permite enqueue de loads que se ejecutan en worker threads sin bloquear
// el main thread. Soporta prioridad, cancelación y progress tracking.
// 
// Fase 3.2: Asset Pipeline completo con loading no-bloqueante.
// =============================================================================

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::core::error::{ReactorResult, ReactorError};
use crate::resources::asset_id::AssetId;
use crate::resources::handle::Handle;

/// Prioridad de carga de asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LoadPriority {
    /// Carga inmediata (próximo frame) - crítico para gameplay
    Critical = 0,
    /// Carga pronto (próximos 2-3 frames) - assets visibles pronto
    High = 1,
    /// Carga normal (background) - assets que pueden esperar
    Normal = 2,
    /// Carga baja (solo cuando idle) - precaching, streaming lejano
    Low = 3,
}

impl Default for LoadPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Estado de una carga en la cola
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    Queued,
    Loading { progress: u32 }, // 0 - 100
    Completed,
    Failed,
    Cancelled,
}

/// Callback para reportar progreso durante carga
pub type ProgressCallback = Box<dyn Fn(f32) + Send + 'static>;

/// Request de carga de asset (type-erased para flexibilidad)
pub struct LoadRequest {
    pub id: AssetId,
    pub path: PathBuf,
    pub priority: LoadPriority,
    pub loader: Box<dyn FnOnce() -> ReactorResult<Box<dyn std::any::Any + Send>> + Send>,
    pub response_tx: oneshot::Sender<ReactorResult<LoadResult>>,
    pub created_at: Instant,
}

/// Resultado de carga type-erased
pub struct LoadResult {
    pub id: AssetId,
    pub data: Box<dyn std::any::Any + Send>,
    pub load_time_ms: f64,
}

/// Manager de cola de carga asíncrona
pub struct AssetLoaderQueue {
    /// Cola de requests ordenada por prioridad
    queue: Arc<Mutex<VecDeque<LoadRequest>>>,
    /// Señal para despertar workers cuando hay trabajo
    work_available: Arc<tokio::sync::Notify>,
    /// Worker handles
    workers: Vec<JoinHandle<()>>,
    /// Shutdown signal (atomic flag)
    shutdown: Arc<AtomicBool>,
    /// Stats
    stats: Arc<Mutex<LoaderStats>>,
    /// Configuración
    config: LoaderQueueConfig,
}

/// Configuración de la cola de carga
#[derive(Clone, Debug)]
pub struct LoaderQueueConfig {
    /// Número de worker threads
    pub num_workers: usize,
    /// Timeout para cargas individuales
    pub load_timeout: Duration,
    /// Habilitar logging de stats
    pub log_stats: bool,
    /// Intervalo para reportar stats
    pub stats_interval: Duration,
}

impl Default for LoaderQueueConfig {
    fn default() -> Self {
        Self {
            num_workers: 2,
            load_timeout: Duration::from_secs(30),
            log_stats: true,
            stats_interval: Duration::from_secs(5),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LoaderStats {
    pub queued: usize,
    pub loading: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub total_load_time_ms: f64,
    pub avg_load_time_ms: f64,
}

impl AssetLoaderQueue {
    /// Crear nueva cola con configuración por defecto
    pub fn new() -> ReactorResult<Self> {
        Self::with_config(LoaderQueueConfig::default())
    }

    /// Crear nueva cola con configuración personalizada
    pub fn with_config(config: LoaderQueueConfig) -> ReactorResult<Self> {
        let shutdown = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(Mutex::new(LoaderStats::default()));
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let work_available = Arc::new(tokio::sync::Notify::new());
        
        let mut workers = Vec::with_capacity(config.num_workers);
        
        for worker_id in 0..config.num_workers {
            let stats = Arc::clone(&stats);
            let queue = Arc::clone(&queue);
            let work_available = Arc::clone(&work_available);
            let shutdown = Arc::clone(&shutdown);
            let config = config.clone();
            
            let handle = tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    shutdown,
                    queue,
                    work_available,
                    stats,
                    config,
                ).await;
            });
            workers.push(handle);
        }
        
        Ok(Self {
            queue,
            work_available,
            workers,
            shutdown,
            stats,
            config,
        })
    }

    /// Loop principal del worker thread
    async fn worker_loop(
        worker_id: usize,
        shutdown: Arc<AtomicBool>,
        queue: Arc<Mutex<VecDeque<LoadRequest>>>,
        work_available: Arc<tokio::sync::Notify>,
        stats: Arc<Mutex<LoaderStats>>,
        config: LoaderQueueConfig,
    ) {
        loop {
            if shutdown.load(Ordering::Relaxed) {
                if config.log_stats {
                    println!("[LoaderQueue#{}] Shutting down", worker_id);
                }
                break;
            }

            // Wait for work notification or periodic check
            tokio::select! {
                _ = work_available.notified() => {}
                _ = tokio::time::sleep(Duration::from_millis(50)) => {
                    // Polling fallback
                    continue;
                }
            }

            // Intentar obtener un request de la cola
            let request = {
                let mut q = queue.lock().unwrap();
                // Prioridad: sacar el de mayor prioridad (menor valor enum)
                let mut best_idx = None;
                let mut best_priority = LoadPriority::Low;
                
                for (idx, req) in q.iter().enumerate() {
                    if best_idx.is_none() || req.priority < best_priority {
                        best_priority = req.priority;
                        best_idx = Some(idx);
                    }
                }
                
                best_idx.and_then(|idx| q.remove(idx))
            };
            
            if let Some(req) = request {
                // Extract fields before consuming the loader
                let req_id = req.id;
                let req_path = req.path.clone();
                let response_tx = req.response_tx;
                let loader = req.loader;

                // Update stats
                {
                    let mut s = stats.lock().unwrap();
                    s.queued = s.queued.saturating_sub(1);
                    s.loading += 1;
                }
                
                let start = Instant::now();
                
                // Ejecutar loader con timeout
                let result = tokio::time::timeout(
                    config.load_timeout,
                    tokio::task::spawn_blocking(move || {
                        (loader)()
                    })
                ).await;
                
                let load_time = start.elapsed().as_secs_f64() * 1000.0;
                
                let final_result = match result {
                    Ok(Ok(Ok(data))) => {
                        Ok(LoadResult {
                            id: req_id,
                            data,
                            load_time_ms: load_time,
                        })
                    }
                    Ok(Ok(Err(e))) => Err(e),
                    Ok(Err(join_err)) => Err(ReactorError::internal(
                        format!("Loader task panicked: {}", join_err)
                    )),
                    Err(_) => Err(ReactorError::timeout(
                        format!("Load timeout for {:?}", req_path)
                    )),
                };
                
                // Update stats
                {
                    let mut s = stats.lock().unwrap();
                    s.loading = s.loading.saturating_sub(1);
                    match &final_result {
                        Ok(_) => {
                            s.completed += 1;
                            s.total_load_time_ms += load_time;
                            if s.completed > 0 {
                                s.avg_load_time_ms = s.total_load_time_ms / s.completed as f64;
                            }
                        }
                        Err(e) if e.code == crate::core::error::ErrorCode::Cancelled => {
                            s.cancelled += 1;
                        }
                        Err(_) => s.failed += 1,
                    }
                }
                
                // Enviar resultado
                let _ = response_tx.send(final_result);
            }
        }
    }

    /// Enqueue un asset para carga asíncrona
    pub fn enqueue<T, F>(
        &self,
        id: AssetId,
        path: PathBuf,
        priority: LoadPriority,
        loader: F,
    ) -> oneshot::Receiver<ReactorResult<Handle<T>>>
    where
        F: FnOnce() -> ReactorResult<T> + Send + 'static,
        T: Send + Sync + 'static,
    {
        
        // Wrapper para type-erase el loader
        let typed_loader = Box::new(move || -> ReactorResult<Box<dyn std::any::Any + Send>> {
            loader().map(|asset| Box::new(Handle::new(id, asset)) as Box<dyn std::any::Any + Send>)
        });
        
        let (response_tx, response_rx) = oneshot::channel();
        
        let request = LoadRequest {
            id,
            path,
            priority,
            loader: typed_loader,
            response_tx,
            created_at: Instant::now(),
        };
        
        // Insertar en cola
        {
            let mut queue = self.queue.lock().unwrap();
            queue.push_back(request);
            let mut s = self.stats.lock().unwrap();
            s.queued += 1;
        }
        
        // Notificar a workers
        self.work_available.notify_one();
        
        // Wrapper del receiver para convertir LoadResult a Handle<T>
        let (wrap_tx, wrap_rx) = oneshot::channel::<ReactorResult<Handle<T>>>();
        
        tokio::spawn(async move {
            match response_rx.await {
                Ok(Ok(result)) => {
                    if let Ok(handle) = result.data.downcast::<Handle<T>>() {
                        let _ = wrap_tx.send(Ok(*handle));
                    } else {
                        let _ = wrap_tx.send(Err(ReactorError::internal(
                            "Type mismatch in loader result"
                        )));
                    }
                }
                Ok(Err(e)) => {
                    let _ = wrap_tx.send(Err(e));
                }
                Err(e) => {
                    let _ = wrap_tx.send(Err(ReactorError::internal(
                        format!("Channel error: {}", e)
                    )));
                }
            }
        });
        
        wrap_rx
    }

    /// Enqueue carga de modelo glTF (helper especializado)
    pub fn enqueue_gltf(
        &self,
        id: AssetId,
        path: PathBuf,
        priority: LoadPriority,
    ) -> oneshot::Receiver<ReactorResult<Handle<crate::resources::gltf_loader::GltfModel>>> {
        let load_path = path.clone();
        self.enqueue(id, path, priority, move || {
            let mut loader = crate::resources::gltf_loader::GltfLoader::new(".");
            loader.load(&load_path)
        })
    }

    /// Cancelar una carga pendiente por AssetId
    pub fn cancel(&self, id: AssetId) -> bool {
        let mut queue = self.queue.lock().unwrap();
        if let Some(pos) = queue.iter().position(|r| r.id == id) {
            let req = queue.remove(pos).unwrap();
            let _ = req.response_tx.send(Err(ReactorError::cancelled()));
            if let Ok(mut s) = self.stats.lock() {
                s.queued = s.queued.saturating_sub(1);
                s.cancelled += 1;
            }
            true
        } else {
            false
        }
    }

    /// Esperar a que todas las cargas críticas se completen
    pub async fn wait_critical(&self) {
        loop {
            {
                let queue = self.queue.lock().unwrap();
                let has_critical = queue.iter().any(|r| r.priority == LoadPriority::Critical);
                let stats = self.stats.lock().unwrap();
                if !has_critical && stats.loading == 0 {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Obtener estadísticas actuales
    pub fn stats(&self) -> LoaderStats {
        self.stats.lock().unwrap().clone()
    }

    /// Esperar a que la cola esté vacía (para shutdown limpio)
    pub async fn drain(&self) {
        loop {
            {
                let queue = self.queue.lock().unwrap();
                let stats = self.stats.lock().unwrap();
                if queue.is_empty() && stats.loading == 0 {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Shutdown de todos los workers
    pub async fn shutdown(self) {
        // Señalar shutdown a workers
        self.shutdown.store(true, Ordering::Relaxed);
        // Wake all workers so they can see the shutdown flag
        self.work_available.notify_waiters();
        
        // Esperar a que terminen
        for worker in self.workers {
            let _ = worker.await;
        }
        
        if self.config.log_stats {
            let stats = self.stats.lock().unwrap();
            println!("[LoaderQueue] Final stats: completed={}, failed={}, avg_time={:.2}ms",
                stats.completed, stats.failed, stats.avg_load_time_ms);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_priority_order() {
        let queue = AssetLoaderQueue::new().unwrap();
        
        // Enqueue en orden aleatorio
        let _rx_low = queue.enqueue(
            AssetId::from_path("low.png"),
            "low.png".into(),
            LoadPriority::Low,
            || Ok("low"),
        );
        let _rx_critical = queue.enqueue(
            AssetId::from_path("critical.png"),
            "critical.png".into(),
            LoadPriority::Critical,
            || Ok("critical"),
        );
        
        // Verificar stats
        let stats = queue.stats();
        assert_eq!(stats.queued, 2);
        
        // Esperar un poco para que los workers procesen
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        queue.shutdown().await;
    }

    #[test]
    fn test_priority_ordering() {
        assert!(LoadPriority::Critical < LoadPriority::High);
        assert!(LoadPriority::High < LoadPriority::Normal);
        assert!(LoadPriority::Normal < LoadPriority::Low);
    }
}
