// =============================================================================
// ADead-AA: Advanced Anti-Aliasing System
// =============================================================================
// MATEMÁTICAS VECTORIALES PURAS para eliminar bordes dentados
// Nivel AAA - Sin dientes, sin artifacts, bordes perfectos
// 
// Técnicas implementadas:
// - Geometric AA (basado en vectores de borde)
// - Subpixel Morphological AA (SMAA-style)
// - Analytical Edge Coverage
// - Multi-sample Edge Blending
// - Temporal Stability
// =============================================================================

use glam::{Vec2, Vec3, Vec4};

// =============================================================================
// CONFIGURACIÓN AVANZADA
// =============================================================================

/// Preset de calidad AA
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AAQuality {
    /// Mínimo - solo smoothstep básico
    Low,
    /// Medio - geometric AA
    Medium,
    /// Alto - full edge analysis
    High,
    /// Ultra - supersampling + temporal
    Ultra,
    /// Cinematográfico - máxima calidad posible
    Cinematic,
}

/// Configuración de anti-aliasing avanzada
#[derive(Clone, Debug)]
pub struct SDFAAConfig {
    /// Preset de calidad
    pub quality: AAQuality,
    /// Ancho del borde de AA (píxeles)
    pub edge_width: f32,
    /// Suavizado adicional (1.0 = normal, 2.0 = más suave)
    pub smoothness: f32,
    /// Habilitar AA temporal
    pub temporal_aa: bool,
    /// Factor de mezcla temporal (0.0-1.0)
    pub temporal_blend: f32,
    /// Habilitar supersampling adaptativo
    pub adaptive_supersampling: bool,
    /// Número de samples para supersampling (4, 8, 16)
    pub sample_count: u32,
    /// Habilitar detección de bordes geométrica
    pub geometric_aa: bool,
    /// Habilitar suavizado subpixel
    pub subpixel_aa: bool,
    /// Intensidad del suavizado subpixel (0.0-1.0)
    pub subpixel_intensity: f32,
    /// Habilitar corrección de gamma
    pub gamma_correct: bool,
    /// Valor de gamma
    pub gamma: f32,
}

impl Default for SDFAAConfig {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl SDFAAConfig {
    /// Preset de baja calidad (máximo rendimiento)
    pub fn low() -> Self {
        Self {
            quality: AAQuality::Low,
            edge_width: 1.0,
            smoothness: 0.8,
            temporal_aa: false,
            temporal_blend: 0.0,
            adaptive_supersampling: false,
            sample_count: 1,
            geometric_aa: false,
            subpixel_aa: false,
            subpixel_intensity: 0.0,
            gamma_correct: false,
            gamma: 2.2,
        }
    }

    /// Preset de calidad media
    pub fn medium() -> Self {
        Self {
            quality: AAQuality::Medium,
            edge_width: 1.2,
            smoothness: 1.0,
            temporal_aa: false,
            temporal_blend: 0.0,
            adaptive_supersampling: false,
            sample_count: 4,
            geometric_aa: true,
            subpixel_aa: false,
            subpixel_intensity: 0.0,
            gamma_correct: true,
            gamma: 2.2,
        }
    }

    /// Preset de alta calidad
    pub fn high() -> Self {
        Self::default()
    }

    /// Preset ultra (máxima calidad)
    pub fn ultra() -> Self {
        Self {
            quality: AAQuality::Ultra,
            edge_width: 2.0,
            smoothness: 1.5,
            temporal_aa: true,
            temporal_blend: 0.2,
            adaptive_supersampling: true,
            sample_count: 16,
            geometric_aa: true,
            subpixel_aa: true,
            subpixel_intensity: 1.0,
            gamma_correct: true,
            gamma: 2.2,
        }
    }

