use glam::{Mat4, Vec3, Quat};

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            rotation: Quat::IDENTITY,
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 16.0 / 9.0,
        }
    }

    pub fn perspective(fov_degrees: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            fov: fov_degrees.to_radians(),
            near,
            far,
            aspect_ratio,
        }
    }

    pub fn look_at(mut self, eye: Vec3, target: Vec3, _up: Vec3) -> Self {
        self.position = eye;
        let forward = (target - eye).normalize();
        self.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
        self
    }

    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn view_matrix(&self) -> Mat4 {
        let target = self.position + self.forward();
        Mat4::look_at_rh(self.position, target, Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        let mut proj = Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far);
        proj.y_axis.y *= -1.0; // Vulkan Y-flip
        proj
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    // FPS-style camera controls
    pub fn rotate_yaw(&mut self, angle: f32) {
        self.rotation = Quat::from_rotation_y(angle) * self.rotation;
    }

    pub fn rotate_pitch(&mut self, angle: f32) {
        self.rotation = self.rotation * Quat::from_rotation_x(angle);
    }

    /// Set camera rotation from yaw and pitch angles (radians)
    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.rotation = Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, 0.0);
    }

    /// Get current yaw angle in radians
    pub fn yaw(&self) -> f32 {
        let (yaw, _, _) = self.rotation.to_euler(glam::EulerRot::YXZ);
        yaw
    }

    /// Get current pitch angle in radians
    pub fn pitch(&self) -> f32 {
        let (_, pitch, _) = self.rotation.to_euler(glam::EulerRot::YXZ);
        pitch
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.position += self.forward() * distance;
    }

    pub fn move_right(&mut self, distance: f32) {
        self.position += self.right() * distance;
    }

    pub fn move_up(&mut self, distance: f32) {
        self.position += Vec3::Y * distance;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

// Orthographic camera for 2D
pub struct Camera2D {
    pub position: glam::Vec2,
    pub zoom: f32,
    pub rotation: f32,
}

impl Camera2D {
    pub fn new() -> Self {
        Self {
            position: glam::Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
        }
    }

    pub fn view_matrix(&self, width: f32, height: f32) -> Mat4 {
        let half_w = width / 2.0 / self.zoom;
        let half_h = height / 2.0 / self.zoom;

        let proj = Mat4::orthographic_rh(-half_w, half_w, half_h, -half_h, -1.0, 1.0);
        let view = Mat4::from_rotation_z(self.rotation)
            * Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0));

        proj * view
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new()
    }
}
