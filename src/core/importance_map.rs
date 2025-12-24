// =============================================================================
// Importance Map Universal - REACTOR Framework
// =============================================================================
// Un buffer universal que indica importancia para múltiples sistemas:
// - Importancia visual (shading rate)
// - Importancia física (simulation detail)
// - Importancia AI (decision frequency)
// - Importancia audio (spatial audio detail)
//
// Un solo concepto, múltiples sistemas.
// =============================================================================

use glam::{Vec2, Vec3};

/// Tipo de importancia
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportanceType {
    /// Importancia visual (para shading rate)
    Visual,
    /// Importancia física (para simulation detail)
    Physics,
    /// Importancia AI (para decision frequency)
    AI,
    /// Importancia audio (para spatial audio)
    Audio,
    /// Importancia combinada (promedio ponderado)
    Combined,
}

/// Configuración del Importance Map
#[derive(Clone, Debug)]
pub struct ImportanceMapConfig {
    /// Ancho en tiles
    pub tile_width: u32,
    /// Alto en tiles
    pub tile_height: u32,
    /// Tamaño de cada tile en píxeles
    pub tile_size: u32,
    /// Peso de importancia visual
    pub visual_weight: f32,
    /// Peso de importancia física
    pub physics_weight: f32,
    /// Peso de importancia AI
    pub ai_weight: f32,
    /// Peso de importancia audio
    pub audio_weight: f32,
    /// Umbral para considerar "importante"
    pub importance_threshold: f32,
    /// Decaimiento temporal (0-1, cuánto se mantiene del frame anterior)
    pub temporal_decay: f32,
}

impl Default for ImportanceMapConfig {
    fn default() -> Self {
        Self {
            tile_width: 80,  // 1280 / 16
            tile_height: 45, // 720 / 16
            tile_size: 16,
            visual_weight: 0.4,
            physics_weight: 0.3,
            ai_weight: 0.2,
            audio_weight: 0.1,
            importance_threshold: 0.3,
            temporal_decay: 0.8,
        }
    }
}

/// Datos de un tile de importancia
#[derive(Clone, Debug, Default)]
pub struct ImportanceTileData {
    /// Importancia visual (0-1)
    pub visual: f32,
    /// Importancia física (0-1)
    pub physics: f32,
    /// Importancia AI (0-1)
    pub ai: f32,
    /// Importancia audio (0-1)
    pub audio: f32,
    /// Importancia combinada calculada
    pub combined: f32,
    /// Posición del centro del tile en world space
    pub world_center: Vec3,
    /// Distancia a la cámara
    pub camera_distance: f32,
    /// Si el tile es visible
    pub visible: bool,
    /// Frame en que se actualizó por última vez
    pub last_update_frame: u64,
}

impl ImportanceTileData {
    /// Calcular importancia combinada
    pub fn calculate_combined(&mut self, config: &ImportanceMapConfig) {
        self.combined = 
            self.visual * config.visual_weight +
            self.physics * config.physics_weight +
            self.ai * config.ai_weight +
            self.audio * config.audio_weight;
    }

    /// Verificar si es importante según umbral
    pub fn is_important(&self, threshold: f32) -> bool {
        self.combined >= threshold
    }

    /// Obtener nivel de detalle sugerido (0 = máximo, 3 = mínimo)
    pub fn suggested_lod(&self) -> u8 {
        if self.combined > 0.75 { 0 }
        else if self.combined > 0.5 { 1 }
        else if self.combined > 0.25 { 2 }
        else { 3 }
    }

    /// Obtener frecuencia de actualización sugerida (cada N frames)
    pub fn suggested_update_frequency(&self) -> u32 {
        if self.combined > 0.75 { 1 }      // Cada frame
        else if self.combined > 0.5 { 2 }  // Cada 2 frames
        else if self.combined > 0.25 { 4 } // Cada 4 frames
        else { 8 }                          // Cada 8 frames
    }
}

/// Importance Map Universal
pub struct ImportanceMap {
    /// Configuración
    pub config: ImportanceMapConfig,
    /// Datos de tiles
    tiles: Vec<ImportanceTileData>,
    /// Frame actual
    current_frame: u64,
    /// Estadísticas
    pub stats: ImportanceMapStats,
}

/// Estadísticas del Importance Map
#[derive(Clone, Debug, Default)]
pub struct ImportanceMapStats {
    /// Tiles totales
    pub total_tiles: u32,
    /// Tiles visibles
    pub visible_tiles: u32,
    /// Tiles importantes (sobre umbral)
    pub important_tiles: u32,
    /// Importancia promedio
    pub average_importance: f32,
    /// Distribución por LOD
    pub lod_distribution: [u32; 4],
}

