// =============================================================================
// ADead-AA: Global Anti-Aliasing Pipeline
// =============================================================================
// Sistema de AA global que se aplica automáticamente en el motor gráfico
// Usa matemáticas SDF puras para eliminar bordes dentados
// =============================================================================

use glam::{Vec2, Vec3, Vec4};
use crate::adead::antialiasing::{
    SDFAntiAliasing, SDFAAConfig, AAQuality,
    smoothstep, smootherstep, smootherstep_ultra,
};

/// Pipeline global de Anti-Aliasing
pub struct AAGlobalPipeline {
    /// Sistema de AA principal
    pub aa: SDFAntiAliasing,
    /// Habilitado globalmente
    pub enabled: bool,
    /// Resolución actual
    width: u32,
    height: u32,
    /// Estadísticas
    pub stats: AAStats,
}

/// Estadísticas del pipeline de AA
#[derive(Clone, Debug, Default)]
pub struct AAStats {
    /// Píxeles procesados
    pub pixels_processed: u64,
    /// Bordes detectados
    pub edges_detected: u64,
    /// Píxeles suavizados
    pub pixels_smoothed: u64,
    /// Tiempo de procesamiento (ms)
    pub processing_time_ms: f32,
    /// Calidad actual
    pub current_quality: String,
}

impl AAGlobalPipeline {
    /// Crear nuevo pipeline de AA
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            aa: SDFAntiAliasing::with_quality(AAQuality::High),
            enabled: true,
            width,
            height,
            stats: AAStats::default(),
        }
    }

    /// Crear con calidad específica
    pub fn with_quality(width: u32, height: u32, quality: AAQuality) -> Self {
        let mut pipeline = Self::new(width, height);
        pipeline.set_quality(quality);
        pipeline
    }

    /// Establecer calidad
    pub fn set_quality(&mut self, quality: AAQuality) {
        self.aa = SDFAntiAliasing::with_quality(quality);
        self.stats.current_quality = format!("{:?}", quality);
    }

    /// Redimensionar
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    // =========================================================================
    // FUNCIONES DE PROCESAMIENTO DE COLOR
    // =========================================================================

    /// Aplicar AA a un color basado en distancia SDF
    /// Esta es la función principal para eliminar dientes
    #[inline]
    pub fn apply_aa_to_color(&self, color: Vec4, sdf_distance: f32, gradient: Vec2) -> Vec4 {
        if !self.enabled {
            return color;
        }

        let coverage = self.aa.compute_edge_coverage(sdf_distance, gradient);
        
        // Mezclar con transparencia basada en cobertura
        Vec4::new(color.x, color.y, color.z, color.w * coverage)
    }

    /// Aplicar AA geométrico a un color
    #[inline]
    pub fn apply_geometric_aa(&self, color: Vec4, edge_distance: f32, normal: Vec2, pixel_size: f32) -> Vec4 {
        if !self.enabled {
            return color;
        }

        let coverage = self.aa.geometric_aa(edge_distance, normal, pixel_size);
        Vec4::new(color.x, color.y, color.z, color.w * coverage)
    }

    /// Suavizar color en borde usando matemáticas SDF puras
    #[inline]
    pub fn smooth_edge_color(&self, inside_color: Vec4, outside_color: Vec4, sdf_distance: f32, edge_width: f32) -> Vec4 {
        if !self.enabled {
            return if sdf_distance < 0.0 { inside_color } else { outside_color };
        }

        // Usar smootherstep_ultra para máxima suavidad
        let t = smootherstep_ultra(-edge_width, edge_width, sdf_distance);
        
        // Interpolar colores
        inside_color * (1.0 - t) + outside_color * t
    }

    /// Calcular factor de suavizado para un píxel
    #[inline]
    pub fn calculate_smooth_factor(&self, sdf_distance: f32, screen_derivative: f32) -> f32 {
        if !self.enabled {
            return if sdf_distance < 0.0 { 1.0 } else { 0.0 };
        }

        self.aa.compute_aa(sdf_distance, screen_derivative)
    }

    // =========================================================================
    // PROCESAMIENTO DE BUFFER DE COLORES
    // =========================================================================

    /// Procesar un buffer de colores completo con AA
    /// Útil para post-proceso
    pub fn process_color_buffer(&mut self, colors: &mut [Vec4], sdf_values: &[f32]) {
        if !self.enabled || colors.len() != sdf_values.len() {
            return;
        }

        let pixel_count = colors.len();
        let mut edges_detected = 0u64;
        let mut pixels_smoothed = 0u64;

        for i in 0..pixel_count {
            let sdf = sdf_values[i];
            
            // Detectar si estamos cerca de un borde
            let is_edge = sdf.abs() < self.aa.config.edge_width * 0.1;
            
            if is_edge {
                edges_detected += 1;
                
                // Calcular gradiente aproximado
                let gradient = self.estimate_gradient(sdf_values, i, self.width as usize);
                
                // Aplicar AA
                let coverage = self.aa.compute_edge_coverage(sdf, gradient);
                colors[i].w *= coverage;
                
                pixels_smoothed += 1;
            }
        }

        // Actualizar estadísticas
        self.stats.pixels_processed = pixel_count as u64;
        self.stats.edges_detected = edges_detected;
        self.stats.pixels_smoothed = pixels_smoothed;
    }

    /// Estimar gradiente desde buffer de SDF
    fn estimate_gradient(&self, sdf_values: &[f32], index: usize, width: usize) -> Vec2 {
        let len = sdf_values.len();
        
        // Obtener valores vecinos
        let center = sdf_values[index];
        let right = if index + 1 < len { sdf_values[index + 1] } else { center };
        let up = if index >= width { sdf_values[index - width] } else { center };
        
        Vec2::new(right - center, up - center)
    }

    // =========================================================================
    // FUNCIONES DE UTILIDAD
    // =========================================================================

    /// Obtener tamaño de píxel en unidades de pantalla
    #[inline]
    pub fn pixel_size(&self) -> f32 {
        1.0 / self.width.max(self.height) as f32
    }

    /// Avanzar frame (para AA temporal)
    pub fn next_frame(&mut self) {
        self.aa.next_frame();
    }

    /// Obtener jitter temporal para TAA
    pub fn get_temporal_jitter(&self) -> Vec2 {
        self.aa.get_temporal_jitter()
    }

    /// Imprimir estadísticas
    pub fn print_stats(&self) {
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                    AA Pipeline Stats                             ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║ Quality:          {:20}                         ║", self.stats.current_quality);
        println!("║ Enabled:          {:20}                         ║", if self.enabled { "Yes" } else { "No" });
        println!("║ Resolution:       {:4}x{:<4}                                      ║", self.width, self.height);
        println!("║ Pixels Processed: {:10}                                    ║", self.stats.pixels_processed);
        println!("║ Edges Detected:   {:10}                                    ║", self.stats.edges_detected);
        println!("║ Pixels Smoothed:  {:10}                                    ║", self.stats.pixels_smoothed);
        println!("╚══════════════════════════════════════════════════════════════════╝");
    }
}

