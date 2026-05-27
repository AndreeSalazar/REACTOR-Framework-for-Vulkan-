//! # Shader Hot-Reloader
//!
//! Watch de archivos de shader (WGSL, GLSL, SPIR-V) que detecta cambios en disco,
//! recompila automáticamente vía `ShaderCompiler`, e invalida los PSOs cacheados
//! que usaban el shader viejo.
//!
//! ## Arquitectura
//!
//! ```text
//! ┌─────────────────────┐     ┌───────────────────┐     ┌────────────────┐
//! │  notify::Watcher    │────►│ ShaderHotReloader │────►│ PsoCache       │
//! │  (file system evt)  │     │ (recompile SPIR-V)│     │ (invalidate)   │
//! └─────────────────────┘     └───────────────────┘     └────────────────┘
//!                                       │
//!                                       ▼
//!                             ┌──────────────────┐
//!                             │ ShaderCompiler   │
//!                             │ (naga → SPIR-V)  │
//!                             └──────────────────┘
//! ```
//!
//! ## Uso
//!
//! ```rust,ignore
//! let mut reloader = ShaderHotReloader::new(
//!     vec!["shaders/pbr.vert", "shaders/pbr.frag"],
//!     ShaderStage::Vertex, // o Fragment
//!     "main",
//! )?;
//!
//! // En el game loop:
//! if let Some(recompiled) = reloader.poll()? {
//!     println!("Shader recompilado! hash: {}", recompiled.spirv_hash);
//!     // Invalidar PSOs que usaban el hash viejo
//!     pso_cache.invalidate_by_spirv_hash(old_hash);
//! }
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::graphics::shader_compiler::{CompiledShader, ShaderCompiler, ShaderStage};

// ═══════════════════════════════════════════════════════════════════════════
// Tipos
// ═══════════════════════════════════════════════════════════════════════════

/// Resultado de una recompilación exitosa.
#[derive(Debug, Clone)]
pub struct ShaderReloadEvent {
    /// Path del shader que cambió.
    pub path: PathBuf,
    /// Stage del shader.
    pub stage: ShaderStage,
    /// SPIR-V hash anterior (para invalidar PSOs viejos).
    pub old_spirv_hash: u64,
    /// Shader recién compilado.
    pub compiled: CompiledShader,
}

/// Entrada de un shader bajo watch.
#[derive(Debug)]
struct WatchedShader {
    path: PathBuf,
    stage: ShaderStage,
    entry_point: String,
    last_spirv_hash: u64,
    last_modified: Option<std::time::SystemTime>,
}

// ═══════════════════════════════════════════════════════════════════════════
// ShaderHotReloader
// ═══════════════════════════════════════════════════════════════════════════

pub struct ShaderHotReloader {
    compiler: ShaderCompiler,
    watched: Vec<WatchedShader>,
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
    rx: mpsc::Receiver<Result<Event, notify::Error>>,
    /// Cooldown para evitar recompilaciones múltiples del mismo archivo
    /// cuando el editor guarda varias veces en rápida sucesión.
    last_recompile: HashMap<PathBuf, Instant>,
    debounce_ms: u64,
}

impl ShaderHotReloader {
    /// Crea un nuevo reloader que vigila los paths dados.
    ///
    /// Compila todos los shaders una vez al inicio para obtener los hashes iniciales.
    pub fn new(shaders: Vec<(PathBuf, ShaderStage)>, entry_point: &str) -> ReactorResult<Self> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .map_err(|e| {
            ReactorError::with_source(ErrorCode::IoError, "Failed to create file watcher", e)
        })?;

        let mut compiler = ShaderCompiler::new();
        let mut watched = Vec::with_capacity(shaders.len());

