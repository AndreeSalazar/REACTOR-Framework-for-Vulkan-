// =============================================================================
// EditorContext — The heart of the REACTOR Editor
// =============================================================================
// Equivalent to Unreal's EditorWorld. Controls everything:
//   - Scene state
//   - Selection
//   - Asset registry
//   - Command stack (undo/redo)
//   - Event bus
// =============================================================================

use glam::{Vec3, Quat, Mat4};
use std::collections::HashMap;

// =============================================================================
// Entity ID
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

// =============================================================================
// Component Types
// =============================================================================

#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl TransformComponent {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

#[derive(Debug, Clone)]
pub struct MeshComponent {
    pub mesh_path: String,
    pub material_path: String,
}

#[derive(Debug, Clone)]
pub struct LightComponent {
    pub light_type: LightType,
    pub color: Vec3,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

impl std::fmt::Display for LightType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LightType::Directional => write!(f, "Directional"),
            LightType::Point => write!(f, "Point"),
            LightType::Spot => write!(f, "Spot"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CameraComponent {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { fov: 60.0, near: 0.1, far: 1000.0 }
    }
}

// =============================================================================
// Entity — Scene node with components
// =============================================================================

#[derive(Debug, Clone)]
pub struct EditorEntity {
    pub id: EntityId,
    pub name: String,
    pub transform: TransformComponent,
    pub mesh: Option<MeshComponent>,
    pub light: Option<LightComponent>,
    pub camera: Option<CameraComponent>,
    pub children: Vec<EntityId>,
    pub parent: Option<EntityId>,
    pub visible: bool,
    pub locked: bool,
}

impl EditorEntity {
    pub fn new(id: EntityId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            transform: TransformComponent::default(),
            mesh: None,
            light: None,
            camera: None,
            children: Vec::new(),
            parent: None,
            visible: true,
            locked: false,
        }
    }

    pub fn with_mesh(mut self, mesh_path: impl Into<String>, material_path: impl Into<String>) -> Self {
        self.mesh = Some(MeshComponent {
            mesh_path: mesh_path.into(),
            material_path: material_path.into(),
        });
        self
    }

    pub fn with_light(mut self, light_type: LightType, color: Vec3, intensity: f32) -> Self {
        self.light = Some(LightComponent { light_type, color, intensity });
        self
    }

    pub fn with_camera(mut self) -> Self {
        self.camera = Some(CameraComponent::default());
        self
    }

    pub fn component_list(&self) -> Vec<String> {
        let mut list = vec!["Transform".to_string()];
        if self.mesh.is_some() { list.push("Mesh Renderer".to_string()); }
        if self.light.is_some() { list.push("Light".to_string()); }
        if self.camera.is_some() { list.push("Camera".to_string()); }
        list
    }
}

// =============================================================================
// Scene — Collection of entities
// =============================================================================

#[derive(Debug, Clone)]
pub struct EditorScene {
    pub name: String,
    entities: HashMap<EntityId, EditorEntity>,
    root_entities: Vec<EntityId>,
    next_id: u64,
}

