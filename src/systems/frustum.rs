use glam::{Vec3, Mat4};
use crate::systems::physics::{AABB, Sphere};

/// Frustum plane
#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn from_point_normal(point: Vec3, normal: Vec3) -> Self {
        let n = normal.normalize();
        Self {
            normal: n,
            distance: -n.dot(point),
        }
    }

    pub fn normalize(&mut self) {
        let len = self.normal.length();
        if len > 0.0 {
            self.normal /= len;
            self.distance /= len;
        }
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }

    pub fn is_point_in_front(&self, point: Vec3) -> bool {
        self.distance_to_point(point) > 0.0
    }
}

/// View frustum for culling
#[derive(Clone, Debug)]
pub struct Frustum {
    pub planes: [Plane; 6], // Left, Right, Bottom, Top, Near, Far
}

impl Frustum {
    pub fn from_view_projection(vp: Mat4) -> Self {
        let m = vp.to_cols_array_2d();
        
        // Extract planes from view-projection matrix
        let mut planes = [
            // Left
            Plane::new(
                Vec3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]),
                m[3][3] + m[3][0],
            ),
            // Right
            Plane::new(
                Vec3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]),
                m[3][3] - m[3][0],
            ),
            // Bottom
            Plane::new(
                Vec3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]),
                m[3][3] + m[3][1],
            ),
            // Top
            Plane::new(
                Vec3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]),
                m[3][3] - m[3][1],
            ),
            // Near
            Plane::new(
                Vec3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]),
                m[3][3] + m[3][2],
            ),
            // Far
            Plane::new(
                Vec3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]),
                m[3][3] - m[3][2],
            ),
        ];

        // Normalize all planes
        for plane in &mut planes {
            plane.normalize();
        }

        Self { planes }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(point) < 0.0 {
                return false;
            }
        }
        true
    }

    pub fn intersects_sphere(&self, sphere: &Sphere) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(sphere.center) < -sphere.radius {
                return false;
            }
        }
        true
    }

    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        for plane in &self.planes {
            // Get the positive vertex (furthest along plane normal)
            let p = Vec3::new(
                if plane.normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
                if plane.normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
                if plane.normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
            );

            if plane.distance_to_point(p) < 0.0 {
                return false;
            }
        }
        true
    }

    /// More precise AABB test that returns intersection state
    pub fn test_aabb(&self, aabb: &AABB) -> FrustumTestResult {
        let mut result = FrustumTestResult::Inside;

        for plane in &self.planes {
            let p = Vec3::new(
                if plane.normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
                if plane.normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
                if plane.normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
            );

            let n = Vec3::new(
                if plane.normal.x >= 0.0 { aabb.min.x } else { aabb.max.x },
                if plane.normal.y >= 0.0 { aabb.min.y } else { aabb.max.y },
                if plane.normal.z >= 0.0 { aabb.min.z } else { aabb.max.z },
            );

            if plane.distance_to_point(p) < 0.0 {
                return FrustumTestResult::Outside;
            }

            if plane.distance_to_point(n) < 0.0 {
                result = FrustumTestResult::Intersecting;
            }
        }

        result
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrustumTestResult {
    Outside,
    Intersecting,
    Inside,
}

/// Culling system that tracks visible objects
pub struct CullingSystem {
    frustum: Frustum,
    visible_count: usize,
    total_count: usize,
}

impl CullingSystem {
    pub fn new() -> Self {
        Self {
            frustum: Frustum::from_view_projection(Mat4::IDENTITY),
            visible_count: 0,
            total_count: 0,
        }
    }

    pub fn update_frustum(&mut self, view_projection: Mat4) {
        self.frustum = Frustum::from_view_projection(view_projection);
        self.visible_count = 0;
        self.total_count = 0;
    }

    pub fn is_visible_aabb(&mut self, aabb: &AABB) -> bool {
        self.total_count += 1;
        let visible = self.frustum.intersects_aabb(aabb);
        if visible {
            self.visible_count += 1;
        }
        visible
    }

    pub fn is_visible_sphere(&mut self, sphere: &Sphere) -> bool {
        self.total_count += 1;
        let visible = self.frustum.intersects_sphere(sphere);
        if visible {
            self.visible_count += 1;
        }
        visible
    }

    pub fn visible_count(&self) -> usize {
        self.visible_count
    }

    pub fn total_count(&self) -> usize {
        self.total_count
    }

    pub fn culled_count(&self) -> usize {
        self.total_count - self.visible_count
    }

    pub fn cull_percentage(&self) -> f32 {
        if self.total_count == 0 {
            0.0
        } else {
            (self.culled_count() as f32 / self.total_count as f32) * 100.0
        }
    }
}

impl Default for CullingSystem {
    fn default() -> Self {
        Self::new()
    }
}
