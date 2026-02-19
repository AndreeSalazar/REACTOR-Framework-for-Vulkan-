// =============================================================================
// ConsolePanel — Editor log output
// =============================================================================

use crate::editor::core::editor_context::{ConsoleEntry, EditorContext, LogLevel};
use egui::{Color32, RichText, Ui};

pub struct ConsolePanel {
    pub filter_info: bool,
    pub filter_warn: bool,
    pub filter_error: bool,
    pub search: String,
    pub auto_scroll: bool,
}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {
            filter_info: true,
            filter_warn: true,
            filter_error: true,
            search: String::new(),
            auto_scroll: true,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &mut EditorContext) {
        // Header toolbar
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("🖥 Console")
                    .strong()
                    .color(Color32::from_rgb(200, 200, 200)),
            );

            ui.separator();

            // Level toggles
            ui.toggle_value(
                &mut self.filter_info,
                RichText::new("INFO")
                    .color(Color32::from_rgb(180, 220, 180))
                    .small(),
            );
            ui.toggle_value(
                &mut self.filter_warn,
                RichText::new("WARN")
                    .color(Color32::from_rgb(255, 200, 60))
                    .small(),
            );
            ui.toggle_value(
                &mut self.filter_error,
                RichText::new("ERROR")
                    .color(Color32::from_rgb(255, 80, 80))
                    .small(),
            );

            ui.separator();

            // Search
            ui.label("🔍");
            ui.add(egui::TextEdit::singleline(&mut self.search).desired_width(120.0));

            ui.separator();

            // Auto-scroll toggle
            ui.toggle_value(&mut self.auto_scroll, "↓ Auto");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("🗑 Clear").clicked() {
                    ctx.console_log.clear();
                }

                // Counts
                let info_count = ctx
                    .console_log
                    .iter()
                    .filter(|e| e.level == LogLevel::Info)
                    .count();
                let warn_count = ctx
                    .console_log
                    .iter()
                    .filter(|e| e.level == LogLevel::Warning)
                    .count();
                let err_count = ctx
                    .console_log
                    .iter()
                    .filter(|e| e.level == LogLevel::Error)
                    .count();

                ui.label(
                    RichText::new(format!("{} ✖", err_count))
                        .color(Color32::from_rgb(255, 80, 80))
                        .small(),
                );
                ui.label(
                    RichText::new(format!("{} ⚠", warn_count))
                        .color(Color32::from_rgb(255, 200, 60))
                        .small(),
                );
                ui.label(
                    RichText::new(format!("{} ℹ", info_count))
                        .color(Color32::from_rgb(100, 200, 100))
                        .small(),
                );
            });
        });

        ui.separator();

        // Log entries
        let scroll = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll);

        scroll.show(ui, |ui| {
            let entries: Vec<ConsoleEntry> = ctx
                .console_log
                .iter()
                .filter(|e| match e.level {
                    LogLevel::Info => self.filter_info,
                    LogLevel::Warning => self.filter_warn,
                    LogLevel::Error => self.filter_error,
                })
                .filter(|e| {
                    self.search.is_empty()
                        || e.message
                            .to_lowercase()
                            .contains(&self.search.to_lowercase())
                })
                .cloned()
                .collect();

            for entry in &entries {
                let (icon, color) = match entry.level {
                    LogLevel::Info => ("ℹ", Color32::from_rgb(160, 220, 160)),
                    LogLevel::Warning => ("⚠", Color32::from_rgb(255, 200, 60)),
                    LogLevel::Error => ("✖", Color32::from_rgb(255, 100, 100)),
                };

                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).color(color).small());
                    ui.label(
                        RichText::new(&entry.message)
                            .color(color)
                            .monospace()
                            .small(),
                    );
                });
            }

            if entries.is_empty() {
                ui.label(
                    RichText::new("No log entries.")
                        .color(Color32::from_rgb(100, 100, 100))
                        .italics()
                        .small(),
                );
            }
        });
    }
}