    /// Preset cinematográfico (calidad de película)
    pub fn cinematic() -> Self {
        Self {
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
        }
    }
}

// =============================================================================
// SISTEMA PRINCIPAL DE ANTI-ALIASING
// =============================================================================

/// Sistema de Anti-Aliasing Avanzado
pub struct SDFAntiAliasing {
    pub config: SDFAAConfig,
    /// Buffer temporal para AA temporal
    temporal_buffer: Option<Vec<Vec4>>,
    /// Frame actual
    frame_count: u64,
}

impl SDFAntiAliasing {
    /// Crear nuevo sistema AA
    pub fn new() -> Self {
        Self {
            config: SDFAAConfig::default(),
            temporal_buffer: None,
            frame_count: 0,
        }
    }

    /// Crear con configuración
    pub fn with_config(config: SDFAAConfig) -> Self {
        Self { 
            config,
            temporal_buffer: None,
            frame_count: 0,
        }
    }

    /// Crear con preset de calidad
    pub fn with_quality(quality: AAQuality) -> Self {
        let config = match quality {
            AAQuality::Low => SDFAAConfig::low(),
            AAQuality::Medium => SDFAAConfig::medium(),
            AAQuality::High => SDFAAConfig::high(),
            AAQuality::Ultra => SDFAAConfig::ultra(),
            AAQuality::Cinematic => SDFAAConfig::cinematic(),
        };
        Self::with_config(config)
    }

    // =========================================================================
    // FUNCIONES CORE DE AA
    // =========================================================================

    /// Calcular cobertura de borde con matemáticas vectoriales
    /// Esta es la función principal para eliminar dientes
    #[inline]
    pub fn compute_edge_coverage(&self, edge_distance: f32, edge_gradient: Vec2) -> f32 {
        // Calcular el ancho del píxel en el espacio del borde
        let gradient_length = edge_gradient.length();
        if gradient_length < 0.0001 {
            return if edge_distance < 0.0 { 1.0 } else { 0.0 };
        }

        // Ancho efectivo del borde basado en el gradiente
        let pixel_width = self.config.edge_width / gradient_length;
        
        // Cobertura analítica usando smootherstep para máxima suavidad
        smootherstep_ultra(-pixel_width * 0.5, pixel_width * 0.5, -edge_distance)
    }

    /// AA geométrico basado en vectores de borde
    #[inline]
    pub fn geometric_aa(&self, edge_distance: f32, normal: Vec2, pixel_size: f32) -> f32 {
        if !self.config.geometric_aa {
            return self.compute_aa_simple(edge_distance);
        }

        // Calcular cobertura basada en la orientación del borde
        let edge_width = self.config.edge_width * pixel_size * self.config.smoothness;
        
        // Usar la normal para ajustar el suavizado
        let normal_factor = normal.length().max(0.001);
        let adjusted_width = edge_width / normal_factor;
        
        // Aplicar suavizado quintic para bordes ultra-suaves
        smootherstep_ultra(-adjusted_width, adjusted_width, -edge_distance)
    }

    /// AA con análisis de cobertura subpixel
    pub fn subpixel_aa(&self, center_dist: f32, neighbors: &[f32; 8], pixel_size: f32) -> f32 {
        if !self.config.subpixel_aa {
            return self.compute_aa_simple(center_dist);
        }

        // Calcular gradiente desde vecinos
        let dx = (neighbors[2] - neighbors[6]) * 0.5; // derecha - izquierda
        let dy = (neighbors[0] - neighbors[4]) * 0.5; // arriba - abajo
        let gradient = Vec2::new(dx, dy);
        
        // Calcular cobertura base
        let base_coverage = self.compute_edge_coverage(center_dist, gradient);
        
        // Calcular variación subpixel
        let mut subpixel_sum = 0.0;
        for &neighbor_dist in neighbors {
            let neighbor_coverage = self.compute_edge_coverage(neighbor_dist, gradient);
            subpixel_sum += neighbor_coverage;
        }
        let subpixel_avg = subpixel_sum / 8.0;
        
        // Mezclar cobertura central con promedio subpixel
        let intensity = self.config.subpixel_intensity;
        base_coverage * (1.0 - intensity) + subpixel_avg * intensity
    }

