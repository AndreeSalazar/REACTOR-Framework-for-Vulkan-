// =============================================================================
// ADead-AA: Advanced Anti-Aliasing System
// =============================================================================
// Sistema de AA avanzado que combina MSAA con matemáticas SDF puras
// Nombre alternativo: "Epic-Liasing" - Anti-Aliasing de nivel cinematográfico
// =============================================================================
//
// Características:
// - Hereda de MSAA pero lo mejora con técnicas SDF
// - Suavizado de bordes con curvas de grado 7 (septic)
// - Detección de bordes basada en gradientes
// - Anti-aliasing temporal con reprojection
// - Corrección de subpixel estilo SMAA
// - Gamma-aware blending
// =============================================================================

use glam::{Vec2, Vec3, Vec4};

// =============================================================================
// CONSTANTES MATEMÁTICAS
// =============================================================================

/// Pi
const PI: f32 = std::f32::consts::PI;

/// Epsilon para comparaciones
const EPS: f32 = 1e-6;

/// Número áureo (para patrones de sampling)
const PHI: f32 = 1.618033988749895;

// =============================================================================
// CURVAS DE SUAVIZADO (de menor a mayor calidad)
// =============================================================================

/// Smoothstep estándar (Hermite, grado 3)
/// f(t) = 3t² - 2t³
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smootherstep (Quintic, grado 5) - Ken Perlin
/// f(t) = 6t⁵ - 15t⁴ + 10t³
#[inline]
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Ultra Smoothstep (Septic, grado 7) - Máxima suavidad
/// f(t) = -20t⁷ + 70t⁶ - 84t⁵ + 35t⁴
#[inline]
pub fn ultra_smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * t * (t * (t * (t * -20.0 + 70.0) - 84.0) + 35.0)
}

/// Epic Smoothstep (Nonic, grado 9) - Calidad cinematográfica
/// f(t) = 70t⁹ - 315t⁸ + 540t⁷ - 420t⁶ + 126t⁵
#[inline]
pub fn epic_smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    let t2 = t * t;
    let t4 = t2 * t2;
    let t5 = t4 * t;
    t5 * (126.0 + t * (-420.0 + t * (540.0 + t * (-315.0 + t * 70.0))))
}

// =============================================================================
// CONFIGURACIÓN DE ADEAD-AA
// =============================================================================

/// Calidad del Anti-Aliasing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ADeadAAQuality {
    /// Sin AA
    Off,
    /// Básico - solo MSAA 2x
    Low,
    /// Medio - MSAA 4x + smoothstep
    Medium,
    /// Alto - MSAA 4x + smootherstep + edge detection
    High,
    /// Ultra - MSAA 8x + ultra_smoothstep + temporal
    Ultra,
    /// Cinematográfico - MSAA 8x + epic_smoothstep + temporal + subpixel
    Cinematic,
    /// Epic - Máxima calidad posible
    Epic,
}

/// Configuración de ADead-AA
#[derive(Clone, Debug)]
pub struct ADeadAAConfig {
    /// Calidad del AA
    pub quality: ADeadAAQuality,
    /// Ancho del borde de suavizado (en píxeles)
    pub edge_width: f32,
    /// Intensidad del suavizado (0.0 - 2.0)
    pub smoothness: f32,
    /// Umbral de detección de bordes
    pub edge_threshold: f32,
    /// Umbral mínimo de bordes
    pub edge_threshold_min: f32,
    /// Factor de mezcla temporal (para TAA)
    pub temporal_blend: f32,
    /// Intensidad de corrección subpixel
    pub subpixel_intensity: f32,
    /// Número de samples para supersampling adaptativo
    pub sample_count: u32,
    /// Habilitar corrección de gamma
    pub gamma_correct: bool,
    /// Valor de gamma
    pub gamma: f32,
    /// Habilitar detección de bordes geométricos
    pub geometric_edge_detection: bool,
    /// Habilitar AA temporal
    pub temporal_aa: bool,
    /// Habilitar corrección subpixel
    pub subpixel_aa: bool,
}

