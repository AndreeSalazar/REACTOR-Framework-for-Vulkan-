// =============================================================================
// REACTOR Audio System — Real audio playback via rodio
// =============================================================================
// Provides:
//   • AudioClip     – metadata for loaded audio clips
//   • AudioSource   – spatial/non-spatial audio emitter
//   • AudioSystem   – manager with real playback (rodio backend)
//   • AudioListener – camera-attached listener for spatial attenuation
// =============================================================================

use glam::Vec3;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

/// Audio clip handle
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioClipId(pub u32);

/// Audio source handle
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioSourceId(pub u32);

/// Audio clip metadata + raw PCM bytes for playback
#[derive(Clone, Debug)]
pub struct AudioClip {
    pub id: AudioClipId,
    pub name: String,
    pub duration: f32,
    pub channels: u32,
    pub sample_rate: u32,
    /// Raw file bytes for decoding via rodio
    pub(crate) raw_bytes: Arc<Vec<u8>>,
}

impl AudioClip {
    /// Load an audio clip from file (WAV, OGG, MP3, FLAC)
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = std::fs::read(path.as_ref())?;
        Self::from_bytes(path.as_ref().to_string_lossy().to_string(), &bytes)
    }

    /// Load an audio clip from raw bytes
    pub fn from_bytes(
        name: String,
        bytes: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut channels = 2;
        let mut sample_rate = 44100;
        let mut duration = 0.0;

        // Try to parse WAV header for metadata
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
                    let bytes_per_sample = 2; // Default to 16-bit
                    let total_samples = chunk_size / (channels as usize * bytes_per_sample);
                    duration = total_samples as f32 / sample_rate as f32;
                    break;
                }
                cursor += chunk_size;
            }
        }

        // Fallback duration estimation for OGG / MP3
        if duration == 0.0 {
            duration = (bytes.len() as f32 / 176400.0).max(0.1);
        }

        Ok(Self {
            id: AudioClipId(0), // Assigned by AudioSystem::register_clip_data
            name,
            duration,
            channels,
            sample_rate,
            raw_bytes: Arc::new(bytes.to_vec()),
        })
    }
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
    pub fn from_camera(camera: &crate::scene::camera::Camera) -> Self {
        Self {
            position: camera.position,
            forward: camera.forward(),
            up: camera.up(),
            velocity: Vec3::ZERO,
        }
    }
}

/// Audio system manager with real playback via rodio
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
    /// rodio output stream handle — kept alive to maintain audio output
    _stream: Option<rodio::OutputStream>,
    /// rodio stream handle for creating sinks
    stream_handle: Option<rodio::OutputStreamHandle>,
    /// Active sinks for currently playing sounds
    active_sinks: HashMap<AudioSourceId, rodio::Sink>,
}

impl AudioSystem {
    pub fn new() -> Self {
        // Try to initialize rodio audio output
        let (stream, stream_handle) = match rodio::OutputStream::try_default() {
            Ok((s, h)) => {
                log::info!("🔊 REACTOR Audio: rodio backend initialized");
                (Some(s), Some(h))
            }
            Err(e) => {
                log::warn!("🔇 REACTOR Audio: failed to init rodio ({e}), running in silent mode");
                (None, None)
            }
        };

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
            _stream: stream,
            stream_handle,
            active_sinks: HashMap::new(),
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

    /// Register an audio clip by name and estimated duration (metadata-only, no bytes)
    pub fn register_clip(&mut self, name: &str, duration: f32) -> AudioClipId {
        let id = AudioClipId(self.next_clip_id);
        self.next_clip_id += 1;

        self.clips.insert(
            id,
            AudioClip {
                id,
                name: name.to_string(),
                duration,
                channels: 2,
                sample_rate: 44100,
                raw_bytes: Arc::new(Vec::new()),
            },
        );

        id
    }

    /// Register a fully-loaded AudioClip and return its assigned ID
    pub fn register_clip_data(&mut self, mut clip: AudioClip) -> AudioClipId {
        let id = AudioClipId(self.next_clip_id);
        self.next_clip_id += 1;
        clip.id = id;
        self.clips.insert(id, clip);
        id
    }

    /// Load and register an audio clip from a file path
    pub fn load_clip<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<AudioClipId, Box<dyn std::error::Error + Send + Sync>> {
        let clip = AudioClip::from_file(path)?;
        Ok(self.register_clip_data(clip))
    }

    /// Create a new audio source
    pub fn create_source(&mut self) -> AudioSourceId {
        let id = AudioSourceId(self.next_source_id);
        self.next_source_id += 1;

        let source = AudioSource { id, ..Default::default() };
        self.sources.insert(id, source);

        id
    }

    /// Get mutable reference to a source
    pub fn get_source_mut(&mut self, id: AudioSourceId) -> Option<&mut AudioSource> {
        self.sources.get_mut(&id)
    }

    /// Play a source using rodio
    pub fn play(&mut self, id: AudioSourceId) {
        let (clip_id, volume, looping) = {
            if let Some(source) = self.sources.get_mut(&id) {
                source.playing = true;
                source.time = 0.0;
                match source.clip {
                    Some(cid) => (cid, source.volume, source.looping),
                    None => return,
                }
            } else {
                return;
            }
        };
        self.play_clip_on_sink(id, clip_id, volume, looping);
    }

    /// Stop a source
    pub fn stop(&mut self, id: AudioSourceId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.playing = false;
            source.time = 0.0;
        }
        if let Some(sink) = self.active_sinks.remove(&id) {
            sink.stop();
        }
    }

