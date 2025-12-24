use glam::{Vec3, Quat};
use std::collections::HashMap;

/// Keyframe for animation
#[derive(Clone, Debug)]
pub struct Keyframe<T: Clone> {
    pub time: f32,
    pub value: T,
}

impl<T: Clone> Keyframe<T> {
    pub fn new(time: f32, value: T) -> Self {
        Self { time, value }
    }
}

/// Animation track for a single property
#[derive(Clone, Debug)]
pub struct AnimationTrack<T: Clone + Interpolate> {
    pub keyframes: Vec<Keyframe<T>>,
    pub loop_mode: LoopMode,
}

impl<T: Clone + Interpolate> AnimationTrack<T> {
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
            loop_mode: LoopMode::Once,
        }
    }

    pub fn add_keyframe(&mut self, time: f32, value: T) {
        self.keyframes.push(Keyframe::new(time, value));
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
    }

    pub fn sample(&self, time: f32) -> Option<T> {
        if self.keyframes.is_empty() {
            return None;
        }

        if self.keyframes.len() == 1 {
            return Some(self.keyframes[0].value.clone());
        }

        let duration = self.duration();
        let t = match self.loop_mode {
            LoopMode::Once => time.min(duration),
            LoopMode::Loop => time % duration,
            LoopMode::PingPong => {
                let cycle = (time / duration) as i32;
                if cycle % 2 == 0 {
                    time % duration
                } else {
                    duration - (time % duration)
                }
            }
        };

        // Find surrounding keyframes
        let mut prev_idx = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time <= t {
                prev_idx = i;
            } else {
                break;
            }
        }

        let next_idx = (prev_idx + 1).min(self.keyframes.len() - 1);

        if prev_idx == next_idx {
            return Some(self.keyframes[prev_idx].value.clone());
        }

        let prev = &self.keyframes[prev_idx];
        let next = &self.keyframes[next_idx];

        let factor = (t - prev.time) / (next.time - prev.time);
        Some(T::interpolate(&prev.value, &next.value, factor))
    }
}

impl<T: Clone + Interpolate> Default for AnimationTrack<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Loop mode for animations
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
}

/// Trait for interpolatable values
pub trait Interpolate {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        a + (b - a) * t
    }
}

impl Interpolate for Vec3 {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        a.lerp(*b, t)
    }
}

impl Interpolate for Quat {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        a.slerp(*b, t)
    }
}

/// Animation clip containing multiple tracks
#[derive(Clone, Debug)]
pub struct AnimationClip {
    pub name: String,
    pub position_track: Option<AnimationTrack<Vec3>>,
    pub rotation_track: Option<AnimationTrack<Quat>>,
    pub scale_track: Option<AnimationTrack<Vec3>>,
    pub loop_mode: LoopMode,
}

impl AnimationClip {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            position_track: None,
            rotation_track: None,
            scale_track: None,
            loop_mode: LoopMode::Once,
        }
    }

    pub fn duration(&self) -> f32 {
        let mut max_duration = 0.0f32;
        
        if let Some(track) = &self.position_track {
            max_duration = max_duration.max(track.duration());
        }
        if let Some(track) = &self.rotation_track {
            max_duration = max_duration.max(track.duration());
        }
        if let Some(track) = &self.scale_track {
            max_duration = max_duration.max(track.duration());
        }
        
        max_duration
    }

    pub fn sample(&self, time: f32) -> AnimationSample {
        AnimationSample {
            position: self.position_track.as_ref().and_then(|t| t.sample(time)),
            rotation: self.rotation_track.as_ref().and_then(|t| t.sample(time)),
            scale: self.scale_track.as_ref().and_then(|t| t.sample(time)),
        }
    }
}

/// Sampled animation values
#[derive(Clone, Debug)]
pub struct AnimationSample {
    pub position: Option<Vec3>,
    pub rotation: Option<Quat>,
    pub scale: Option<Vec3>,
}

