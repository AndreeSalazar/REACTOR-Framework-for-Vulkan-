use glam::{Vec3, Vec4};

/// Single particle
#[derive(Clone, Debug)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub color: Vec4,
    pub size: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub alive: bool,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            color: Vec4::ONE,
            size: 1.0,
            rotation: 0.0,
            angular_velocity: 0.0,
            lifetime: 0.0,
            max_lifetime: 1.0,
            alive: true,
        }
    }
}

impl Particle {
    pub fn age(&self) -> f32 {
        self.lifetime / self.max_lifetime
    }

    pub fn update(&mut self, dt: f32) {
        if !self.alive {
            return;
        }

        self.lifetime += dt;
        if self.lifetime >= self.max_lifetime {
            self.alive = false;
            return;
        }

        self.velocity += self.acceleration * dt;
        self.position += self.velocity * dt;
        self.rotation += self.angular_velocity * dt;
    }
}

/// Particle emitter shape
#[derive(Clone, Debug)]
pub enum EmitterShape {
    Point,
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
    Cone { angle: f32, radius: f32 },
    Circle { radius: f32 },
}

/// Value that can vary over particle lifetime
#[derive(Clone, Debug)]
pub enum ValueOverLifetime<T: Clone> {
    Constant(T),
    Linear { start: T, end: T },
    Curve(Vec<(f32, T)>), // (time 0-1, value)
}

impl<T: Clone + Lerp> ValueOverLifetime<T> {
    pub fn sample(&self, t: f32) -> T {
        match self {
            Self::Constant(v) => v.clone(),
            Self::Linear { start, end } => T::lerp(start, end, t),
            Self::Curve(points) => {
                if points.is_empty() {
                    panic!("Empty curve");
                }
                if points.len() == 1 {
                    return points[0].1.clone();
                }

                let mut prev = &points[0];
                for point in points.iter().skip(1) {
                    if point.0 >= t {
                        let local_t = (t - prev.0) / (point.0 - prev.0);
                        return T::lerp(&prev.1, &point.1, local_t);
                    }
                    prev = point;
                }
                points.last().unwrap().1.clone()
            }
        }
    }
}

pub trait Lerp {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        a + (b - a) * t
    }
}

impl Lerp for Vec3 {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        a.lerp(*b, t)
    }
}

impl Lerp for Vec4 {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        a.lerp(*b, t)
    }
}

/// Random range for particle properties
#[derive(Clone, Debug)]
pub struct RandomRange<T> {
    pub min: T,
    pub max: T,
}

impl<T: Clone> RandomRange<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    pub fn constant(value: T) -> Self {
        Self { min: value.clone(), max: value }
    }
}

impl RandomRange<f32> {
    pub fn sample(&self) -> f32 {
        // Simple pseudo-random using time
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as f32 / 1_000_000_000.0;
        self.min + (self.max - self.min) * t
    }
}

impl RandomRange<Vec3> {
    pub fn sample(&self) -> Vec3 {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as f32 / 1_000_000_000.0;
        self.min.lerp(self.max, t)
    }
}

/// Particle system configuration
#[derive(Clone, Debug)]
pub struct ParticleSystemConfig {
    pub max_particles: usize,
    pub emission_rate: f32,
    pub burst_count: u32,
    pub shape: EmitterShape,
    pub lifetime: RandomRange<f32>,
    pub start_speed: RandomRange<f32>,
    pub start_size: RandomRange<f32>,
    pub start_rotation: RandomRange<f32>,
    pub start_color: Vec4,
    pub gravity_modifier: f32,
    pub size_over_lifetime: ValueOverLifetime<f32>,
    pub color_over_lifetime: ValueOverLifetime<Vec4>,
    pub velocity_over_lifetime: Option<Vec3>,
    pub rotation_over_lifetime: f32,
    pub world_space: bool,
    pub looping: bool,
    pub duration: f32,
}