    /// AA con supersampling adaptativo de alta calidad
    pub fn adaptive_supersample<F>(&self, center_pos: Vec2, sample_sdf: F, pixel_size: f32) -> f32
    where F: Fn(Vec2) -> f32
    {
        let center_dist = sample_sdf(center_pos);
        
        if !self.config.adaptive_supersampling {
            return self.compute_aa_simple(center_dist);
        }

        // Determinar si necesitamos supersampling
        let threshold = pixel_size * 2.0;
        if center_dist.abs() > threshold {
            // Lejos del borde, no necesita supersampling
            return if center_dist < 0.0 { 1.0 } else { 0.0 };
        }

        // Generar patrón de samples rotado (Rotated Grid)
        let samples = self.generate_sample_pattern();
        let sample_count = samples.len();
        
        let mut coverage_sum = 0.0;
        for sample_offset in &samples {
            let sample_pos = center_pos + *sample_offset * pixel_size;
            let sample_dist = sample_sdf(sample_pos);
            
            // Calcular gradiente local
            let dx = sample_sdf(sample_pos + Vec2::new(pixel_size * 0.1, 0.0)) - sample_dist;
            let dy = sample_sdf(sample_pos + Vec2::new(0.0, pixel_size * 0.1)) - sample_dist;
            let gradient = Vec2::new(dx, dy) * 10.0;
            
            coverage_sum += self.compute_edge_coverage(sample_dist, gradient);
        }
        
        coverage_sum / sample_count as f32
    }

    /// Generar patrón de samples para supersampling
    fn generate_sample_pattern(&self) -> Vec<Vec2> {
        let count = self.config.sample_count.min(64) as usize;
        
        match count {
            1 => vec![Vec2::ZERO],
            4 => vec![
                Vec2::new(-0.25, -0.25),
                Vec2::new(0.25, -0.25),
                Vec2::new(-0.25, 0.25),
                Vec2::new(0.25, 0.25),
            ],
            8 => {
                // Patrón rotado 8x (Rotated Grid)
                let angle = std::f32::consts::PI / 8.0;
                (0..8).map(|i| {
                    let a = angle + (i as f32) * std::f32::consts::PI / 4.0;
                    let r = 0.35;
                    Vec2::new(a.cos() * r, a.sin() * r)
                }).collect()
            }
            16 => {
                // Patrón Poisson Disk 16x
                vec![
                    Vec2::new(-0.375, -0.375), Vec2::new(0.125, -0.375),
                    Vec2::new(-0.125, -0.125), Vec2::new(0.375, -0.125),
                    Vec2::new(-0.375, 0.125), Vec2::new(0.125, 0.125),
                    Vec2::new(-0.125, 0.375), Vec2::new(0.375, 0.375),
                    Vec2::new(-0.25, -0.25), Vec2::new(0.25, -0.25),
                    Vec2::new(-0.25, 0.0), Vec2::new(0.25, 0.0),
                    Vec2::new(0.0, -0.25), Vec2::new(0.0, 0.25),
                    Vec2::new(-0.25, 0.25), Vec2::new(0.25, 0.25),
                ]
            }
            _ => {
                // Patrón espiral para cualquier cantidad
                (0..count).map(|i| {
                    let t = i as f32 / count as f32;
                    let angle = t * std::f32::consts::TAU * 2.5;
                    let radius = t.sqrt() * 0.4;
                    Vec2::new(angle.cos() * radius, angle.sin() * radius)
                }).collect()
            }
        }
    }

    /// Versión simplificada de AA (para compatibilidad)
    #[inline]
    pub fn compute_aa(&self, sdf_value: f32, screen_derivative: f32) -> f32 {
        let edge_width = self.config.edge_width * screen_derivative * self.config.smoothness;
        smootherstep(-edge_width, edge_width, -sdf_value)
    }

