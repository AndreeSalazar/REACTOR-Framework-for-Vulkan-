// =============================================================================
// ViewportPanel — Professional 3D Viewport (UE5/Blender style)
// =============================================================================
// Features:
//   - Functional transform gizmos (translate/rotate/scale in real-time)
//   - UE5-style top bar with view modes and settings
//   - Bottom bar with grid/snap settings
//   - Orbit / Pan / Zoom camera controls
//   - Infinite ground grid with fade
//   - Solid shaded + wireframe objects
//   - Light visualization (radius, direction)
//   - Selection highlights and bounding boxes
//   - Orientation gizmo (top-right corner)
//   - Click-selection with multi-select
// =============================================================================

use egui::{Color32, Painter, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use glam::{Vec3, Vec4, Mat4};
use crate::editor::core::editor_context::{EditorContext, EntityId, GizmoMode, MeshPrimitive};

// ─── Cube geometry ───────────────────────────────────────────────────────────
const CUBE_VERTS: [Vec3; 8] = [
    Vec3::new(-0.5, -0.5, -0.5), Vec3::new( 0.5, -0.5, -0.5),
    Vec3::new( 0.5,  0.5, -0.5), Vec3::new(-0.5,  0.5, -0.5),
    Vec3::new(-0.5, -0.5,  0.5), Vec3::new( 0.5, -0.5,  0.5),
    Vec3::new( 0.5,  0.5,  0.5), Vec3::new(-0.5,  0.5,  0.5),
];
const CUBE_EDGES: [(usize, usize); 12] = [
    (0,1),(1,2),(2,3),(3,0), (4,5),(5,6),(6,7),(7,4), (0,4),(1,5),(2,6),(3,7),
];
const CUBE_FACES: [(usize, usize, usize, Vec3); 12] = [
    (0,1,2, Vec3::new(0.0, 0.0,-1.0)), (0,2,3, Vec3::new(0.0, 0.0,-1.0)),
    (5,4,7, Vec3::new(0.0, 0.0, 1.0)), (5,7,6, Vec3::new(0.0, 0.0, 1.0)),
    (3,2,6, Vec3::new(0.0, 1.0, 0.0)), (3,6,7, Vec3::new(0.0, 1.0, 0.0)),
    (4,5,1, Vec3::new(0.0,-1.0, 0.0)), (4,1,0, Vec3::new(0.0,-1.0, 0.0)),
    (1,5,6, Vec3::new(1.0, 0.0, 0.0)), (1,6,2, Vec3::new(1.0, 0.0, 0.0)),
    (4,0,3, Vec3::new(-1.0,0.0, 0.0)), (4,3,7, Vec3::new(-1.0,0.0, 0.0)),
];

// ─── Gizmo interaction state ─────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoAxis {
    None,
    X, Y, Z,
    XY, XZ, YZ,
    All,
}

