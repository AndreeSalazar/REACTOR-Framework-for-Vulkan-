// =============================================================================
// ADead-SDF: Signed Distance Functions Library
// =============================================================================
// Primitivas matemáticas para ray marching y anti-aliasing
// Basado en las técnicas de Inigo Quilez y ADead-GPU
// =============================================================================

use glam::{Vec2, Vec3, Vec4, Quat};

// =============================================================================
// PRIMITIVAS SDF BÁSICAS
// =============================================================================

/// SDF de una esfera
#[inline]
pub fn sd_sphere(p: Vec3, radius: f32) -> f32 {
    p.length() - radius
}

/// SDF de un cubo (box)
#[inline]
pub fn sd_box(p: Vec3, half_size: Vec3) -> f32 {
    let q = p.abs() - half_size;
    q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0)
}

/// SDF de un cubo redondeado
#[inline]
pub fn sd_round_box(p: Vec3, half_size: Vec3, radius: f32) -> f32 {
    let q = p.abs() - half_size + radius;
    q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0) - radius
}

/// SDF de un cilindro (eje Y)
#[inline]
pub fn sd_cylinder(p: Vec3, height: f32, radius: f32) -> f32 {
    let d = Vec2::new(Vec2::new(p.x, p.z).length(), p.y).abs() - Vec2::new(radius, height);
    d.x.max(d.y).min(0.0) + d.max(Vec2::ZERO).length()
}

/// SDF de un toro (en plano XZ)
#[inline]
pub fn sd_torus(p: Vec3, major_radius: f32, minor_radius: f32) -> f32 {
    let q = Vec2::new(Vec2::new(p.x, p.z).length() - major_radius, p.y);
    q.length() - minor_radius
}

/// SDF de una cápsula (línea con radio)
#[inline]
pub fn sd_capsule(p: Vec3, a: Vec3, b: Vec3, radius: f32) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let t = (ap.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
    let c = a + ab * t;
    (p - c).length() - radius
}

/// SDF de un cono
#[inline]
pub fn sd_cone(p: Vec3, height: f32, radius: f32) -> f32 {
    let q = Vec2::new(Vec2::new(p.x, p.z).length(), p.y);
    let tip = Vec2::new(0.0, height);
    let base = Vec2::new(radius, 0.0);
    
    let e = base - tip;
    let w = q - tip;
    
    let d1 = w - e * (w.dot(e) / e.dot(e)).clamp(0.0, 1.0);
    let d2 = w - Vec2::new(base.x * (w.x / base.x).clamp(0.0, 1.0), base.y);
    
    let s = (-w.y).max(0.0).signum();
    d1.length().min(d2.length()) * s
}

/// SDF de un plano infinito
#[inline]
pub fn sd_plane(p: Vec3, normal: Vec3, offset: f32) -> f32 {
    p.dot(normal.normalize()) + offset
}

/// SDF de un elipsoide
#[inline]
pub fn sd_ellipsoid(p: Vec3, radii: Vec3) -> f32 {
    let k0 = (p / radii).length();
    let k1 = (p / (radii * radii)).length();
    k0 * (k0 - 1.0) / k1
}

/// SDF de una pirámide
pub fn sd_pyramid(p: Vec3, height: f32) -> f32 {
    let m2 = height * height + 0.25;
    let mut p = p;
    p.x = p.x.abs();
    p.z = p.z.abs();
    if p.z > p.x {
        std::mem::swap(&mut p.x, &mut p.z);
    }
    p.x -= 0.5;
    p.z -= 0.5;
    
    let q = Vec3::new(p.z, height * p.y - 0.5 * p.x, height * p.x + 0.5 * p.y);
    let s = (-q.x).max(0.0);
    let t = ((q.y - 0.5 * p.z) / (m2 + 0.25)).clamp(0.0, 1.0);
    
    let a = m2 * (q.x + s) * (q.x + s) + q.y * q.y;
    let b = m2 * (q.x + 0.5 * t) * (q.x + 0.5 * t) + (q.y - m2 * t) * (q.y - m2 * t);
    
    let d2 = if q.y.min(-q.x * m2 - q.y * 0.5) > 0.0 { 0.0 } else { a.min(b) };
    ((d2 + q.z * q.z) / m2).sqrt() * q.z.max(-p.y).signum()
}

// =============================================================================
// OPERACIONES CSG (Constructive Solid Geometry)
// =============================================================================

/// Unión de dos SDFs
#[inline]
pub fn op_union(d1: f32, d2: f32) -> f32 {
    d1.min(d2)
}

