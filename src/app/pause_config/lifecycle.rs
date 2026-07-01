use winit::keyboard::KeyCode;

use super::adjust;
use super::display;
use super::input;
use super::{PauseConfig, PauseConfigPage, PauseConfigResult};
use crate::app::context::ReactorContext;

impl PauseConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn show(&mut self, ctx: &mut ReactorContext) {
        ctx.reactor.post_process.enabled = true;
        self.dirty = true;
        self.sync_overlay(ctx);
    }

    pub fn hide(&mut self, ctx: &mut ReactorContext) {
        ctx.reactor.post_process.settings.pause_overlay_alpha = 0.0;
        self.dirty = true;
    }

    pub fn update(&mut self, ctx: &mut ReactorContext) -> PauseConfigResult {
        self.sync_overlay(ctx);
        let input = input::PauseConfigInput::capture(ctx);
        let mut result = PauseConfigResult::default();

        if input.resume {
            self.hide(ctx);
            result.requested_resume = true;
        }
        if input.quit {
            result.requested_quit = true;
        }

        if let Some(page) = input.page {
            self.page = page;
            self.selected = self.selected.min(self.row_count().saturating_sub(1));
            self.dirty = true;
        }

        if input.prev_page || input.next_page {
            self.step_page(if input.next_page { 1 } else { -1 });
            self.dirty = true;
        }

        if input.up || input.down {
            self.step_selected(if input.down { 1 } else { -1 });
            self.dirty = true;
        }

        if input.left || input.right || input.activate {
            let dir = if input.left { -1 } else { 1 };
            result.changed |= self.adjust_selected(ctx, dir, input.activate);
        }

        result.changed |= self.apply_direct_hotkeys(ctx, &input);
        if result.changed {
            self.dirty = true;
        }

        if self.print_on_change && self.dirty {
            display::print(self, ctx);
            self.dirty = false;
        }

        self.sync_overlay(ctx);
        result
    }

    fn sync_overlay(&self, ctx: &mut ReactorContext) {
        ctx.reactor.post_process.enabled = true;
        let settings = &mut ctx.reactor.post_process.settings;
        settings.pause_overlay_alpha = self.overlay_alpha;
        settings.pause_page = self.page_index() as f32;
        settings.pause_selected = self.selected as f32;
        settings.pause_row_count = self.row_count() as f32;
    }

    fn row_count(&self) -> usize {
        match self.page {
            PauseConfigPage::Display => 12,
            PauseConfigPage::Lighting => 13,
            PauseConfigPage::Color => 13,
            PauseConfigPage::Performance => 10,
            PauseConfigPage::Presets => 5,
        }
    }

    pub(super) fn page_index(&self) -> usize {
        PauseConfigPage::ALL
            .iter()
            .position(|p| *p == self.page)
            .unwrap_or(0)
    }

    fn step_page(&mut self, dir: i32) {
        let len = PauseConfigPage::ALL.len() as i32;
        let next = (self.page_index() as i32 + dir).rem_euclid(len) as usize;
        self.page = PauseConfigPage::ALL[next];
        self.selected = self.selected.min(self.row_count().saturating_sub(1));
    }

    fn step_selected(&mut self, dir: i32) {
        let len = self.row_count() as i32;
        self.selected = (self.selected as i32 + dir).rem_euclid(len) as usize;
    }
}
