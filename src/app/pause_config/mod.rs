mod adjust;
mod display;
mod input;
mod lifecycle;
mod utils;

use crate::app::context::ReactorContext;
use crate::core::PixelIntelligentProfile;
use crate::graphics::post_process::{PostProcessEffect, PostProcessSettings};
use winit::keyboard::KeyCode;
use winit::window::Fullscreen;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseConfigPage {
    Display,
    Lighting,
    Color,
    Performance,
    Presets,
}

impl PauseConfigPage {
    const ALL: [Self; 5] = [
        Self::Display,
        Self::Lighting,
        Self::Color,
        Self::Performance,
        Self::Presets,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Display => "DISPLAY",
            Self::Lighting => "LIGHTING / PT",
            Self::Color => "COLOR / POST FX",
            Self::Performance => "PERFORMANCE",
            Self::Presets => "PRESETS",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PauseConfigResult {
    pub changed: bool,
    pub requested_resume: bool,
    pub requested_quit: bool,
}

#[derive(Debug, Clone)]
pub struct PauseConfig {
    pub title: String,
    pub page: PauseConfigPage,
    pub selected: usize,
    pub print_on_change: bool,
    pub overlay_alpha: f32,
    dirty: bool,
}

impl Default for PauseConfig {
    fn default() -> Self {
        Self {
            title: "REACTOR PAUSE CONFIG".to_string(),
            page: PauseConfigPage::Display,
            selected: 0,
            print_on_change: true,
            overlay_alpha: 0.78,
            dirty: true,
        }
    }
}

pub type PauseConfiguration = PauseConfig;
pub type PauseConfiguracion = PauseConfig;
