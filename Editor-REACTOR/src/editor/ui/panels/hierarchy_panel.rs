// =============================================================================
// HierarchyPanel — Scene entity tree (like Unreal's Outliner)
// =============================================================================

use egui::{Color32, RichText, Ui, Vec2};
use crate::editor::core::editor_context::{EditorContext, EntityId};

pub struct HierarchyPanel {
    pub search_filter: String,
    rename_target: Option<EntityId>,
    rename_buffer: String,
}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {
            search_filter: String::new(),
            rename_target: None,
            rename_buffer: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &mut EditorContext) {
        // Header toolbar
        ui.horizontal(|ui| {
            ui.label(RichText::new("🌍 Hierarchy").strong().color(Color32::from_rgb(200, 200, 200)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("➕").on_hover_text("Spawn empty entity").clicked() {
                    ctx.spawn_entity("New Entity");
                }
            });
        });

        ui.separator();

        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_filter);
            if !self.search_filter.is_empty() {
                if ui.small_button("✖").clicked() {
                    self.search_filter.clear();
                }
            }
        });

        ui.add_space(2.0);

        // Scene name header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("📁 {}", ctx.scene.name))
                    .color(Color32::from_rgb(180, 180, 120))
                    .small()
            );
            ui.label(
                RichText::new(format!("({} entities)", ctx.scene.entity_count()))
                    .color(Color32::from_rgb(120, 120, 120))
                    .small()
            );
        });

        ui.separator();

        // Entity list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let root_ids: Vec<EntityId> = ctx.scene.root_entities().to_vec();

                for id in root_ids {
                    self.draw_entity_row(ui, ctx, id, 0);
                }

                // Empty area click to deselect
                let remaining = ui.available_rect_before_wrap();
                if remaining.height() > 0.0 {
                    let response = ui.allocate_rect(remaining, egui::Sense::click());
                    if response.clicked() {
                        ctx.select(None);
                    }
                }
            });

        // Context menu for selected entity
        if let Some(selected) = ctx.selected_entity {
            ui.separator();
            ui.horizontal(|ui| {
                if ui.small_button("🗑 Delete").clicked() {
                    ctx.delete_selected();
                }
                if ui.small_button("📋 Duplicate").clicked() {
                    if let Some(e) = ctx.scene.get(selected) {
                        let name = format!("{} (Copy)", e.name.clone());
                        ctx.spawn_entity(name);
                    }
                }
            });
        }
    }

    fn draw_entity_row(&mut self, ui: &mut Ui, ctx: &mut EditorContext, id: EntityId, depth: usize) {
        let entity = match ctx.scene.get(id) {
            Some(e) => e.clone(),
            None => return,
        };

        // Filter
        if !self.search_filter.is_empty() {
            if !entity.name.to_lowercase().contains(&self.search_filter.to_lowercase()) {
                return;
            }
        }

        let is_selected = ctx.selected_entity == Some(id);
        let indent = depth as f32 * 16.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

            // Visibility toggle
            let vis_icon = if entity.visible { "👁" } else { "🚫" };
            if ui.small_button(vis_icon).on_hover_text("Toggle visibility").clicked() {
                if let Some(e) = ctx.scene.get_mut(id) {
                    e.visible = !e.visible;
                }
            }

            // Entity type icon
            let icon = entity_icon(&entity);

            // Rename mode
            if self.rename_target == Some(id) {
                let response = ui.text_edit_singleline(&mut self.rename_buffer);
                if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(e) = ctx.scene.get_mut(id) {
                        if !self.rename_buffer.is_empty() {
                            e.name = self.rename_buffer.clone();
                        }
                    }
                    self.rename_target = None;
                    self.rename_buffer.clear();
                }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.rename_target = None;
                    self.rename_buffer.clear();
                }
            } else {
                // Selectable row
                let label_text = format!("{} {}", icon, entity.name);
                let label = if is_selected {
                    RichText::new(label_text)
                        .color(Color32::from_rgb(255, 200, 80))
                        .strong()
                } else {
                    RichText::new(label_text)
                        .color(Color32::from_rgb(200, 200, 200))
                };

                let response = ui.selectable_label(is_selected, label);

                if response.clicked() {
                    ctx.select(Some(id));
                }

                // Double-click to rename
                if response.double_clicked() {
                    self.rename_target = Some(id);
                    self.rename_buffer = entity.name.clone();
                }

                // Right-click context menu
                response.context_menu(|ui| {
                    if ui.button("✏ Rename").clicked() {
                        self.rename_target = Some(id);
                        self.rename_buffer = entity.name.clone();
                        ui.close_menu();
                    }
                    if ui.button("📋 Duplicate").clicked() {
                        let name = format!("{} (Copy)", entity.name);
                        ctx.spawn_entity(name);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🗑 Delete").clicked() {
                        ctx.scene.remove(id);
                        if ctx.selected_entity == Some(id) {
                            ctx.select(None);
                        }
                        ui.close_menu();
                    }
                });
            }
        });

        // Children (recursive)
        let children: Vec<EntityId> = entity.children.clone();
        for child_id in children {
            self.draw_entity_row(ui, ctx, child_id, depth + 1);
        }
    }
}

fn entity_icon(entity: &crate::editor::core::editor_context::EditorEntity) -> &'static str {
    if entity.camera.is_some() { return "🎥"; }
    if entity.light.is_some() { return "💡"; }
    if entity.mesh.is_some() { return "📦"; }
    "⬜"
}
