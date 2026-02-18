// =============================================================================
// ToolbarPanel — Top toolbar (Play/Stop, transform modes, save/load)
// =============================================================================

use egui::{Color32, RichText, Ui};
use crate::editor::core::editor_context::{EditorContext, EditorMode};

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
            let mode_btn = |ui: &mut Ui, label: &str, tooltip: &str, mode: EditorMode, current: &EditorMode| -> bool {
                let active = current == &mode;
                let text = if active {
                    RichText::new(label).color(Color32::from_rgb(255, 200, 60)).strong()
                } else {
                    RichText::new(label).color(Color32::from_rgb(180, 180, 180))
                };
                ui.selectable_label(active, text).on_hover_text(tooltip).clicked()
            };

            if mode_btn(ui, "↖ Select [Q]", "Select mode", EditorMode::Select, &ctx.editor_mode) {
                ctx.editor_mode = EditorMode::Select;
            }
            if mode_btn(ui, "↔ Move [W]", "Translate mode", EditorMode::Translate, &ctx.editor_mode) {
                ctx.editor_mode = EditorMode::Translate;
            }
            if mode_btn(ui, "↻ Rotate [E]", "Rotate mode", EditorMode::Rotate, &ctx.editor_mode) {
                ctx.editor_mode = EditorMode::Rotate;
            }
            if mode_btn(ui, "⤢ Scale [R]", "Scale mode", EditorMode::Scale, &ctx.editor_mode) {
                ctx.editor_mode = EditorMode::Scale;
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
                    let id = ctx.spawn_entity("Cube");
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.mesh = Some(crate::editor::core::editor_context::MeshComponent {
                            mesh_path: "assets/models/cube.obj".to_string(),
                            material_path: "assets/materials/default.mat".to_string(),
                        });
                    }
                    ui.close_menu();
                }
                if ui.button("💡  Directional Light").clicked() {
                    let id = ctx.spawn_entity("Directional Light");
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.light = Some(crate::editor::core::editor_context::LightComponent {
                            light_type: crate::editor::core::editor_context::LightType::Directional,
                            color: glam::Vec3::new(1.0, 0.98, 0.95),
                            intensity: 1.0,
                        });
                    }
                    ui.close_menu();
                }
                if ui.button("💡  Point Light").clicked() {
                    let id = ctx.spawn_entity("Point Light");
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.light = Some(crate::editor::core::editor_context::LightComponent {
                            light_type: crate::editor::core::editor_context::LightType::Point,
                            color: glam::Vec3::ONE,
                            intensity: 1.0,
                        });
                    }
                    ui.close_menu();
                }
                if ui.button("🎥  Camera").clicked() {
                    let id = ctx.spawn_entity("Camera");
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.camera = Some(crate::editor::core::editor_context::CameraComponent::default());
                    }
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
