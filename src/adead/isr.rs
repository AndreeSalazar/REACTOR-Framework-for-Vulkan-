// =============================================================================
// ADead-ISR: Intelligent Shading Rate 2.0
// =============================================================================
// "Adaptive Resolution Shading sin AI, sin Tensor Cores, Matemáticas Puras"
//
// Concepto: No todos los píxeles necesitan el mismo esfuerzo
//   - Píxel en BORDE:     Importante    → 1x1 (full detail)
//   - Píxel en CIELO:     No importante → 4x4 (low detail)
//   - Píxel en TEXTURA:   Medio         → 2x2 (medium detail)
//
// Comparación vs DLSS:
//   DLSS: Requiere Tensor Cores, ghosting, artifacts, input lag
//   ISR:  Cualquier GPU, sin ghosting, native resolution, zero lag
// =============================================================================

use glam::{Vec2, Vec3};

/// Nivel de importancia de un píxel/región
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ImportanceLevel {
    #[default]
    /// Máxima importancia - bordes, detalles finos (1x1 shading)
    Critical = 0,
    /// Alta importancia - texturas complejas (1x1 shading)
    High = 1,
    /// Media importancia - superficies con variación (2x2 shading)
    Medium = 2,
    /// Baja importancia - áreas uniformes (4x4 shading)
    Low = 3,
    /// Mínima importancia - cielo, fondos (8x8 shading)
    Minimal = 4,
}

impl ImportanceLevel {
    /// Obtener el tamaño del super-pixel para este nivel
    pub fn pixel_size(&self) -> u32 {
        match self {
            ImportanceLevel::Critical => 1,
            ImportanceLevel::High => 1,
            ImportanceLevel::Medium => 2,
            ImportanceLevel::Low => 4,
            ImportanceLevel::Minimal => 8,
        }
    }

    /// Obtener el factor de ahorro de GPU
    pub fn savings_factor(&self) -> f32 {
        let size = self.pixel_size() as f32;
        1.0 - (1.0 / (size * size))
    }
}

/// Configuración del sistema ISR
#[derive(Clone, Debug)]
pub struct ISRConfig {
    /// Umbral para importancia crítica (bordes)
    pub critical_threshold: f32,
    /// Umbral para importancia alta
    pub high_threshold: f32,
    /// Umbral para importancia media
    pub medium_threshold: f32,
    /// Umbral para importancia baja
    pub low_threshold: f32,
    
    /// Peso del factor de bordes (SDF distance)
    pub edge_weight: f32,
    /// Peso del factor de variación de normales
    pub normal_weight: f32,
    /// Peso del factor de movimiento
    pub motion_weight: f32,
    /// Peso del factor de distancia a cámara
    pub distance_weight: f32,
    
    /// Habilitar coherencia temporal
    pub temporal_coherence: bool,
    /// Umbral de cambio para invalidar cache temporal
    pub temporal_threshold: f32,
    
    /// Habilitar análisis jerárquico (mipmap-style)
    pub hierarchical_analysis: bool,
    /// Niveles de jerarquía
    pub hierarchy_levels: u32,
    
    /// Habilitar foveated rendering (VR/eye-tracking)
    pub foveated_rendering: bool,
    /// Radio del área foveal (en píxeles normalizados 0-1)
    pub fovea_radius: f32,
    /// Centro de la fóvea (posición de la mirada)
    pub fovea_center: Vec2,
}

impl Default for ISRConfig {
    fn default() -> Self {
        Self {
            critical_threshold: 0.8,
            high_threshold: 0.6,
            medium_threshold: 0.4,
            low_threshold: 0.2,
            
            edge_weight: 0.5,
            normal_weight: 0.3,
            motion_weight: 0.2,
            distance_weight: 1.0,
            
            temporal_coherence: true,
            temporal_threshold: 0.1,
            
            hierarchical_analysis: true,
            hierarchy_levels: 3,
            
            foveated_rendering: false,
            fovea_radius: 0.15,
            fovea_center: Vec2::new(0.5, 0.5),
        }
    }
}