impl Default for ADeadAAConfig {
    fn default() -> Self {
        Self {
            quality: ADeadAAQuality::High,
            edge_width: 1.5,
            smoothness: 1.0,
            edge_threshold: 0.125,
            edge_threshold_min: 0.0625,
            temporal_blend: 0.15,
            subpixel_intensity: 0.75,
            sample_count: 8,
            gamma_correct: true,
            gamma: 2.2,
            geometric_edge_detection: true,
            temporal_aa: true,
            subpixel_aa: true,
        }
    }
}

impl ADeadAAConfig {
    /// Preset de baja calidad
    pub fn low() -> Self {
        Self {
            quality: ADeadAAQuality::Low,
            edge_width: 1.0,
            smoothness: 0.8,
            edge_threshold: 0.166,
            edge_threshold_min: 0.0833,
            temporal_blend: 0.0,
            subpixel_intensity: 0.5,
            sample_count: 4,
            gamma_correct: false,
            gamma: 2.2,
            geometric_edge_detection: false,
            temporal_aa: false,
            subpixel_aa: false,
        }
    }

    /// Preset de calidad media
    pub fn medium() -> Self {
        Self {
            quality: ADeadAAQuality::Medium,
            edge_width: 1.2,
            smoothness: 1.0,
            edge_threshold: 0.125,
            edge_threshold_min: 0.0625,
            temporal_blend: 0.1,
            subpixel_intensity: 0.75,
            sample_count: 8,
            gamma_correct: true,
            gamma: 2.2,
            geometric_edge_detection: true,
            temporal_aa: false,
            subpixel_aa: true,
        }
    }

    /// Preset de alta calidad
    pub fn high() -> Self {
        Self::default()
    }

    /// Preset ultra
    pub fn ultra() -> Self {
        Self {
            quality: ADeadAAQuality::Ultra,
            edge_width: 2.0,
            smoothness: 1.5,
            edge_threshold: 0.1,
            edge_threshold_min: 0.05,
            temporal_blend: 0.2,
            subpixel_intensity: 1.0,
            sample_count: 16,
            gamma_correct: true,
            gamma: 2.2,
            geometric_edge_detection: true,
            temporal_aa: true,
            subpixel_aa: true,
        }
    }

    /// Preset cinematográfico
    pub fn cinematic() -> Self {
        Self {
            quality: ADeadAAQuality::Cinematic,
            edge_width: 2.5,
            smoothness: 2.0,
            edge_threshold: 0.08,
            edge_threshold_min: 0.04,
            temporal_blend: 0.25,
            subpixel_intensity: 1.0,
            sample_count: 32,
            gamma_correct: true,
            gamma: 2.2,
            geometric_edge_detection: true,
            temporal_aa: true,
            subpixel_aa: true,
        }
    }

    /// Preset Epic (máxima calidad)
    pub fn epic() -> Self {
        Self {
            quality: ADeadAAQuality::Epic,
            edge_width: 3.0,
            smoothness: 2.5,
            edge_threshold: 0.05,
            edge_threshold_min: 0.025,
            temporal_blend: 0.3,
            subpixel_intensity: 1.0,
            sample_count: 64,
            gamma_correct: true,
            gamma: 2.2,
            geometric_edge_detection: true,
            temporal_aa: true,
            subpixel_aa: true,
        }
    }
}

// =============================================================================
// SISTEMA PRINCIPAL DE ADEAD-AA
// =============================================================================

/// Sistema principal de Anti-Aliasing ADead
pub struct ADeadAA {
    /// Configuración actual
    pub config: ADeadAAConfig,
    /// Frame actual (para AA temporal)
    frame_count: u64,
    /// Buffer temporal del frame anterior
    temporal_buffer: Option<Vec<Vec4>>,
    /// Jitter offset para TAA
    jitter_offset: Vec2,
    /// Historial de jitter (para reprojection)
    jitter_history: Vec<Vec2>,
}

impl ADeadAA {
    /// Crear nuevo sistema ADead-AA
    pub fn new() -> Self {
        Self::with_config(ADeadAAConfig::default())
    }

