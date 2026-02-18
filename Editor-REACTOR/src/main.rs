// =============================================================================
// REACTOR Editor — Main Entry Point
// =============================================================================
// Architecture:
//   Engine (reactor lib) — independent
//   Editor (this crate)  — client of the engine
//
// Layout:
//   Toolbar
//   ┌──────────────┬──────────────────────────┐
//   │  Hierarchy   │       Viewport           │
//   ├──────────────┼──────────────────────────┤
//   │  Inspector   │     Asset Browser        │
//   └──────────────┴──────────────────────────┘
//   Console
// =============================================================================

#![allow(clippy::new_without_default)]

mod editor;

use eframe::{egui, App, CreationContext, Frame, NativeOptions};
use egui::{Color32, Context, RichText, Visuals};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};

use editor::core::command_system::CommandSystem;
use editor::core::editor_context::EditorContext;
use editor::core::event_bus::EventBus;
use editor::ui::panels::{
    asset_browser_panel::AssetBrowserPanel,
    console_panel::ConsolePanel,
    hierarchy_panel::HierarchyPanel,
    inspector_panel::InspectorPanel,
    toolbar_panel::ToolbarPanel,
    viewport_panel::ViewportPanel,
};

// =============================================================================
// Panel identifiers for the dock system
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
enum PanelId {
    Viewport,
    Hierarchy,
    Inspector,
    AssetBrowser,
    Console,
}

impl std::fmt::Display for PanelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelId::Viewport     => write!(f, "🖥  Viewport"),
            PanelId::Hierarchy    => write!(f, "🌍  Hierarchy"),
            PanelId::Inspector    => write!(f, "🔍  Inspector"),
            PanelId::AssetBrowser => write!(f, "📂  Assets"),
            PanelId::Console      => write!(f, "🖥  Console"),
        }
    }
}

// =============================================================================
// TabViewer — bridges egui_dock with our panels
// =============================================================================

struct ReactorTabViewer<'a> {
    ctx: &'a mut EditorContext,
    viewport: &'a mut ViewportPanel,
    hierarchy: &'a mut HierarchyPanel,
    inspector: &'a mut InspectorPanel,
    asset_browser: &'a mut AssetBrowserPanel,
    console: &'a mut ConsolePanel,
}

impl<'a> TabViewer for ReactorTabViewer<'a> {
    type Tab = PanelId;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelId::Viewport     => self.viewport.show(ui, self.ctx),
            PanelId::Hierarchy    => self.hierarchy.show(ui, self.ctx),
            PanelId::Inspector    => self.inspector.show(ui, self.ctx),
            PanelId::AssetBrowser => self.asset_browser.show(ui, self.ctx),
            PanelId::Console      => self.console.show(ui, self.ctx),
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }
}

// =============================================================================
// ReactorEditor — the main eframe App
// =============================================================================

struct ReactorEditor {
    // Core state
    editor_ctx: EditorContext,
    commands: CommandSystem,
    events: EventBus,

    // UI panels
    viewport: ViewportPanel,
    hierarchy: HierarchyPanel,
    inspector: InspectorPanel,
    asset_browser: AssetBrowserPanel,
    console: ConsolePanel,
    toolbar: ToolbarPanel,

    // Dock layout
    dock_state: DockState<PanelId>,

    // Frame timing
    last_frame_time: std::time::Instant,
    frame_count: u64,
}

impl ReactorEditor {
    fn new(_cc: &CreationContext) -> Self {
        // ── Build dock layout ──────────────────────────────────────────────
        //
        //  [Hierarchy | Viewport]
        //  [Inspector | AssetBrowser]
        //  [Console (bottom, full width)]
        //
        let mut dock_state = DockState::new(vec![PanelId::Viewport]);

        let surface = dock_state.main_surface_mut();

        // Split left: Hierarchy (25% width)
        let [left, _right] = surface.split_left(NodeIndex::root(), 0.22, vec![PanelId::Hierarchy]);

        // Split the right side bottom: Console (20% height)
        let [_top, _bottom] = surface.split_below(NodeIndex::root(), 0.78, vec![PanelId::Console]);

        // Split left side bottom: Inspector below Hierarchy
        let [_hier, _insp] = surface.split_below(left, 0.55, vec![PanelId::Inspector]);

        // Split viewport right: Asset Browser (30% width)
        surface.split_right(NodeIndex::root(), 0.72, vec![PanelId::AssetBrowser]);

        Self {
            editor_ctx: EditorContext::new(),
            commands: CommandSystem::new(),
            events: EventBus::new(),
            viewport: ViewportPanel::new(),
            hierarchy: HierarchyPanel::new(),
            inspector: InspectorPanel::new(),
            asset_browser: AssetBrowserPanel::new(),
            console: ConsolePanel::new(),
            toolbar: ToolbarPanel::new(),
            dock_state,
            last_frame_time: std::time::Instant::now(),
            frame_count: 0,
        }
    }

    fn apply_dark_theme(ctx: &Context) {
        let mut visuals = Visuals::dark();

        // Custom REACTOR dark theme
        visuals.window_fill = Color32::from_rgb(28, 28, 32);
        visuals.panel_fill = Color32::from_rgb(32, 32, 36);
        visuals.faint_bg_color = Color32::from_rgb(38, 38, 44);
        visuals.extreme_bg_color = Color32::from_rgb(20, 20, 24);

        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(38, 38, 44);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(48, 48, 54);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 70);
        visuals.widgets.active.bg_fill = Color32::from_rgb(70, 100, 160);

        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(80, 120, 200, 120);
        visuals.selection.stroke.color = Color32::from_rgb(100, 160, 255);