impl ImportanceMap {
    /// Crear nuevo Importance Map
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        let tile_size = 16;
        let tile_width = (screen_width + tile_size - 1) / tile_size;
        let tile_height = (screen_height + tile_size - 1) / tile_size;
        
        let config = ImportanceMapConfig {
            tile_width,
            tile_height,
            tile_size,
            ..Default::default()
        };

        let total_tiles = (tile_width * tile_height) as usize;
        let tiles = vec![ImportanceTileData::default(); total_tiles];

        Self {
            config,
            tiles,
            current_frame: 0,
            stats: ImportanceMapStats::default(),
        }
    }

    /// Crear con configuración personalizada
    pub fn with_config(config: ImportanceMapConfig) -> Self {
        let total_tiles = (config.tile_width * config.tile_height) as usize;
        let tiles = vec![ImportanceTileData::default(); total_tiles];

        Self {
            config,
            tiles,
            current_frame: 0,
            stats: ImportanceMapStats::default(),
        }
    }

    /// Obtener índice de tile desde coordenadas
    fn tile_index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.config.tile_width && y < self.config.tile_height {
            Some((y * self.config.tile_width + x) as usize)
        } else {
            None
        }
    }

    /// Obtener tile en coordenadas
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&ImportanceTileData> {
        self.tile_index(x, y).map(|i| &self.tiles[i])
    }

    /// Obtener tile mutable
    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> Option<&mut ImportanceTileData> {
        self.tile_index(x, y).map(|i| &mut self.tiles[i])
    }

    /// Obtener tile desde coordenadas de pantalla
    pub fn get_tile_at_screen(&self, screen_x: u32, screen_y: u32) -> Option<&ImportanceTileData> {
        let tile_x = screen_x / self.config.tile_size;
        let tile_y = screen_y / self.config.tile_size;
        self.get_tile(tile_x, tile_y)
    }

    /// Establecer importancia visual para un tile
    pub fn set_visual_importance(&mut self, x: u32, y: u32, importance: f32) {
        let current_frame = self.current_frame;
        if let Some(idx) = self.tile_index(x, y) {
            self.tiles[idx].visual = importance.clamp(0.0, 1.0);
            self.tiles[idx].last_update_frame = current_frame;
        }
    }

    /// Establecer importancia física para un tile
    pub fn set_physics_importance(&mut self, x: u32, y: u32, importance: f32) {
        let current_frame = self.current_frame;
        if let Some(idx) = self.tile_index(x, y) {
            self.tiles[idx].physics = importance.clamp(0.0, 1.0);
            self.tiles[idx].last_update_frame = current_frame;
        }
    }

    /// Establecer importancia AI para un tile
    pub fn set_ai_importance(&mut self, x: u32, y: u32, importance: f32) {
        let current_frame = self.current_frame;
        if let Some(idx) = self.tile_index(x, y) {
            self.tiles[idx].ai = importance.clamp(0.0, 1.0);
            self.tiles[idx].last_update_frame = current_frame;
        }
    }

    /// Establecer importancia audio para un tile
    pub fn set_audio_importance(&mut self, x: u32, y: u32, importance: f32) {
        let current_frame = self.current_frame;
        if let Some(idx) = self.tile_index(x, y) {
            self.tiles[idx].audio = importance.clamp(0.0, 1.0);
            self.tiles[idx].last_update_frame = current_frame;
        }
    }

    /// Establecer todas las importancias de un tile
    pub fn set_tile_importance(&mut self, x: u32, y: u32, visual: f32, physics: f32, ai: f32, audio: f32) {
        let current_frame = self.current_frame;
        if let Some(idx) = self.tile_index(x, y) {
            self.tiles[idx].visual = visual.clamp(0.0, 1.0);
            self.tiles[idx].physics = physics.clamp(0.0, 1.0);
            self.tiles[idx].ai = ai.clamp(0.0, 1.0);
            self.tiles[idx].audio = audio.clamp(0.0, 1.0);
            self.tiles[idx].last_update_frame = current_frame;
        }
    }

    /// Actualizar importancia basada en posición de cámara
    pub fn update_from_camera(&mut self, camera_pos: Vec3, camera_forward: Vec3) {
        for tile in &mut self.tiles {
            if tile.world_center != Vec3::ZERO {
                // Calcular distancia
                tile.camera_distance = (tile.world_center - camera_pos).length();
                
                // Importancia visual basada en distancia
                let dist_factor = 1.0 / (tile.camera_distance * 0.1 + 1.0);
                
                // Importancia basada en dirección de vista
                let to_tile = (tile.world_center - camera_pos).normalize();
                let view_factor = camera_forward.dot(to_tile).max(0.0);
                
                tile.visual = (dist_factor * 0.5 + view_factor * 0.5).clamp(0.0, 1.0);
            }
        }
    }

    /// Actualizar importancia desde visibilidad (frustum culling)
    pub fn update_visibility(&mut self, visible_tiles: &[(u32, u32)]) {
        // Marcar todos como no visibles
        for tile in &mut self.tiles {
            tile.visible = false;
        }
        
        // Marcar visibles
        for &(x, y) in visible_tiles {
            if let Some(tile) = self.get_tile_mut(x, y) {
                tile.visible = true;
            }
        }
    }

    /// Recalcular importancia combinada de todos los tiles
    pub fn recalculate_combined(&mut self) {
        for tile in &mut self.tiles {
            tile.calculate_combined(&self.config);
        }
    }

    /// Aplicar decaimiento temporal
    pub fn apply_temporal_decay(&mut self) {
        let decay = self.config.temporal_decay;
        for tile in &mut self.tiles {
            // Si no se actualizó este frame, aplicar decay
            if tile.last_update_frame < self.current_frame {
                tile.visual *= decay;
                tile.physics *= decay;
                tile.ai *= decay;
                tile.audio *= decay;
            }
        }
    }

    /// Avanzar frame
    pub fn next_frame(&mut self) {
        self.current_frame += 1;
        self.apply_temporal_decay();
        self.recalculate_combined();
        self.update_stats();
    }

    /// Actualizar estadísticas
    fn update_stats(&mut self) {
        self.stats.total_tiles = self.tiles.len() as u32;
        self.stats.visible_tiles = self.tiles.iter().filter(|t| t.visible).count() as u32;
        self.stats.important_tiles = self.tiles.iter()
            .filter(|t| t.is_important(self.config.importance_threshold))
            .count() as u32;
        
        let sum: f32 = self.tiles.iter().map(|t| t.combined).sum();
        self.stats.average_importance = sum / self.tiles.len() as f32;
        
        self.stats.lod_distribution = [0; 4];
        for tile in &self.tiles {
            let lod = tile.suggested_lod() as usize;
            if lod < 4 {
                self.stats.lod_distribution[lod] += 1;
            }
        }
    }

    /// Obtener importancia en coordenadas de pantalla
    pub fn get_importance_at(&self, screen_x: u32, screen_y: u32, importance_type: ImportanceType) -> f32 {
        if let Some(tile) = self.get_tile_at_screen(screen_x, screen_y) {
            match importance_type {
                ImportanceType::Visual => tile.visual,
                ImportanceType::Physics => tile.physics,
                ImportanceType::AI => tile.ai,
                ImportanceType::Audio => tile.audio,
                ImportanceType::Combined => tile.combined,
            }
        } else {
            0.0
        }
    }

    /// Verificar si un objeto debe actualizarse este frame
    pub fn should_update(&self, screen_x: u32, screen_y: u32) -> bool {
        if let Some(tile) = self.get_tile_at_screen(screen_x, screen_y) {
            let freq = tile.suggested_update_frequency();
            (self.current_frame % freq as u64) == 0
        } else {
            true // Por defecto, actualizar
        }
    }

    /// Obtener LOD sugerido para una posición
    pub fn get_suggested_lod(&self, screen_x: u32, screen_y: u32) -> u8 {
        if let Some(tile) = self.get_tile_at_screen(screen_x, screen_y) {
            tile.suggested_lod()
        } else {
            0 // Máximo detalle por defecto
        }
    }

    /// Redimensionar para nueva resolución
    pub fn resize(&mut self, screen_width: u32, screen_height: u32) {
        self.config.tile_width = (screen_width + self.config.tile_size - 1) / self.config.tile_size;
        self.config.tile_height = (screen_height + self.config.tile_size - 1) / self.config.tile_size;
        
        let total_tiles = (self.config.tile_width * self.config.tile_height) as usize;
        self.tiles = vec![ImportanceTileData::default(); total_tiles];
    }

    /// Imprimir estadísticas
    pub fn print_stats(&self) {
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                   Importance Map Stats                           ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║ Total Tiles:     {:5}                                           ║", self.stats.total_tiles);
        println!("║ Visible Tiles:   {:5}                                           ║", self.stats.visible_tiles);
        println!("║ Important Tiles: {:5}                                           ║", self.stats.important_tiles);
        println!("║ Avg Importance:  {:5.2}                                           ║", self.stats.average_importance);
        println!("║ LOD Distribution:                                                ║");
        println!("║   LOD 0 (Max):   {:5}                                           ║", self.stats.lod_distribution[0]);
        println!("║   LOD 1:         {:5}                                           ║", self.stats.lod_distribution[1]);
        println!("║   LOD 2:         {:5}                                           ║", self.stats.lod_distribution[2]);
        println!("║   LOD 3 (Min):   {:5}                                           ║", self.stats.lod_distribution[3]);
        println!("╚══════════════════════════════════════════════════════════════════╝");
    }
}

impl Default for ImportanceMap {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}