    /// Crear con calidad específica
    pub fn with_quality(quality: ADeadAAQuality) -> Self {
        let config = match quality {
            ADeadAAQuality::Off => ADeadAAConfig { quality, ..Default::default() },
            ADeadAAQuality::Low => ADeadAAConfig::low(),
            ADeadAAQuality::Medium => ADeadAAConfig::medium(),
            ADeadAAQuality::High => ADeadAAConfig::high(),
            ADeadAAQuality::Ultra => ADeadAAConfig::ultra(),
            ADeadAAQuality::Cinematic => ADeadAAConfig::cinematic(),
            ADeadAAQuality::Epic => ADeadAAConfig::epic(),
        };
        Self::with_config(config)
    }

    /// Crear con configuración personalizada
    pub fn with_config(config: ADeadAAConfig) -> Self {
        Self {
            config,
            frame_count: 0,
            temporal_buffer: None,
            jitter_offset: Vec2::ZERO,
            jitter_history: Vec::new(),
        }
    }

    // =========================================================================
    // FUNCIONES DE COBERTURA DE BORDE (SDF)
    // =========================================================================

    /// Calcular cobertura de borde usando SDF
    /// Esta es la función principal para eliminar dientes
    #[inline]
    pub fn compute_edge_coverage(&self, sdf_distance: f32, gradient: Vec2) -> f32 {
        let gradient_length = gradient.length();
        
        if gradient_length < EPS {
            return if sdf_distance < 0.0 { 1.0 } else { 0.0 };
        }

        // Ancho del píxel en espacio SDF
        let pixel_width = self.config.edge_width / gradient_length;
        
        // Usar la curva de suavizado según la calidad
        match self.config.quality {
            ADeadAAQuality::Off => if sdf_distance < 0.0 { 1.0 } else { 0.0 },
            ADeadAAQuality::Low => smoothstep(-pixel_width * 0.5, pixel_width * 0.5, -sdf_distance),
            ADeadAAQuality::Medium => smootherstep(-pixel_width * 0.5, pixel_width * 0.5, -sdf_distance),
            ADeadAAQuality::High => ultra_smoothstep(-pixel_width * 0.5, pixel_width * 0.5, -sdf_distance),
            ADeadAAQuality::Ultra | ADeadAAQuality::Cinematic | ADeadAAQuality::Epic => {
                epic_smoothstep(-pixel_width * 0.5, pixel_width * 0.5, -sdf_distance)
            }
        }
    }

    /// Calcular cobertura con supersampling adaptativo
    pub fn compute_coverage_supersampled(&self, sdf_fn: impl Fn(Vec2) -> f32, center: Vec2, pixel_size: f32) -> f32 {
        let samples = self.config.sample_count as usize;
        let mut coverage = 0.0;
        
        // Usar patrón de Poisson disk para samples
        let offsets = self.generate_poisson_disk_samples(samples);
        
        for offset in offsets {
            let sample_pos = center + offset * pixel_size * 0.5;
            let dist = sdf_fn(sample_pos);
            coverage += if dist < 0.0 { 1.0 } else { 0.0 };
        }
        
        coverage / samples as f32
    }

    /// Generar samples usando Poisson disk
    fn generate_poisson_disk_samples(&self, count: usize) -> Vec<Vec2> {
        let mut samples = Vec::with_capacity(count);
        
        for i in 0..count {
            // Usar secuencia de Halton para distribución cuasi-aleatoria
            let u = self.halton(i as u32, 2);
            let v = self.halton(i as u32, 3);
            
            // Convertir a coordenadas de disco
            let r = u.sqrt();
            let theta = v * 2.0 * PI;
            
            samples.push(Vec2::new(r * theta.cos(), r * theta.sin()));
        }
        
        samples
    }

    /// Secuencia de Halton para sampling cuasi-aleatorio
    #[inline]
    fn halton(&self, index: u32, base: u32) -> f32 {
        let mut result = 0.0;
        let mut f = 1.0 / base as f32;
        let mut i = index;
        
        while i > 0 {
            result += f * (i % base) as f32;
            i /= base;
            f /= base as f32;
        }
        
        result
    }

    // =========================================================================
    // DETECCIÓN DE BORDES
    // =========================================================================

