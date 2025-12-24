// =============================================================================
// ADead-RT: Ray Marching Engine
// =============================================================================
// Ray Tracing sin RT Cores - Funciona en CUALQUIER GPU
// Basado en Sphere Tracing / Ray Marching con SDF
// =============================================================================

use glam::{Vec2, Vec3, Vec4, Mat4};
use crate::adead::sdf::SDFPrimitive;

/// Configuración del ray marcher
#[derive(Clone, Debug)]
pub struct RayMarchConfig {
    /// Número máximo de pasos
    pub max_steps: u32,
    /// Distancia máxima de ray
    pub max_distance: f32,
    /// Umbral de superficie (epsilon)
    pub surface_threshold: f32,
    /// Habilitar sombras suaves
    pub soft_shadows: bool,
    /// Dureza de sombras (k parameter)
    pub shadow_hardness: f32,
    /// Habilitar oclusión ambiental
    pub ambient_occlusion: bool,
    /// Pasos de AO
    pub ao_steps: u32,
    /// Habilitar reflexiones
    pub reflections: bool,
    /// Número de bounces de reflexión
    pub reflection_bounces: u32,
    /// Habilitar refracción
    pub refraction: bool,
    /// Índice de refracción
    pub refraction_index: f32,
}

impl Default for RayMarchConfig {
    fn default() -> Self {
        Self {
            max_steps: 128,
            max_distance: 100.0,
            surface_threshold: 0.001,
            soft_shadows: true,
            shadow_hardness: 16.0,
            ambient_occlusion: true,
            ao_steps: 5,
            reflections: false,
            reflection_bounces: 2,
            refraction: false,
            refraction_index: 1.5,
        }
    }
}

/// Resultado de un ray march
#[derive(Clone, Debug)]
pub struct RayMarchHit {
    /// Si hubo hit
    pub hit: bool,
    /// Posición del hit
    pub position: Vec3,
    /// Normal en el punto de hit
    pub normal: Vec3,
    /// Distancia recorrida
    pub distance: f32,
    /// Número de pasos usados
    pub steps: u32,
    /// Índice de la primitiva golpeada
    pub primitive_index: Option<usize>,
    /// Color/material del hit
    pub color: Vec4,
}

impl Default for RayMarchHit {
    fn default() -> Self {
        Self {
            hit: false,
            position: Vec3::ZERO,
            normal: Vec3::Y,
            distance: 0.0,
            steps: 0,
            primitive_index: None,
            color: Vec4::ZERO,
        }
    }
}

/// Escena SDF para ray marching
#[derive(Clone, Debug, Default)]
pub struct SDFScene {
    /// Primitivas en la escena
    pub primitives: Vec<SDFPrimitive>,
    /// Color de fondo/cielo
    pub background_color: Vec4,
    /// Dirección de luz principal
    pub light_direction: Vec3,
    /// Color de luz principal
    pub light_color: Vec3,
    /// Intensidad de luz ambiental
    pub ambient_intensity: f32,
}

impl SDFScene {
    /// Crear escena vacía
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
            background_color: Vec4::new(0.5, 0.7, 1.0, 1.0),
            light_direction: Vec3::new(-0.5, -1.0, -0.3).normalize(),
            light_color: Vec3::ONE,
            ambient_intensity: 0.1,
        }
    }

    /// Agregar primitiva
    pub fn add(&mut self, primitive: SDFPrimitive) -> usize {
        let idx = self.primitives.len();
        self.primitives.push(primitive);
        idx
    }

    /// Evaluar SDF de toda la escena
    pub fn evaluate(&self, p: Vec3) -> (f32, Option<usize>) {
        let mut min_dist = f32::MAX;
        let mut closest_idx = None;

        for (i, prim) in self.primitives.iter().enumerate() {
            let d = prim.evaluate(p);
            if d < min_dist {
                min_dist = d;
                closest_idx = Some(i);
            }
        }

        (min_dist, closest_idx)
    }

    /// Obtener color de una primitiva
    pub fn get_color(&self, idx: usize) -> Vec4 {
        self.primitives.get(idx).map(|p| p.color).unwrap_or(Vec4::ONE)
    }
}

