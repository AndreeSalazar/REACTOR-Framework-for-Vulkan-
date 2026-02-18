// =============================================================================
// ViewportPanel — Vulkan render output panel
// =============================================================================
// This is the most important panel. It displays the Vulkan-rendered scene.
// Architecture: Vulkan renders to texture → egui displays that texture.
// For now it renders a placeholder with grid and gizmo overlay.
// =============================================================================

use egui::{Color32, Painter, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use crate::editor::core::editor_context::{EditorContext, EditorMode};

pub struct ViewportPanel {
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_distance: f32,
    pub camera_target: egui::Vec2,
    pub is_focused: bool,
    last_drag_pos: Option<Pos2>,
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {
            camera_yaw: 0.3,
            camera_pitch: 0.4,
            camera_distance: 8.0,
            camera_target: Vec2::ZERO,
            is_focused: false,
            last_drag_pos: None,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &mut EditorContext) {
        let available = ui.available_rect_before_wrap();

        // Viewport background (dark, like a real 3D viewport)
        ui.painter().rect_filled(available, 0.0, Color32::from_rgb(30, 30, 35));

        // Allocate the full viewport area as interactive
        let response = ui.allocate_rect(available, Sense::click_and_drag());
        self.is_focused = response.has_focus() || response.hovered();

        let painter = ui.painter_at(available);

        // Draw grid
        self.draw_grid(&painter, available);

        // Draw scene objects (placeholder representations)
        self.draw_scene_objects(&painter, available, ctx);

        // Draw gizmo overlay (top-right corner)
        self.draw_orientation_gizmo(&painter, available);

        // Draw viewport info overlay (top-left)
        self.draw_info_overlay(ui, available, ctx);

        // Draw mode indicator (bottom-left)
        self.draw_mode_indicator(&painter, available, ctx);

        // Handle mouse orbit
        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = response.drag_delta();
            self.camera_yaw += delta.x * 0.005;
            self.camera_pitch += delta.y * 0.005;
            self.camera_pitch = self.camera_pitch.clamp(-1.4, 1.4);
        }

        // Scroll to zoom
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if response.hovered() && scroll != 0.0 {
            self.camera_distance -= scroll * 0.1;
            self.camera_distance = self.camera_distance.clamp(0.5, 100.0);
        }

        // Middle mouse pan
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = response.drag_delta();
            self.camera_target += delta * 0.01;
        }

        // Play mode overlay
        if ctx.play_mode {
            self.draw_play_mode_overlay(&painter, available);
        }
    }

    fn draw_grid(&self, painter: &Painter, rect: Rect) {
        let center = rect.center();
        let grid_color = Color32::from_rgba_premultiplied(60, 60, 65, 200);
        let axis_color_x = Color32::from_rgba_premultiplied(180, 60, 60, 220);
        let axis_color_z = Color32::from_rgba_premultiplied(60, 60, 180, 220);

        let grid_size = 400.0_f32;
        let cell_size = 30.0_f32;
        let half = grid_size / 2.0;
        let steps = (grid_size / cell_size) as i32;

        for i in -steps..=steps {
            let t = i as f32 * cell_size;
            let is_axis = i == 0;
            let color = if is_axis { Color32::from_rgba_premultiplied(80, 80, 85, 255) } else { grid_color };
            let stroke = Stroke::new(if is_axis { 1.5 } else { 0.5 }, color);

            painter.line_segment(
                [Pos2::new(center.x - half, center.y + t), Pos2::new(center.x + half, center.y + t)],
                stroke,
            );
            painter.line_segment(
                [Pos2::new(center.x + t, center.y - half), Pos2::new(center.x + t, center.y + half)],
                stroke,
            );
        }

        // X axis (red)
        painter.line_segment(
            [Pos2::new(center.x, center.y), Pos2::new(center.x + 60.0, center.y)],
            Stroke::new(2.0, axis_color_x),
        );
        // Z axis (blue)
        painter.line_segment(
            [Pos2::new(center.x, center.y), Pos2::new(center.x, center.y - 60.0)],
            Stroke::new(2.0, axis_color_z),
        );
    }

    fn draw_scene_objects(&self, painter: &Painter, rect: Rect, ctx: &EditorContext) {
        let center = rect.center();

        for entity in ctx.scene.all_entities() {
            if !entity.visible { continue; }

            let pos = entity.transform.position;

            // Simple 3D → 2D projection (orthographic approximation)
            let screen_x = center.x + pos.x * 30.0 - pos.z * 15.0;
            let screen_y = center.y - pos.y * 30.0 + pos.z * 8.0;
            let screen_pos = Pos2::new(screen_x, screen_y);

            if !rect.contains(screen_pos) { continue; }

            let is_selected = ctx.selected_entity == Some(entity.id);

            if entity.mesh.is_some() {
                let scale = (entity.transform.scale.x * 15.0).max(8.0);
                let color = if is_selected {
                    Color32::from_rgb(255, 165, 0)
                } else {
                    Color32::from_rgb(100, 160, 220)
                };

                // Draw cube outline
                let half = scale;
                let corners = [
                    Pos2::new(screen_pos.x - half, screen_pos.y - half),
                    Pos2::new(screen_pos.x + half, screen_pos.y - half),
                    Pos2::new(screen_pos.x + half, screen_pos.y + half),
                    Pos2::new(screen_pos.x - half, screen_pos.y + half),
                ];
                painter.rect_stroke(
                    Rect::from_min_max(corners[0], corners[2]),
                    2.0,
                    Stroke::new(if is_selected { 2.5 } else { 1.5 }, color),
                );

                // Fill with semi-transparent color
                let fill = if is_selected {
                    Color32::from_rgba_premultiplied(255, 165, 0, 30)
                } else {
                    Color32::from_rgba_premultiplied(100, 160, 220, 20)
                };
                painter.rect_filled(Rect::from_min_max(corners[0], corners[2]), 2.0, fill);

            } else if entity.light.is_some() {
                let color = Color32::from_rgb(255, 240, 100);
                painter.circle_filled(screen_pos, 6.0, color);
                painter.circle_stroke(screen_pos, 10.0, Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 240, 100, 120)));

            } else if entity.camera.is_some() {
                let color = Color32::from_rgb(100, 220, 100);
                painter.circle_filled(screen_pos, 5.0, color);
                // Camera frustum lines
                painter.line_segment([screen_pos, Pos2::new(screen_pos.x - 12.0, screen_pos.y - 10.0)], Stroke::new(1.0, color));
                painter.line_segment([screen_pos, Pos2::new(screen_pos.x + 12.0, screen_pos.y - 10.0)], Stroke::new(1.0, color));
            }

            // Entity name label
            if is_selected {
                painter.text(
                    Pos2::new(screen_pos.x + 12.0, screen_pos.y - 8.0),
                    egui::Align2::LEFT_CENTER,
                    &entity.name,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(255, 220, 100),
                );
            }
        }
    }

    fn draw_orientation_gizmo(&self, painter: &Painter, rect: Rect) {
        let origin = Pos2::new(rect.max.x - 55.0, rect.min.y + 55.0);
        let len = 30.0_f32;

        let yaw = self.camera_yaw;
        let pitch = self.camera_pitch;

        // X axis (red)
        let x_end = Pos2::new(
            origin.x + yaw.cos() * len,
            origin.y + pitch.sin() * yaw.sin() * len * 0.5,
        );
        painter.line_segment([origin, x_end], Stroke::new(2.5, Color32::from_rgb(220, 60, 60)));
        painter.text(x_end, egui::Align2::CENTER_CENTER, "X", egui::FontId::proportional(11.0), Color32::from_rgb(220, 60, 60));

        // Y axis (green)
        let y_end = Pos2::new(origin.x, origin.y - pitch.cos() * len);
        painter.line_segment([origin, y_end], Stroke::new(2.5, Color32::from_rgb(60, 200, 60)));
        painter.text(y_end, egui::Align2::CENTER_CENTER, "Y", egui::FontId::proportional(11.0), Color32::from_rgb(60, 200, 60));

        // Z axis (blue)
        let z_end = Pos2::new(
            origin.x - yaw.sin() * len,
            origin.y + pitch.sin() * yaw.cos() * len * 0.5,
        );
        painter.line_segment([origin, z_end], Stroke::new(2.5, Color32::from_rgb(60, 60, 220)));
        painter.text(z_end, egui::Align2::CENTER_CENTER, "Z", egui::FontId::proportional(11.0), Color32::from_rgb(60, 60, 220));

        // Center dot
        painter.circle_filled(origin, 3.0, Color32::WHITE);
    }

    fn draw_info_overlay(&self, ui: &mut Ui, rect: Rect, ctx: &EditorContext) {
        let pos = Pos2::new(rect.min.x + 8.0, rect.min.y + 8.0);
        let painter = ui.painter();

        let bg = Color32::from_rgba_premultiplied(0, 0, 0, 140);
        let text_color = Color32::from_rgb(200, 200, 200);

        let info = format!(
            "FPS: {:.0}  |  {:.1}ms  |  Entities: {}",
            ctx.stats.fps, ctx.stats.frame_time_ms, ctx.stats.entity_count
        );

        let galley = ui.fonts(|f| f.layout_no_wrap(info.clone(), egui::FontId::monospace(11.0), text_color));
        let text_rect = Rect::from_min_size(pos, galley.size() + Vec2::splat(6.0));
        painter.rect_filled(text_rect, 3.0, bg);
        painter.galley(pos + Vec2::splat(3.0), galley, text_color);

        // Camera info
        let cam_info = format!(
            "Cam  Yaw: {:.1}°  Pitch: {:.1}°  Dist: {:.1}",
            self.camera_yaw.to_degrees(),
            self.camera_pitch.to_degrees(),
            self.camera_distance
        );
        let cam_pos = Pos2::new(rect.min.x + 8.0, rect.min.y + 30.0);
        let galley2 = ui.fonts(|f| f.layout_no_wrap(cam_info, egui::FontId::monospace(10.0), Color32::from_rgb(160, 160, 160)));
        let text_rect2 = Rect::from_min_size(cam_pos, galley2.size() + Vec2::splat(4.0));
        painter.rect_filled(text_rect2, 3.0, bg);
        painter.galley(cam_pos + Vec2::splat(2.0), galley2, Color32::from_rgb(160, 160, 160));
    }

    fn draw_mode_indicator(&self, painter: &Painter, rect: Rect, ctx: &EditorContext) {
        let mode_str = match ctx.editor_mode {
            EditorMode::Select    => "SELECT  [Q]",
            EditorMode::Translate => "TRANSLATE  [W]",
            EditorMode::Rotate    => "ROTATE  [E]",
            EditorMode::Scale     => "SCALE  [R]",
        };
        let color = match ctx.editor_mode {
            EditorMode::Select    => Color32::from_rgb(180, 180, 180),
            EditorMode::Translate => Color32::from_rgb(100, 200, 100),
            EditorMode::Rotate    => Color32::from_rgb(100, 150, 255),
            EditorMode::Scale     => Color32::from_rgb(255, 180, 60),
        };

        painter.text(
            Pos2::new(rect.min.x + 10.0, rect.max.y - 12.0),
            egui::Align2::LEFT_BOTTOM,
            mode_str,
            egui::FontId::monospace(12.0),
            color,
        );
    }

    fn draw_play_mode_overlay(&self, painter: &Painter, rect: Rect) {
        // Orange border when in play mode
        painter.rect_stroke(rect, 0.0, Stroke::new(3.0, Color32::from_rgb(255, 140, 0)));

        painter.text(
            Pos2::new(rect.center().x, rect.min.y + 16.0),
            egui::Align2::CENTER_CENTER,
            "▶  PLAY MODE",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(255, 180, 60),
        );
    }
}
