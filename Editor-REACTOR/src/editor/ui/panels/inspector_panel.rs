// =============================================================================
// InspectorPanel — Component inspector (like Unreal's Details panel)
// =============================================================================

use egui::{Color32, RichText, Ui};
use crate::editor::core::editor_context::{EditorContext, LightType};

pub struct InspectorPanel;

impl InspectorPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &mut EditorContext) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍 Inspector").strong().color(Color32::from_rgb(200, 200, 200)));
        });
        ui.separator();

        let selected_id = match ctx.selected {
            Some(id) => id,
            None => {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("No entity selected")
                        .color(Color32::from_rgb(120, 120, 120))
                        .italics()
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Click an entity in the Hierarchy or Viewport to inspect it.")
                        .color(Color32::from_rgb(100, 100, 100))
                        .small()
                );
                return;
            }
        };

        let entity = match ctx.scene.get(selected_id) {
            Some(e) => e.clone(),
            None => return,
        };

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // ── Entity Header ──────────────────────────────────────────
                ui.horizontal(|ui| {
                    let icon = entity.icon();
                    ui.label(RichText::new(icon).size(20.0));
                    ui.vertical(|ui| {
                        // Editable name
                        let mut name = entity.name.clone();
                        if ui.text_edit_singleline(&mut name).changed() {
                            if let Some(e) = ctx.scene.get_mut(selected_id) {
                                e.name = name;
                            }
                        }
                        ui.label(
                            RichText::new(format!("ID: {}", selected_id.0))
                                .color(Color32::from_rgb(100, 100, 100))
                                .small()
                        );
                    });
                });

                // Visible / Locked toggles
                ui.horizontal(|ui| {
                    let mut visible = entity.visible;
                    if ui.checkbox(&mut visible, "Visible").changed() {
                        if let Some(e) = ctx.scene.get_mut(selected_id) {
                            e.visible = visible;
                        }
                    }
                    let mut locked = entity.locked;
                    if ui.checkbox(&mut locked, "Locked").changed() {
                        if let Some(e) = ctx.scene.get_mut(selected_id) {
                            e.locked = locked;
                        }
                    }
                });

                ui.add_space(4.0);
                ui.separator();

                // ── Transform Component ────────────────────────────────────
                self.show_transform_component(ui, ctx, selected_id, &entity);

                // ── Mesh Component ─────────────────────────────────────────
                if entity.mesh.is_some() {
                    self.show_mesh_component(ui, ctx, selected_id, &entity);
                }

                // ── Light Component ────────────────────────────────────────
                if entity.light.is_some() {
                    self.show_light_component(ui, ctx, selected_id, &entity);
                }

                // ── Camera Component ───────────────────────────────────────
                if entity.camera.is_some() {
                    self.show_camera_component(ui, ctx, selected_id, &entity);
                }

                ui.add_space(8.0);
                ui.separator();

                // ── Add Component button ───────────────────────────────────
                ui.add_space(4.0);
                ui.centered_and_justified(|ui| {
                    ui.menu_button("➕  Add Component", |ui| {
                        if ui.button("📦  Mesh Renderer").clicked() {
                            if let Some(e) = ctx.scene.get_mut(selected_id) {
                                if e.mesh.is_none() {
                                    e.mesh = Some(crate::editor::core::editor_context::MeshComponent {
                                        mesh_path: "primitives://cube".to_string(),
                                        material_path: "default".to_string(),
                                        primitive: crate::editor::core::editor_context::MeshPrimitive::Cube,
                                    });
                                }
                            }
                            ui.close_menu();
                        }
                        if ui.button("💡  Light").clicked() {
                            if let Some(e) = ctx.scene.get_mut(selected_id) {
                                if e.light.is_none() {
                                    e.light = Some(crate::editor::core::editor_context::LightComponent {
                                        light_type: LightType::Point,
                                        color: glam::Vec3::ONE,
                                        intensity: 1.0,
                                        range: 10.0,
                                        spot_angle: 45.0,
                                    });
                                }
                            }
                            ui.close_menu();
                        }
                        if ui.button("🎥  Camera").clicked() {
                            if let Some(e) = ctx.scene.get_mut(selected_id) {
                                if e.camera.is_none() {
                                    e.camera = Some(crate::editor::core::editor_context::CameraComponent::default());
                                }
                            }
                            ui.close_menu();
                        }
                    });
                });
            });
    }

    fn show_transform_component(
        &self, ui: &mut Ui, ctx: &mut EditorContext,
        id: crate::editor::core::editor_context::EntityId,
        entity: &crate::editor::core::editor_context::EditorEntity,
    ) {
        egui::CollapsingHeader::new(RichText::new("⚙  Transform").strong())
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("transform_grid")
                    .num_columns(4)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        // Position
                        ui.label(RichText::new("Position").color(Color32::from_rgb(180, 180, 180)));
                        let mut pos = entity.transform.position;
                        let changed_x = ui.add(egui::DragValue::new(&mut pos.x).speed(0.01).prefix("X ")).changed();
                        let changed_y = ui.add(egui::DragValue::new(&mut pos.y).speed(0.01).prefix("Y ")).changed();
                        let changed_z = ui.add(egui::DragValue::new(&mut pos.z).speed(0.01).prefix("Z ")).changed();
                        if changed_x || changed_y || changed_z {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                e.transform.position = pos;
                            }
                        }
                        ui.end_row();

                        // Rotation (Euler degrees)
                        ui.label(RichText::new("Rotation").color(Color32::from_rgb(180, 180, 180)));
                        let (yaw, pitch, roll) = entity.transform.rotation.to_euler(glam::EulerRot::YXZ);
                        let mut yaw_deg = yaw.to_degrees();
                        let mut pitch_deg = pitch.to_degrees();
                        let mut roll_deg = roll.to_degrees();
                        let cy = ui.add(egui::DragValue::new(&mut yaw_deg).speed(0.5).suffix("°").prefix("Y ")).changed();
                        let cp = ui.add(egui::DragValue::new(&mut pitch_deg).speed(0.5).suffix("°").prefix("P ")).changed();
                        let cr = ui.add(egui::DragValue::new(&mut roll_deg).speed(0.5).suffix("°").prefix("R ")).changed();
                        if cy || cp || cr {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                e.transform.rotation = glam::Quat::from_euler(
                                    glam::EulerRot::YXZ,
                                    yaw_deg.to_radians(),
                                    pitch_deg.to_radians(),
                                    roll_deg.to_radians(),
                                );
                            }
                        }
                        ui.end_row();

                        // Scale
                        ui.label(RichText::new("Scale").color(Color32::from_rgb(180, 180, 180)));
                        let mut scale = entity.transform.scale;
                        let sx = ui.add(egui::DragValue::new(&mut scale.x).speed(0.01).prefix("X ")).changed();
                        let sy = ui.add(egui::DragValue::new(&mut scale.y).speed(0.01).prefix("Y ")).changed();
                        let sz = ui.add(egui::DragValue::new(&mut scale.z).speed(0.01).prefix("Z ")).changed();
                        if sx || sy || sz {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                e.transform.scale = scale;
                            }
                        }
                        ui.end_row();
                    });

                // Reset button
                ui.horizontal(|ui| {
                    if ui.small_button("↺ Reset Position").clicked() {
                        if let Some(e) = ctx.scene.get_mut(id) {
                            e.transform.position = glam::Vec3::ZERO;
                        }
                    }
                    if ui.small_button("↺ Reset Scale").clicked() {
                        if let Some(e) = ctx.scene.get_mut(id) {
                            e.transform.scale = glam::Vec3::ONE;
                        }
                    }
                });
            });
    }

    fn show_mesh_component(
        &self, ui: &mut Ui, ctx: &mut EditorContext,
        id: crate::editor::core::editor_context::EntityId,
        entity: &crate::editor::core::editor_context::EditorEntity,
    ) {
        let mesh = match &entity.mesh { Some(m) => m.clone(), None => return };

        egui::CollapsingHeader::new(RichText::new("📦  Mesh Renderer").strong())
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("mesh_grid")
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Mesh").color(Color32::from_rgb(180, 180, 180)));
                        let mut mesh_path = mesh.mesh_path.clone();
                        if ui.text_edit_singleline(&mut mesh_path).changed() {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                if let Some(m) = &mut e.mesh {
                                    m.mesh_path = mesh_path;
                                }
                            }
                        }
                        ui.end_row();

                        ui.label(RichText::new("Material").color(Color32::from_rgb(180, 180, 180)));
                        let mut mat_path = mesh.material_path.clone();
                        if ui.text_edit_singleline(&mut mat_path).changed() {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                if let Some(m) = &mut e.mesh {
                                    m.material_path = mat_path;
                                }
                            }
                        }
                        ui.end_row();
                    });

                if ui.small_button("🗑 Remove Component").clicked() {
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.mesh = None;
                    }
                }
            });
    }

    fn show_light_component(
        &self, ui: &mut Ui, ctx: &mut EditorContext,
        id: crate::editor::core::editor_context::EntityId,
        entity: &crate::editor::core::editor_context::EditorEntity,
    ) {
        let light = match &entity.light { Some(l) => l.clone(), None => return };

        egui::CollapsingHeader::new(RichText::new("💡  Light").strong())
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("light_grid")
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Type").color(Color32::from_rgb(180, 180, 180)));
                        ui.label(format!("{}", light.light_type));
                        ui.end_row();

                        ui.label(RichText::new("Color").color(Color32::from_rgb(180, 180, 180)));
                        let mut color = [light.color.x, light.color.y, light.color.z];
                        if ui.color_edit_button_rgb(&mut color).changed() {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                if let Some(l) = &mut e.light {
                                    l.color = glam::Vec3::new(color[0], color[1], color[2]);
                                }
                            }
                        }
                        ui.end_row();

                        ui.label(RichText::new("Intensity").color(Color32::from_rgb(180, 180, 180)));
                        let mut intensity = light.intensity;
                        if ui.add(egui::DragValue::new(&mut intensity).speed(0.01).range(0.0..=100.0_f32)).changed() {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                if let Some(l) = &mut e.light {
                                    l.intensity = intensity;
                                }
                            }
                        }
                        ui.end_row();
                    });

                if ui.small_button("🗑 Remove Component").clicked() {
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.light = None;
                    }
                }
            });
    }

    fn show_camera_component(
        &self, ui: &mut Ui, ctx: &mut EditorContext,
        id: crate::editor::core::editor_context::EntityId,
        entity: &crate::editor::core::editor_context::EditorEntity,
    ) {
        let camera = match &entity.camera { Some(c) => c.clone(), None => return };

        egui::CollapsingHeader::new(RichText::new("🎥  Camera").strong())
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("camera_grid")
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("FOV").color(Color32::from_rgb(180, 180, 180)));
                        let mut fov = camera.fov;
                        if ui.add(egui::DragValue::new(&mut fov).speed(0.5).suffix("°").range(10.0..=170.0_f32)).changed() {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                if let Some(c) = &mut e.camera {
                                    c.fov = fov;
                                }
                            }
                        }
                        ui.end_row();

                        ui.label(RichText::new("Near").color(Color32::from_rgb(180, 180, 180)));
                        let mut near = camera.near;
                        if ui.add(egui::DragValue::new(&mut near).speed(0.001).range(0.001..=10.0_f32)).changed() {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                if let Some(c) = &mut e.camera { c.near = near; }
                            }
                        }
                        ui.end_row();

                        ui.label(RichText::new("Far").color(Color32::from_rgb(180, 180, 180)));
                        let mut far = camera.far;
                        if ui.add(egui::DragValue::new(&mut far).speed(1.0).range(1.0..=100000.0_f32)).changed() {
                            if let Some(e) = ctx.scene.get_mut(id) {
                                if let Some(c) = &mut e.camera { c.far = far; }
                            }
                        }
                        ui.end_row();
                    });

                if ui.small_button("🗑 Remove Component").clicked() {
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.camera = None;
                    }
                }
            });
    }
}
