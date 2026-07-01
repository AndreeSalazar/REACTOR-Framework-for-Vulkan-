use std::sync::Arc;
use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioClipId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioSourceId(pub u32);

#[derive(Clone, Debug)]
pub struct AudioClip {
    pub id: AudioClipId,
    pub name: String,
    pub duration: f32,
    pub channels: u32,
    pub sample_rate: u32,
    pub(crate) raw_bytes: Arc<Vec<u8>>,
}

impl AudioClip {
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = std::fs::read(path.as_ref())?;
        Self::from_bytes(path.as_ref().to_string_lossy().to_string(), &bytes)
    }

    pub fn from_bytes(
        name: String,
        bytes: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut channels = 2;
        let mut sample_rate = 44100;
        let mut duration = 0.0;

        if bytes.len() >= 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
            let mut cursor = 12;
            while cursor + 8 < bytes.len() {
                let chunk_id = &bytes[cursor..cursor + 4];
                let chunk_size =
                    u32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into()?) as usize;
                cursor += 8;

                if chunk_id == b"fmt " && cursor + 16 <= bytes.len() {
                    channels = u16::from_le_bytes(bytes[cursor + 2..cursor + 4].try_into()?) as u32;
                    sample_rate = u32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into()?);
                } else if chunk_id == b"data" {
                    let bytes_per_sample = 2;
                    let total_samples = chunk_size / (channels as usize * bytes_per_sample);
                    duration = total_samples as f32 / sample_rate as f32;
                    break;
                }
                cursor += chunk_size;
            }
        }

        if duration == 0.0 {
            duration = (bytes.len() as f32 / 176400.0).max(0.1);
        }

        Ok(Self {
            id: AudioClipId(0),
            name,
            duration,
            channels,
            sample_rate,
            raw_bytes: Arc::new(bytes.to_vec()),
        })
    }
}

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
    pub fn from_camera(camera: &crate::scene::camera::Camera) -> Self {
        Self {
            position: camera.position,
            forward: camera.forward(),
            up: camera.up(),
            velocity: Vec3::ZERO,
        }
    }
}
