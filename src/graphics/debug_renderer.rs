use glam::{Vec3, Vec4, Mat4};

/// Simple AABB for debug rendering
#[derive(Clone, Copy, Debug)]
pub struct DebugAABB {
    pub min: Vec3,
    pub max: Vec3,
}

/// Simple Sphere for debug rendering
#[derive(Clone, Copy, Debug)]
pub struct DebugSphere {
    pub center: Vec3,
    pub radius: f32,
}

/// Simple Ray for debug rendering
#[derive(Clone, Copy, Debug)]
pub struct DebugRay {
    pub origin: Vec3,
    pub direction: Vec3,
}

/// Debug line for rendering
#[derive(Clone, Copy, Debug)]
pub struct DebugLine {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec4,
}

impl DebugLine {
    pub fn new(start: Vec3, end: Vec3, color: Vec4) -> Self {
        Self { start, end, color }
    }

    pub fn white(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::ONE)
    }

    pub fn red(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::new(1.0, 0.0, 0.0, 1.0))
    }

    pub fn green(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::new(0.0, 1.0, 0.0, 1.0))
    }

    pub fn blue(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::new(0.0, 0.0, 1.0, 1.0))
    }

    pub fn yellow(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::new(1.0, 1.0, 0.0, 1.0))
    }
}

/// Debug renderer for visualizing shapes, rays, etc.
pub struct DebugRenderer {
    lines: Vec<DebugLine>,
    persistent_lines: Vec<DebugLine>,
    enabled: bool,
    max_lines: usize,
}

impl DebugRenderer {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            persistent_lines: Vec::new(),
            enabled: true,
            max_lines: 10000,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn clear_persistent(&mut self) {
        self.persistent_lines.clear();
    }

    pub fn line(&mut self, start: Vec3, end: Vec3, color: Vec4) {
        if self.enabled && self.lines.len() < self.max_lines {
            self.lines.push(DebugLine::new(start, end, color));
        }
    }

    pub fn line_persistent(&mut self, start: Vec3, end: Vec3, color: Vec4) {
        if self.enabled && self.persistent_lines.len() < self.max_lines {
            self.persistent_lines.push(DebugLine::new(start, end, color));
        }
    }

    pub fn ray(&mut self, ray: &DebugRay, length: f32, color: Vec4) {
        self.line(ray.origin, ray.origin + ray.direction * length, color);
    }

