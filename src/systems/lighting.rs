use glam::Vec3;
use crate::graphics::uniform_buffer::{LightData, LightUniformData, MAX_LIGHTS};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

#[derive(Clone, Debug)]
pub struct Light {
    pub light_type: LightType,
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32,
    pub spot_angle: f32,
    pub cast_shadows: bool,
    pub enabled: bool,
}

impl Light {
    pub fn directional(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional,
            position: Vec3::ZERO,
            direction: direction.normalize(),
            color,
            intensity,
            range: f32::MAX,
            spot_angle: 0.0,
            cast_shadows: true,
            enabled: true,
        }
    }

    pub fn point(position: Vec3, color: Vec3, intensity: f32, range: f32) -> Self {
        Self {
            light_type: LightType::Point,
            position,
            direction: Vec3::NEG_Y,
            color,
            intensity,
            range,
            spot_angle: 0.0,
            cast_shadows: false,
            enabled: true,
        }
    }

    pub fn spot(position: Vec3, direction: Vec3, color: Vec3, intensity: f32, range: f32, angle_degrees: f32) -> Self {
        Self {
            light_type: LightType::Spot,
            position,
            direction: direction.normalize(),
            color,
            intensity,
            range,
            spot_angle: angle_degrees.to_radians(),
            cast_shadows: true,
            enabled: true,
        }
    }

    pub fn sun() -> Self {
        Self::directional(
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Vec3::new(1.0, 0.98, 0.9),
            1.0,
        )
    }

    pub fn to_gpu_data(&self) -> LightData {
        LightData {
            position: [self.position.x, self.position.y, self.position.z, self.range],
            direction: [self.direction.x, self.direction.y, self.direction.z, self.spot_angle],
            color: [self.color.x * self.intensity, self.color.y * self.intensity, self.color.z * self.intensity, self.intensity],
            light_type: match self.light_type {
                LightType::Directional => 0,
                LightType::Point => 1,
                LightType::Spot => 2,
            },
            cast_shadows: if self.cast_shadows { 1 } else { 0 },
            _padding: [0, 0],
        }
    }
}

pub struct LightingSystem {
    pub lights: Vec<Light>,
    pub ambient_color: Vec3,
    pub ambient_intensity: f32,
}

impl LightingSystem {
    pub fn new() -> Self {
        Self {
            lights: Vec::new(),
            ambient_color: Vec3::splat(0.1),
            ambient_intensity: 1.0,
        }
    }

    pub fn with_sun() -> Self {
        let mut system = Self::new();
        system.add_light(Light::sun());
        system
    }

    pub fn add_light(&mut self, light: Light) -> usize {
        let index = self.lights.len();
        self.lights.push(light);
        index
    }

    pub fn remove_light(&mut self, index: usize) {
        if index < self.lights.len() {
            self.lights.remove(index);
        }
    }

    pub fn get_light(&self, index: usize) -> Option<&Light> {
        self.lights.get(index)
    }

    pub fn get_light_mut(&mut self, index: usize) -> Option<&mut Light> {
        self.lights.get_mut(index)
    }

    pub fn set_ambient(&mut self, color: Vec3, intensity: f32) {
        self.ambient_color = color;
        self.ambient_intensity = intensity;
    }

    pub fn to_gpu_data(&self) -> LightUniformData {
        let mut data = LightUniformData::default();
        
        data.ambient_color = [
            self.ambient_color.x * self.ambient_intensity,
            self.ambient_color.y * self.ambient_intensity,
            self.ambient_color.z * self.ambient_intensity,
            1.0,
        ];

        let enabled_lights: Vec<_> = self.lights.iter().filter(|l| l.enabled).collect();
        data.light_count = enabled_lights.len().min(MAX_LIGHTS) as u32;

        for (i, light) in enabled_lights.iter().take(MAX_LIGHTS).enumerate() {
            data.lights[i] = light.to_gpu_data();
        }

        data
    }

    pub fn light_count(&self) -> usize {
        self.lights.iter().filter(|l| l.enabled).count()
    }
}

impl Default for LightingSystem {
    fn default() -> Self {
        Self::new()
    }
}