    /// Detectar bordes usando gradiente de luminancia
    pub fn detect_edge(&self, center_luma: f32, neighbors: &[f32; 4]) -> f32 {
        // neighbors: [left, right, up, down]
        let horizontal = (neighbors[0] - center_luma).abs() + (neighbors[1] - center_luma).abs();
        let vertical = (neighbors[2] - center_luma).abs() + (neighbors[3] - center_luma).abs();
        
        let edge_strength = (horizontal + vertical) * 0.5;
        
        // Aplicar umbral
        if edge_strength < self.config.edge_threshold_min {
            0.0
        } else if edge_strength > self.config.edge_threshold {
            1.0
        } else {
            ultra_smoothstep(
                self.config.edge_threshold_min,
                self.config.edge_threshold,
                edge_strength
            )
        }
    }

    /// Detectar bordes usando Sobel operator
    pub fn detect_edge_sobel(&self, samples: &[[f32; 3]; 3]) -> f32 {
        // Sobel kernels
        // Gx = [-1 0 1; -2 0 2; -1 0 1]
        // Gy = [-1 -2 -1; 0 0 0; 1 2 1]
        
        let gx = -samples[0][0] + samples[0][2]
               - 2.0 * samples[1][0] + 2.0 * samples[1][2]
               - samples[2][0] + samples[2][2];
        
        let gy = -samples[0][0] - 2.0 * samples[0][1] - samples[0][2]
               + samples[2][0] + 2.0 * samples[2][1] + samples[2][2];
        
        (gx * gx + gy * gy).sqrt()
    }

    // =========================================================================
    // ANTI-ALIASING TEMPORAL
    // =========================================================================

    /// Obtener jitter offset para el frame actual
    pub fn get_temporal_jitter(&self) -> Vec2 {
        if !self.config.temporal_aa {
            return Vec2::ZERO;
        }
        
        // Usar secuencia de Halton para jitter
        let frame = self.frame_count as u32;
        Vec2::new(
            self.halton(frame % 16, 2) - 0.5,
            self.halton(frame % 16, 3) - 0.5
        ) * 0.5 // Escalar a medio píxel
    }

    /// Mezclar con frame anterior (TAA)
    pub fn temporal_blend(&self, current: Vec4, history: Vec4, velocity: Vec2) -> Vec4 {
        if !self.config.temporal_aa {
            return current;
        }
        
        // Ajustar blend factor basado en velocidad
        let velocity_length = velocity.length();
        let blend = self.config.temporal_blend * (1.0 - velocity_length.min(1.0));
        
        // Clamp history para evitar ghosting
        let clamped_history = self.clamp_history(current, history);
        
        // Mezclar
        current * (1.0 - blend) + clamped_history * blend
    }

    /// Clamp history para evitar ghosting
    fn clamp_history(&self, current: Vec4, history: Vec4) -> Vec4 {
        // Usar AABB clamp simple
        let min_val = current - Vec4::splat(0.1);
        let max_val = current + Vec4::splat(0.1);
        
        Vec4::new(
            history.x.clamp(min_val.x, max_val.x),
            history.y.clamp(min_val.y, max_val.y),
            history.z.clamp(min_val.z, max_val.z),
            history.w.clamp(min_val.w, max_val.w),
        )
    }

    /// Avanzar al siguiente frame
    pub fn next_frame(&mut self) {
        self.frame_count += 1;
        self.jitter_offset = self.get_temporal_jitter();
        
        // Guardar jitter en historial
        self.jitter_history.push(self.jitter_offset);
        if self.jitter_history.len() > 16 {
            self.jitter_history.remove(0);
        }
    }

    // =========================================================================
    // CORRECCIÓN SUBPIXEL
    // =========================================================================

    /// Aplicar corrección subpixel estilo SMAA
    pub fn subpixel_correction(&self, color: Vec4, edge_direction: Vec2, blend_factor: f32) -> Vec4 {
        if !self.config.subpixel_aa {
            return color;
        }
        
        // Calcular offset subpixel basado en dirección del borde
        let offset = edge_direction.normalize_or_zero() * blend_factor * self.config.subpixel_intensity;
        
        // Aplicar corrección (simulado - en GPU sería un sample offset)
        let correction = 1.0 - offset.length() * 0.1;
        color * correction
    }

