use glam::{Vec3, Quat};
use crate::systems::transform::Transform;

/// Basic physics body component
#[derive(Clone, Debug)]
pub struct RigidBody {
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub drag: f32,
    pub angular_drag: f32,
    pub gravity_scale: f32,
    pub is_kinematic: bool,
    pub freeze_rotation: bool,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            drag: 0.0,
            angular_drag: 0.05,
            gravity_scale: 1.0,
            is_kinematic: false,
            freeze_rotation: false,
        }
    }
}

impl RigidBody {
    pub fn kinematic() -> Self {
        Self {
            is_kinematic: true,
            ..Default::default()
        }
    }

    pub fn add_force(&mut self, force: Vec3) {
        if !self.is_kinematic && self.mass > 0.0 {
            self.velocity += force / self.mass;
        }
    }

    pub fn add_impulse(&mut self, impulse: Vec3) {
        if !self.is_kinematic {
            self.velocity += impulse;
        }
    }

    pub fn add_torque(&mut self, torque: Vec3) {
        if !self.is_kinematic && !self.freeze_rotation {
            self.angular_velocity += torque;
        }
    }
}

/// Axis-Aligned Bounding Box
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn extents(&self) -> Vec3 {
        self.size() * 0.5
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub fn expand(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn transformed(&self, transform: &Transform) -> AABB {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let mat = transform.matrix();
        let mut result = AABB::new(Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        
        for corner in corners {
            let transformed = mat.transform_point3(corner);
            result.expand(transformed);
        }

        result
    }
}

/// Sphere collider
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }

    pub fn intersects_sphere(&self, other: &Sphere) -> bool {
        let dist_sq = (other.center - self.center).length_squared();
        let radius_sum = self.radius + other.radius;
        dist_sq <= radius_sum * radius_sum
    }

    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        let closest = Vec3::new(
            self.center.x.clamp(aabb.min.x, aabb.max.x),
            self.center.y.clamp(aabb.min.y, aabb.max.y),
            self.center.z.clamp(aabb.min.z, aabb.max.z),
        );
        (closest - self.center).length_squared() <= self.radius * self.radius
    }
}

/// Ray for raycasting
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn from_screen(
        screen_x: f32,
        screen_y: f32,
        screen_width: f32,
        screen_height: f32,
        inv_view_proj: glam::Mat4,
    ) -> Self {
        let ndc_x = (2.0 * screen_x / screen_width) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_y / screen_height); // Flip Y for Vulkan

        let near = inv_view_proj.project_point3(Vec3::new(ndc_x, ndc_y, 0.0));
        let far = inv_view_proj.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));

        Self::new(near, far - near)
    }

    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    pub fn intersects_aabb(&self, aabb: &AABB) -> Option<f32> {
        let inv_dir = Vec3::new(1.0 / self.direction.x, 1.0 / self.direction.y, 1.0 / self.direction.z);
        
        let t1 = (aabb.min.x - self.origin.x) * inv_dir.x;
        let t2 = (aabb.max.x - self.origin.x) * inv_dir.x;
        let t3 = (aabb.min.y - self.origin.y) * inv_dir.y;
        let t4 = (aabb.max.y - self.origin.y) * inv_dir.y;
        let t5 = (aabb.min.z - self.origin.z) * inv_dir.z;
        let t6 = (aabb.max.z - self.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some(if tmin < 0.0 { tmax } else { tmin })
        }
    }

    pub fn intersects_sphere(&self, sphere: &Sphere) -> Option<f32> {
        let oc = self.origin - sphere.center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - sphere.radius * sphere.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.0 { Some(t) } else { None }
        }
    }

    pub fn intersects_plane(&self, plane_normal: Vec3, plane_d: f32) -> Option<f32> {
        let denom = plane_normal.dot(self.direction);
        if denom.abs() > 1e-6 {
            let t = -(plane_normal.dot(self.origin) + plane_d) / denom;
            if t >= 0.0 { Some(t) } else { None }
        } else {
            None
        }
    }
}

/// Simple physics world
pub struct PhysicsWorld {
    pub gravity: Vec3,
    pub fixed_timestep: f32,
    accumulator: f32,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            fixed_timestep: 1.0 / 60.0,
            accumulator: 0.0,
        }
    }

    pub fn step(&mut self, delta_time: f32) -> u32 {
        self.accumulator += delta_time;
        let mut steps = 0;

        while self.accumulator >= self.fixed_timestep {
            self.accumulator -= self.fixed_timestep;
            steps += 1;
        }

        steps
    }

    pub fn integrate(&self, transform: &mut Transform, body: &mut RigidBody) {
        if body.is_kinematic {
            return;
        }

        let dt = self.fixed_timestep;

        // Apply gravity
        body.velocity += self.gravity * body.gravity_scale * dt;

        // Apply drag
        body.velocity *= 1.0 - body.drag * dt;
        body.angular_velocity *= 1.0 - body.angular_drag * dt;

        // Integrate position
        transform.position += body.velocity * dt;

        // Integrate rotation
        if !body.freeze_rotation && body.angular_velocity.length_squared() > 1e-6 {
            let angle = body.angular_velocity.length() * dt;
            let axis = body.angular_velocity.normalize();
            transform.rotation = Quat::from_axis_angle(axis, angle) * transform.rotation;
        }
    }

    pub fn interpolation_alpha(&self) -> f32 {
        self.accumulator / self.fixed_timestep
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}