impl Default for ParticleSystemConfig {
    fn default() -> Self {
        Self {
            max_particles: 1000,
            emission_rate: 10.0,
            burst_count: 0,
            shape: EmitterShape::Point,
            lifetime: RandomRange::new(1.0, 2.0),
            start_speed: RandomRange::new(1.0, 2.0),
            start_size: RandomRange::new(0.1, 0.2),
            start_rotation: RandomRange::new(0.0, std::f32::consts::TAU),
            start_color: Vec4::ONE,
            gravity_modifier: 0.0,
            size_over_lifetime: ValueOverLifetime::Constant(1.0),
            color_over_lifetime: ValueOverLifetime::Constant(Vec4::ONE),
            velocity_over_lifetime: None,
            rotation_over_lifetime: 0.0,
            world_space: true,
            looping: true,
            duration: 5.0,
        }
    }
}

/// Particle system
pub struct ParticleSystem {
    pub config: ParticleSystemConfig,
    pub position: Vec3,
    pub rotation: glam::Quat,
    particles: Vec<Particle>,
    emission_accumulator: f32,
    time: f32,
    playing: bool,
}

impl ParticleSystem {
    pub fn new(config: ParticleSystemConfig) -> Self {
        let max = config.max_particles;
        Self {
            config,
            position: Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            particles: Vec::with_capacity(max),
            emission_accumulator: 0.0,
            time: 0.0,
            playing: true,
        }
    }

