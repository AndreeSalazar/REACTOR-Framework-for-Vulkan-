// Extracted from xenofall.rs — Audio
// =============================================================================
// AUDIO IDs
// =============================================================================

use reactor_vulkan::prelude::*;

pub struct GameAudio {
    pub gunshot: Option<AudioClipId>,
    pub reload: Option<AudioClipId>,
    pub zombie_groan: Option<AudioClipId>,
    pub impact: Option<AudioClipId>,
    pub death: Option<AudioClipId>,
    pub combo: Option<AudioClipId>,
    pub card_select: Option<AudioClipId>,
    pub damage: Option<AudioClipId>,
    pub wave_start: Option<AudioClipId>,
    pub victory: Option<AudioClipId>,
}

impl GameAudio {
    pub fn new() -> Self {
        Self {
            gunshot: None,
            reload: None,
            zombie_groan: None,
            impact: None,
            death: None,
            combo: None,
            card_select: None,
            damage: None,
            wave_start: None,
            victory: None,
        }
    }

    pub fn load_all(&mut self, audio: &mut AudioSystem) {
        self.gunshot = Self::try_load(audio, "assets/audio/gunshot.wav");
        self.reload = Self::try_load(audio, "assets/audio/reload.wav");
        self.zombie_groan = Self::try_load(audio, "assets/audio/zombie_groan.wav");
        self.impact = Self::try_load(audio, "assets/audio/impact.wav");
        self.death = Self::try_load(audio, "assets/audio/death.wav");
        self.combo = Self::try_load(audio, "assets/audio/combo.wav");
        self.card_select = Self::try_load(audio, "assets/audio/card_select.wav");
        self.damage = Self::try_load(audio, "assets/audio/damage.wav");
        self.wave_start = Self::try_load(audio, "assets/audio/wave_start.wav");
        self.victory = Self::try_load(audio, "assets/audio/victory.wav");
    }

    fn try_load(audio: &mut AudioSystem, path: &str) -> Option<AudioClipId> {
        match audio.load_clip(path) {
            Ok(id) => {
                Log::audio(&format!("Loaded: {}", path));
                Some(id)
            }
            Err(e) => {
                Log::error(&format!("Could not load {}: {}", path, e));
                None
            }
        }
    }
}
