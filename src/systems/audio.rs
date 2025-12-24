// Audio System Placeholder
// NOTE: Full audio implementation would require a library like rodio, kira, or cpal
// This provides the API structure for future implementation

use glam::Vec3;
use std::collections::HashMap;

/// Audio clip handle
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioClipId(pub u32);

/// Audio source handle
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioSourceId(pub u32);

/// Audio clip metadata
#[derive(Clone, Debug)]
pub struct AudioClip {
    pub id: AudioClipId,
    pub name: String,
    pub duration: f32,
    pub channels: u32,
    pub sample_rate: u32,
}

/// Audio source component for 3D spatial audio
#[derive(Clone, Debug)]
pub struct AudioSource {
    pub id: AudioSourceId,
    pub clip: Option<AudioClipId>,
    pub volume: f32,
    pub pitch: f32,
    pub looping: bool,
    pub spatial: bool,
    pub min_distance: f32,
    pub max_distance: f32,
    pub position: Vec3,
    pub playing: bool,
    pub time: f32,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            id: AudioSourceId(0),
            clip: None,
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            spatial: true,
            min_distance: 1.0,
            max_distance: 100.0,
            position: Vec3::ZERO,
            playing: false,
            time: 0.0,
        }
    }
}

impl AudioSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_clip(mut self, clip: AudioClipId) -> Self {
        self.clip = Some(clip);
        self
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.1, 3.0);
        self
    }

    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    pub fn spatial_3d(mut self, min_dist: f32, max_dist: f32) -> Self {
        self.spatial = true;
        self.min_distance = min_dist;
        self.max_distance = max_dist;
        self
    }

    pub fn non_spatial(mut self) -> Self {
        self.spatial = false;
        self
    }
}

/// Audio listener (usually attached to camera)
#[derive(Clone, Debug)]
pub struct AudioListener {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub velocity: Vec3,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: Vec3::NEG_Z,
            up: Vec3::Y,
            velocity: Vec3::ZERO,
        }
    }
}

impl AudioListener {
    pub fn from_camera(camera: &crate::systems::camera::Camera) -> Self {
        Self {
            position: camera.position,
            forward: camera.forward(),
            up: camera.up(),
            velocity: Vec3::ZERO,
        }
    }
}

/// Audio system manager
pub struct AudioSystem {
    clips: HashMap<AudioClipId, AudioClip>,
    sources: HashMap<AudioSourceId, AudioSource>,
    listener: AudioListener,
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    next_clip_id: u32,
    next_source_id: u32,
    enabled: bool,
}

impl AudioSystem {
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
            sources: HashMap::new(),
            listener: AudioListener::default(),
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            next_clip_id: 1,
            next_source_id: 1,
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    pub fn update_listener(&mut self, listener: AudioListener) {
        self.listener = listener;
    }

    /// Register an audio clip (placeholder - would load actual audio data)
    pub fn register_clip(&mut self, name: &str, duration: f32) -> AudioClipId {
        let id = AudioClipId(self.next_clip_id);
        self.next_clip_id += 1;

        self.clips.insert(id, AudioClip {
            id,
            name: name.to_string(),
            duration,
            channels: 2,
            sample_rate: 44100,
        });

        id
    }

    /// Create a new audio source
    pub fn create_source(&mut self) -> AudioSourceId {
        let id = AudioSourceId(self.next_source_id);
        self.next_source_id += 1;

        let mut source = AudioSource::default();
        source.id = id;
        self.sources.insert(id, source);

        id
    }

    /// Get mutable reference to a source
    pub fn get_source_mut(&mut self, id: AudioSourceId) -> Option<&mut AudioSource> {
        self.sources.get_mut(&id)
    }

    /// Play a source
    pub fn play(&mut self, id: AudioSourceId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.playing = true;
            source.time = 0.0;
        }
    }

    /// Stop a source
    pub fn stop(&mut self, id: AudioSourceId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.playing = false;
            source.time = 0.0;
        }
    }

    /// Pause a source
    pub fn pause(&mut self, id: AudioSourceId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.playing = false;
        }
    }

    /// Play a one-shot sound effect
    pub fn play_sfx(&mut self, clip: AudioClipId, position: Option<Vec3>, volume: f32) -> AudioSourceId {
        let id = self.create_source();
        if let Some(source) = self.sources.get_mut(&id) {
            source.clip = Some(clip);
            source.volume = volume;
            source.looping = false;
            if let Some(pos) = position {
                source.spatial = true;
                source.position = pos;
            } else {
                source.spatial = false;
            }
            source.playing = true;
        }
        id
    }

    /// Update audio system (call each frame)
    pub fn update(&mut self, delta_time: f32) {
        if !self.enabled {
            return;
        }

        let mut to_remove = Vec::new();

        for (id, source) in &mut self.sources {
            if !source.playing {
                continue;
            }

            source.time += delta_time * source.pitch;

            if let Some(clip_id) = source.clip {
                if let Some(clip) = self.clips.get(&clip_id) {
                    if source.time >= clip.duration {
                        if source.looping {
                            source.time %= clip.duration;
                        } else {
                            source.playing = false;
                            // Mark non-looping finished sources for cleanup
                            if !source.looping {
                                to_remove.push(*id);
                            }
                        }
                    }
                }
            }
        }

        // Clean up finished one-shot sources
        for id in to_remove {
            self.sources.remove(&id);
        }
    }

    /// Calculate volume based on distance (for spatial audio)
    pub fn calculate_spatial_volume(&self, source: &AudioSource) -> f32 {
        if !source.spatial {
            return source.volume;
        }

        let distance = (source.position - self.listener.position).length();
        
        if distance <= source.min_distance {
            source.volume
        } else if distance >= source.max_distance {
            0.0
        } else {
            let range = source.max_distance - source.min_distance;
            let attenuation = 1.0 - (distance - source.min_distance) / range;
            source.volume * attenuation
        }
    }

    pub fn active_source_count(&self) -> usize {
        self.sources.values().filter(|s| s.playing).count()
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}
