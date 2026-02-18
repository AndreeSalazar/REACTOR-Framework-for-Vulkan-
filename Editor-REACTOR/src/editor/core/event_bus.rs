// =============================================================================
// EventBus — Decoupled editor event system
// =============================================================================

use crate::editor::core::editor_context::EntityId;

// =============================================================================
// Editor Events
// =============================================================================

#[derive(Debug, Clone)]
pub enum EditorEvent {
    EntitySelected(EntityId),
    EntityDeselected,
    EntitySpawned(EntityId),
    EntityDeleted(EntityId),
    EntityTransformChanged(EntityId),
    SceneLoaded(String),
    SceneSaved(String),
    PlayModeEntered,
    PlayModeStopped,
    AssetDroppedOnViewport { asset_path: String, x: f32, y: f32 },
    ConsoleMessage(String),
}

// =============================================================================
// EventBus
// =============================================================================

pub struct EventBus {
    queue: Vec<EditorEvent>,
    history: Vec<EditorEvent>,
    max_history: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            history: Vec::new(),
            max_history: 256,
        }
    }

    pub fn emit(&mut self, event: EditorEvent) {
        self.queue.push(event);
    }

    pub fn drain(&mut self) -> Vec<EditorEvent> {
        let events = self.queue.drain(..).collect::<Vec<_>>();
        for e in &events {
            self.history.push(e.clone());
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
        events
    }

    pub fn has_events(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn history(&self) -> &[EditorEvent] {
        &self.history
    }
}