pub struct ViewportPanel {
    pub is_focused: bool,
    // Gizmo interaction
    pub active_axis: GizmoAxis,
    pub drag_start_pos: Option<glam::Vec2>,
    pub drag_start_transform: Option<glam::Vec3>,
    pub drag_start_scale: Option<glam::Vec3>,
    pub drag_start_rotation: Option<glam::Quat>,
    // View settings
    pub view_mode: ViewMode,
    pub shading_mode: ShadingMode,
    pub show_stats: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Perspective,
    Top,
    Front,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadingMode {
    Solid,
    Wireframe,
    SolidWireframe,
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {
            is_focused: false,
            active_axis: GizmoAxis::None,
            drag_start_pos: None,
            drag_start_transform: None,
            drag_start_scale: None,
            drag_start_rotation: None,
            view_mode: ViewMode::Perspective,
            shading_mode: ShadingMode::SolidWireframe,
            show_stats: true,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &mut EditorContext) {
        let full_rect = ui.available_rect_before_wrap();
        
        // Reserve space for top and bottom bars
        let top_bar_height = 26.0;
        let bottom_bar_height = 22.0;
        
        let viewport_rect = Rect::from_min_max(
            Pos2::new(full_rect.min.x, full_rect.min.y + top_bar_height),
            Pos2::new(full_rect.max.x, full_rect.max.y - bottom_bar_height),
        );
        let vp_size = glam::Vec2::new(viewport_rect.width(), viewport_rect.height());

        // ── Top bar (UE5 style) ──────────────────────────────────────────
        self.draw_top_bar(ui, full_rect, ctx);

        // ── Viewport background ──────────────────────────────────────────
        let bg = Color32::from_rgb(32, 32, 38);
        ui.painter().rect_filled(viewport_rect, 0.0, bg);

        // Allocate interactive area for viewport
        let response = ui.allocate_rect(viewport_rect, Sense::click_and_drag());
        self.is_focused = response.hovered();

        let painter = ui.painter_at(viewport_rect);

        // ── Draw layers ──────────────────────────────────────────────────
        if ctx.show_grid {
            self.draw_ground_grid(&painter, viewport_rect, &ctx.camera, vp_size);
        }
        self.draw_scene_entities(&painter, viewport_rect, ctx, vp_size);
        self.draw_transform_gizmo_interactive(&painter, viewport_rect, ctx, vp_size);
        self.draw_orientation_gizmo(&painter, viewport_rect, &ctx.camera);
        
        if self.show_stats {
            self.draw_stats_overlay(&painter, viewport_rect, ctx);
        }

        if ctx.play_mode {
            self.draw_play_overlay(&painter, viewport_rect);
        }

        // ── Bottom bar ───────────────────────────────────────────────────
        self.draw_bottom_bar(ui, full_rect, ctx);

        // ── Input handling ───────────────────────────────────────────────
        self.handle_gizmo_interaction(ui, &response, viewport_rect, ctx, vp_size);
        self.handle_camera_input(ui, &response, ctx);
        self.handle_click_selection(ui, &response, viewport_rect, ctx, vp_size);
    }

    // =====================================================================
    // Top bar — UE5 style with view modes, shading, etc.
    // =====================================================================
    fn draw_top_bar(&mut self, ui: &mut Ui, full_rect: Rect, ctx: &mut EditorContext) {
        let bar_rect = Rect::from_min_size(full_rect.min, Vec2::new(full_rect.width(), 26.0));
        
        // Pre-calculate all rects
        let mut x = bar_rect.min.x + 8.0;
        let y = bar_rect.center().y;
        
        let view_rect = Rect::from_min_size(Pos2::new(x, bar_rect.min.y + 3.0), Vec2::new(90.0, 20.0));
        x += 96.0;
        let sep1_x = x;
        x += 8.0;
        let shading_rect = Rect::from_min_size(Pos2::new(x, bar_rect.min.y + 3.0), Vec2::new(80.0, 20.0));
        x += 86.0;
        let sep2_x = x;
        x += 8.0;
        let grid_rect = Rect::from_min_size(Pos2::new(x, bar_rect.min.y + 3.0), Vec2::new(50.0, 20.0));
        x += 56.0;
        let stats_rect = Rect::from_min_size(Pos2::new(x, bar_rect.min.y + 3.0), Vec2::new(50.0, 20.0));

        // Allocate all interactive rects first (mutable borrow)
        let view_resp = ui.allocate_rect(view_rect, Sense::click());
        let shading_resp = ui.allocate_rect(shading_rect, Sense::click());
        let grid_resp = ui.allocate_rect(grid_rect, Sense::click());
        let stats_resp = ui.allocate_rect(stats_rect, Sense::click());

        // Now get painter (immutable borrow)
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(bar_rect, 0.0, Color32::from_rgb(22, 22, 26));
        painter.line_segment(
            [Pos2::new(bar_rect.min.x, bar_rect.max.y), Pos2::new(bar_rect.max.x, bar_rect.max.y)],
            Stroke::new(1.0, Color32::from_rgb(45, 45, 50)),
        );

        // View mode button
        let view_label = match self.view_mode {
            ViewMode::Perspective => "⬡ Perspective",
            ViewMode::Top => "⬒ Top",
            ViewMode::Front => "⬒ Front", 
            ViewMode::Right => "⬒ Right",
        };
        let view_bg = if view_resp.hovered() { Color32::from_rgb(50, 50, 58) } else { Color32::from_rgb(35, 35, 42) };
        painter.rect_filled(view_rect, 3.0, view_bg);
        painter.text(view_rect.center(), egui::Align2::CENTER_CENTER, view_label, 
            egui::FontId::proportional(10.0), Color32::from_rgb(200, 200, 200));

        // Separator 1
        painter.line_segment(
            [Pos2::new(sep1_x, bar_rect.min.y + 5.0), Pos2::new(sep1_x, bar_rect.max.y - 5.0)],
            Stroke::new(1.0, Color32::from_rgb(55, 55, 60)),
        );

        // Shading mode button
        let shading_label = match self.shading_mode {
            ShadingMode::Solid => "◼ Solid",
            ShadingMode::Wireframe => "◻ Wire",
            ShadingMode::SolidWireframe => "◧ Solid+Wire",
        };
        let shading_bg = if shading_resp.hovered() { Color32::from_rgb(50, 50, 58) } else { Color32::from_rgb(35, 35, 42) };
        painter.rect_filled(shading_rect, 3.0, shading_bg);
        painter.text(shading_rect.center(), egui::Align2::CENTER_CENTER, shading_label,
            egui::FontId::proportional(10.0), Color32::from_rgb(200, 200, 200));

        // Separator 2
        painter.line_segment(
            [Pos2::new(sep2_x, bar_rect.min.y + 5.0), Pos2::new(sep2_x, bar_rect.max.y - 5.0)],
            Stroke::new(1.0, Color32::from_rgb(55, 55, 60)),
        );

        // Grid toggle
        let grid_bg = if ctx.show_grid { Color32::from_rgb(60, 80, 100) } else { Color32::from_rgb(35, 35, 42) };
        painter.rect_filled(grid_rect, 3.0, grid_bg);
        painter.text(grid_rect.center(), egui::Align2::CENTER_CENTER, "⊞ Grid",
            egui::FontId::proportional(10.0), 
            if ctx.show_grid { Color32::from_rgb(150, 200, 255) } else { Color32::from_rgb(150, 150, 150) });

        // Stats toggle
        let stats_bg = if self.show_stats { Color32::from_rgb(60, 80, 100) } else { Color32::from_rgb(35, 35, 42) };
        painter.rect_filled(stats_rect, 3.0, stats_bg);
        painter.text(stats_rect.center(), egui::Align2::CENTER_CENTER, "📊 Stats",
            egui::FontId::proportional(10.0),
            if self.show_stats { Color32::from_rgb(150, 200, 255) } else { Color32::from_rgb(150, 150, 150) });

        // Right side: Gizmo mode indicator
        let mode_label = match ctx.gizmo_mode {
            GizmoMode::Select => ("↖ Select", Color32::from_rgb(180, 180, 180)),
            GizmoMode::Translate => ("↔ Move", Color32::from_rgb(100, 220, 100)),
            GizmoMode::Rotate => ("↻ Rotate", Color32::from_rgb(100, 150, 255)),
            GizmoMode::Scale => ("⤢ Scale", Color32::from_rgb(255, 180, 60)),
        };
        painter.text(
            Pos2::new(bar_rect.max.x - 70.0, y),
            egui::Align2::CENTER_CENTER,
            mode_label.0,
            egui::FontId::proportional(11.0),
            mode_label.1,
        );

        // Handle clicks (after painting)
        if view_resp.clicked() {
            self.view_mode = match self.view_mode {
                ViewMode::Perspective => ViewMode::Top,
                ViewMode::Top => ViewMode::Front,
                ViewMode::Front => ViewMode::Right,
                ViewMode::Right => ViewMode::Perspective,
            };
            match self.view_mode {
                ViewMode::Top => ctx.camera.set_top(),
                ViewMode::Front => ctx.camera.set_front(),
                ViewMode::Right => ctx.camera.set_right(),
                ViewMode::Perspective => {},
            }
        }
        if shading_resp.clicked() {
            self.shading_mode = match self.shading_mode {
                ShadingMode::Solid => ShadingMode::Wireframe,
                ShadingMode::Wireframe => ShadingMode::SolidWireframe,
                ShadingMode::SolidWireframe => ShadingMode::Solid,
            };
            ctx.show_wireframe = matches!(self.shading_mode, ShadingMode::Wireframe | ShadingMode::SolidWireframe);
        }
        if grid_resp.clicked() { ctx.show_grid = !ctx.show_grid; }
        if stats_resp.clicked() { self.show_stats = !self.show_stats; }
    }

    // =====================================================================
    // Bottom bar — Grid/snap settings, coordinates
    // =====================================================================
    fn draw_bottom_bar(&self, ui: &mut Ui, full_rect: Rect, ctx: &EditorContext) {
        let bar_rect = Rect::from_min_max(
            Pos2::new(full_rect.min.x, full_rect.max.y - 22.0),
            full_rect.max,
        );
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(bar_rect, 0.0, Color32::from_rgb(22, 22, 26));
        painter.line_segment(
            [Pos2::new(bar_rect.min.x, bar_rect.min.y), Pos2::new(bar_rect.max.x, bar_rect.min.y)],
            Stroke::new(1.0, Color32::from_rgb(45, 45, 50)),
        );

        let y = bar_rect.center().y;

        // Left: Grid info
        painter.text(
            Pos2::new(bar_rect.min.x + 10.0, y),
            egui::Align2::LEFT_CENTER,
            format!("Grid: 1.0m  |  Snap: {:.1}m / {:.0}° / {:.1}x", 
                ctx.snap_translate, ctx.snap_rotate, ctx.snap_scale),
            egui::FontId::monospace(9.0),
            Color32::from_rgb(120, 120, 130),
        );

        // Center: Selected entity position
        if let Some(id) = ctx.selected {
            if let Some(entity) = ctx.scene.get(id) {
                let pos = entity.transform.position;
                painter.text(
                    Pos2::new(bar_rect.center().x, y),
                    egui::Align2::CENTER_CENTER,
                    format!("X: {:.2}  Y: {:.2}  Z: {:.2}", pos.x, pos.y, pos.z),
                    egui::FontId::monospace(10.0),
                    Color32::from_rgb(180, 180, 180),
                );
            }
        }

        // Right: Camera distance
        painter.text(
            Pos2::new(bar_rect.max.x - 10.0, y),
            egui::Align2::RIGHT_CENTER,
            format!("Dist: {:.1}m", ctx.camera.distance),
            egui::FontId::monospace(9.0),
            Color32::from_rgb(100, 100, 110),
        );
    }

    // =====================================================================
    // Stats overlay (top-left)
    // =====================================================================
    fn draw_stats_overlay(&self, painter: &Painter, rect: Rect, ctx: &EditorContext) {
        let bg = Color32::from_rgba_premultiplied(15, 15, 18, 220);
        let stats_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + 6.0, rect.min.y + 6.0),
            Vec2::new(140.0, 52.0),
        );
        painter.rect_filled(stats_rect, 4.0, bg);
        painter.rect_stroke(stats_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(50, 50, 55)));

        let mut y = stats_rect.min.y + 10.0;
        let x = stats_rect.min.x + 8.0;

        painter.text(Pos2::new(x, y), egui::Align2::LEFT_CENTER,
            format!("{:.0} FPS  ({:.1}ms)", ctx.stats.fps, ctx.stats.frame_time_ms),
            egui::FontId::monospace(10.0), Color32::from_rgb(100, 220, 100));
        y += 14.0;

        painter.text(Pos2::new(x, y), egui::Align2::LEFT_CENTER,
            format!("{} entities  {} tris", ctx.stats.entity_count, ctx.stats.triangles),
            egui::FontId::monospace(9.0), Color32::from_rgb(180, 180, 180));
        y += 14.0;

        painter.text(Pos2::new(x, y), egui::Align2::LEFT_CENTER,
            format!("{} draw calls", ctx.stats.draw_calls),
            egui::FontId::monospace(9.0), Color32::from_rgb(140, 140, 150));
    }

    // =====================================================================
    // Camera input — Blender style
    // =====================================================================
    fn handle_camera_input(
        &self, ui: &Ui, response: &egui::Response, ctx: &mut EditorContext,
    ) {
        // Middle-mouse orbit
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = response.drag_delta();
            let shift = ui.input(|i| i.modifiers.shift);
            if shift {
                ctx.camera.pan(delta.x, delta.y);
            } else {
                ctx.camera.orbit(delta.x, delta.y);
            }
        }

        // Right-mouse orbit (alternative)
        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = response.drag_delta();
            ctx.camera.orbit(delta.x, delta.y);
        }

        // Scroll zoom
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                ctx.camera.zoom(scroll * 0.01);
            }
        }

        // Keyboard shortcuts when viewport is hovered
        if response.hovered() {
            ui.input(|i| {
                // Numpad views
                if i.key_pressed(egui::Key::Num1) { ctx.camera.set_front(); }
                if i.key_pressed(egui::Key::Num3) { ctx.camera.set_right(); }
                if i.key_pressed(egui::Key::Num7) { ctx.camera.set_top(); }
                // F to focus
                if i.key_pressed(egui::Key::F) { ctx.focus_selected(); }
            });
        }
    }

    // =====================================================================
    // Click selection
    // =====================================================================
    fn handle_click_selection(
        &self, ui: &Ui, response: &egui::Response,
        rect: Rect, ctx: &mut EditorContext, vp_size: glam::Vec2,
    ) {
        if !response.clicked() { return; }
        let Some(mouse) = response.interact_pointer_pos() else { return; };
        let mouse_vp = glam::Vec2::new(mouse.x - rect.min.x, mouse.y - rect.min.y);

        let mut best: Option<(EntityId, f32)> = None;

        let entities: Vec<_> = ctx.scene.all_entities()
            .filter(|e| e.visible)
            .map(|e| (e.id, e.transform.position))
            .collect();

        for (id, pos) in entities {
            if let Some(screen) = ctx.camera.project(pos, vp_size) {
                let dist = (screen - mouse_vp).length();
                if dist < 25.0 {
                    if best.is_none() || dist < best.unwrap().1 {
                        best = Some((id, dist));
                    }
                }
            }
        }

        let ctrl = ui.input(|i| i.modifiers.ctrl);
        if let Some((id, _)) = best {
            if ctrl {
                ctx.toggle_select(id);
            } else {
                ctx.select(Some(id));
            }
        } else if !ctrl {
            ctx.select(None);
        }
    }

    // =====================================================================
    // Ground grid — perspective-projected infinite grid
    // =====================================================================
    fn draw_ground_grid(
        &self, painter: &Painter, rect: Rect,
        cam: &crate::editor::core::editor_context::OrbitCamera,
        vp_size: glam::Vec2,
    ) {
        let grid_extent: i32 = 20;
        let grid_step = 1.0_f32;

        for i in -grid_extent..=grid_extent {
            let t = i as f32 * grid_step;
            let is_axis = i == 0;

            // Lines parallel to X
            let a = Vec3::new(-grid_extent as f32, 0.0, t);
            let b = Vec3::new( grid_extent as f32, 0.0, t);
            // Lines parallel to Z
            let c = Vec3::new(t, 0.0, -grid_extent as f32);
            let d = Vec3::new(t, 0.0,  grid_extent as f32);

            let alpha = if is_axis { 100 } else {
                let fade = 1.0 - (i.unsigned_abs() as f32 / grid_extent as f32).powf(1.5);
                (fade * 50.0) as u8
            };
            let color = Color32::from_rgba_premultiplied(120, 120, 130, alpha);
            let width = if is_axis { 1.2 } else { 0.6 };

            self.draw_3d_line(painter, rect, cam, vp_size, a, b, Stroke::new(width, color));
            self.draw_3d_line(painter, rect, cam, vp_size, c, d, Stroke::new(width, color));
        }

        // X axis (red)
        self.draw_3d_line(painter, rect, cam, vp_size,
            Vec3::ZERO, Vec3::new(grid_extent as f32, 0.0, 0.0),
            Stroke::new(1.8, Color32::from_rgba_premultiplied(200, 50, 50, 180)));
        // Z axis (blue)
        self.draw_3d_line(painter, rect, cam, vp_size,
            Vec3::ZERO, Vec3::new(0.0, 0.0, grid_extent as f32),
            Stroke::new(1.8, Color32::from_rgba_premultiplied(50, 50, 200, 180)));
        // Y axis (green) — short vertical
        self.draw_3d_line(painter, rect, cam, vp_size,
            Vec3::ZERO, Vec3::new(0.0, 2.0, 0.0),
            Stroke::new(1.8, Color32::from_rgba_premultiplied(50, 200, 50, 180)));
    }

    // =====================================================================
    // Scene entities — solid shaded + wireframe
    // =====================================================================
    fn draw_scene_entities(
        &self, painter: &Painter, rect: Rect,
        ctx: &EditorContext, vp_size: glam::Vec2,
    ) {
        let cam = &ctx.camera;
        let light_dir = Vec3::new(0.4, 0.8, 0.3).normalize();

        // Collect entities sorted by distance (back to front for painter's algorithm)
        let mut sorted: Vec<_> = ctx.scene.all_entities()
            .filter(|e| e.visible)
            .map(|e| {
                let dist = (e.transform.position - cam.eye()).length();
                (e.clone(), dist)
            })
            .collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (entity, _dist) in &sorted {
            let is_selected = ctx.selected == Some(entity.id)
                || ctx.multi_selected.contains(&entity.id);
            let model = entity.transform.matrix();

            if let Some(mesh) = &entity.mesh {
                let base_color = if is_selected {
                    Vec3::new(1.0, 0.6, 0.1)
                } else {
                    Vec3::new(0.55, 0.6, 0.65)
                };

                match mesh.primitive {
                    MeshPrimitive::Cube => {
                        self.draw_solid_cube(painter, rect, cam, vp_size, &model, base_color, light_dir, is_selected);
                    }
                    MeshPrimitive::Sphere => {
                        self.draw_sphere_proxy(painter, rect, cam, vp_size, &model, base_color, is_selected);
                    }
                    _ => {
                        self.draw_solid_cube(painter, rect, cam, vp_size, &model, base_color, light_dir, is_selected);
                    }
                }

                // Wireframe overlay
                if ctx.show_wireframe || is_selected {
                    let wire_color = if is_selected {
                        Color32::from_rgb(255, 180, 40)
                    } else {
                        Color32::from_rgba_premultiplied(200, 200, 200, 60)
                    };
                    self.draw_wireframe_cube(painter, rect, cam, vp_size, &model, wire_color, is_selected);
                }

            } else if entity.light.is_some() {
                self.draw_light_icon(painter, rect, cam, vp_size, &entity.transform.position, is_selected);
            } else if entity.camera.is_some() {
                self.draw_camera_icon(painter, rect, cam, vp_size, &entity.transform.position, is_selected);
            } else {
                // Empty entity — small cross
                self.draw_empty_icon(painter, rect, cam, vp_size, &entity.transform.position, is_selected);
            }

            // Selected entity name label
            if is_selected {
                if let Some(sp) = cam.project(entity.transform.position, vp_size) {
                    let screen = Pos2::new(rect.min.x + sp.x, rect.min.y + sp.y - 18.0);
                    if rect.contains(screen) {
                        painter.text(
                            screen, egui::Align2::CENTER_BOTTOM,
                            &entity.name,
                            egui::FontId::proportional(11.0),
                            Color32::from_rgb(255, 220, 80),
                        );
                    }
                }
            }
        }
    }

    // =====================================================================
    // Solid cube rendering with flat shading
    // =====================================================================
    fn draw_solid_cube(
        &self, painter: &Painter, rect: Rect,
        cam: &crate::editor::core::editor_context::OrbitCamera,
        vp_size: glam::Vec2, model: &Mat4,
        base_color: Vec3, light_dir: Vec3, is_selected: bool,
    ) {
        // Transform vertices
        let transformed: Vec<Vec3> = CUBE_VERTS.iter()
            .map(|&v| {
                let w = *model * Vec4::new(v.x, v.y, v.z, 1.0);
                Vec3::new(w.x, w.y, w.z)
            })
            .collect();

        // Project vertices
        let projected: Vec<Option<glam::Vec2>> = transformed.iter()
            .map(|&v| cam.project(v, vp_size))
            .collect();

        let cam_forward = cam.forward();

        // Draw faces (back-face culled, lit)
        for &(a, b, c, face_normal) in &CUBE_FACES {
            // Transform normal
            let world_normal = {
                let n4 = *model * Vec4::new(face_normal.x, face_normal.y, face_normal.z, 0.0);
                Vec3::new(n4.x, n4.y, n4.z).normalize()
            };

            // Back-face culling
            if world_normal.dot(-cam_forward) < -0.05 { continue; }

            let (Some(pa), Some(pb), Some(pc)) = (projected[a], projected[b], projected[c]) else { continue };

            // Lighting
            let ndl = world_normal.dot(light_dir).max(0.0);
            let ambient = 0.25;
            let lit = base_color * (ambient + ndl * 0.75);
            let r = (lit.x * 255.0).min(255.0) as u8;
            let g = (lit.y * 255.0).min(255.0) as u8;
            let b_val = (lit.z * 255.0).min(255.0) as u8;
            let fill = Color32::from_rgba_premultiplied(r, g, b_val, if is_selected { 220 } else { 200 });

            let points = vec![
                Pos2::new(rect.min.x + pa.x, rect.min.y + pa.y),
                Pos2::new(rect.min.x + pb.x, rect.min.y + pb.y),
                Pos2::new(rect.min.x + pc.x, rect.min.y + pc.y),
            ];

            // Check if triangle is on screen
            let any_visible = points.iter().any(|p| rect.contains(*p));
            if !any_visible { continue; }

            painter.add(egui::Shape::convex_polygon(
                points,
                fill,
                Stroke::NONE,
            ));
        }
    }

    // =====================================================================
    // Wireframe cube
    // =====================================================================
    fn draw_wireframe_cube(
        &self, painter: &Painter, rect: Rect,
        cam: &crate::editor::core::editor_context::OrbitCamera,
        vp_size: glam::Vec2, model: &Mat4,
        color: Color32, is_selected: bool,
    ) {
        let transformed: Vec<Vec3> = CUBE_VERTS.iter()
            .map(|&v| {
                let w = *model * Vec4::new(v.x, v.y, v.z, 1.0);
                Vec3::new(w.x, w.y, w.z)
            })
            .collect();

        let projected: Vec<Option<glam::Vec2>> = transformed.iter()
            .map(|&v| cam.project(v, vp_size))
            .collect();

        let width = if is_selected { 1.8 } else { 0.8 };

        for &(a, b) in &CUBE_EDGES {
            if let (Some(pa), Some(pb)) = (projected[a], projected[b]) {
                let sa = Pos2::new(rect.min.x + pa.x, rect.min.y + pa.y);
                let sb = Pos2::new(rect.min.x + pb.x, rect.min.y + pb.y);
                painter.line_segment([sa, sb], Stroke::new(width, color));
            }
        }
    }

    // =====================================================================
    // Sphere proxy (circle in screen space)
    // =====================================================================
    fn draw_sphere_proxy(
        &self, painter: &Painter, rect: Rect,
        cam: &crate::editor::core::editor_context::OrbitCamera,
        vp_size: glam::Vec2, model: &Mat4,
        base_color: Vec3, is_selected: bool,
    ) {
        let center_w = *model * Vec4::new(0.0, 0.0, 0.0, 1.0);
        let center3 = Vec3::new(center_w.x, center_w.y, center_w.z);
        let edge_w = *model * Vec4::new(0.5, 0.0, 0.0, 1.0);
        let edge3 = Vec3::new(edge_w.x, edge_w.y, edge_w.z);

        let Some(center_s) = cam.project(center3, vp_size) else { return };
        let Some(edge_s) = cam.project(edge3, vp_size) else { return };

        let radius = (center_s - edge_s).length().max(4.0);
        let sp = Pos2::new(rect.min.x + center_s.x, rect.min.y + center_s.y);

        if !rect.contains(sp) { return; }

        // Gradient sphere approximation
        let r = (base_color.x * 200.0) as u8;
        let g = (base_color.y * 200.0) as u8;
        let b = (base_color.z * 200.0) as u8;
        let fill = Color32::from_rgba_premultiplied(r, g, b, if is_selected { 220 } else { 180 });
        let highlight = Color32::from_rgba_premultiplied(
            (r as u16 + 40).min(255) as u8,
            (g as u16 + 40).min(255) as u8,
            (b as u16 + 40).min(255) as u8,
            100,
        );

        painter.circle_filled(sp, radius, fill);
        // Specular highlight
        painter.circle_filled(
            Pos2::new(sp.x - radius * 0.25, sp.y - radius * 0.25),
            radius * 0.35,
            highlight,
        );

        if is_selected {
            painter.circle_stroke(sp, radius + 2.0, Stroke::new(2.0, Color32::from_rgb(255, 180, 40)));
        }
    }

    // =====================================================================
    // Light icon
    // =====================================================================
    fn draw_light_icon(
        &self, painter: &Painter, rect: Rect,
        cam: &crate::editor::core::editor_context::OrbitCamera,
        vp_size: glam::Vec2, pos: &Vec3, is_selected: bool,
    ) {
        let Some(sp) = cam.project(*pos, vp_size) else { return };
        let screen = Pos2::new(rect.min.x + sp.x, rect.min.y + sp.y);
        if !rect.contains(screen) { return; }

        let color = Color32::from_rgb(255, 220, 60);
        painter.circle_filled(screen, 7.0, color);

        // Rays
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::TAU / 8.0;
            let inner = 9.0;
            let outer = 14.0;
            let a = Pos2::new(screen.x + angle.cos() * inner, screen.y + angle.sin() * inner);
            let b = Pos2::new(screen.x + angle.cos() * outer, screen.y + angle.sin() * outer);
            painter.line_segment([a, b], Stroke::new(1.2, Color32::from_rgba_premultiplied(255, 220, 60, 140)));
        }

        if is_selected {
            painter.circle_stroke(screen, 16.0, Stroke::new(2.0, Color32::from_rgb(255, 180, 40)));
        }
    }

    // =====================================================================
    // Camera icon
    // =====================================================================
    fn draw_camera_icon(
        &self, painter: &Painter, rect: Rect,
        cam: &crate::editor::core::editor_context::OrbitCamera,
        vp_size: glam::Vec2, pos: &Vec3, is_selected: bool,
    ) {
        let Some(sp) = cam.project(*pos, vp_size) else { return };
        let screen = Pos2::new(rect.min.x + sp.x, rect.min.y + sp.y);
        if !rect.contains(screen) { return; }

        let color = if is_selected { Color32::from_rgb(100, 255, 100) } else { Color32::from_rgb(80, 180, 80) };

        // Camera body
        let body = Rect::from_center_size(screen, Vec2::new(14.0, 10.0));
        painter.rect_filled(body, 2.0, color);

        // Lens
        let lens_pts = vec![
            Pos2::new(screen.x - 10.0, screen.y - 7.0),
            Pos2::new(screen.x - 16.0, screen.y - 12.0),
            Pos2::new(screen.x - 16.0, screen.y + 12.0),
            Pos2::new(screen.x - 10.0, screen.y + 7.0),
        ];
        painter.add(egui::Shape::convex_polygon(
            lens_pts, color, Stroke::NONE,
        ));

        if is_selected {
            painter.rect_stroke(
                Rect::from_center_size(screen, Vec2::new(36.0, 28.0)),
                3.0,
                Stroke::new(2.0, Color32::from_rgb(255, 180, 40)),
            );
        }
    }

    // =====================================================================
    // Empty entity icon
    // =====================================================================
    fn draw_empty_icon(
        &self, painter: &Painter, rect: Rect,
        cam: &crate::editor::core::editor_context::OrbitCamera,
        vp_size: glam::Vec2, pos: &Vec3, is_selected: bool,
    ) {
        let Some(sp) = cam.project(*pos, vp_size) else { return };
        let screen = Pos2::new(rect.min.x + sp.x, rect.min.y + sp.y);
        if !rect.contains(screen) { return; }

        let color = if is_selected { Color32::from_rgb(255, 180, 40) } else { Color32::from_rgb(150, 150, 150) };
        let s = 6.0;
        painter.line_segment(
            [Pos2::new(screen.x - s, screen.y), Pos2::new(screen.x + s, screen.y)],
            Stroke::new(1.5, color),
        );
        painter.line_segment(
            [Pos2::new(screen.x, screen.y - s), Pos2::new(screen.x, screen.y + s)],
            Stroke::new(1.5, color),
        );
    }

    // =====================================================================
    // FUNCTIONAL Gizmo Interaction — Real-time transform manipulation
    // =====================================================================
    fn handle_gizmo_interaction(
        &mut self, ui: &Ui, response: &egui::Response,
        rect: Rect, ctx: &mut EditorContext, vp_size: glam::Vec2,
    ) {
        let Some(id) = ctx.selected else {
            self.active_axis = GizmoAxis::None;
            return;
        };
        
        // Skip if not in a transform mode
        if ctx.gizmo_mode == GizmoMode::Select {
            self.active_axis = GizmoAxis::None;
            return;
        }

        let Some(entity) = ctx.scene.get(id) else { return };
        let entity_pos = entity.transform.position;
        let Some(center_screen) = ctx.camera.project(entity_pos, vp_size) else { return };
        let gizmo_center = Pos2::new(rect.min.x + center_screen.x, rect.min.y + center_screen.y);

        // Get mouse position
        let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO);
        let mouse_vp = glam::Vec2::new(mouse_pos.x - rect.min.x, mouse_pos.y - rect.min.y);

        let gizmo_len = 55.0;

        // Check for drag start on gizmo axes
        if response.drag_started_by(egui::PointerButton::Primary) {
            let dx = mouse_pos.x - gizmo_center.x;
            let dy = mouse_pos.y - gizmo_center.y;
            
            // Detect which axis was clicked
            let axis = self.detect_gizmo_axis(dx, dy, gizmo_len, ctx.gizmo_mode);
            
            if axis != GizmoAxis::None {
                self.active_axis = axis;
                self.drag_start_pos = Some(mouse_vp);
                self.drag_start_transform = Some(entity_pos);
                self.drag_start_scale = Some(entity.transform.scale);
                self.drag_start_rotation = Some(entity.transform.rotation);
            }
        }

        // Handle active drag
        if self.active_axis != GizmoAxis::None && response.dragged_by(egui::PointerButton::Primary) {
            let Some(start_pos) = self.drag_start_pos else { return };
            let delta = mouse_vp - start_pos;
            
            // Calculate world-space movement based on camera
            let cam_right = ctx.camera.right();
            let cam_up = ctx.camera.up();
            
            // Sensitivity factors
            let translate_sensitivity = 0.02 * ctx.camera.distance;
            let rotate_sensitivity = 0.5;
            let scale_sensitivity = 0.01;

            match ctx.gizmo_mode {
                GizmoMode::Translate => {
                    let Some(start_transform) = self.drag_start_transform else { return };
                    let mut new_pos = start_transform;
                    
                    match self.active_axis {
                        GizmoAxis::X => {
                            // Project screen delta onto X axis
                            let x_delta = delta.x * translate_sensitivity;
                            new_pos.x = start_transform.x + x_delta;
                        }
                        GizmoAxis::Y => {
                            // Y is up, so use negative screen Y
                            let y_delta = -delta.y * translate_sensitivity;
                            new_pos.y = start_transform.y + y_delta;
                        }
                        GizmoAxis::Z => {
                            // Z uses combined screen movement
                            let z_delta = (-delta.x * 0.5 + delta.y * 0.3) * translate_sensitivity;
                            new_pos.z = start_transform.z + z_delta;
                        }
                        GizmoAxis::XY => {
                            new_pos.x = start_transform.x + delta.x * translate_sensitivity;
                            new_pos.y = start_transform.y - delta.y * translate_sensitivity;
                        }
                        GizmoAxis::XZ => {
                            new_pos.x = start_transform.x + delta.x * translate_sensitivity;
                            new_pos.z = start_transform.z + delta.y * translate_sensitivity;
                        }
                        GizmoAxis::All => {
                            new_pos = start_transform + cam_right * delta.x * translate_sensitivity
                                                      + cam_up * (-delta.y) * translate_sensitivity;
                        }
                        _ => {}
                    }
                    
                    // Apply snap if enabled
                    if ctx.snap_translate > 0.0 {
                        new_pos.x = (new_pos.x / ctx.snap_translate).round() * ctx.snap_translate;
                        new_pos.y = (new_pos.y / ctx.snap_translate).round() * ctx.snap_translate;
                        new_pos.z = (new_pos.z / ctx.snap_translate).round() * ctx.snap_translate;
                    }
                    
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.transform.position = new_pos;
                    }
                }
                GizmoMode::Rotate => {
                    let Some(start_rot) = self.drag_start_rotation else { return };
                    let angle = delta.x * rotate_sensitivity;
                    
                    // Apply snap
                    let snapped_angle = if ctx.snap_rotate > 0.0 {
                        (angle / ctx.snap_rotate).round() * ctx.snap_rotate
                    } else {
                        angle
                    };
                    
                    let rotation = match self.active_axis {
                        GizmoAxis::X => glam::Quat::from_rotation_x(snapped_angle.to_radians()),
                        GizmoAxis::Y => glam::Quat::from_rotation_y(snapped_angle.to_radians()),
                        GizmoAxis::Z => glam::Quat::from_rotation_z(snapped_angle.to_radians()),
                        _ => glam::Quat::IDENTITY,
                    };
                    
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.transform.rotation = rotation * start_rot;
                    }
                }
                GizmoMode::Scale => {
                    let Some(start_scale) = self.drag_start_scale else { return };
                    let scale_delta = 1.0 + delta.x * scale_sensitivity;
                    let scale_factor = scale_delta.max(0.1);
                    
                    let mut new_scale = start_scale;
                    match self.active_axis {
                        GizmoAxis::X => new_scale.x = start_scale.x * scale_factor,
                        GizmoAxis::Y => new_scale.y = start_scale.y * scale_factor,
                        GizmoAxis::Z => new_scale.z = start_scale.z * scale_factor,
                        GizmoAxis::All => new_scale = start_scale * scale_factor,
                        _ => {}
                    }
                    
                    if let Some(e) = ctx.scene.get_mut(id) {
                        e.transform.scale = new_scale;
                    }
                }
                _ => {}
            }
        }

        // End drag
        if response.drag_stopped() {
            self.active_axis = GizmoAxis::None;
            self.drag_start_pos = None;
            self.drag_start_transform = None;
            self.drag_start_scale = None;
            self.drag_start_rotation = None;
        }
    }

    fn detect_gizmo_axis(&self, dx: f32, dy: f32, gizmo_len: f32, mode: GizmoMode) -> GizmoAxis {
        let hit_radius = 12.0;
        
        match mode {
            GizmoMode::Translate | GizmoMode::Scale => {
                // X axis (right)
                if dx > 10.0 && dx < gizmo_len + hit_radius && dy.abs() < hit_radius {
                    return GizmoAxis::X;
                }
                // Y axis (up)
                if dy < -10.0 && dy > -(gizmo_len + hit_radius) && dx.abs() < hit_radius {
                    return GizmoAxis::Y;
                }
                // Z axis (diagonal)
                let z_dx = -gizmo_len * 0.5;
                let z_dy = gizmo_len * 0.3;
                let dist_to_z = ((dx - z_dx * 0.5).powi(2) + (dy - z_dy * 0.5).powi(2)).sqrt();
                if dist_to_z < gizmo_len * 0.6 && dx < 0.0 && dy > 0.0 {
                    return GizmoAxis::Z;
                }
                // Center (all axes)
                if dx.abs() < 15.0 && dy.abs() < 15.0 {
                    return GizmoAxis::All;
                }
            }
            GizmoMode::Rotate => {
                let dist = (dx * dx + dy * dy).sqrt();
                let r = gizmo_len * 0.8;
                // X ring (outermost)
                if (dist - r).abs() < hit_radius {
                    return GizmoAxis::X;
                }
                // Y ring (middle)
                if (dist - r * 0.85).abs() < hit_radius {
                    return GizmoAxis::Y;
                }
                // Z ring (innermost)
                if (dist - r * 0.7).abs() < hit_radius {
                    return GizmoAxis::Z;
                }
            }
            _ => {}
        }
        
        GizmoAxis::None
    }

    // =====================================================================
    // Interactive Transform Gizmo — with hover highlights
    // =====================================================================
    fn draw_transform_gizmo_interactive(
        &self, painter: &Painter, rect: Rect,
        ctx: &EditorContext, vp_size: glam::Vec2,
    ) {
        let Some(id) = ctx.selected else { return };
        let Some(entity) = ctx.scene.get(id) else { return };
        let pos = entity.transform.position;
        let Some(center) = ctx.camera.project(pos, vp_size) else { return };
        let screen = Pos2::new(rect.min.x + center.x, rect.min.y + center.y);

        if !rect.contains(screen) { return; }

        let gizmo_len = 55.0;
        let active = self.active_axis;

        match ctx.gizmo_mode {
            GizmoMode::Select => {
                painter.circle_stroke(screen, 10.0, Stroke::new(2.0, Color32::from_rgb(255, 200, 60)));
                painter.circle_filled(screen, 4.0, Color32::from_rgb(255, 200, 60));
            }
            GizmoMode::Translate => {
                // Center square (all axes)
                let center_color = if active == GizmoAxis::All {
                    Color32::from_rgb(255, 255, 100)
                } else {
                    Color32::from_rgba_premultiplied(200, 200, 200, 100)
                };
                painter.rect_filled(Rect::from_center_size(screen, Vec2::splat(12.0)), 2.0, center_color);

                // X arrow (red)
                let x_color = if active == GizmoAxis::X { Color32::from_rgb(255, 100, 100) } else { Color32::from_rgb(230, 50, 50) };
                let x_width = if active == GizmoAxis::X { 4.0 } else { 3.0 };
                let x_tip = Pos2::new(screen.x + gizmo_len, screen.y);
                painter.line_segment([screen, x_tip], Stroke::new(x_width, x_color));
                self.draw_arrow_head(painter, x_tip, Vec2::new(1.0, 0.0), x_color);
                painter.text(Pos2::new(x_tip.x + 8.0, x_tip.y), egui::Align2::LEFT_CENTER, "X", egui::FontId::proportional(11.0), x_color);

                // Y arrow (green)
                let y_color = if active == GizmoAxis::Y { Color32::from_rgb(100, 255, 100) } else { Color32::from_rgb(50, 200, 50) };
                let y_width = if active == GizmoAxis::Y { 4.0 } else { 3.0 };
                let y_tip = Pos2::new(screen.x, screen.y - gizmo_len);
                painter.line_segment([screen, y_tip], Stroke::new(y_width, y_color));
                self.draw_arrow_head(painter, y_tip, Vec2::new(0.0, -1.0), y_color);
                painter.text(Pos2::new(y_tip.x, y_tip.y - 10.0), egui::Align2::CENTER_BOTTOM, "Y", egui::FontId::proportional(11.0), y_color);

                // Z arrow (blue)
                let z_color = if active == GizmoAxis::Z { Color32::from_rgb(100, 150, 255) } else { Color32::from_rgb(50, 80, 230) };
                let z_width = if active == GizmoAxis::Z { 4.0 } else { 3.0 };
                let z_tip = Pos2::new(screen.x - gizmo_len * 0.5, screen.y + gizmo_len * 0.4);
                painter.line_segment([screen, z_tip], Stroke::new(z_width, z_color));
                self.draw_arrow_head(painter, z_tip, Vec2::new(-0.5, 0.4).normalized(), z_color);
                painter.text(Pos2::new(z_tip.x - 8.0, z_tip.y), egui::Align2::RIGHT_CENTER, "Z", egui::FontId::proportional(11.0), z_color);

                // XY plane indicator
                let plane_size = 18.0;
                let xy_pts = vec![
                    screen,
                    Pos2::new(screen.x + plane_size, screen.y),
                    Pos2::new(screen.x + plane_size, screen.y - plane_size),
                    Pos2::new(screen.x, screen.y - plane_size),
                ];
                painter.add(egui::Shape::convex_polygon(xy_pts, Color32::from_rgba_premultiplied(255, 255, 100, 40), Stroke::NONE));
            }
            GizmoMode::Rotate => {
                let r = gizmo_len * 0.85;
                // X ring (red - pitch)
                let x_color = if active == GizmoAxis::X { Color32::from_rgb(255, 100, 100) } else { Color32::from_rgb(230, 50, 50) };
                let x_width = if active == GizmoAxis::X { 4.0 } else { 2.5 };
                painter.circle_stroke(screen, r, Stroke::new(x_width, x_color));
                
                // Y ring (green - yaw)
                let y_color = if active == GizmoAxis::Y { Color32::from_rgb(100, 255, 100) } else { Color32::from_rgb(50, 200, 50) };
                let y_width = if active == GizmoAxis::Y { 4.0 } else { 2.5 };
                painter.circle_stroke(screen, r * 0.85, Stroke::new(y_width, y_color));
                
                // Z ring (blue - roll)
                let z_color = if active == GizmoAxis::Z { Color32::from_rgb(100, 150, 255) } else { Color32::from_rgb(50, 80, 230) };
                let z_width = if active == GizmoAxis::Z { 4.0 } else { 2.5 };
                painter.circle_stroke(screen, r * 0.7, Stroke::new(z_width, z_color));

                // Labels
                painter.text(Pos2::new(screen.x + r + 8.0, screen.y), egui::Align2::LEFT_CENTER, "X", egui::FontId::proportional(10.0), x_color);
                painter.text(Pos2::new(screen.x, screen.y - r * 0.85 - 8.0), egui::Align2::CENTER_BOTTOM, "Y", egui::FontId::proportional(10.0), y_color);
            }
            GizmoMode::Scale => {
                let s = gizmo_len;
                
                // Center cube (uniform scale)
                let center_color = if active == GizmoAxis::All { Color32::from_rgb(255, 255, 100) } else { Color32::from_rgb(180, 180, 180) };
                painter.rect_filled(Rect::from_center_size(screen, Vec2::splat(10.0)), 0.0, center_color);

                // X
                let x_color = if active == GizmoAxis::X { Color32::from_rgb(255, 100, 100) } else { Color32::from_rgb(230, 50, 50) };
                let x_width = if active == GizmoAxis::X { 4.0 } else { 3.0 };
                painter.line_segment([screen, Pos2::new(screen.x + s, screen.y)], Stroke::new(x_width, x_color));
                painter.rect_filled(Rect::from_center_size(Pos2::new(screen.x + s, screen.y), Vec2::splat(8.0)), 0.0, x_color);

                // Y
                let y_color = if active == GizmoAxis::Y { Color32::from_rgb(100, 255, 100) } else { Color32::from_rgb(50, 200, 50) };
                let y_width = if active == GizmoAxis::Y { 4.0 } else { 3.0 };
                painter.line_segment([screen, Pos2::new(screen.x, screen.y - s)], Stroke::new(y_width, y_color));
                painter.rect_filled(Rect::from_center_size(Pos2::new(screen.x, screen.y - s), Vec2::splat(8.0)), 0.0, y_color);

                // Z
                let z_color = if active == GizmoAxis::Z { Color32::from_rgb(100, 150, 255) } else { Color32::from_rgb(50, 80, 230) };
                let z_width = if active == GizmoAxis::Z { 4.0 } else { 3.0 };
                let zt = Pos2::new(screen.x - s * 0.5, screen.y + s * 0.4);
                painter.line_segment([screen, zt], Stroke::new(z_width, z_color));
                painter.rect_filled(Rect::from_center_size(zt, Vec2::splat(8.0)), 0.0, z_color);
            }
        }

        // Show active axis feedback
        if active != GizmoAxis::None {
            let axis_name = match active {
                GizmoAxis::X => "X",
                GizmoAxis::Y => "Y", 
                GizmoAxis::Z => "Z",
                GizmoAxis::XY => "XY",
                GizmoAxis::XZ => "XZ",
                GizmoAxis::YZ => "YZ",
                GizmoAxis::All => "ALL",
                GizmoAxis::None => "",
            };
            painter.text(
                Pos2::new(screen.x, screen.y + gizmo_len + 20.0),
                egui::Align2::CENTER_TOP,
                format!("Dragging: {}", axis_name),
                egui::FontId::proportional(10.0),
                Color32::from_rgb(255, 220, 80),
            );
        }
    }

    fn draw_arrow_head(&self, painter: &Painter, tip: Pos2, dir: Vec2, color: Color32) {
        let perp = Vec2::new(-dir.y, dir.x);
        let back = tip - dir * 8.0;
        let pts = vec![
            tip,
            Pos2::new(back.x + perp.x * 3.5, back.y + perp.y * 3.5),
            Pos2::new(back.x - perp.x * 3.5, back.y - perp.y * 3.5),
        ];
        painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
    }

    // =====================================================================
    // Orientation gizmo (top-right corner)
    // =====================================================================
    fn draw_orientation_gizmo(
        &self, painter: &Painter, rect: Rect,
        cam: &crate::editor::core::editor_context::OrbitCamera,
    ) {
        let origin = Pos2::new(rect.max.x - 50.0, rect.min.y + 50.0);
        let len = 28.0;

        // Use actual camera rotation to compute axis directions
        let view = cam.view_matrix();
        let axes = [
            (Vec3::X, Color32::from_rgb(230, 60, 60), "X"),
            (Vec3::Y, Color32::from_rgb(60, 200, 60), "Y"),
            (Vec3::Z, Color32::from_rgb(60, 80, 230), "Z"),
        ];

        // Background circle
        painter.circle_filled(origin, 38.0, Color32::from_rgba_premultiplied(20, 20, 24, 200));
        painter.circle_stroke(origin, 38.0, Stroke::new(1.0, Color32::from_rgba_premultiplied(80, 80, 90, 120)));

        // Sort axes by depth for proper draw order
        let mut axis_data: Vec<(Pos2, Color32, &str, f32)> = axes.iter().map(|&(axis, color, label)| {
            let v = view.transform_vector3(axis);
            let end = Pos2::new(origin.x + v.x * len, origin.y - v.y * len);
            (end, color, label, v.z)
        }).collect();
        axis_data.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));

        for (end, color, label, _depth) in &axis_data {
            painter.line_segment([origin, *end], Stroke::new(2.5, *color));
            painter.circle_filled(*end, 5.0, *color);
            painter.text(*end, egui::Align2::CENTER_CENTER, *label, egui::FontId::proportional(9.0), Color32::WHITE);
        }

        painter.circle_filled(origin, 2.5, Color32::from_rgb(200, 200, 200));
    }

    // =====================================================================
    // HUD overlay
    // =====================================================================
    fn draw_hud(&self, ui: &mut Ui, rect: Rect, ctx: &EditorContext) {
        let painter = ui.painter();
        let bg = Color32::from_rgba_premultiplied(10, 10, 14, 180);

        // Top-left: stats
        {
            let info = format!(
                " {:.0} FPS  {:.1}ms  {} entities  {} tris ",
                ctx.stats.fps, ctx.stats.frame_time_ms,
                ctx.stats.entity_count, ctx.stats.triangles,
            );
            let pos = Pos2::new(rect.min.x + 6.0, rect.min.y + 6.0);
            let galley = ui.fonts(|f| f.layout_no_wrap(
                info, egui::FontId::monospace(10.0), Color32::from_rgb(180, 180, 180),
            ));
            let tr = Rect::from_min_size(pos, galley.size() + Vec2::new(4.0, 2.0));
            painter.rect_filled(tr, 3.0, bg);
            painter.galley(pos + Vec2::new(2.0, 1.0), galley, Color32::from_rgb(180, 180, 180));
        }

        // Bottom-left: gizmo mode
        {
            let (label, color) = match ctx.gizmo_mode {
                GizmoMode::Select    => ("SELECT [Q]",    Color32::from_rgb(180, 180, 180)),
                GizmoMode::Translate => ("MOVE [W]",      Color32::from_rgb(100, 220, 100)),
                GizmoMode::Rotate    => ("ROTATE [E]",    Color32::from_rgb(100, 150, 255)),
                GizmoMode::Scale     => ("SCALE [R]",     Color32::from_rgb(255, 180, 60)),
            };
            painter.text(
                Pos2::new(rect.min.x + 10.0, rect.max.y - 10.0),
                egui::Align2::LEFT_BOTTOM,
                label,
                egui::FontId::monospace(11.0),
                color,
            );
        }

        // Bottom-right: camera info
        {
            let cam = &ctx.camera;
            let info = format!(
                "Eye ({:.1}, {:.1}, {:.1})  Dist {:.1}",
                cam.eye().x, cam.eye().y, cam.eye().z, cam.distance,
            );
            painter.text(
                Pos2::new(rect.max.x - 10.0, rect.max.y - 10.0),
                egui::Align2::RIGHT_BOTTOM,
                info,
                egui::FontId::monospace(9.0),
                Color32::from_rgb(120, 120, 130),
            );
        }

        // Bottom-center: controls hint
        {
            let hint = "MMB: Orbit  Shift+MMB: Pan  Scroll: Zoom  F: Focus  1/3/7: Views";
            painter.text(
                Pos2::new(rect.center().x, rect.max.y - 10.0),
                egui::Align2::CENTER_BOTTOM,
                hint,
                egui::FontId::monospace(8.0),
                Color32::from_rgb(80, 80, 90),
            );
        }
    }

    // =====================================================================
    // Play mode overlay
    // =====================================================================
    fn draw_play_overlay(&self, painter: &Painter, rect: Rect) {
        painter.rect_stroke(rect, 0.0, Stroke::new(3.0, Color32::from_rgb(255, 140, 0)));
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(rect.center().x - 80.0, rect.min.y + 4.0), Vec2::new(160.0, 22.0)),
            4.0,
            Color32::from_rgba_premultiplied(200, 100, 0, 200),
        );
        painter.text(
            Pos2::new(rect.center().x, rect.min.y + 15.0),
            egui::Align2::CENTER_CENTER,
            "▶  PLAY MODE  [F5]",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
    }

    // =====================================================================
    // Utility: draw a 3D line projected to screen
    // =====================================================================
    fn draw_3d_line(
        &self, painter: &Painter, rect: Rect,
        cam: &crate::editor::core::editor_context::OrbitCamera,
        vp_size: glam::Vec2,
        a: Vec3, b: Vec3, stroke: Stroke,
    ) {
        let Some(sa) = cam.project(a, vp_size) else { return };
        let Some(sb) = cam.project(b, vp_size) else { return };
        let pa = Pos2::new(rect.min.x + sa.x, rect.min.y + sa.y);
        let pb = Pos2::new(rect.min.x + sb.x, rect.min.y + sb.y);
        painter.line_segment([pa, pb], stroke);
    }
}