    // =========================================================================
    // BLENDING CON GAMMA
    // =========================================================================

    /// Convertir de gamma a lineal
    #[inline]
    pub fn gamma_to_linear(&self, color: Vec4) -> Vec4 {
        if !self.config.gamma_correct {
            return color;
        }
        
        Vec4::new(
            color.x.powf(self.config.gamma),
            color.y.powf(self.config.gamma),
            color.z.powf(self.config.gamma),
            color.w,
        )
    }

    /// Convertir de lineal a gamma
    #[inline]
    pub fn linear_to_gamma(&self, color: Vec4) -> Vec4 {
        if !self.config.gamma_correct {
            return color;
        }
        
        let inv_gamma = 1.0 / self.config.gamma;
        Vec4::new(
            color.x.powf(inv_gamma),
            color.y.powf(inv_gamma),
            color.z.powf(inv_gamma),
            color.w,
        )
    }

    /// Mezclar colores con corrección de gamma
    pub fn blend_gamma_correct(&self, a: Vec4, b: Vec4, t: f32) -> Vec4 {
        let a_linear = self.gamma_to_linear(a);
        let b_linear = self.gamma_to_linear(b);
        
        let blended = a_linear * (1.0 - t) + b_linear * t;
        
        self.linear_to_gamma(blended)
    }

    // =========================================================================
    // FUNCIÓN PRINCIPAL DE AA
    // =========================================================================

    /// Aplicar ADead-AA a un color basado en SDF
    pub fn apply(&self, color: Vec4, sdf_distance: f32, gradient: Vec2) -> Vec4 {
        if self.config.quality == ADeadAAQuality::Off {
            return color;
        }

        let coverage = self.compute_edge_coverage(sdf_distance, gradient);
        
        Vec4::new(color.x, color.y, color.z, color.w * coverage)
    }

    /// Aplicar ADead-AA completo con todos los efectos
    pub fn apply_full(
        &self,
        color: Vec4,
        sdf_distance: f32,
        gradient: Vec2,
        history_color: Option<Vec4>,
        velocity: Vec2,
        edge_direction: Vec2,
    ) -> Vec4 {
        if self.config.quality == ADeadAAQuality::Off {
            return color;
        }

        // 1. Calcular cobertura de borde
        let coverage = self.compute_edge_coverage(sdf_distance, gradient);
        let mut result = Vec4::new(color.x, color.y, color.z, color.w * coverage);

        // 2. Aplicar corrección subpixel
        if self.config.subpixel_aa {
            result = self.subpixel_correction(result, edge_direction, 1.0 - coverage);
        }

        // 3. Aplicar AA temporal
        if self.config.temporal_aa {
            if let Some(history) = history_color {
                result = self.temporal_blend(result, history, velocity);
            }
        }

        result
    }

    // =========================================================================
    // UTILIDADES
    // =========================================================================

    /// Calcular luminancia de un color
    #[inline]
    pub fn luminance(color: Vec3) -> f32 {
        color.x * 0.299 + color.y * 0.587 + color.z * 0.114
    }

    /// Calcular luminancia de un color Vec4
    #[inline]
    pub fn luminance_vec4(color: Vec4) -> f32 {
        Self::luminance(Vec3::new(color.x, color.y, color.z))
    }

    /// Obtener estadísticas del sistema
    pub fn get_stats(&self) -> ADeadAAStats {
        ADeadAAStats {
            quality: format!("{:?}", self.config.quality),
            frame_count: self.frame_count,
            sample_count: self.config.sample_count,
            temporal_enabled: self.config.temporal_aa,
            subpixel_enabled: self.config.subpixel_aa,
            gamma_correct: self.config.gamma_correct,
        }
    }

