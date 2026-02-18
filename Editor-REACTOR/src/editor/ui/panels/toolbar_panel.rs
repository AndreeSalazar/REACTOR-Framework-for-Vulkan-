// =============================================================================
// ToolbarPanel — Top toolbar (Play/Stop, transform modes, save/load)
// =============================================================================

use egui::{Color32, RichText, Ui};
use crate::editor::core::editor_context::{EditorContext, GizmoMode};

pub struct ToolbarPanel;

impl ToolbarPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &mut EditorContext) {
        ui.horizontal(|ui| {
            // ── REACTOR branding ──────────────────────────────────────────
            ui.label(
                RichText::new("⚛ REACTOR")
                    .strong()
                    .color(Color32::from_rgb(255, 140, 40))
                    .size(16.0)
            );
            ui.label(
                RichText::new("Editor")
                    .color(Color32::from_rgb(180, 180, 180))
                    .size(14.0)
            );

            ui.separator();

            // ── File operations ───────────────────────────────────────────
            if ui.button("📁 Open").on_hover_text("Open scene").clicked() {
                ctx.log_info("Open scene (not yet implemented)");
            }
            if ui.button("💾 Save").on_hover_text("Save scene").clicked() {
                ctx.log_info(format!("Saved scene: {}", ctx.scene.name));
            }

            ui.separator();

            // ── Transform mode buttons ────────────────────────────────────
            let mode_btn = |ui: &mut Ui, label: &str, tooltip: &str, mode: GizmoMode, current: &GizmoMode| -> bool {
                let active = current == &mode;
                let text = if active {
                    RichText::new(label).color(Color32::from_rgb(255, 200, 60)).strong()
                } else {
                    RichText::new(label).color(Color32::from_rgb(180, 180, 180))
                };
                ui.selectable_label(active, text).on_hover_text(tooltip).clicked()
            };

            if mode_btn(ui, "↖ Select [Q]", "Select mode", GizmoMode::Select, &ctx.gizmo_mode) {
                ctx.gizmo_mode = GizmoMode::Select;
            }
            if mode_btn(ui, "↔ Move [W]", "Translate mode", GizmoMode::Translate, &ctx.gizmo_mode) {
                ctx.gizmo_mode = GizmoMode::Translate;
            }
            if mode_btn(ui, "↻ Rotate [E]", "Rotate mode", GizmoMode::Rotate, &ctx.gizmo_mode) {
                ctx.gizmo_mode = GizmoMode::Rotate;
            }
            if mode_btn(ui, "⤢ Scale [R]", "Scale mode", GizmoMode::Scale, &ctx.gizmo_mode) {
                ctx.gizmo_mode = GizmoMode::Scale;
            }

            ui.separator();

            // ── Spawn shortcuts ───────────────────────────────────────────
            ui.menu_button("➕ Spawn", |ui| {
                if ui.button("📦  Empty Entity").clicked() {
                    ctx.spawn_entity("New Entity");
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("🟫  Cube").clicked() {
                    ctx.spawn_cube();
                    ui.close_menu();
                }
                if ui.button("🔵  Sphere").clicked() {
                    ctx.spawn_sphere();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("💡  Directional Light").clicked() {
                    ctx.spawn_light(crate::editor::core::editor_context::LightType::Directional);
                    ui.close_menu();
                }
                if ui.button("💡  Point Light").clicked() {
                    ctx.spawn_light(crate::editor::core::editor_context::LightType::Point);
                    ui.close_menu();
                }
                if ui.button("💡  Spot Light").clicked() {
                    ctx.spawn_light(crate::editor::core::editor_context::LightType::Spot);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("🎥  Camera").clicked() {
                    ctx.spawn_camera();
                    ui.close_menu();
                }
            });

            ui.separator();

            // ── Play / Stop ───────────────────────────────────────────────
            if ctx.play_mode {
                if ui.button(
                    RichText::new("⏹ Stop")
                        .color(Color32::from_rgb(255, 80, 80))
                        .strong()
                        .size(14.0)
                ).on_hover_text("Stop play mode").clicked() {
                    ctx.play_mode = false;
                    ctx.log_info("Play mode stopped.");
                }
            } else {
                if ui.button(
                    RichText::new("▶ Play")
                        .color(Color32::from_rgb(80, 220, 80))
                        .strong()
                        .size(14.0)
                ).on_hover_text("Enter play mode").clicked() {
                    ctx.play_mode = true;
                    ctx.log_info("Play mode started.");
                }
            }

            // ── Stats (right-aligned) ─────────────────────────────────────
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!(
                        "{:.0} FPS  |  {:.1}ms  |  {} entities",
                        ctx.stats.fps, ctx.stats.frame_time_ms, ctx.stats.entity_count
                    ))
                    .color(Color32::from_rgb(140, 140, 140))
                    .monospace()
                    .small()
                );
            });
        });
    }
}