impl EditorScene {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entities: HashMap::new(),
            root_entities: Vec::new(),
            next_id: 1,
        }
    }

    pub fn spawn(&mut self, name: impl Into<String>) -> EntityId {
        let id = EntityId::new(self.next_id);
        self.next_id += 1;
        let entity = EditorEntity::new(id, name);
        self.entities.insert(id, entity);
        self.root_entities.push(id);
        id
    }

    pub fn get(&self, id: EntityId) -> Option<&EditorEntity> {
        self.entities.get(&id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut EditorEntity> {
        self.entities.get_mut(&id)
    }

    pub fn remove(&mut self, id: EntityId) {
        self.entities.remove(&id);
        self.root_entities.retain(|&e| e != id);
    }

    pub fn root_entities(&self) -> &[EntityId] {
        &self.root_entities
    }

    pub fn all_entities(&self) -> impl Iterator<Item = &EditorEntity> {
        self.entities.values()
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

// =============================================================================
// Asset Registry
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Shader,
    Scene,
    Audio,
    Script,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::Mesh => write!(f, "Mesh"),
            AssetType::Texture => write!(f, "Texture"),
            AssetType::Material => write!(f, "Material"),
            AssetType::Shader => write!(f, "Shader"),
            AssetType::Scene => write!(f, "Scene"),
            AssetType::Audio => write!(f, "Audio"),
            AssetType::Script => write!(f, "Script"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetEntry {
    pub name: String,
    pub path: String,
    pub asset_type: AssetType,
    pub size_bytes: u64,
}

impl AssetEntry {
    pub fn new(name: impl Into<String>, path: impl Into<String>, asset_type: AssetType) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            asset_type,
            size_bytes: 0,
        }
    }

    pub fn icon(&self) -> &str {
        match self.asset_type {
            AssetType::Mesh => "📦",
            AssetType::Texture => "🖼",
            AssetType::Material => "🎨",
            AssetType::Shader => "⚡",
            AssetType::Scene => "🌍",
            AssetType::Audio => "🔊",
            AssetType::Script => "📜",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetRegistry {
    pub assets: Vec<AssetEntry>,
    pub current_folder: String,
}

impl AssetRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            assets: Vec::new(),
            current_folder: "assets/".to_string(),
        };
        registry.populate_defaults();
        registry
    }

    fn populate_defaults(&mut self) {
        self.assets.push(AssetEntry::new("cube.obj", "assets/models/cube.obj", AssetType::Mesh));
        self.assets.push(AssetEntry::new("pyramid.obj", "assets/models/pyramid.obj", AssetType::Mesh));
        self.assets.push(AssetEntry::new("container.jpg", "assets/textures/container.jpg", AssetType::Texture));
        self.assets.push(AssetEntry::new("vert.spv", "shaders/vert.spv", AssetType::Shader));
        self.assets.push(AssetEntry::new("frag.spv", "shaders/frag.spv", AssetType::Shader));
        self.assets.push(AssetEntry::new("texture_vert.spv", "shaders/texture_vert.spv", AssetType::Shader));
        self.assets.push(AssetEntry::new("texture_frag.spv", "shaders/texture_frag.spv", AssetType::Shader));
    }

    pub fn filtered(&self, filter: &str) -> Vec<&AssetEntry> {
        if filter.is_empty() {
            self.assets.iter().collect()
        } else {
            self.assets.iter()
                .filter(|a| a.name.to_lowercase().contains(&filter.to_lowercase()))
                .collect()
        }
    }
}

// =============================================================================
// EditorContext — Master state
// =============================================================================

pub struct EditorContext {
    pub scene: EditorScene,
    pub selected_entity: Option<EntityId>,
    pub assets: AssetRegistry,
    pub console_log: Vec<ConsoleEntry>,
    pub drag_payload: Option<DragPayload>,
    pub editor_mode: EditorMode,
    pub play_mode: bool,
    pub stats: EditorStats,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorMode {
    Select,
    Translate,
    Rotate,
    Scale,
}

#[derive(Debug, Clone)]
pub struct DragPayload {
    pub asset_path: String,
    pub asset_type: AssetType,
    pub asset_name: String,
}

#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EditorStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub entity_count: usize,
    pub draw_calls: u32,
    pub triangles: u64,
}

impl EditorContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            scene: EditorScene::new("Untitled Scene"),
            selected_entity: None,
            assets: AssetRegistry::new(),
            console_log: Vec::new(),
            drag_payload: None,
            editor_mode: EditorMode::Select,
            play_mode: false,
            stats: EditorStats::default(),
        };
        ctx.populate_default_scene();
        ctx.log_info("REACTOR Editor initialized.");
        ctx.log_info("Vulkan backend ready.");
        ctx
    }

    fn populate_default_scene(&mut self) {
        let cam_id = self.scene.spawn("Main Camera");
        if let Some(e) = self.scene.get_mut(cam_id) {
            e.transform.position = Vec3::new(0.0, 2.0, 5.0);
            e.camera = Some(CameraComponent::default());
        }

        let light_id = self.scene.spawn("Directional Light");
        if let Some(e) = self.scene.get_mut(light_id) {
            e.transform.position = Vec3::new(5.0, 10.0, 5.0);
            e.light = Some(LightComponent {
                light_type: LightType::Directional,
                color: Vec3::new(1.0, 0.98, 0.95),
                intensity: 1.0,
            });
        }

        let cube_id = self.scene.spawn("Cube");
        if let Some(e) = self.scene.get_mut(cube_id) {
            e.transform.position = Vec3::ZERO;
            e.mesh = Some(MeshComponent {
                mesh_path: "assets/models/cube.obj".to_string(),
                material_path: "assets/materials/default.mat".to_string(),
            });
        }

        let floor_id = self.scene.spawn("Floor");
        if let Some(e) = self.scene.get_mut(floor_id) {
            e.transform.position = Vec3::new(0.0, -1.0, 0.0);
            e.transform.scale = Vec3::new(10.0, 0.1, 10.0);
            e.mesh = Some(MeshComponent {
                mesh_path: "assets/models/cube.obj".to_string(),
                material_path: "assets/materials/floor.mat".to_string(),
            });
        }
    }

    pub fn log_info(&mut self, msg: impl Into<String>) {
        self.console_log.push(ConsoleEntry {
            level: LogLevel::Info,
            message: msg.into(),
            timestamp: 0.0,
        });
    }

    pub fn log_warn(&mut self, msg: impl Into<String>) {
        self.console_log.push(ConsoleEntry {
            level: LogLevel::Warning,
            message: msg.into(),
            timestamp: 0.0,
        });
    }

    pub fn log_error(&mut self, msg: impl Into<String>) {
        self.console_log.push(ConsoleEntry {
            level: LogLevel::Error,
            message: msg.into(),
            timestamp: 0.0,
        });
    }

    pub fn select(&mut self, id: Option<EntityId>) {
        self.selected_entity = id;
    }

    pub fn spawn_entity(&mut self, name: impl Into<String>) -> EntityId {
        let id = self.scene.spawn(name);
        self.log_info(format!("Spawned entity: {}", id));
        id
    }

    pub fn delete_selected(&mut self) {
        if let Some(id) = self.selected_entity.take() {
            self.scene.remove(id);
            self.log_info(format!("Deleted entity: {}", id));
        }
    }

    pub fn update_stats(&mut self, fps: f32, frame_time_ms: f32) {
        self.stats.fps = fps;
        self.stats.frame_time_ms = frame_time_ms;
        self.stats.entity_count = self.scene.entity_count();
    }
}