/// Motor de Ray Marching
pub struct RayMarcher {
    /// Configuración
    pub config: RayMarchConfig,
}

impl RayMarcher {
    /// Crear nuevo ray marcher
    pub fn new() -> Self {
        Self {
            config: RayMarchConfig::default(),
        }
    }

    /// Crear con configuración personalizada
    pub fn with_config(config: RayMarchConfig) -> Self {
        Self { config }
    }

    /// Ejecutar ray march
    pub fn march(&self, scene: &SDFScene, ray_origin: Vec3, ray_direction: Vec3) -> RayMarchHit {
        let mut t = 0.0;
        let mut hit = RayMarchHit::default();

        for step in 0..self.config.max_steps {
            let p = ray_origin + ray_direction * t;
            let (d, idx) = scene.evaluate(p);

            if d < self.config.surface_threshold {
                hit.hit = true;
                hit.position = p;
                hit.distance = t;
                hit.steps = step;
                hit.primitive_index = idx;
                
                // Calcular normal
                hit.normal = self.calc_scene_normal(scene, p);
                
                // Obtener color
                if let Some(i) = idx {
                    hit.color = scene.get_color(i);
                }
                
                return hit;
            }

            t += d;

            if t > self.config.max_distance {
                break;
            }
        }

        hit.steps = self.config.max_steps;
        hit.distance = t;
        hit
    }

    /// Calcular normal de la escena
    fn calc_scene_normal(&self, scene: &SDFScene, p: Vec3) -> Vec3 {
        const EPS: f32 = 0.0001;
        let dx = Vec3::new(EPS, 0.0, 0.0);
        let dy = Vec3::new(0.0, EPS, 0.0);
        let dz = Vec3::new(0.0, 0.0, EPS);

        Vec3::new(
            scene.evaluate(p + dx).0 - scene.evaluate(p - dx).0,
            scene.evaluate(p + dy).0 - scene.evaluate(p - dy).0,
            scene.evaluate(p + dz).0 - scene.evaluate(p - dz).0,
        ).normalize()
    }

    /// Calcular iluminación completa
    pub fn shade(&self, scene: &SDFScene, hit: &RayMarchHit) -> Vec4 {
        if !hit.hit {
            return scene.background_color;
        }

        let base_color = hit.color;
        let normal = hit.normal;
        let position = hit.position;

        // Luz difusa
        let n_dot_l = normal.dot(-scene.light_direction).max(0.0);
        let diffuse = scene.light_color * n_dot_l;

        // Sombras suaves
        let shadow = if self.config.soft_shadows {
            self.calc_shadow(scene, position, -scene.light_direction)
        } else {
            1.0
        };

        // Oclusión ambiental
        let ao = if self.config.ambient_occlusion {
            self.calc_ao(scene, position, normal)
        } else {
            1.0
        };

        // Combinar iluminación
        let ambient = scene.ambient_intensity * ao;
        let lighting = ambient + diffuse * shadow;

        Vec4::new(
            base_color.x * lighting.x,
            base_color.y * lighting.y,
            base_color.z * lighting.z,
            base_color.w,
        )
    }

    /// Calcular sombra suave
    fn calc_shadow(&self, scene: &SDFScene, origin: Vec3, light_dir: Vec3) -> f32 {
        let mut res: f32 = 1.0;
        let mut t: f32 = 0.01;
        let max_t = 50.0;

        while t < max_t {
            let p = origin + light_dir * t;
            let (h, _) = scene.evaluate(p);

            if h < 0.001 {
                return 0.0;
            }

            res = res.min(self.config.shadow_hardness * h / t);
            t += h;
        }

        res.clamp(0.0, 1.0)
    }