    pub fn fire() -> Self {
        Self::new(ParticleSystemConfig {
            emission_rate: 50.0,
            lifetime: RandomRange::new(0.5, 1.5),
            start_speed: RandomRange::new(2.0, 4.0),
            start_size: RandomRange::new(0.1, 0.3),
            start_color: Vec4::new(1.0, 0.5, 0.0, 1.0),
            gravity_modifier: -0.5,
            color_over_lifetime: ValueOverLifetime::Linear {
                start: Vec4::new(1.0, 0.8, 0.0, 1.0),
                end: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
            size_over_lifetime: ValueOverLifetime::Linear {
                start: 1.0,
                end: 0.0,
            },
            shape: EmitterShape::Cone { angle: 15.0, radius: 0.1 },
            ..Default::default()
        })
    }

    pub fn smoke() -> Self {
        Self::new(ParticleSystemConfig {
            emission_rate: 20.0,
            lifetime: RandomRange::new(2.0, 4.0),
            start_speed: RandomRange::new(0.5, 1.0),
            start_size: RandomRange::new(0.2, 0.5),
            start_color: Vec4::new(0.5, 0.5, 0.5, 0.5),
            gravity_modifier: -0.2,
            color_over_lifetime: ValueOverLifetime::Linear {
                start: Vec4::new(0.5, 0.5, 0.5, 0.5),
                end: Vec4::new(0.3, 0.3, 0.3, 0.0),
            },
            size_over_lifetime: ValueOverLifetime::Linear {
                start: 1.0,
                end: 2.0,
            },
            shape: EmitterShape::Circle { radius: 0.2 },
            ..Default::default()
        })
    }

    pub fn explosion() -> Self {
        Self::new(ParticleSystemConfig {
            emission_rate: 0.0,
            burst_count: 100,
            lifetime: RandomRange::new(0.5, 1.0),
            start_speed: RandomRange::new(5.0, 10.0),
            start_size: RandomRange::new(0.1, 0.2),
            start_color: Vec4::new(1.0, 0.7, 0.0, 1.0),
            gravity_modifier: 1.0,
            color_over_lifetime: ValueOverLifetime::Linear {
                start: Vec4::new(1.0, 0.7, 0.0, 1.0),
                end: Vec4::new(0.5, 0.0, 0.0, 0.0),
            },
            shape: EmitterShape::Sphere { radius: 0.1 },
            looping: false,
            duration: 1.0,
            ..Default::default()
        })
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.time = 0.0;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn burst(&mut self, count: u32) {
        for _ in 0..count {
            self.emit_particle();
        }
    }

    fn emit_particle(&mut self) {
        if self.particles.len() >= self.config.max_particles {
            // Find dead particle to reuse
            let dead_idx = self.particles.iter().position(|p| !p.alive);
            if let Some(idx) = dead_idx {
                self.init_particle_at(idx);
                return;
            }
            return;
        }

        let particle = self.create_new_particle();
        self.particles.push(particle);
    }

    fn init_particle_at(&mut self, idx: usize) {
        let particle = self.create_new_particle();
        self.particles[idx] = particle;
    }

    fn create_new_particle(&self) -> Particle {
        let mut particle = Particle::default();
        particle.alive = true;
        particle.lifetime = 0.0;
        particle.max_lifetime = self.config.lifetime.sample();
        particle.size = self.config.start_size.sample();
        particle.rotation = self.config.start_rotation.sample();
        particle.color = self.config.start_color;

        // Position and velocity based on shape
        let (pos_offset, direction) = match &self.config.shape {
            EmitterShape::Point => (Vec3::ZERO, Vec3::Y),
            EmitterShape::Sphere { radius } => {
                let dir = random_unit_sphere();
                (dir * *radius, dir)
            }
            EmitterShape::Box { half_extents } => {
                let pos = Vec3::new(
                    random_range(-half_extents.x, half_extents.x),
                    random_range(-half_extents.y, half_extents.y),
                    random_range(-half_extents.z, half_extents.z),
                );
                (pos, Vec3::Y)
            }
            EmitterShape::Cone { angle, radius } => {
                let angle_rad = angle.to_radians();
                let r = random_range(0.0, *radius);
                let theta = random_range(0.0, std::f32::consts::TAU);
                let pos = Vec3::new(r * theta.cos(), 0.0, r * theta.sin());
                let spread = random_range(0.0, angle_rad);
                let dir = Vec3::new(spread.sin() * theta.cos(), spread.cos(), spread.sin() * theta.sin());
                (pos, dir.normalize())
            }
            EmitterShape::Circle { radius } => {
                let theta = random_range(0.0, std::f32::consts::TAU);
                let r = random_range(0.0, *radius);
                (Vec3::new(r * theta.cos(), 0.0, r * theta.sin()), Vec3::Y)
            }
        };

        if self.config.world_space {
            particle.position = self.position + self.rotation * pos_offset;
            particle.velocity = self.rotation * direction * self.config.start_speed.sample();
        } else {
            particle.position = pos_offset;
            particle.velocity = direction * self.config.start_speed.sample();
        }

        particle.acceleration = Vec3::new(0.0, -9.81 * self.config.gravity_modifier, 0.0);
        particle
    }

    pub fn update(&mut self, dt: f32) {
        if !self.playing {
            // Still update existing particles
            for particle in &mut self.particles {
                particle.update(dt);
            }
            return;
        }

        self.time += dt;

        // Check duration
        if !self.config.looping && self.time >= self.config.duration {
            self.playing = false;
        }

        // Emit new particles
        if self.config.emission_rate > 0.0 {
            self.emission_accumulator += dt * self.config.emission_rate;
            while self.emission_accumulator >= 1.0 {
                self.emission_accumulator -= 1.0;
                self.emit_particle();
            }
        }

        // Initial burst
        if self.time <= dt && self.config.burst_count > 0 {
            self.burst(self.config.burst_count);
        }

        // Update particles
        for particle in &mut self.particles {
            if !particle.alive {
                continue;
            }

            particle.update(dt);

            // Apply over-lifetime modifiers
            let age = particle.age();
            let _size_mult = self.config.size_over_lifetime.sample(age);
            let color = self.config.color_over_lifetime.sample(age);
            
            particle.color = color;
            // Size is base size * multiplier
            // (stored size is the base, we'd apply multiplier at render time)
        }
    }

    pub fn alive_count(&self) -> usize {
        self.particles.iter().filter(|p| p.alive).count()
    }

    pub fn particles(&self) -> impl Iterator<Item = &Particle> {
        self.particles.iter().filter(|p| p.alive)
    }

    pub fn is_finished(&self) -> bool {
        !self.playing && self.alive_count() == 0
    }
}

// Helper functions
fn random_range(min: f32, max: f32) -> f32 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as f32 / 1_000_000_000.0;
    min + (max - min) * t
}

fn random_unit_sphere() -> Vec3 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as f32;
    let theta = t * 0.001 % std::f32::consts::TAU;
    let phi = (t * 0.0001 % 1.0) * std::f32::consts::PI;
    Vec3::new(
        phi.sin() * theta.cos(),
        phi.cos(),
        phi.sin() * theta.sin(),
    )
}
