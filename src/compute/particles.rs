// =============================================================================
// GPU Particles — Compute shader based particle system
// =============================================================================

use glam::{Vec3, Vec4};

/// GPU Particle data (matches shader layout)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GPUParticle {
    pub position: [f32; 4],
    pub velocity: [f32; 4],
    pub color: [f32; 4],
    pub life_size: [f32; 4], // life, max_life, size, size_end
}

impl GPUParticle {
    pub fn new(position: Vec3, velocity: Vec3, color: Vec4, life: f32, size: f32) -> Self {
        Self {
            position: [position.x, position.y, position.z, 1.0],
            velocity: [velocity.x, velocity.y, velocity.z, 0.0],
            color: color.to_array(),
            life_size: [life, life, size, size * 0.5],
        }
    }

    pub fn is_alive(&self) -> bool {
        self.life_size[0] > 0.0
    }
}

/// GPU Particle emitter configuration
#[derive(Clone, Debug)]
pub struct GPUParticleEmitterConfig {
    pub max_particles: u32,
    pub emit_rate: f32,
    pub lifetime: f32,
    pub lifetime_variance: f32,
    pub initial_speed: f32,
    pub speed_variance: f32,
    pub gravity: Vec3,
    pub drag: f32,
    pub start_size: f32,
    pub end_size: f32,
    pub start_color: Vec4,
    pub end_color: Vec4,
    pub emit_shape: EmitShape,
}

impl Default for GPUParticleEmitterConfig {
    fn default() -> Self {
        Self {
            max_particles: 10000,
            emit_rate: 100.0,
            lifetime: 2.0,
            lifetime_variance: 0.5,
            initial_speed: 5.0,
            speed_variance: 1.0,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            drag: 0.1,
            start_size: 0.1,
            end_size: 0.02,
            start_color: Vec4::new(1.0, 0.8, 0.2, 1.0),
            end_color: Vec4::new(1.0, 0.2, 0.0, 0.0),
            emit_shape: EmitShape::Point,
        }
    }
}

/// Emission shape
#[derive(Clone, Debug)]
pub enum EmitShape {
    Point,
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
    Cone { angle: f32, height: f32 },
    Ring { radius: f32, width: f32 },
}

impl Default for EmitShape {
    fn default() -> Self {
        Self::Point
    }
}

/// GPU Particle system push constants
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ParticlePushConstants {
    pub delta_time: f32,
    pub emit_count: u32,
    pub gravity: [f32; 3],
    pub drag: f32,
    pub time: f32,
    pub _padding: [f32; 3],
}

/// GPU Particle system state
#[derive(Clone, Debug)]
pub struct GPUParticleSystem {
    pub config: GPUParticleEmitterConfig,
    pub position: Vec3,
    pub emit_accumulator: f32,
    pub active_count: u32,
    pub enabled: bool,
}

impl GPUParticleSystem {
    pub fn new(config: GPUParticleEmitterConfig) -> Self {
        Self {
            config,
            position: Vec3::ZERO,
            emit_accumulator: 0.0,
            active_count: 0,
            enabled: true,
        }
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// Calculate how many particles to emit this frame
    pub fn calculate_emit_count(&mut self, delta_time: f32) -> u32 {
        if !self.enabled {
            return 0;
        }

        self.emit_accumulator += self.config.emit_rate * delta_time;
        let emit_count = self.emit_accumulator as u32;
        self.emit_accumulator -= emit_count as f32;

        emit_count.min(self.config.max_particles - self.active_count)
    }

    /// Get push constants for compute shader
    pub fn get_push_constants(&self, delta_time: f32, emit_count: u32, time: f32) -> ParticlePushConstants {
        ParticlePushConstants {
            delta_time,
            emit_count,
            gravity: self.config.gravity.to_array(),
            drag: self.config.drag,
            time,
            _padding: [0.0; 3],
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
}

impl Default for GPUParticleSystem {
    fn default() -> Self {
        Self::new(GPUParticleEmitterConfig::default())
    }
}

// =============================================================================
// Presets
// =============================================================================

impl GPUParticleEmitterConfig {
    pub fn fire() -> Self {
        Self {
            max_particles: 5000,
            emit_rate: 200.0,
            lifetime: 1.5,
            lifetime_variance: 0.3,
            initial_speed: 3.0,
            speed_variance: 1.0,
            gravity: Vec3::new(0.0, 2.0, 0.0), // Fire rises
            drag: 0.5,
            start_size: 0.15,
            end_size: 0.02,
            start_color: Vec4::new(1.0, 0.6, 0.1, 1.0),
            end_color: Vec4::new(1.0, 0.1, 0.0, 0.0),
            emit_shape: EmitShape::Sphere { radius: 0.2 },
        }
    }

    pub fn smoke() -> Self {
        Self {
            max_particles: 3000,
            emit_rate: 50.0,
            lifetime: 4.0,
            lifetime_variance: 1.0,
            initial_speed: 1.0,
            speed_variance: 0.5,
            gravity: Vec3::new(0.0, 0.5, 0.0),
            drag: 0.8,
            start_size: 0.1,
            end_size: 0.5,
            start_color: Vec4::new(0.3, 0.3, 0.3, 0.8),
            end_color: Vec4::new(0.5, 0.5, 0.5, 0.0),
            emit_shape: EmitShape::Sphere { radius: 0.1 },
        }
    }

    pub fn sparks() -> Self {
        Self {
            max_particles: 2000,
            emit_rate: 500.0,
            lifetime: 0.5,
            lifetime_variance: 0.2,
            initial_speed: 10.0,
            speed_variance: 3.0,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            drag: 0.1,
            start_size: 0.02,
            end_size: 0.01,
            start_color: Vec4::new(1.0, 0.9, 0.5, 1.0),
            end_color: Vec4::new(1.0, 0.5, 0.0, 0.0),
            emit_shape: EmitShape::Cone { angle: 0.5, height: 0.1 },
        }
    }

    pub fn snow() -> Self {
        Self {
            max_particles: 10000,
            emit_rate: 100.0,
            lifetime: 10.0,
            lifetime_variance: 2.0,
            initial_speed: 0.5,
            speed_variance: 0.2,
            gravity: Vec3::new(0.0, -1.0, 0.0),
            drag: 0.9,
            start_size: 0.03,
            end_size: 0.03,
            start_color: Vec4::new(1.0, 1.0, 1.0, 0.8),
            end_color: Vec4::new(1.0, 1.0, 1.0, 0.0),
            emit_shape: EmitShape::Box { half_extents: Vec3::new(10.0, 0.1, 10.0) },
        }
    }

    pub fn explosion() -> Self {
        Self {
            max_particles: 1000,
            emit_rate: 10000.0, // Burst
            lifetime: 1.0,
            lifetime_variance: 0.3,
            initial_speed: 15.0,
            speed_variance: 5.0,
            gravity: Vec3::new(0.0, -5.0, 0.0),
            drag: 0.3,
            start_size: 0.2,
            end_size: 0.05,
            start_color: Vec4::new(1.0, 0.8, 0.3, 1.0),
            end_color: Vec4::new(0.5, 0.1, 0.0, 0.0),
            emit_shape: EmitShape::Sphere { radius: 0.5 },
        }
    }
}