        visuals.override_text_color = Some(Color32::from_rgb(210, 210, 215));

        ctx.set_visuals(visuals);
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &Context) {
        ctx.input(|i| {
            // Transform mode shortcuts
            if i.key_pressed(egui::Key::Q) {
                self.editor_ctx.gizmo_mode = editor::core::editor_context::GizmoMode::Select;
            }
            if i.key_pressed(egui::Key::W) {
                self.editor_ctx.gizmo_mode = editor::core::editor_context::GizmoMode::Translate;
            }
            if i.key_pressed(egui::Key::E) {
                self.editor_ctx.gizmo_mode = editor::core::editor_context::GizmoMode::Rotate;
            }
            if i.key_pressed(egui::Key::R) {
                self.editor_ctx.gizmo_mode = editor::core::editor_context::GizmoMode::Scale;
            }

            // Undo / Redo
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                // undo handled below (needs mut)
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Y) {
                // redo handled below
            }

            // Delete selected
            if i.key_pressed(egui::Key::Delete) {
                // handled below
            }

            // Play/Stop
            if i.key_pressed(egui::Key::F5) {
                // handled below
            }
        });

        // Undo/Redo (needs mutable borrow)
        let (do_undo, do_redo, do_delete, do_play, do_duplicate, do_focus) = ctx.input(|i| (
            i.modifiers.ctrl && i.key_pressed(egui::Key::Z),
            i.modifiers.ctrl && i.key_pressed(egui::Key::Y),
            i.key_pressed(egui::Key::Delete),
            i.key_pressed(egui::Key::F5),
            i.modifiers.ctrl && i.key_pressed(egui::Key::D),
            i.key_pressed(egui::Key::F),
        ));

        if do_undo {
            self.commands.undo(&mut self.editor_ctx);
        }
        if do_redo {
            self.commands.redo(&mut self.editor_ctx);
        }
        if do_delete {
            self.editor_ctx.delete_selected();
        }
        if do_play {
            self.editor_ctx.play_mode = !self.editor_ctx.play_mode;
            if self.editor_ctx.play_mode {
                self.editor_ctx.log_info("Play mode started (F5).");
            } else {
                self.editor_ctx.log_info("Play mode stopped (F5).");
            }
        }
        if do_duplicate {
            self.editor_ctx.duplicate_selected();
        }
        if do_focus {
            self.editor_ctx.focus_selected();
        }
    }

    fn update_stats(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.frame_count += 1;

        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };
        let frame_ms = dt * 1000.0;
        self.editor_ctx.update_stats(fps, frame_ms);
    }
}

impl App for ReactorEditor {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        Self::apply_dark_theme(ctx);
        self.update_stats();
        self.handle_keyboard_shortcuts(ctx);

        // ── Top toolbar (outside dock) ─────────────────────────────────────
        egui::TopBottomPanel::top("toolbar")
            .exact_height(32.0)
            .show(ctx, |ui| {
                self.toolbar.show(ui, &mut self.editor_ctx);
            });

        // ── Status bar (bottom) ────────────────────────────────────────────
        egui::TopBottomPanel::bottom("statusbar")
            .exact_height(20.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("⚛ REACTOR Editor  v0.1.0")
                            .color(Color32::from_rgb(100, 100, 100))
                            .small()
                    );
                    ui.separator();

                    // Undo/Redo status
                    if self.commands.can_undo() {
                        ui.label(
                            RichText::new(format!("Undo: {}  [Ctrl+Z]", self.commands.undo_description().unwrap_or("")))
                                .color(Color32::from_rgb(120, 180, 120))
                                .small()
                        );
                    }
                    if self.commands.can_redo() {
                        ui.label(
                            RichText::new(format!("Redo: {}  [Ctrl+Y]", self.commands.redo_description().unwrap_or("")))
                                .color(Color32::from_rgb(120, 120, 180))
                                .small()
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.editor_ctx.play_mode {
                            ui.label(
                                RichText::new("▶ PLAY MODE  [F5 to stop]")
                                    .color(Color32::from_rgb(255, 160, 40))
                                    .small()
                                    .strong()
                            );
                        } else {
                            ui.label(
                                RichText::new("EDITOR MODE  [F5 to play]")
                                    .color(Color32::from_rgb(100, 100, 100))
                                    .small()
                            );
                        }
                        ui.separator();
                        ui.label(
                            RichText::new(format!("Scene: {}  |  {} entities",
                                self.editor_ctx.scene.name,
                                self.editor_ctx.stats.entity_count
                            ))
                            .color(Color32::from_rgb(120, 120, 120))
                            .small()
                        );
                    });
                });
            });

        // ── Main dock area ─────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(20, 20, 24)))
            .show(ctx, |ui| {
                let style = Style::from_egui(ctx.style().as_ref());

                let mut tab_viewer = ReactorTabViewer {
                    ctx: &mut self.editor_ctx,
                    viewport: &mut self.viewport,
                    hierarchy: &mut self.hierarchy,
                    inspector: &mut self.inspector,
                    asset_browser: &mut self.asset_browser,
                    console: &mut self.console,
                };

                DockArea::new(&mut self.dock_state)
                    .style(style)
                    .show_inside(ui, &mut tab_viewer);
            });

        // Request continuous repaint for live FPS counter
        ctx.request_repaint();
    }
}

// =============================================================================
// Main
// =============================================================================

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("⚛ REACTOR Editor")
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "REACTOR Editor",
        options,
        Box::new(|cc| Ok(Box::new(ReactorEditor::new(cc)))),
    )
}