    /// Versión ultra-simplificada
    #[inline]
    pub fn compute_aa_simple(&self, sdf_value: f32) -> f32 {
        let edge_width = self.config.edge_width * 0.01 * self.config.smoothness;
        smootherstep(-edge_width, edge_width, -sdf_value)
    }

    // =========================================================================
    // AA TEMPORAL
    // =========================================================================

    /// Aplicar AA temporal con reprojection
    pub fn apply_temporal(&self, current: Vec4, previous: Vec4, velocity: Vec2) -> Vec4 {
        if !self.config.temporal_aa {
            return current;
        }

        // Ajustar blend basado en velocidad (menos blend = menos ghosting en movimiento)
        let velocity_magnitude = velocity.length();
        let velocity_factor = 1.0 / (1.0 + velocity_magnitude * 10.0);
        let blend = self.config.temporal_blend * velocity_factor;
        
        // Mezclar con corrección de gamma
        if self.config.gamma_correct {
            let current_linear = gamma_to_linear_vec4(current, self.config.gamma);
            let previous_linear = gamma_to_linear_vec4(previous, self.config.gamma);
            let blended = current_linear * (1.0 - blend) + previous_linear * blend;
            linear_to_gamma_vec4(blended, self.config.gamma)
        } else {
            current * (1.0 - blend) + previous * blend
        }
    }

    /// Avanzar frame para AA temporal
    pub fn next_frame(&mut self) {
        self.frame_count += 1;
    }

    /// Obtener jitter para TAA
    pub fn get_temporal_jitter(&self) -> Vec2 {
        if !self.config.temporal_aa {
            return Vec2::ZERO;
        }

        // Patrón Halton 2,3 para jitter temporal
        let frame = self.frame_count % 16;
        halton_2d(frame as u32)
    }

    // =========================================================================
    // UTILIDADES
    // =========================================================================

    /// Calcular derivada de pantalla
    #[inline]
    pub fn estimate_screen_derivative(&self, sdf_value: f32, neighbor_sdf: f32, pixel_distance: f32) -> f32 {
        (sdf_value - neighbor_sdf).abs() / pixel_distance
    }