/// Datos de importancia por tile/región
#[derive(Clone, Debug, Default)]
pub struct ImportanceTile {
    /// Posición del tile en la pantalla
    pub position: Vec2,
    /// Tamaño del tile
    pub size: Vec2,
    /// Nivel de importancia calculado
    pub importance: f32,
    /// Nivel discretizado
    pub level: ImportanceLevel,
    /// Importancia del frame anterior (para temporal coherence)
    pub prev_importance: f32,
    /// Si el tile necesita re-análisis
    pub needs_update: bool,
}

/// Mapa de importancia para toda la pantalla
#[derive(Clone, Debug)]
pub struct ImportanceMap {
    /// Ancho en tiles
    pub width: u32,
    /// Alto en tiles
    pub height: u32,
    /// Tamaño de cada tile en píxeles
    pub tile_size: u32,
    /// Tiles de importancia
    pub tiles: Vec<ImportanceTile>,
    /// Estadísticas
    pub stats: ISRStats,
}

/// Estadísticas del sistema ISR
#[derive(Clone, Debug, Default)]
pub struct ISRStats {
    /// Total de píxeles en la pantalla
    pub total_pixels: u64,
    /// Píxeles que realmente se renderizan
    pub rendered_pixels: u64,
    /// Porcentaje de ahorro
    pub savings_percent: f32,
    /// Distribución por nivel de importancia
    pub distribution: [f32; 5],
    /// Tiempo de análisis en ms
    pub analysis_time_ms: f32,
}

impl ImportanceMap {
    /// Crear un nuevo mapa de importancia
    pub fn new(screen_width: u32, screen_height: u32, tile_size: u32) -> Self {
        let width = (screen_width + tile_size - 1) / tile_size;
        let height = (screen_height + tile_size - 1) / tile_size;
        let total_tiles = (width * height) as usize;
        
        let mut tiles = Vec::with_capacity(total_tiles);
        for y in 0..height {
            for x in 0..width {
                tiles.push(ImportanceTile {
                    position: Vec2::new(
                        (x * tile_size) as f32,
                        (y * tile_size) as f32
                    ),
                    size: Vec2::new(tile_size as f32, tile_size as f32),
                    importance: 0.5,
                    level: ImportanceLevel::Medium,
                    prev_importance: 0.5,
                    needs_update: true,
                });
            }
        }
        
        Self {
            width,
            height,
            tile_size,
            tiles,
            stats: ISRStats::default(),
        }
    }

