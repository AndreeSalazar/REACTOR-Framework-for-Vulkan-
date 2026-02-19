// =============================================================================
// AssetBrowserPanel — File/asset browser with drag & drop support
// =============================================================================

use crate::editor::core::editor_context::{AssetEntry, DragPayload, EditorContext};
use egui::{Color32, RichText, Ui, Vec2};

pub struct AssetBrowserPanel {
    pub search_filter: String,
    pub selected_asset: Option<usize>,
    pub view_mode: AssetViewMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssetViewMode {
    Grid,
    List,
}

impl AssetBrowserPanel {
    pub fn new() -> Self {
        Self {
            search_filter: String::new(),
            selected_asset: None,
            view_mode: AssetViewMode::Grid,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &mut EditorContext) {
        // Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("📂 Asset Browser")
                    .strong()
                    .color(Color32::from_rgb(200, 200, 200)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .selectable_label(self.view_mode == AssetViewMode::List, "☰")
                    .clicked()
                {
                    self.view_mode = AssetViewMode::List;
                }
                if ui
                    .selectable_label(self.view_mode == AssetViewMode::Grid, "⊞")
                    .clicked()
                {
                    self.view_mode = AssetViewMode::Grid;
                }
            });
        });

        ui.separator();

        // Search + path bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_filter);
            if !self.search_filter.is_empty() {
                if ui.small_button("✖").clicked() {
                    self.search_filter.clear();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("📁 {}", ctx.assets.current_folder))
                    .color(Color32::from_rgb(150, 150, 100))
                    .small(),
            );
        });

        ui.separator();

        // Asset list
        let filtered: Vec<(usize, AssetEntry)> = ctx
            .assets
            .assets
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                self.search_filter.is_empty()
                    || a.name
                        .to_lowercase()
                        .contains(&self.search_filter.to_lowercase())
            })
            .map(|(i, a)| (i, a.clone()))
            .collect();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.view_mode {
                AssetViewMode::List => self.show_list(ui, ctx, &filtered),
                AssetViewMode::Grid => self.show_grid(ui, ctx, &filtered),
            });

        // Drag payload info
        if let Some(payload) = &ctx.drag_payload {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Dragging: {}", payload.asset_name))
                        .color(Color32::from_rgb(255, 200, 60))
                        .small(),
                );
            });
        }
    }

    fn show_list(&mut self, ui: &mut Ui, ctx: &mut EditorContext, assets: &[(usize, AssetEntry)]) {
        for (idx, asset) in assets {
            let is_selected = self.selected_asset == Some(*idx);

            let label = format!("{} {}", asset.icon(), asset.name);
            let text = if is_selected {
                RichText::new(label)
                    .color(Color32::from_rgb(255, 200, 80))
                    .strong()
            } else {
                RichText::new(label).color(Color32::from_rgb(200, 200, 200))
            };

            let response = ui.selectable_label(is_selected, text);

            if response.clicked() {
                self.selected_asset = Some(*idx);
            }

            // Double-click to start drag
            if response.double_clicked() {
                ctx.drag_payload = Some(DragPayload {
                    asset_name: asset.name.clone(),
                    asset_type: asset.asset_type.clone(),
                });
                ctx.log_info(format!("Dragging asset: {}", asset.name));
            }

            // Hover tooltip
            response.on_hover_ui(|ui| {
                ui.label(RichText::new(format!("{} {}", asset.icon(), asset.name)).strong());
                ui.label(RichText::new(format!("Type: {}", asset.asset_type)).small());
                ui.label(
                    RichText::new(format!("Path: {}", asset.path))
                        .small()
                        .color(Color32::from_rgb(150, 150, 150)),
                );
                ui.label(
                    RichText::new("Double-click to drag into viewport")
                        .small()
                        .italics()
                        .color(Color32::from_rgb(120, 180, 120)),
                );
            });
        }
    }

    fn show_grid(&mut self, ui: &mut Ui, ctx: &mut EditorContext, assets: &[(usize, AssetEntry)]) {
        let tile_size = Vec2::new(72.0, 72.0);
        let available_width = ui.available_width();
        let cols = ((available_width / (tile_size.x + 8.0)) as usize).max(1);

        egui::Grid::new("asset_grid")
            .num_columns(cols)
            .spacing([6.0, 6.0])
            .show(ui, |ui| {
                for (col_idx, (idx, asset)) in assets.iter().enumerate() {
                    let is_selected = self.selected_asset == Some(*idx);

                    let bg_color = if is_selected {
                        Color32::from_rgba_premultiplied(80, 120, 200, 80)
                    } else {
                        Color32::from_rgba_premultiplied(50, 50, 55, 200)
                    };

                    let (rect, response) = ui.allocate_exact_size(tile_size, egui::Sense::click());
                    let painter = ui.painter_at(rect);

                    // Background
                    painter.rect_filled(rect, 4.0, bg_color);
                    if is_selected {
                        painter.rect_stroke(
                            rect,
                            4.0,
                            egui::Stroke::new(1.5, Color32::from_rgb(100, 160, 255)),
                        );
                    }

                    // Icon (large)
                    painter.text(
                        rect.center() - Vec2::new(0.0, 8.0),
                        egui::Align2::CENTER_CENTER,
                        asset.icon(),
                        egui::FontId::proportional(28.0),
                        Color32::WHITE,
                    );

                    // Name (small, truncated)
                    let name = if asset.name.len() > 10 {
                        format!("{}…", &asset.name[..9])
                    } else {
                        asset.name.clone()
                    };
                    painter.text(
                        egui::Pos2::new(rect.center().x, rect.max.y - 10.0),
                        egui::Align2::CENTER_CENTER,
                        name,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(200, 200, 200),
                    );

                    if response.clicked() {
                        self.selected_asset = Some(*idx);
                    }

                    if response.double_clicked() {
                        ctx.drag_payload = Some(DragPayload {
                            asset_name: asset.name.clone(),
                            asset_type: asset.asset_type.clone(),
                        });
                        ctx.log_info(format!("Dragging asset: {}", asset.name));
                    }

                    response.on_hover_ui(|ui| {
                        ui.label(
                            RichText::new(format!("{} {}", asset.icon(), asset.name)).strong(),
                        );
                        ui.label(RichText::new(format!("Type: {}", asset.asset_type)).small());
                        ui.label(
                            RichText::new(format!("Path: {}", asset.path))
                                .small()
                                .color(Color32::from_rgb(150, 150, 150)),
                        );
                        ui.label(
                            RichText::new("Double-click to drag into viewport")
                                .small()
                                .italics()
                                .color(Color32::from_rgb(120, 180, 120)),
                        );
                    });

                    if (col_idx + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
    }
}