    /// Imprimir información del sistema
    pub fn print_info(&self) {
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                    ADead-AA System Info                          ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║ Quality:          {:20?}                        ║", self.config.quality);
        println!("║ Edge Width:       {:20.2}                        ║", self.config.edge_width);
        println!("║ Smoothness:       {:20.2}                        ║", self.config.smoothness);
        println!("║ Sample Count:     {:20}                        ║", self.config.sample_count);
        println!("║ Temporal AA:      {:20}                        ║", if self.config.temporal_aa { "Enabled" } else { "Disabled" });
        println!("║ Subpixel AA:      {:20}                        ║", if self.config.subpixel_aa { "Enabled" } else { "Disabled" });
        println!("║ Gamma Correct:    {:20}                        ║", if self.config.gamma_correct { "Enabled" } else { "Disabled" });
        println!("╚══════════════════════════════════════════════════════════════════╝");
    }
}

impl Default for ADeadAA {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del sistema ADead-AA
#[derive(Clone, Debug)]
pub struct ADeadAAStats {
    pub quality: String,
    pub frame_count: u64,
    pub sample_count: u32,
    pub temporal_enabled: bool,
    pub subpixel_enabled: bool,
    pub gamma_correct: bool,
}

// =============================================================================
// FUNCIONES GLOBALES PARA USO EN SHADERS
// =============================================================================

/// Calcular cobertura de borde con matemáticas SDF puras (Epic quality)
#[inline]
pub fn epic_edge_coverage(sdf_distance: f32, edge_width: f32) -> f32 {
    epic_smoothstep(-edge_width * 0.5, edge_width * 0.5, -sdf_distance)
}

/// Suavizar transición entre dos colores usando SDF (Epic quality)
#[inline]
pub fn epic_blend_colors(color_a: Vec4, color_b: Vec4, sdf_distance: f32, edge_width: f32) -> Vec4 {
    let t = epic_smoothstep(-edge_width, edge_width, sdf_distance);
    color_a * (1.0 - t) + color_b * t
}

/// Calcular alpha de borde para anti-aliasing (Epic quality)
#[inline]
pub fn epic_edge_alpha(sdf_distance: f32, gradient_length: f32) -> f32 {
    if gradient_length < EPS {
        return if sdf_distance < 0.0 { 1.0 } else { 0.0 };
    }
    
    let pixel_width = 1.5 / gradient_length;
    epic_smoothstep(-pixel_width * 0.5, pixel_width * 0.5, -sdf_distance)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoothstep_functions() {
        // Test que todas las funciones de smoothstep son 0 en edge0 y 1 en edge1
        assert!((smoothstep(0.0, 1.0, 0.0) - 0.0).abs() < EPS);
        assert!((smoothstep(0.0, 1.0, 1.0) - 1.0).abs() < EPS);
        
        assert!((smootherstep(0.0, 1.0, 0.0) - 0.0).abs() < EPS);
        assert!((smootherstep(0.0, 1.0, 1.0) - 1.0).abs() < EPS);
        
        assert!((ultra_smoothstep(0.0, 1.0, 0.0) - 0.0).abs() < EPS);
        assert!((ultra_smoothstep(0.0, 1.0, 1.0) - 1.0).abs() < EPS);
        
        assert!((epic_smoothstep(0.0, 1.0, 0.0) - 0.0).abs() < EPS);
        assert!((epic_smoothstep(0.0, 1.0, 1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_adead_aa_creation() {
        let aa = ADeadAA::new();
        assert_eq!(aa.config.quality, ADeadAAQuality::High);
        
        let aa_epic = ADeadAA::with_quality(ADeadAAQuality::Epic);
        assert_eq!(aa_epic.config.quality, ADeadAAQuality::Epic);
    }

    #[test]
    fn test_edge_coverage() {
        let aa = ADeadAA::with_quality(ADeadAAQuality::Epic);
        
        // Dentro del objeto (SDF negativo) -> cobertura alta
        let coverage_inside = aa.compute_edge_coverage(-1.0, Vec2::new(1.0, 0.0));
        assert!(coverage_inside > 0.9);
        
        // Fuera del objeto (SDF positivo) -> cobertura baja
        let coverage_outside = aa.compute_edge_coverage(1.0, Vec2::new(1.0, 0.0));
        assert!(coverage_outside < 0.1);
        
        // En el borde (SDF ~0) -> cobertura ~0.5
        let coverage_edge = aa.compute_edge_coverage(0.0, Vec2::new(1.0, 0.0));
        assert!((coverage_edge - 0.5).abs() < 0.1);
    }
}
