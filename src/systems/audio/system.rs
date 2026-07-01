use std::collections::HashMap;
use std::io::Cursor;
use glam::Vec3;
use crate::systems::audio::types::*;

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
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
    active_sinks: HashMap<AudioSourceId, rodio::Sink>,
}

impl AudioSystem {
    pub fn new() -> Self {
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
                raw_bytes: std::sync::Arc::new(Vec::new()),
            },
        );

        id
    }

    pub fn register_clip_data(&mut self, mut clip: AudioClip) -> AudioClipId {
        let id = AudioClipId(self.next_clip_id);
        self.next_clip_id += 1;
        clip.id = id;
        self.clips.insert(id, clip);
        id
    }

    pub fn load_clip<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<AudioClipId, Box<dyn std::error::Error + Send + Sync>> {
        let clip = AudioClip::from_file(path)?;
        Ok(self.register_clip_data(clip))
    }

    pub fn create_source(&mut self) -> AudioSourceId {
        let id = AudioSourceId(self.next_source_id);
        self.next_source_id += 1;

        let source = AudioSource { id, ..Default::default() };
        self.sources.insert(id, source);

        id
    }

    pub fn get_source_mut(&mut self, id: AudioSourceId) -> Option<&mut AudioSource> {
        self.sources.get_mut(&id)
    }

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

    pub fn stop(&mut self, id: AudioSourceId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.playing = false;
            source.time = 0.0;
        }
        if let Some(sink) = self.active_sinks.remove(&id) {
            sink.stop();
        }
    }

    pub fn pause(&mut self, id: AudioSourceId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.playing = false;
        }
        if let Some(sink) = self.active_sinks.get(&id) {
            sink.pause();
        }
    }

    pub fn resume(&mut self, id: AudioSourceId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.playing = true;
        }
        if let Some(sink) = self.active_sinks.get(&id) {
            sink.play();
        }
    }

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

        let effective_volume = if let Some(source) = self.sources.get(&id) {
            self.calculate_spatial_volume_for(source) * self.master_volume * self.sfx_volume
        } else {
            volume * self.master_volume * self.sfx_volume
        };

        self.play_clip_on_sink(id, clip, effective_volume, false);
        id
    }

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
            return;
        }

        let sink = match rodio::Sink::try_new(stream_handle) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to create audio sink: {e}");
                return;
            }
        };

        sink.set_volume(volume * self.master_volume);

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

        self.active_sinks.insert(source_id, sink);
    }

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

        for (id, sink) in &self.active_sinks {
            if sink.empty() {
                finished_sinks.push(*id);
            }
        }

        for id in &to_remove {
            self.sources.remove(id);
            self.active_sinks.remove(id);
        }
        for id in &finished_sinks {
            if !to_remove.contains(id) {
                self.active_sinks.remove(id);
                if let Some(source) = self.sources.get_mut(id) {
                    source.playing = false;
                }
            }
        }
    }

    pub fn calculate_spatial_volume(&self, source: &AudioSource) -> f32 {
        self.calculate_spatial_volume_for(source)
    }

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

    pub fn active_sink_count(&self) -> usize {
        self.active_sinks.values().filter(|s| !s.empty()).count()
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}
