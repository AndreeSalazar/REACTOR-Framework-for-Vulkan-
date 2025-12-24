use std::time::{Duration, Instant};

pub struct Time {
    start_time: Instant,
    last_frame: Instant,
    delta_time: Duration,
    frame_count: u64,
    fps: f32,
    fps_update_timer: Duration,
    frames_this_second: u32,
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame: now,
            delta_time: Duration::ZERO,
            frame_count: 0,
            fps: 0.0,
            fps_update_timer: Duration::ZERO,
            frames_this_second: 0,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta_time = now - self.last_frame;
        self.last_frame = now;
        self.frame_count += 1;
        self.frames_this_second += 1;

        self.fps_update_timer += self.delta_time;
        if self.fps_update_timer >= Duration::from_secs(1) {
            self.fps = self.frames_this_second as f32 / self.fps_update_timer.as_secs_f32();
            self.fps_update_timer = Duration::ZERO;
            self.frames_this_second = 0;
        }
    }

    pub fn delta(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    pub fn delta_duration(&self) -> Duration {
        self.delta_time
    }

    pub fn elapsed(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }

    pub fn elapsed_duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn fps_string(&self) -> String {
        format!("{:.1} FPS", self.fps)
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

// Fixed timestep for physics
pub struct FixedTimestep {
    accumulator: Duration,
    timestep: Duration,
}

impl FixedTimestep {
    pub fn new(hz: u32) -> Self {
        Self {
            accumulator: Duration::ZERO,
            timestep: Duration::from_secs_f64(1.0 / hz as f64),
        }
    }

    pub fn update(&mut self, delta: Duration) -> u32 {
        self.accumulator += delta;
        let mut steps = 0;
        
        while self.accumulator >= self.timestep {
            self.accumulator -= self.timestep;
            steps += 1;
        }
        
        steps
    }

    pub fn timestep(&self) -> f32 {
        self.timestep.as_secs_f32()
    }

    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.timestep.as_secs_f32()
    }
}