impl Default for AAGlobalPipeline {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}

// =============================================================================
// FUNCIONES GLOBALES DE AA (para uso directo en shaders/CPU)
// =============================================================================

/// Calcular cobertura de borde con matemáticas SDF puras
/// Función global para uso en cualquier parte del motor
#[inline]
pub fn sdf_edge_coverage(sdf_distance: f32, edge_width: f32) -> f32 {
    smootherstep_ultra(-edge_width * 0.5, edge_width * 0.5, -sdf_distance)
}

/// Suavizar transición entre dos colores usando SDF
#[inline]
pub fn sdf_blend_colors(color_a: Vec4, color_b: Vec4, sdf_distance: f32, edge_width: f32) -> Vec4 {
    let t = smootherstep_ultra(-edge_width, edge_width, sdf_distance);
    color_a * (1.0 - t) + color_b * t
}

/// Calcular alpha de borde para anti-aliasing
#[inline]
pub fn sdf_edge_alpha(sdf_distance: f32, gradient_length: f32) -> f32 {
    if gradient_length < 0.0001 {
        return if sdf_distance < 0.0 { 1.0 } else { 0.0 };
    }
    
    let pixel_width = 1.5 / gradient_length;
    smootherstep_ultra(-pixel_width * 0.5, pixel_width * 0.5, -sdf_distance)
}

/// Aplicar AA a normal de superficie
#[inline]
pub fn smooth_normal(normal: Vec3, neighbor_normals: &[Vec3], smoothness: f32) -> Vec3 {
    if neighbor_normals.is_empty() {
        return normal;
    }
    
    let mut avg = normal;
    for n in neighbor_normals {
        avg += *n;
    }
    avg /= (neighbor_normals.len() + 1) as f32;
    
    // Interpolar entre normal original y promedio
    let t = smootherstep_ultra(0.0, 1.0, smoothness);
    (normal * (1.0 - t) + avg * t).normalize()
}

/// Calcular factor de fresnel suavizado (para bordes de silueta)
#[inline]
pub fn smooth_fresnel(normal: Vec3, view_dir: Vec3, power: f32) -> f32 {
    let n_dot_v = normal.dot(view_dir).abs();
    let fresnel = (1.0 - n_dot_v).powf(power);
    
    // Suavizar el fresnel
    smootherstep_ultra(0.0, 1.0, fresnel)
}

// =============================================================================
// PRESETS DE AA PARA DIFERENTES ESCENARIOS
// =============================================================================

/// Preset de AA para UI/2D
pub fn aa_preset_ui() -> SDFAntiAliasing {
    SDFAntiAliasing::with_config(SDFAAConfig {
        quality: AAQuality::High,
        edge_width: 1.0,
        smoothness: 1.5,
        temporal_aa: false,
        adaptive_supersampling: false,
        sample_count: 1,
        geometric_aa: true,
        subpixel_aa: true,
        subpixel_intensity: 1.0,
        gamma_correct: true,
        gamma: 2.2,
        ..Default::default()
    })
}

/// Preset de AA para 3D en tiempo real
pub fn aa_preset_realtime_3d() -> SDFAntiAliasing {
    SDFAntiAliasing::with_config(SDFAAConfig {
        quality: AAQuality::High,
        edge_width: 1.5,
        smoothness: 1.0,
        temporal_aa: true,
        temporal_blend: 0.15,
        adaptive_supersampling: true,
        sample_count: 8,
        geometric_aa: true,
        subpixel_aa: true,
        subpixel_intensity: 0.75,
        gamma_correct: true,
        gamma: 2.2,
    })
}

/// Preset de AA para renderizado offline (máxima calidad)
pub fn aa_preset_offline() -> SDFAntiAliasing {
    SDFAntiAliasing::with_config(SDFAAConfig {
        quality: AAQuality::Cinematic,
        edge_width: 2.5,
        smoothness: 2.0,
        temporal_aa: true,
        temporal_blend: 0.25,
        adaptive_supersampling: true,
        sample_count: 32,
        geometric_aa: true,
        subpixel_aa: true,
        subpixel_intensity: 1.0,
        gamma_correct: true,
        gamma: 2.2,
    })
}