    pub fn aabb(&mut self, aabb: &DebugAABB, color: Vec4) {
        let min = aabb.min;
        let max = aabb.max;

        // Bottom face
        self.line(Vec3::new(min.x, min.y, min.z), Vec3::new(max.x, min.y, min.z), color);
        self.line(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, min.y, max.z), color);
        self.line(Vec3::new(max.x, min.y, max.z), Vec3::new(min.x, min.y, max.z), color);
        self.line(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, min.y, min.z), color);

        // Top face
        self.line(Vec3::new(min.x, max.y, min.z), Vec3::new(max.x, max.y, min.z), color);
        self.line(Vec3::new(max.x, max.y, min.z), Vec3::new(max.x, max.y, max.z), color);
        self.line(Vec3::new(max.x, max.y, max.z), Vec3::new(min.x, max.y, max.z), color);
        self.line(Vec3::new(min.x, max.y, max.z), Vec3::new(min.x, max.y, min.z), color);

        // Vertical edges
        self.line(Vec3::new(min.x, min.y, min.z), Vec3::new(min.x, max.y, min.z), color);
        self.line(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, max.y, min.z), color);
        self.line(Vec3::new(max.x, min.y, max.z), Vec3::new(max.x, max.y, max.z), color);
        self.line(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, max.y, max.z), color);
    }

    pub fn sphere(&mut self, sphere: &DebugSphere, color: Vec4, segments: u32) {
        let segments = segments.max(8);
        let step = std::f32::consts::TAU / segments as f32;

        // XY circle
        for i in 0..segments {
            let a1 = i as f32 * step;
            let a2 = (i + 1) as f32 * step;
            let p1 = sphere.center + Vec3::new(a1.cos(), a1.sin(), 0.0) * sphere.radius;
            let p2 = sphere.center + Vec3::new(a2.cos(), a2.sin(), 0.0) * sphere.radius;
            self.line(p1, p2, color);
        }

        // XZ circle
        for i in 0..segments {
            let a1 = i as f32 * step;
            let a2 = (i + 1) as f32 * step;
            let p1 = sphere.center + Vec3::new(a1.cos(), 0.0, a1.sin()) * sphere.radius;
            let p2 = sphere.center + Vec3::new(a2.cos(), 0.0, a2.sin()) * sphere.radius;
            self.line(p1, p2, color);
        }

        // YZ circle
        for i in 0..segments {
            let a1 = i as f32 * step;
            let a2 = (i + 1) as f32 * step;
            let p1 = sphere.center + Vec3::new(0.0, a1.cos(), a1.sin()) * sphere.radius;
            let p2 = sphere.center + Vec3::new(0.0, a2.cos(), a2.sin()) * sphere.radius;
            self.line(p1, p2, color);
        }
    }

    pub fn axes(&mut self, origin: Vec3, size: f32) {
        self.line(origin, origin + Vec3::X * size, Vec4::new(1.0, 0.0, 0.0, 1.0));
        self.line(origin, origin + Vec3::Y * size, Vec4::new(0.0, 1.0, 0.0, 1.0));
        self.line(origin, origin + Vec3::Z * size, Vec4::new(0.0, 0.0, 1.0, 1.0));
    }

    pub fn transform(&mut self, transform: &crate::systems::transform::Transform, size: f32) {
        let origin = transform.position;
        self.line(origin, origin + transform.right() * size, Vec4::new(1.0, 0.0, 0.0, 1.0));
        self.line(origin, origin + transform.up() * size, Vec4::new(0.0, 1.0, 0.0, 1.0));
        self.line(origin, origin + transform.forward() * size, Vec4::new(0.0, 0.0, 1.0, 1.0));
    }

    pub fn grid(&mut self, center: Vec3, size: f32, divisions: u32, color: Vec4) {
        let half = size / 2.0;
        let step = size / divisions as f32;

        for i in 0..=divisions {
            let offset = -half + i as f32 * step;
            
            // X lines
            self.line(
                center + Vec3::new(-half, 0.0, offset),
                center + Vec3::new(half, 0.0, offset),
                color,
            );
            
            // Z lines
            self.line(
                center + Vec3::new(offset, 0.0, -half),
                center + Vec3::new(offset, 0.0, half),
                color,
            );
        }
    }

    pub fn frustum(&mut self, inv_view_proj: Mat4, color: Vec4) {
        // NDC corners
        let ndc_corners = [
            Vec3::new(-1.0, -1.0, 0.0), // near bottom left
            Vec3::new( 1.0, -1.0, 0.0), // near bottom right
            Vec3::new( 1.0,  1.0, 0.0), // near top right
            Vec3::new(-1.0,  1.0, 0.0), // near top left
            Vec3::new(-1.0, -1.0, 1.0), // far bottom left
            Vec3::new( 1.0, -1.0, 1.0), // far bottom right
            Vec3::new( 1.0,  1.0, 1.0), // far top right
            Vec3::new(-1.0,  1.0, 1.0), // far top left
        ];

        let world_corners: Vec<Vec3> = ndc_corners
            .iter()
            .map(|&ndc| inv_view_proj.project_point3(ndc))
            .collect();

        // Near plane
        self.line(world_corners[0], world_corners[1], color);
        self.line(world_corners[1], world_corners[2], color);
        self.line(world_corners[2], world_corners[3], color);
        self.line(world_corners[3], world_corners[0], color);

        // Far plane
        self.line(world_corners[4], world_corners[5], color);
        self.line(world_corners[5], world_corners[6], color);
        self.line(world_corners[6], world_corners[7], color);
        self.line(world_corners[7], world_corners[4], color);

        // Connecting edges
        self.line(world_corners[0], world_corners[4], color);
        self.line(world_corners[1], world_corners[5], color);
        self.line(world_corners[2], world_corners[6], color);
        self.line(world_corners[3], world_corners[7], color);
    }

    pub fn get_lines(&self) -> impl Iterator<Item = &DebugLine> {
        self.lines.iter().chain(self.persistent_lines.iter())
    }

    pub fn line_count(&self) -> usize {
        self.lines.len() + self.persistent_lines.len()
    }
}

impl Default for DebugRenderer {
    fn default() -> Self {
        Self::new()
    }
}