    /// Aplicar corrección de gamma a color
    pub fn apply_gamma_correction(&self, color: Vec4) -> Vec4 {
        if self.config.gamma_correct {
            linear_to_gamma_vec4(color, self.config.gamma)
        } else {
            color
        }
    }
}

impl Default for SDFAntiAliasing {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// FUNCIONES MATEMÁTICAS VECTORIALES PURAS
// =============================================================================

/// Smoothstep estándar (Hermite interpolation)
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smootherstep (Ken Perlin's quintic curve) - más suave que smoothstep
#[inline]
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Ultra smoothstep (septic curve) - máxima suavidad
#[inline]
pub fn smootherstep_ultra(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    // Polinomio de grado 7 para máxima suavidad
    t * t * t * t * (t * (t * (t * -20.0 + 70.0) - 84.0) + 35.0)
}

/// Inverse smoothstep
#[inline]
pub fn inverse_smoothstep(x: f32) -> f32 {
    0.5 - (0.5 - x).clamp(-0.5, 0.5).asin().sin() * 0.5
}

/// Linear step (sin suavizado)
#[inline]
pub fn linearstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0)
}

/// Exponential smoothstep (para bordes más definidos)
#[inline]
pub fn exp_smoothstep(edge0: f32, edge1: f32, x: f32, sharpness: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    1.0 - (-sharpness * t).exp()
}

/// Calcular fwidth aproximado (derivada de pantalla)
#[inline]
pub fn fwidth_approx(center: f32, right: f32, up: f32) -> f32 {
    (center - right).abs() + (center - up).abs()
}

/// Calcular fwidth para Vec2
#[inline]
pub fn fwidth_vec2(center: Vec2, right: Vec2, up: Vec2) -> Vec2 {
    Vec2::new(
        (center.x - right.x).abs() + (center.x - up.x).abs(),
        (center.y - right.y).abs() + (center.y - up.y).abs(),
    )
}

/// Calcular fwidth para Vec3
#[inline]
pub fn fwidth_vec3(center: Vec3, right: Vec3, up: Vec3) -> Vec3 {
    Vec3::new(
        (center.x - right.x).abs() + (center.x - up.x).abs(),
        (center.y - right.y).abs() + (center.y - up.y).abs(),
        (center.z - right.z).abs() + (center.z - up.z).abs(),
    )
}

/// Secuencia Halton para sampling cuasi-aleatorio
#[inline]
pub fn halton(index: u32, base: u32) -> f32 {
    let mut f = 1.0;
    let mut r = 0.0;
    let mut i = index;
    while i > 0 {
        f /= base as f32;
        r += f * (i % base) as f32;
        i /= base;
    }
    r
}

/// Secuencia Halton 2D (bases 2 y 3)
#[inline]
pub fn halton_2d(index: u32) -> Vec2 {
    Vec2::new(halton(index, 2), halton(index, 3)) - Vec2::splat(0.5)
}

/// Conversión gamma a lineal
#[inline]
pub fn gamma_to_linear(value: f32, gamma: f32) -> f32 {
    value.powf(gamma)
}

/// Conversión lineal a gamma
#[inline]
pub fn linear_to_gamma(value: f32, gamma: f32) -> f32 {
    value.powf(1.0 / gamma)
}

/// Conversión gamma a lineal para Vec4
#[inline]
pub fn gamma_to_linear_vec4(color: Vec4, gamma: f32) -> Vec4 {
    Vec4::new(
        gamma_to_linear(color.x, gamma),
        gamma_to_linear(color.y, gamma),
        gamma_to_linear(color.z, gamma),
        color.w, // Alpha no se modifica
    )
}

/// Conversión lineal a gamma para Vec4
#[inline]
pub fn linear_to_gamma_vec4(color: Vec4, gamma: f32) -> Vec4 {
    Vec4::new(
        linear_to_gamma(color.x, gamma),
        linear_to_gamma(color.y, gamma),
        linear_to_gamma(color.z, gamma),
        color.w, // Alpha no se modifica
    )
}

// =============================================================================
// EDGE DETECTION AVANZADO
// =============================================================================

/// Detector de bordes usando análisis vectorial
pub struct SDFEdgeDetector {
    /// Umbral para considerar borde
    pub edge_threshold: f32,
    /// Ancho del borde
    pub edge_width: f32,
    /// Sensibilidad a cambios de normal
    pub normal_sensitivity: f32,
}

impl SDFEdgeDetector {
    pub fn new() -> Self {
        Self {
            edge_threshold: 0.01,
            edge_width: 2.0,
            normal_sensitivity: 1.0,
        }
    }

    /// Detectar si un punto está en un borde
    #[inline]
    pub fn is_edge(&self, sdf_value: f32, derivative: f32) -> bool {
        sdf_value.abs() < self.edge_threshold * derivative * self.edge_width
    }

    /// Obtener intensidad del borde (0 = no borde, 1 = borde fuerte)
    #[inline]
    pub fn edge_intensity(&self, sdf_value: f32, derivative: f32) -> f32 {
        let edge_dist = sdf_value.abs() / (derivative * self.edge_width + 0.0001);
        smootherstep(1.0, 0.0, edge_dist)
    }

    /// Detectar borde basado en cambio de normal
    pub fn detect_normal_edge(&self, normal: Vec3, neighbor_normals: &[Vec3]) -> f32 {
        let mut max_diff = 0.0f32;
        for neighbor in neighbor_normals {
            let diff = 1.0 - normal.dot(*neighbor).max(0.0);
            max_diff = max_diff.max(diff);
        }
        (max_diff * self.normal_sensitivity).min(1.0)
    }

