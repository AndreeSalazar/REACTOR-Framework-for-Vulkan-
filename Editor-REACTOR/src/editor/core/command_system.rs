// =============================================================================
// CommandSystem — Undo/Redo stack
// =============================================================================

use crate::editor::core::editor_context::{EditorContext, EntityId, TransformComponent};
use glam::Vec3;

// =============================================================================
// Command trait
// =============================================================================

pub trait Command: std::fmt::Debug {
    fn execute(&mut self, ctx: &mut EditorContext);
    fn undo(&mut self, ctx: &mut EditorContext);
    fn description(&self) -> &str;
}

// =============================================================================
// Concrete Commands
// =============================================================================

#[derive(Debug)]
pub struct MoveEntityCommand {
    pub entity_id: EntityId,
    pub old_position: Vec3,
    pub new_position: Vec3,
}

impl Command for MoveEntityCommand {
    fn execute(&mut self, ctx: &mut EditorContext) {
        if let Some(e) = ctx.scene.get_mut(self.entity_id) {
            e.transform.position = self.new_position;
        }
    }

    fn undo(&mut self, ctx: &mut EditorContext) {
        if let Some(e) = ctx.scene.get_mut(self.entity_id) {
            e.transform.position = self.old_position;
        }
    }

    fn description(&self) -> &str {
        "Move Entity"
    }
}

#[derive(Debug)]
pub struct SpawnEntityCommand {
    pub name: String,
    pub spawned_id: Option<EntityId>,
}

impl Command for SpawnEntityCommand {
    fn execute(&mut self, ctx: &mut EditorContext) {
        let id = ctx.scene.spawn(self.name.clone());
        self.spawned_id = Some(id);
        ctx.log_info(format!("Spawned: {}", self.name));
    }

    fn undo(&mut self, ctx: &mut EditorContext) {
        if let Some(id) = self.spawned_id.take() {
            ctx.scene.remove(id);
            ctx.log_info(format!("Undo spawn: {}", self.name));
        }
    }

    fn description(&self) -> &str {
        "Spawn Entity"
    }
}

#[derive(Debug)]
pub struct DeleteEntityCommand {
    pub entity_id: EntityId,
    pub saved_name: String,
    pub saved_transform: Option<TransformComponent>,
}

impl Command for DeleteEntityCommand {
    fn execute(&mut self, ctx: &mut EditorContext) {
        if let Some(e) = ctx.scene.get(self.entity_id) {
            self.saved_name = e.name.clone();
            self.saved_transform = Some(e.transform.clone());
        }
        ctx.scene.remove(self.entity_id);
        ctx.log_info(format!("Deleted: {}", self.saved_name));
    }

    fn undo(&mut self, ctx: &mut EditorContext) {
        let id = ctx.scene.spawn(self.saved_name.clone());
        if let (Some(transform), Some(e)) = (self.saved_transform.clone(), ctx.scene.get_mut(id)) {
            e.transform = transform;
        }
        ctx.log_info(format!("Restored: {}", self.saved_name));
    }

    fn description(&self) -> &str {
        "Delete Entity"
    }
}

// =============================================================================
// CommandSystem — the stack
// =============================================================================

pub struct CommandSystem {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    max_history: usize,
}

impl CommandSystem {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
        }
    }

    pub fn execute(&mut self, mut cmd: Box<dyn Command>, ctx: &mut EditorContext) {
        cmd.execute(ctx);
        self.undo_stack.push(cmd);
        self.redo_stack.clear();

        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self, ctx: &mut EditorContext) {
        if let Some(mut cmd) = self.undo_stack.pop() {
            ctx.log_info(format!("Undo: {}", cmd.description()));
            cmd.undo(ctx);
            self.redo_stack.push(cmd);
        }
    }

    pub fn redo(&mut self, ctx: &mut EditorContext) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            ctx.log_info(format!("Redo: {}", cmd.description()));
            cmd.execute(ctx);
            self.undo_stack.push(cmd);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.description())
    }

    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.description())
    }

    pub fn history_count(&self) -> usize {
        self.undo_stack.len()
    }
}