        for (path, stage) in &shaders {
            // Vigilar el directorio padre (más robusto que vigilar el archivo)
            let parent = path.parent().unwrap_or(Path::new("."));
            watcher
                .watch(parent, RecursiveMode::NonRecursive)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::IoError,
                        format!("Failed to watch directory: {}", parent.display()),
                        e,
                    )
                })?;

            // Compilar una vez para obtener el hash inicial
            let compiled = if path.exists() {
                compiler.compile_file(path, *stage, entry_point).ok()
            } else {
                None
            };

            let (hash, modified) = match &compiled {
                Some(c) => {
                    let modified = fs::metadata(path).ok().and_then(|m| m.modified().ok());
                    (c.spirv_hash, modified)
                }
                None => (0, None),
            };

            watched.push(WatchedShader {
                path: path.clone(),
                stage: *stage,
                entry_point: entry_point.to_string(),
                last_spirv_hash: hash,
                last_modified: modified,
            });
        }

        Ok(Self {
            compiler,
            watched,
            watcher,
            rx,
            last_recompile: HashMap::new(),
            debounce_ms: 100,
        })
    }

    /// Crea un reloader vacío (sin shaders bajo watch).
    /// Útil para añadir shaders dinámicamente con `watch_shader()`.
    pub fn empty() -> ReactorResult<Self> {
        let (tx, rx) = mpsc::channel();
        let watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .map_err(|e| {
            ReactorError::with_source(ErrorCode::IoError, "Failed to create file watcher", e)
        })?;

        Ok(Self {
            compiler: ShaderCompiler::new(),
            watched: Vec::new(),
            watcher,
            rx,
            last_recompile: HashMap::new(),
            debounce_ms: 100,
        })
    }

    /// Añade un shader al sistema de watch.
    pub fn watch_shader(
        &mut self,
        path: &Path,
        stage: ShaderStage,
        entry_point: &str,
    ) -> ReactorResult<Option<CompiledShader>> {
        let parent = path.parent().unwrap_or(Path::new("."));
        self.watcher
            .watch(parent, RecursiveMode::NonRecursive)
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::IoError,
                    format!("Failed to watch: {}", parent.display()),
                    e,
                )
            })?;

        let compiled = if path.exists() {
            self.compiler.compile_file(path, stage, entry_point).ok()
        } else {
            None
        };

        let (hash, modified) = match &compiled {
            Some(c) => {
                let modified = fs::metadata(path).ok().and_then(|m| m.modified().ok());
                (c.spirv_hash, modified)
            }
            None => (0, None),
        };

        self.watched.push(WatchedShader {
            path: path.to_path_buf(),
            stage,
            entry_point: entry_point.to_string(),
            last_spirv_hash: hash,
            last_modified: modified,
        });

        Ok(compiled)
    }

    /// Poll no-bloqueante: retorna eventos de shaders que cambiaron y se recompilaron.
    ///
    /// Llamar una vez por frame. Los eventos se deduplican con un debounce de 100ms.
    pub fn poll(&mut self) -> ReactorResult<Vec<ShaderReloadEvent>> {
        let mut events = Vec::new();

        // Drenar todos los eventos del canal
        let mut changed_paths: Vec<PathBuf> = Vec::new();
        while let Ok(result) = self.rx.try_recv() {
            if let Ok(event) = result {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        for path in event.paths {
                            changed_paths.push(path);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Procesar cada path cambiado
        for changed_path in changed_paths {
            // Encontrar si este path corresponde a un shader vigilado
            let watched_idx = self.watched.iter().position(|w| {
                // Comparar por canonicalized path si es posible, sino por valor
                match (changed_path.canonicalize(), w.path.canonicalize()) {
                    (Ok(a), Ok(b)) => a == b,
                    _ => changed_path == w.path,
                }
            });

            let Some(idx) = watched_idx else { continue };

            // Debounce: no recompilar si ya lo hicimos hace menos de debounce_ms
            let now = Instant::now();
            if let Some(last) = self.last_recompile.get(&self.watched[idx].path) {
                if now.duration_since(*last) < Duration::from_millis(self.debounce_ms) {
                    continue;
                }
            }

            // Verificar que el archivo realmente cambió (por timestamp)
            let current_modified = fs::metadata(&self.watched[idx].path)
                .ok()
                .and_then(|m| m.modified().ok());

            if current_modified == self.watched[idx].last_modified {
                continue; // No cambió realmente
            }

            // Recompilar
            let entry_point = self.watched[idx].entry_point.clone();
            let stage = self.watched[idx].stage;
            let old_hash = self.watched[idx].last_spirv_hash;

            match self
                .compiler
                .compile_file(&self.watched[idx].path, stage, &entry_point)
            {
                Ok(compiled) => {
                    // Solo notificar si el SPIR-V realmente cambió
                    if compiled.spirv_hash != old_hash {
                        self.watched[idx].last_spirv_hash = compiled.spirv_hash;
                        self.watched[idx].last_modified = current_modified;
                        self.last_recompile
                            .insert(self.watched[idx].path.clone(), Instant::now());

                        log::info!(
                            "🔄 Shader hot-reload: {} (hash: {:#x} → {:#x})",
                            self.watched[idx].path.display(),
                            old_hash,
                            compiled.spirv_hash,
                        );

                        events.push(ShaderReloadEvent {
                            path: self.watched[idx].path.clone(),
                            stage,
                            old_spirv_hash: old_hash,
                            compiled,
                        });
                    } else {
                        // El archivo cambió pero el SPIR-V es el mismo
                        // (ej: cambio de comentario). Actualizar timestamp.
                        self.watched[idx].last_modified = current_modified;
                    }
                }
                Err(e) => {
                    log::warn!(
                        "⚠️ Shader recompilation failed for {}: {}",
                        self.watched[idx].path.display(),
                        e,
                    );
                    // No actualizar el hash → la próxima vez que cambie, se reintentará
                }
            }
        }

        Ok(events)
    }

    /// Retorna la cantidad de shaders bajo watch.
    pub fn watched_count(&self) -> usize {
        self.watched.len()
    }

    /// Configura el debounce en milisegundos (default: 100ms).
    pub fn set_debounce(&mut self, ms: u64) {
        self.debounce_ms = ms;
    }

    /// Acceso al compilador interno (para compilar shaders manualmente).
    pub fn compiler(&mut self) -> &mut ShaderCompiler {
        &mut self.compiler
    }
}