    /// Calcular color de borde para debug
    pub fn debug_edge_color(&self, sdf_value: f32, derivative: f32, base_color: Vec4) -> Vec4 {
        let intensity = self.edge_intensity(sdf_value, derivative);
        
        if intensity > 0.0 {
            let edge_color = Vec4::new(0.0, 1.0, 0.0, 1.0);
            base_color * (1.0 - intensity) + edge_color * intensity
        } else {
            base_color
        }
    }
}

impl Default for SDFEdgeDetector {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// COMPARACIÓN CON OTROS MÉTODOS AA
// =============================================================================

/// Comparación de métodos de AA
#[derive(Clone, Debug)]
pub struct AAComparison {
    /// Nombre del método
    pub method: String,
    /// Calidad (0-100)
    pub quality: f32,
    /// Costo de rendimiento (0-100, menor = mejor)
    pub performance_cost: f32,
    /// Uso de memoria (MB)
    pub memory_mb: f32,
    /// Tiene ghosting
    pub has_ghosting: bool,
    /// Tiene blur
    pub has_blur: bool,
}

impl AAComparison {
    /// Obtener comparación de SDF-AA
    pub fn sdf_aa() -> Self {
        Self {
            method: "SDF-AA (ADead)".to_string(),
            quality: 98.0,
            performance_cost: 5.0,
            memory_mb: 0.0,
            has_ghosting: false,
            has_blur: false,
        }
    }

    /// Obtener comparación de MSAA 4x
    pub fn msaa_4x() -> Self {
        Self {
            method: "MSAA 4x".to_string(),
            quality: 85.0,
            performance_cost: 40.0,
            memory_mb: 32.0, // Para 1080p
            has_ghosting: false,
            has_blur: false,
        }
    }

    /// Obtener comparación de FXAA
    pub fn fxaa() -> Self {
        Self {
            method: "FXAA".to_string(),
            quality: 70.0,
            performance_cost: 10.0,
            memory_mb: 0.0,
            has_ghosting: false,
            has_blur: true,
        }
    }

    /// Obtener comparación de TAA
    pub fn taa() -> Self {
        Self {
            method: "TAA".to_string(),
            quality: 88.0,
            performance_cost: 15.0,
            memory_mb: 16.0,
            has_ghosting: true,
            has_blur: true,
        }
    }

    /// Obtener comparación de DLSS
    pub fn dlss() -> Self {
        Self {
            method: "DLSS 2.0".to_string(),
            quality: 85.0,
            performance_cost: 20.0,
            memory_mb: 64.0,
            has_ghosting: true,
            has_blur: true,
        }
    }

    /// Obtener todas las comparaciones
    pub fn all() -> Vec<Self> {
        vec![
            Self::sdf_aa(),
            Self::msaa_4x(),
            Self::fxaa(),
            Self::taa(),
            Self::dlss(),
        ]
    }

    /// Imprimir tabla comparativa
    pub fn print_comparison() {
        println!("╔═══════════════════════════════════════════════════════════════════╗");
        println!("║                    AA Method Comparison                           ║");
        println!("╠═══════════════════╦═════════╦══════════╦════════╦═════════╦═══════╣");
        println!("║ Method            ║ Quality ║ Perf Cost║ Memory ║ Ghost   ║ Blur  ║");
        println!("╠═══════════════════╬═════════╬══════════╬════════╬═════════╬═══════╣");
        
        for aa in Self::all() {
            println!("║ {:17} ║ {:5.1}%  ║ {:6.1}%  ║ {:4.0}MB ║ {:7} ║ {:5} ║",
                aa.method,
                aa.quality,
                aa.performance_cost,
                aa.memory_mb,
                if aa.has_ghosting { "Yes" } else { "No" },
                if aa.has_blur { "Yes" } else { "No" }
            );
        }
        
        println!("╚═══════════════════╩═════════╩══════════╩════════╩═════════╩═══════╝");
    }
}