/// Sustracción de dos SDFs (d1 - d2)
#[inline]
pub fn op_subtract(d1: f32, d2: f32) -> f32 {
    d1.max(-d2)
}

/// Intersección de dos SDFs
#[inline]
pub fn op_intersect(d1: f32, d2: f32) -> f32 {
    d1.max(d2)
}

/// Unión suave de dos SDFs
#[inline]
pub fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    d2 * (1.0 - h) + d1 * h - k * h * (1.0 - h)
}

/// Sustracción suave de dos SDFs
#[inline]
pub fn op_smooth_subtract(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 - 0.5 * (d2 + d1) / k).clamp(0.0, 1.0);
    d1 * (1.0 - h) + (-d2) * h + k * h * (1.0 - h)
}

/// Intersección suave de dos SDFs
#[inline]
pub fn op_smooth_intersect(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 - 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    d2 * (1.0 - h) + d1 * h + k * h * (1.0 - h)
}

// =============================================================================
// TRANSFORMACIONES
// =============================================================================

/// Trasladar un punto para evaluar SDF
#[inline]
pub fn op_translate(p: Vec3, offset: Vec3) -> Vec3 {
    p - offset
}

/// Rotar un punto para evaluar SDF
#[inline]
pub fn op_rotate(p: Vec3, rotation: Quat) -> Vec3 {
    rotation.inverse() * p
}

/// Escalar un punto para evaluar SDF (requiere dividir resultado por scale)
#[inline]
pub fn op_scale(p: Vec3, scale: f32) -> Vec3 {
    p / scale
}

/// Repetición infinita en un eje
#[inline]
pub fn op_repeat(p: Vec3, period: Vec3) -> Vec3 {
    Vec3::new(
        if period.x > 0.0 { ((p.x + period.x * 0.5) % period.x) - period.x * 0.5 } else { p.x },
        if period.y > 0.0 { ((p.y + period.y * 0.5) % period.y) - period.y * 0.5 } else { p.y },
        if period.z > 0.0 { ((p.z + period.z * 0.5) % period.z) - period.z * 0.5 } else { p.z },
    )
}

/// Repetición limitada
#[inline]
pub fn op_repeat_limited(p: Vec3, period: Vec3, limit: Vec3) -> Vec3 {
    Vec3::new(
        p.x - period.x * (p.x / period.x).round().clamp(-limit.x, limit.x),
        p.y - period.y * (p.y / period.y).round().clamp(-limit.y, limit.y),
        p.z - period.z * (p.z / period.z).round().clamp(-limit.z, limit.z),
    )
}

/// Twist alrededor del eje Y
pub fn op_twist(p: Vec3, twist_amount: f32) -> Vec3 {
    let c = (twist_amount * p.y).cos();
    let s = (twist_amount * p.y).sin();
    Vec3::new(
        c * p.x - s * p.z,
        p.y,
        s * p.x + c * p.z,
    )
}

/// Bend alrededor del eje Y
pub fn op_bend(p: Vec3, bend_amount: f32) -> Vec3 {
    let c = (bend_amount * p.x).cos();
    let s = (bend_amount * p.x).sin();
    Vec3::new(
        c * p.x - s * p.y,
        s * p.x + c * p.y,
        p.z,
    )
}

// =============================================================================
// UTILIDADES
// =============================================================================

/// Calcular normal usando gradiente del SDF
pub fn calc_normal<F>(p: Vec3, sdf: F) -> Vec3 
where F: Fn(Vec3) -> f32 
{
    const EPS: f32 = 0.0001;
    let dx = Vec3::new(EPS, 0.0, 0.0);
    let dy = Vec3::new(0.0, EPS, 0.0);
    let dz = Vec3::new(0.0, 0.0, EPS);
    
    Vec3::new(
        sdf(p + dx) - sdf(p - dx),
        sdf(p + dy) - sdf(p - dy),
        sdf(p + dz) - sdf(p - dz),
    ).normalize()
}

/// Calcular oclusión ambiental usando SDF
pub fn calc_ao<F>(p: Vec3, normal: Vec3, sdf: F, steps: u32) -> f32 
where F: Fn(Vec3) -> f32 
{
    let mut occ = 0.0;
    let mut scale = 1.0;
    
    for i in 0..steps {
        let h = 0.01 + 0.12 * i as f32;
        let d = sdf(p + normal * h);
        occ += (h - d) * scale;
        scale *= 0.95;
    }
    
    (1.0 - 3.0 * occ).clamp(0.0, 1.0)
}