    /// Obtener tile en coordenadas de tile
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&ImportanceTile> {
        if x < self.width && y < self.height {
            Some(&self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Obtener tile mutable
    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> Option<&mut ImportanceTile> {
        if x < self.width && y < self.height {
            Some(&mut self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Calcular estadísticas
    pub fn calculate_stats(&mut self) {
        let mut distribution = [0u32; 5];
        let mut rendered = 0u64;
        
        for tile in &self.tiles {
            let level_idx = tile.level as usize;
            distribution[level_idx] += 1;
            
            let pixel_size = tile.level.pixel_size();
            let tile_pixels = (self.tile_size * self.tile_size) as u64;
            rendered += tile_pixels / (pixel_size * pixel_size) as u64;
        }
        
        let total_tiles = self.tiles.len() as f32;
        self.stats.distribution = [
            distribution[0] as f32 / total_tiles,
            distribution[1] as f32 / total_tiles,
            distribution[2] as f32 / total_tiles,
            distribution[3] as f32 / total_tiles,
            distribution[4] as f32 / total_tiles,
        ];
        
        self.stats.total_pixels = (self.width * self.height * self.tile_size * self.tile_size) as u64;
        self.stats.rendered_pixels = rendered;
        self.stats.savings_percent = 1.0 - (rendered as f32 / self.stats.total_pixels as f32);
    }
}

/// Sistema principal de Intelligent Shading Rate
pub struct IntelligentShadingRate {
    /// Configuración
    pub config: ISRConfig,
    /// Mapa de importancia actual
    pub importance_map: ImportanceMap,
    /// Resolución de pantalla
    pub screen_size: (u32, u32),
}

impl IntelligentShadingRate {
    /// Crear nuevo sistema ISR
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            config: ISRConfig::default(),
            importance_map: ImportanceMap::new(width, height, 16),
            screen_size: (width, height),
        }
    }

    /// Crear con configuración personalizada
    pub fn with_config(width: u32, height: u32, config: ISRConfig) -> Self {
        Self {
            config,
            importance_map: ImportanceMap::new(width, height, 16),
            screen_size: (width, height),
        }
    }

    /// Calcular importancia de un punto en el mundo
    pub fn calculate_importance(
        &self,
        world_pos: Vec3,
        normal: Vec3,
        prev_pos: Vec3,
        camera_pos: Vec3,
        sdf_distance: f32,
    ) -> f32 {
        // Factor de bordes (SDF) - más cerca del borde = más importante
        let edge_importance = 1.0 / (sdf_distance.abs() + 0.01);
        let edge_importance = edge_importance.min(1.0);
        
        // Factor de variación de normales (aproximado)
        let normal_variance = (normal - Vec3::Y).length();
        let normal_importance = normal_variance.min(1.0);
        
        // Factor de movimiento
        let motion = (world_pos - prev_pos).length();
        let motion_importance = (motion * 10.0).min(1.0);
        
        // Factor de distancia a cámara
        let distance = (world_pos - camera_pos).length();
        let distance_factor = 1.0 / (distance + 1.0);
        
        // Combinar factores
        let importance = (
            edge_importance * self.config.edge_weight +
            normal_importance * self.config.normal_weight +
            motion_importance * self.config.motion_weight
        ) * distance_factor * self.config.distance_weight;
        
        importance.clamp(0.0, 1.0)
    }

    /// Convertir importancia a nivel discreto
    pub fn importance_to_level(&self, importance: f32) -> ImportanceLevel {
        if importance > self.config.critical_threshold {
            ImportanceLevel::Critical
        } else if importance > self.config.high_threshold {
            ImportanceLevel::High
        } else if importance > self.config.medium_threshold {
            ImportanceLevel::Medium
        } else if importance > self.config.low_threshold {
            ImportanceLevel::Low
        } else {
            ImportanceLevel::Minimal
        }
    }

    /// Aplicar foveated rendering si está habilitado
    pub fn apply_foveated(&self, importance: f32, screen_uv: Vec2) -> f32 {
        if !self.config.foveated_rendering {
            return importance;
        }
        
        let dist_to_fovea = (screen_uv - self.config.fovea_center).length();
        
        if dist_to_fovea < self.config.fovea_radius {
            // En la fóvea: forzar alta importancia
            importance.max(0.9)
        } else {
            // Periferia: reducir importancia
            let falloff = ((dist_to_fovea - self.config.fovea_radius) * 2.0).min(1.0);
            importance * (1.0 - falloff * 0.5)
        }
    }

    /// Actualizar mapa de importancia con coherencia temporal
    pub fn update_tile_importance(&mut self, tile_x: u32, tile_y: u32, new_importance: f32) {
        let temporal_coherence = self.config.temporal_coherence;
        let temporal_threshold = self.config.temporal_threshold;
        let new_level = self.importance_to_level(new_importance);
        
        if let Some(tile) = self.importance_map.get_tile_mut(tile_x, tile_y) {
            if temporal_coherence {
                let diff = (new_importance - tile.prev_importance).abs();
                if diff < temporal_threshold {
                    return;
                }
            }
            
            tile.prev_importance = tile.importance;
            tile.importance = new_importance;
            tile.level = new_level;
            tile.needs_update = false;
        }
    }

    /// Obtener tamaño de pixel adaptativo para una posición de pantalla
    pub fn get_adaptive_pixel_size(&self, screen_x: u32, screen_y: u32) -> u32 {
        let tile_x = screen_x / self.importance_map.tile_size;
        let tile_y = screen_y / self.importance_map.tile_size;
        
        self.importance_map
            .get_tile(tile_x, tile_y)
            .map(|t| t.level.pixel_size())
            .unwrap_or(1)
    }

    /// Verificar si un pixel es el "líder" de su grupo (debe renderizarse)
    pub fn is_leader_pixel(&self, screen_x: u32, screen_y: u32) -> bool {
        let pixel_size = self.get_adaptive_pixel_size(screen_x, screen_y);
        screen_x % pixel_size == 0 && screen_y % pixel_size == 0
    }

    /// Obtener estadísticas actuales
    pub fn stats(&mut self) -> &ISRStats {
        self.importance_map.calculate_stats();
        &self.importance_map.stats
    }

    /// Redimensionar para nueva resolución
    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_size = (width, height);
        self.importance_map = ImportanceMap::new(width, height, 16);
    }

    /// Preset: Máximo rendimiento (más ahorro de GPU)
    pub fn preset_performance() -> ISRConfig {
        ISRConfig {
            critical_threshold: 0.9,
            high_threshold: 0.7,
            medium_threshold: 0.5,
            low_threshold: 0.3,
            ..Default::default()
        }
    }

    /// Preset: Máxima calidad (menos ahorro)
    pub fn preset_quality() -> ISRConfig {
        ISRConfig {
            critical_threshold: 0.6,
            high_threshold: 0.4,
            medium_threshold: 0.2,
            low_threshold: 0.1,
            ..Default::default()
        }
    }

    /// Preset: Balanceado
    pub fn preset_balanced() -> ISRConfig {
        ISRConfig::default()
    }

    /// Preset: VR con foveated rendering
    pub fn preset_vr() -> ISRConfig {
        ISRConfig {
            foveated_rendering: true,
            fovea_radius: 0.12,
            fovea_center: Vec2::new(0.5, 0.5),
            ..Default::default()
        }
    }
}

/// Benchmark del sistema ISR
#[derive(Clone, Debug, Default)]
pub struct ISRBenchmark {
    /// FPS tradicional (sin ISR)
    pub traditional_fps: f32,
    /// FPS con ISR
    pub isr_fps: f32,
    /// Mejora de rendimiento
    pub speedup: f32,
    /// Ahorro de GPU
    pub gpu_savings: f32,
    /// Calidad estimada (0-100%)
    pub quality_estimate: f32,
}

impl ISRBenchmark {
    /// Calcular benchmark comparativo
    pub fn calculate(isr: &mut IntelligentShadingRate, frame_time_ms: f32) -> Self {
        let stats = isr.stats();
        
        // Estimar FPS tradicional basado en tiempo de frame
        let traditional_fps = 1000.0 / frame_time_ms;
        
        // Estimar FPS con ISR basado en ahorro
        let savings = stats.savings_percent;
        let isr_fps = traditional_fps / (1.0 - savings * 0.7); // Factor conservador
        
        // Estimar calidad basada en distribución
        let quality = 
            stats.distribution[0] * 1.0 +   // Critical: 100%
            stats.distribution[1] * 0.98 +  // High: 98%
            stats.distribution[2] * 0.95 +  // Medium: 95%
            stats.distribution[3] * 0.90 +  // Low: 90%
            stats.distribution[4] * 0.85;   // Minimal: 85%
        
        Self {
            traditional_fps,
            isr_fps,
            speedup: isr_fps / traditional_fps,
            gpu_savings: savings * 100.0,
            quality_estimate: quality * 100.0,
        }
    }
}