impl AnimationSample {
    pub fn apply_to_transform(&self, transform: &mut crate::systems::transform::Transform) {
        if let Some(pos) = self.position {
            transform.position = pos;
        }
        if let Some(rot) = self.rotation {
            transform.rotation = rot;
        }
        if let Some(scale) = self.scale {
            transform.scale = scale;
        }
    }
}

/// Animation player component
#[derive(Clone, Debug)]
pub struct AnimationPlayer {
    pub clips: HashMap<String, AnimationClip>,
    pub current_clip: Option<String>,
    pub time: f32,
    pub speed: f32,
    pub playing: bool,
}

impl AnimationPlayer {
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
            current_clip: None,
            time: 0.0,
            speed: 1.0,
            playing: false,
        }
    }

    pub fn add_clip(&mut self, clip: AnimationClip) {
        self.clips.insert(clip.name.clone(), clip);
    }

    pub fn play(&mut self, name: &str) {
        if self.clips.contains_key(name) {
            self.current_clip = Some(name.to_string());
            self.time = 0.0;
            self.playing = true;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.time = 0.0;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn resume(&mut self) {
        self.playing = true;
    }

    pub fn update(&mut self, delta_time: f32) -> Option<AnimationSample> {
        if !self.playing {
            return None;
        }

        let clip_name = self.current_clip.as_ref()?;
        let clip = self.clips.get(clip_name)?;

        self.time += delta_time * self.speed;

        let duration = clip.duration();
        match clip.loop_mode {
            LoopMode::Once => {
                if self.time >= duration {
                    self.time = duration;
                    self.playing = false;
                }
            }
            LoopMode::Loop => {
                if self.time >= duration {
                    self.time %= duration;
                }
            }
            LoopMode::PingPong => {
                // Handled in track sampling
            }
        }

        Some(clip.sample(self.time))
    }

    pub fn is_finished(&self) -> bool {
        if let Some(clip_name) = &self.current_clip {
            if let Some(clip) = self.clips.get(clip_name) {
                return clip.loop_mode == LoopMode::Once && self.time >= clip.duration();
            }
        }
        true
    }
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Tween for simple value animations
pub struct Tween<T: Clone + Interpolate> {
    pub start: T,
    pub end: T,
    pub duration: f32,
    pub elapsed: f32,
    pub easing: EasingFunction,
}

impl<T: Clone + Interpolate> Tween<T> {
    pub fn new(start: T, end: T, duration: f32) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: 0.0,
            easing: EasingFunction::Linear,
        }
    }

    pub fn with_easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }

    pub fn update(&mut self, delta: f32) -> T {
        self.elapsed = (self.elapsed + delta).min(self.duration);
        let t = self.elapsed / self.duration;
        let eased_t = self.easing.apply(t);
        T::interpolate(&self.start, &self.end, eased_t)
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }
}

/// Easing functions
#[derive(Clone, Copy, Debug)]
pub enum EasingFunction {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInElastic,
    EaseOutElastic,
    EaseOutBounce,
}

impl EasingFunction {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseInQuad => t * t,
            Self::EaseOutQuad => t * (2.0 - t),
            Self::EaseInOutQuad => {
                if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t }
            }
            Self::EaseInCubic => t * t * t,
            Self::EaseOutCubic => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t = 2.0 * t - 2.0;
                    0.5 * t * t * t + 1.0
                }
            }
            Self::EaseInElastic => {
                if t == 0.0 || t == 1.0 { return t; }
                let p = 0.3;
                let s = p / 4.0;
                let t = t - 1.0;
                -(2.0_f32.powf(10.0 * t) * ((t - s) * std::f32::consts::TAU / p).sin())
            }
            Self::EaseOutElastic => {
                if t == 0.0 || t == 1.0 { return t; }
                let p = 0.3;
                let s = p / 4.0;
                2.0_f32.powf(-10.0 * t) * ((t - s) * std::f32::consts::TAU / p).sin() + 1.0
            }
            Self::EaseOutBounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
        }
    }
}