    /// Pause a source
    pub fn pause(&mut self, id: AudioSourceId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.playing = false;
        }
        if let Some(sink) = self.active_sinks.get(&id) {
            sink.pause();
        }
    }

    /// Resume a paused source
    pub fn resume(&mut self, id: AudioSourceId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.playing = true;
        }
        if let Some(sink) = self.active_sinks.get(&id) {
            sink.play();
        }
    }

    /// Play a one-shot sound effect via rodio
    pub fn play_sfx(
        &mut self,
        clip: AudioClipId,
        position: Option<Vec3>,
        volume: f32,
    ) -> AudioSourceId {
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

        // Compute effective volume for spatial audio
        let effective_volume = if let Some(source) = self.sources.get(&id) {
            self.calculate_spatial_volume_for(source) * self.master_volume * self.sfx_volume
        } else {
            volume * self.master_volume * self.sfx_volume
        };

        self.play_clip_on_sink(id, clip, effective_volume, false);
        id
    }

    /// Internal: play a clip on a new rodio Sink
    fn play_clip_on_sink(
        &mut self,
        source_id: AudioSourceId,
        clip_id: AudioClipId,
        volume: f32,
        looping: bool,
    ) {
        if !self.enabled {
            return;
        }

        let stream_handle = match &self.stream_handle {
            Some(h) => h,
            None => return,
        };

        let clip = match self.clips.get(&clip_id) {
            Some(c) => c,
            None => return,
        };

        if clip.raw_bytes.is_empty() {
            return; // No audio data to play
        }

        // Create a rodio Sink
        let sink = match rodio::Sink::try_new(stream_handle) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to create audio sink: {e}");
                return;
            }
        };

        sink.set_volume(volume * self.master_volume);

        // Decode and append the audio data
        let cursor = Cursor::new(clip.raw_bytes.as_ref().clone());
        match rodio::Decoder::new(cursor) {
            Ok(decoded) => {
                if looping {
                    use rodio::Source;
                    sink.append(decoded.repeat_infinite());
                } else {
                    sink.append(decoded);
                }
            }
            Err(e) => {
                log::warn!("Failed to decode audio clip '{}': {e}", clip.name);
                return;
            }
        }

        // Store sink to keep it alive
        self.active_sinks.insert(source_id, sink);
    }

    /// Update audio system (call each frame)
    pub fn update(&mut self, delta_time: f32) {
        if !self.enabled {
            return;
        }

        let mut to_remove = Vec::new();
        let mut finished_sinks = Vec::new();

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
                            to_remove.push(*id);
                        }
                    }
                }
            }
        }

        // Check for finished sinks (rodio's Sink reports empty when done)
        for (id, sink) in &self.active_sinks {
            if sink.empty() {
                finished_sinks.push(*id);
            }
        }

        // Clean up finished one-shot sources
        for id in &to_remove {
            self.sources.remove(id);
            self.active_sinks.remove(id);
        }
        for id in &finished_sinks {
            if !to_remove.contains(id) {
                self.active_sinks.remove(id);
                // Also mark the source as not playing
                if let Some(source) = self.sources.get_mut(id) {
                    source.playing = false;
                }
            }
        }
    }

    /// Calculate volume based on distance (for spatial audio)
    pub fn calculate_spatial_volume(&self, source: &AudioSource) -> f32 {
        self.calculate_spatial_volume_for(source)
    }

    /// Internal spatial volume calculation
    fn calculate_spatial_volume_for(&self, source: &AudioSource) -> f32 {
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

    /// Get count of active rodio sinks (actually playing hardware audio)
    pub fn active_sink_count(&self) -> usize {
        self.active_sinks.values().filter(|s| !s.empty()).count()
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}