/// Calcular sombra suave usando SDF
pub fn calc_soft_shadow<F>(ro: Vec3, rd: Vec3, sdf: F, min_t: f32, max_t: f32, k: f32) -> f32 
where F: Fn(Vec3) -> f32 
{
    let mut res: f32 = 1.0;
    let mut t = min_t;
    
    while t < max_t {
        let h = sdf(ro + rd * t);
        if h < 0.001 {
            return 0.0;
        }
        res = res.min(k * h / t);
        t += h;
    }
    
    res.clamp(0.0, 1.0)
}

// =============================================================================
// PRIMITIVA COMPUESTA
// =============================================================================

/// Tipo de primitiva SDF
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SDFPrimitiveType {
    Sphere,
    Box,
    RoundBox,
    Cylinder,
    Torus,
    Capsule,
    Cone,
    Plane,
    Ellipsoid,
    Pyramid,
}

/// Primitiva SDF con transformación
#[derive(Clone, Debug)]
pub struct SDFPrimitive {
    pub primitive_type: SDFPrimitiveType,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub params: Vec4, // Parámetros específicos de cada primitiva
    pub color: Vec4,
    pub material_id: u32,
}

impl SDFPrimitive {
    /// Crear esfera
    pub fn sphere(center: Vec3, radius: f32) -> Self {
        Self {
            primitive_type: SDFPrimitiveType::Sphere,
            position: center,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            params: Vec4::new(radius, 0.0, 0.0, 0.0),
            color: Vec4::ONE,
            material_id: 0,
        }
    }

    /// Crear cubo
    pub fn cube(center: Vec3, half_size: Vec3) -> Self {
        Self {
            primitive_type: SDFPrimitiveType::Box,
            position: center,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            params: Vec4::new(half_size.x, half_size.y, half_size.z, 0.0),
            color: Vec4::ONE,
            material_id: 0,
        }
    }

    /// Crear cilindro
    pub fn cylinder(center: Vec3, height: f32, radius: f32) -> Self {
        Self {
            primitive_type: SDFPrimitiveType::Cylinder,
            position: center,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            params: Vec4::new(height, radius, 0.0, 0.0),
            color: Vec4::ONE,
            material_id: 0,
        }
    }

    /// Crear toro
    pub fn torus(center: Vec3, major_radius: f32, minor_radius: f32) -> Self {
        Self {
            primitive_type: SDFPrimitiveType::Torus,
            position: center,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            params: Vec4::new(major_radius, minor_radius, 0.0, 0.0),
            color: Vec4::ONE,
            material_id: 0,
        }
    }

    /// Evaluar SDF en un punto
    pub fn evaluate(&self, world_pos: Vec3) -> f32 {
        // Transformar punto a espacio local
        let local_pos = self.rotation.inverse() * (world_pos - self.position);
        let scaled_pos = local_pos / self.scale;
        
        let d = match self.primitive_type {
            SDFPrimitiveType::Sphere => sd_sphere(scaled_pos, self.params.x),
            SDFPrimitiveType::Box => sd_box(scaled_pos, Vec3::new(self.params.x, self.params.y, self.params.z)),
            SDFPrimitiveType::RoundBox => sd_round_box(scaled_pos, Vec3::new(self.params.x, self.params.y, self.params.z), self.params.w),
            SDFPrimitiveType::Cylinder => sd_cylinder(scaled_pos, self.params.x, self.params.y),
            SDFPrimitiveType::Torus => sd_torus(scaled_pos, self.params.x, self.params.y),
            SDFPrimitiveType::Capsule => sd_capsule(scaled_pos, Vec3::ZERO, Vec3::new(0.0, self.params.x, 0.0), self.params.y),
            SDFPrimitiveType::Cone => sd_cone(scaled_pos, self.params.x, self.params.y),
            SDFPrimitiveType::Plane => sd_plane(scaled_pos, Vec3::Y, self.params.x),
            SDFPrimitiveType::Ellipsoid => sd_ellipsoid(scaled_pos, Vec3::new(self.params.x, self.params.y, self.params.z)),
            SDFPrimitiveType::Pyramid => sd_pyramid(scaled_pos, self.params.x),
        };
        
        // Escalar distancia de vuelta
        d * self.scale.min_element()
    }

    /// Establecer color
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Establecer material
    pub fn with_material(mut self, material_id: u32) -> Self {
        self.material_id = material_id;
        self
    }
}