    /// Calcular oclusión ambiental
    fn calc_ao(&self, scene: &SDFScene, origin: Vec3, normal: Vec3) -> f32 {
        let mut occ = 0.0;
        let mut scale = 1.0;

        for i in 0..self.config.ao_steps {
            let h = 0.01 + 0.12 * i as f32;
            let p = origin + normal * h;
            let (d, _) = scene.evaluate(p);
            occ += (h - d) * scale;
            scale *= 0.95;
        }

        (1.0 - 3.0 * occ).clamp(0.0, 1.0)
    }

    /// Renderizar un píxel completo
    pub fn render_pixel(
        &self,
        scene: &SDFScene,
        ray_origin: Vec3,
        ray_direction: Vec3,
    ) -> Vec4 {
        let hit = self.march(scene, ray_origin, ray_direction);
        self.shade(scene, &hit)
    }

    /// Generar ray desde coordenadas de pantalla
    pub fn screen_to_ray(
        &self,
        screen_pos: Vec2,
        screen_size: Vec2,
        inv_view_proj: Mat4,
    ) -> (Vec3, Vec3) {
        // Normalizar a NDC (-1 a 1)
        let ndc = Vec2::new(
            (screen_pos.x / screen_size.x) * 2.0 - 1.0,
            1.0 - (screen_pos.y / screen_size.y) * 2.0, // Y invertido
        );

        // Puntos en near y far plane
        let near_point = inv_view_proj.project_point3(Vec3::new(ndc.x, ndc.y, 0.0));
        let far_point = inv_view_proj.project_point3(Vec3::new(ndc.x, ndc.y, 1.0));

        let origin = near_point;
        let direction = (far_point - near_point).normalize();

        (origin, direction)
    }

    /// Preset: Calidad máxima
    pub fn preset_quality() -> RayMarchConfig {
        RayMarchConfig {
            max_steps: 256,
            max_distance: 200.0,
            surface_threshold: 0.0001,
            soft_shadows: true,
            shadow_hardness: 32.0,
            ambient_occlusion: true,
            ao_steps: 8,
            reflections: true,
            reflection_bounces: 3,
            ..Default::default()
        }
    }

    /// Preset: Rendimiento
    pub fn preset_performance() -> RayMarchConfig {
        RayMarchConfig {
            max_steps: 64,
            max_distance: 50.0,
            surface_threshold: 0.01,
            soft_shadows: false,
            ambient_occlusion: false,
            reflections: false,
            ..Default::default()
        }
    }

    /// Preset: Balanceado
    pub fn preset_balanced() -> RayMarchConfig {
        RayMarchConfig::default()
    }
}

impl Default for RayMarcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de ray marching
#[derive(Clone, Debug, Default)]
pub struct RayMarchStats {
    /// Total de rays lanzados
    pub total_rays: u64,
    /// Rays que hicieron hit
    pub hits: u64,
    /// Promedio de pasos por ray
    pub avg_steps: f32,
    /// Máximo de pasos usado
    pub max_steps_used: u32,
    /// Tiempo total en ms
    pub total_time_ms: f32,
}

impl RayMarchStats {
    /// Actualizar estadísticas con un nuevo hit
    pub fn record_hit(&mut self, hit: &RayMarchHit) {
        self.total_rays += 1;
        if hit.hit {
            self.hits += 1;
        }
        self.avg_steps = (self.avg_steps * (self.total_rays - 1) as f32 + hit.steps as f32) 
            / self.total_rays as f32;
        self.max_steps_used = self.max_steps_used.max(hit.steps);
    }

    /// Obtener tasa de hit
    pub fn hit_rate(&self) -> f32 {
        if self.total_rays == 0 {
            0.0
        } else {
            self.hits as f32 / self.total_rays as f32
        }
    }

    /// Reset estadísticas
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
