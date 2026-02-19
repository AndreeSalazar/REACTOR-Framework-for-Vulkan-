// =============================================================================
// ToolbarPanel — Top toolbar (Play/Stop, transform modes, save/load)
// =============================================================================

use crate::editor::core::editor_context::{EditorContext, GizmoMode};
use egui::{Color32, RichText, Ui};

pub struct ToolbarPanel;

fn ue_section_label(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .color(Color32::from_rgb(130, 136, 148))
            .small()
            .strong(),
    );
}

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
                    .color(Color32::from_rgb(255, 156, 76))
                    .size(16.0),
            );
            ui.label(
                RichText::new("Editor Pro")
                    .color(Color32::from_rgb(188, 188, 196))
                    .size(13.0),
            );

            ui.separator();
            ue_section_label(ui, "FILE");

            // ── File operations ───────────────────────────────────────────
            if ui.button("📁 Open").on_hover_text("Open scene").clicked() {
                ctx.log_info("Open scene (not yet implemented)");
            }
            if ui.button("💾 Save").on_hover_text("Save scene").clicked() {
                ctx.log_info(format!("Saved scene: {}", ctx.scene.name));
            }

            ui.separator();
            ue_section_label(ui, "TRANSFORM");

            // ── Transform mode buttons ────────────────────────────────────
            let mode_btn = |ui: &mut Ui,
                            label: &str,
                            tooltip: &str,
                            mode: GizmoMode,
                            current: &GizmoMode|
             -> bool {
                let active = current == &mode;
                let text = if active {
                    RichText::new(label)
                        .color(Color32::from_rgb(255, 200, 60))
                        .strong()
                } else {
                    RichText::new(label).color(Color32::from_rgb(180, 180, 180))
                };
                ui.selectable_label(active, text)
                    .on_hover_text(tooltip)
                    .clicked()
            };

            if mode_btn(
                ui,
                "↖ Select [Q]",
                "Select mode",
                GizmoMode::Select,
                &ctx.gizmo_mode,
            ) {
                ctx.gizmo_mode = GizmoMode::Select;
            }
            if mode_btn(
                ui,
                "↔ Move [W]",
                "Translate mode",
                GizmoMode::Translate,
                &ctx.gizmo_mode,
            ) {
                ctx.gizmo_mode = GizmoMode::Translate;
            }
            if mode_btn(
                ui,
                "↻ Rotate [E]",
                "Rotate mode",
                GizmoMode::Rotate,
                &ctx.gizmo_mode,
            ) {
                ctx.gizmo_mode = GizmoMode::Rotate;
            }
            if mode_btn(
                ui,
                "⤢ Scale [R]",
                "Scale mode",
                GizmoMode::Scale,
                &ctx.gizmo_mode,
            ) {
                ctx.gizmo_mode = GizmoMode::Scale;
            }

            ui.separator();
            ue_section_label(ui, "CREATE");

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
            ue_section_label(ui, "VIEWPORT");

            ui.menu_button("⚙ Viewport", |ui| {
                ui.set_min_width(270.0);
                ui.label(
                    RichText::new("Gizmo Response")
                        .strong()
                        .color(Color32::from_rgb(210, 210, 220)),
                );
                ui.add(
                    egui::Slider::new(&mut ctx.gizmo_translate_sensitivity, 0.10..=1.50)
                        .text("Move speed"),
                );
                ui.add(
                    egui::Slider::new(&mut ctx.gizmo_rotate_sensitivity, 0.05..=1.20)
                        .text("Rotate speed"),
                );
                ui.add(
                    egui::Slider::new(&mut ctx.gizmo_scale_sensitivity, 0.10..=1.50)
                        .text("Scale speed"),
                );

                ui.separator();
                ui.label(
                    RichText::new("Camera Navigation")
                        .strong()
                        .color(Color32::from_rgb(210, 210, 220)),
                );
                ui.add(egui::Slider::new(&mut ctx.camera.orbit_speed, 0.0015..=0.02).text("Orbit"));
                ui.add(egui::Slider::new(&mut ctx.camera.pan_speed, 0.002..=0.03).text("Pan"));
                ui.add(egui::Slider::new(&mut ctx.camera.zoom_speed, 0.03..=0.35).text("Zoom"));

                ui.separator();
                ui.label(
                    RichText::new("Snapping")
                        .strong()
                        .color(Color32::from_rgb(210, 210, 220)),
                );
                ui.add(egui::Slider::new(&mut ctx.snap_translate, 0.0..=2.0).text("Move snap (m)"));
                ui.add(egui::Slider::new(&mut ctx.snap_rotate, 0.0..=90.0).text("Rotate snap (°)"));
                ui.add(egui::Slider::new(&mut ctx.snap_scale, 0.0..=1.0).text("Scale snap"));
            });

            ui.separator();
            ue_section_label(ui, "SIMULATE");

            // ── Play / Stop ───────────────────────────────────────────────
            if ctx.play_mode {
                if ui
                    .button(
                        RichText::new("⏹ Stop")
                            .color(Color32::from_rgb(255, 80, 80))
                            .strong()
                            .size(14.0),
                    )
                    .on_hover_text("Stop play mode")
                    .clicked()
                {
                    ctx.play_mode = false;
                    ctx.log_info("Play mode stopped.");
                }
            } else {
                if ui
                    .button(
                        RichText::new("▶ Play")
                            .color(Color32::from_rgb(80, 220, 80))
                            .strong()
                            .size(14.0),
                    )
                    .on_hover_text("Enter play mode")
                    .clicked()
                {
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
                    .small(),
                );
            });
        });
    }
}
