// =============================================================================
// EditorContext — The heart of the REACTOR Editor
// =============================================================================
// Architecture: Engine (reactor lib) is independent. Editor is a client.
//   - OrbitCamera: Blender-style 3D navigation
//   - EditorScene: entity graph with components
//   - Selection, Assets, Console, Stats
// =============================================================================

use glam::{Vec2, Vec3, Vec4, Quat, Mat4};
use std::collections::HashMap;

// =============================================================================
// OrbitCamera — Blender/Maya-style 3D viewport camera
// =============================================================================

pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub orbit_speed: f32,
    pub pan_speed: f32,
    pub zoom_speed: f32,
}

impl OrbitCamera {
    pub fn new() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 8.0,
            yaw: 0.6,
            pitch: 0.4,
            fov: 45.0_f32.to_radians(),
            near: 0.01,
            far: 5000.0,
            orbit_speed: 0.005,
            pan_speed: 0.01,
            zoom_speed: 0.15,
        }
    }

    pub fn eye(&self) -> Vec3 {
        let cp = self.pitch.cos();
        let sp = self.pitch.sin();
        let cy = self.yaw.cos();
        let sy = self.yaw.sin();
        self.target + Vec3::new(cp * sy, sp, cp * cy) * self.distance
    }

    pub fn forward(&self) -> Vec3 {
        (self.target - self.eye()).normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        let mut proj = Mat4::perspective_rh(self.fov, aspect, self.near, self.far);
        proj.y_axis.y *= -1.0; // Vulkan Y-flip
        proj
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        self.projection_matrix(aspect) * self.view_matrix()
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * self.orbit_speed;
        self.pitch += dy * self.orbit_speed;
        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        let scale = self.distance * self.pan_speed;
        self.target += self.right() * (-dx * scale);
        self.target += self.up() * (dy * scale);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance *= 1.0 - delta * self.zoom_speed;
        self.distance = self.distance.clamp(0.1, 500.0);
    }

    pub fn focus_on(&mut self, pos: Vec3) {
        self.target = pos;
        self.distance = 5.0;
    }

    pub fn set_front(&mut self) { self.yaw = 0.0; self.pitch = 0.0; }
    pub fn set_back(&mut self) { self.yaw = std::f32::consts::PI; self.pitch = 0.0; }
    pub fn set_right(&mut self) { self.yaw = std::f32::consts::FRAC_PI_2; self.pitch = 0.0; }
    pub fn set_left(&mut self) { self.yaw = -std::f32::consts::FRAC_PI_2; self.pitch = 0.0; }
    pub fn set_top(&mut self) { self.yaw = 0.0; self.pitch = 1.5; }
    pub fn set_bottom(&mut self) { self.yaw = 0.0; self.pitch = -1.5; }

    /// Project 3D world point to 2D screen coordinates
    pub fn project(&self, world_pos: Vec3, viewport_size: Vec2) -> Option<Vec2> {
        let aspect = viewport_size.x / viewport_size.y;
        let vp = self.view_projection(aspect);
        let clip = vp * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
        if clip.w <= 0.0 { return None; }
        let ndc = Vec3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w);
        if ndc.z < 0.0 || ndc.z > 1.0 { return None; }
        Some(Vec2::new(
            (ndc.x * 0.5 + 0.5) * viewport_size.x,
            (ndc.y * 0.5 + 0.5) * viewport_size.y,
        ))
    }
}

// =============================================================================
// EntityId
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

// =============================================================================
// Components
// =============================================================================

#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self { position: Vec3::ZERO, rotation: Quat::IDENTITY, scale: Vec3::ONE }
    }
}

impl TransformComponent {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn euler_degrees(&self) -> Vec3 {
        let (y, x, z) = self.rotation.to_euler(glam::EulerRot::YXZ);
        Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
    }

    pub fn set_euler_degrees(&mut self, deg: Vec3) {
        self.rotation = Quat::from_euler(
            glam::EulerRot::YXZ,
            deg.y.to_radians(),
            deg.x.to_radians(),
            deg.z.to_radians(),
        );
    }
}

#[derive(Debug, Clone)]
pub struct MeshComponent {
    pub mesh_path: String,
    pub material_path: String,
    pub primitive: MeshPrimitive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MeshPrimitive {
    Cube,
    Sphere,
    Plane,
    Cylinder,
    Cone,
    Custom,
}

impl std::fmt::Display for MeshPrimitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cube => write!(f, "Cube"),
            Self::Sphere => write!(f, "Sphere"),
            Self::Plane => write!(f, "Plane"),
            Self::Cylinder => write!(f, "Cylinder"),
            Self::Cone => write!(f, "Cone"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LightComponent {
    pub light_type: LightType,
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32,
    pub spot_angle: f32,
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_type: LightType::Point,
            color: Vec3::ONE,
            intensity: 1.0,
            range: 10.0,
            spot_angle: 45.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LightType { Directional, Point, Spot }

impl std::fmt::Display for LightType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Directional => write!(f, "Directional"),
            Self::Point => write!(f, "Point"),
            Self::Spot => write!(f, "Spot"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CameraComponent {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub is_main: bool,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { fov: 60.0, near: 0.1, far: 1000.0, is_main: false }
    }
}

// =============================================================================
// EditorEntity
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
            id, name: name.into(),
            transform: TransformComponent::default(),
            mesh: None, light: None, camera: None,
            children: Vec::new(), parent: None,
            visible: true, locked: false,
        }
    }

    pub fn icon(&self) -> &'static str {
        if self.camera.is_some() { return "\u{1F3A5}"; }  // camera
        if self.light.is_some() { return "\u{1F4A1}"; }   // light
        if self.mesh.is_some() { return "\u{1F4E6}"; }    // mesh
        "\u{2B1C}"  // empty
    }
}

// =============================================================================
// EditorScene
// =============================================================================

#[derive(Debug, Clone)]
pub struct EditorScene {
    pub name: String,
    pub entities: HashMap<EntityId, EditorEntity>,
    pub root_entities: Vec<EntityId>,
    next_id: u64,
}

impl EditorScene {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), entities: HashMap::new(), root_entities: Vec::new(), next_id: 1 }
    }

    pub fn spawn(&mut self, name: impl Into<String>) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        self.entities.insert(id, EditorEntity::new(id, name));
        self.root_entities.push(id);
        id
    }

    pub fn get(&self, id: EntityId) -> Option<&EditorEntity> { self.entities.get(&id) }
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut EditorEntity> { self.entities.get_mut(&id) }

    pub fn remove(&mut self, id: EntityId) {
        // Remove children recursively
        if let Some(entity) = self.entities.get(&id) {
            let children: Vec<EntityId> = entity.children.clone();
            for child in children { self.remove(child); }
        }
        self.entities.remove(&id);
        self.root_entities.retain(|&e| e != id);
        // Remove from parent's children list
        let parent_ids: Vec<EntityId> = self.entities.keys().copied().collect();
        for pid in parent_ids {
            if let Some(parent) = self.entities.get_mut(&pid) {
                parent.children.retain(|&c| c != id);
            }
        }
    }

    pub fn all_entities(&self) -> impl Iterator<Item = &EditorEntity> { self.entities.values() }
    pub fn entity_count(&self) -> usize { self.entities.len() }
}

// =============================================================================
// Asset Registry
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum AssetType { Mesh, Texture, Material, Shader, Scene, Audio, Script }

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mesh => write!(f, "Mesh"), Self::Texture => write!(f, "Texture"),
            Self::Material => write!(f, "Material"), Self::Shader => write!(f, "Shader"),
            Self::Scene => write!(f, "Scene"), Self::Audio => write!(f, "Audio"),
            Self::Script => write!(f, "Script"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetEntry {
    pub name: String,
    pub path: String,
    pub asset_type: AssetType,
}

impl AssetEntry {
    pub fn icon(&self) -> &str {
        match self.asset_type {
            AssetType::Mesh => "\u{1F4E6}", AssetType::Texture => "\u{1F5BC}",
            AssetType::Material => "\u{1F3A8}", AssetType::Shader => "\u{26A1}",
            AssetType::Scene => "\u{1F30D}", AssetType::Audio => "\u{1F50A}",
            AssetType::Script => "\u{1F4DC}",
        }
    }
}

pub struct AssetRegistry {
    pub assets: Vec<AssetEntry>,
    pub current_folder: String,
}

impl AssetRegistry {
    pub fn new() -> Self {
        let mut r = Self { assets: Vec::new(), current_folder: "assets/".into() };
        r.assets.push(AssetEntry { name: "cube.obj".into(), path: "assets/models/cube.obj".into(), asset_type: AssetType::Mesh });
        r.assets.push(AssetEntry { name: "pyramid.obj".into(), path: "assets/models/pyramid.obj".into(), asset_type: AssetType::Mesh });
        r.assets.push(AssetEntry { name: "sphere.obj".into(), path: "assets/models/sphere.obj".into(), asset_type: AssetType::Mesh });
        r.assets.push(AssetEntry { name: "container.jpg".into(), path: "assets/textures/container.jpg".into(), asset_type: AssetType::Texture });
        r.assets.push(AssetEntry { name: "default.mat".into(), path: "assets/materials/default.mat".into(), asset_type: AssetType::Material });
        r.assets.push(AssetEntry { name: "vert.spv".into(), path: "shaders/vert.spv".into(), asset_type: AssetType::Shader });
        r.assets.push(AssetEntry { name: "frag.spv".into(), path: "shaders/frag.spv".into(), asset_type: AssetType::Shader });
        r
    }
}

// =============================================================================
// DragPayload
// =============================================================================

#[derive(Debug, Clone)]
pub struct DragPayload {
    pub asset_name: String,
    pub asset_type: AssetType,
}

// =============================================================================
// Console
// =============================================================================

#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    pub level: LogLevel,
    pub message: String,
    pub frame: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel { Info, Warning, Error }

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Info => write!(f, "INFO"), Self::Warning => write!(f, "WARN"), Self::Error => write!(f, "ERR") }
    }
}

// =============================================================================
// GizmoMode
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoMode { Select, Translate, Rotate, Scale }

// =============================================================================
// EditorStats
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct EditorStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub entity_count: usize,
    pub triangles: u64,
    pub draw_calls: u32,
    pub vram_mb: f32,
}

// =============================================================================
// EditorContext — Master state
// =============================================================================

pub struct EditorContext {
    pub scene: EditorScene,
    pub camera: OrbitCamera,
    pub selected: Option<EntityId>,
    pub multi_selected: Vec<EntityId>,
    pub assets: AssetRegistry,
    pub console_log: Vec<ConsoleEntry>,
    pub drag_payload: Option<DragPayload>,
    pub gizmo_mode: GizmoMode,
    pub gizmo_space: GizmoSpace,
    pub play_mode: bool,
    pub stats: EditorStats,
    pub frame_count: u64,
    pub show_grid: bool,
    pub show_wireframe: bool,
    pub show_bounds: bool,
    pub snap_translate: f32,
    pub snap_rotate: f32,
    pub snap_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoSpace { Local, World }

impl EditorContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            scene: EditorScene::new("Untitled Scene"),
            camera: OrbitCamera::new(),
            selected: None,
            multi_selected: Vec::new(),
            assets: AssetRegistry::new(),
            console_log: Vec::new(),
            drag_payload: None,
            gizmo_mode: GizmoMode::Select,
            gizmo_space: GizmoSpace::World,
            play_mode: false,
            stats: EditorStats::default(),
            frame_count: 0,
            show_grid: true,
            show_wireframe: false,
            show_bounds: false,
            snap_translate: 0.0,
            snap_rotate: 0.0,
            snap_scale: 0.0,
        };
        ctx.build_default_scene();
        ctx.log(LogLevel::Info, "REACTOR Editor v0.1.0 initialized");
        ctx.log(LogLevel::Info, "Vulkan backend: REACTOR Framework");
        ctx
    }

    fn build_default_scene(&mut self) {
        // Camera
        let id = self.scene.spawn("Main Camera");
        if let Some(e) = self.scene.get_mut(id) {
            e.transform.position = Vec3::new(0.0, 2.0, 5.0);
            e.camera = Some(CameraComponent { is_main: true, ..Default::default() });
        }

        // Directional Light
        let id = self.scene.spawn("Directional Light");
        if let Some(e) = self.scene.get_mut(id) {
            e.transform.position = Vec3::new(5.0, 8.0, 3.0);
            e.transform.rotation = Quat::from_euler(glam::EulerRot::YXZ, -0.5, -0.8, 0.0);
            e.light = Some(LightComponent {
                light_type: LightType::Directional,
                color: Vec3::new(1.0, 0.98, 0.92),
                intensity: 1.2,
                ..Default::default()
            });
        }

        // Cube
        let id = self.scene.spawn("Cube");
        if let Some(e) = self.scene.get_mut(id) {
            e.mesh = Some(MeshComponent {
                mesh_path: "primitives://cube".into(),
                material_path: "default".into(),
                primitive: MeshPrimitive::Cube,
            });
        }

        // Floor
        let id = self.scene.spawn("Floor");
        if let Some(e) = self.scene.get_mut(id) {
            e.transform.position = Vec3::new(0.0, -0.5, 0.0);
            e.transform.scale = Vec3::new(10.0, 0.05, 10.0);
            e.mesh = Some(MeshComponent {
                mesh_path: "primitives://cube".into(),
                material_path: "floor".into(),
                primitive: MeshPrimitive::Cube,
            });
        }

        // Point Light
        let id = self.scene.spawn("Point Light");
        if let Some(e) = self.scene.get_mut(id) {
            e.transform.position = Vec3::new(-2.0, 3.0, 1.0);
            e.light = Some(LightComponent {
                light_type: LightType::Point,
                color: Vec3::new(0.4, 0.6, 1.0),
                intensity: 2.0,
                range: 15.0,
                ..Default::default()
            });
        }

        // Sphere
        let id = self.scene.spawn("Sphere");
        if let Some(e) = self.scene.get_mut(id) {
            e.transform.position = Vec3::new(3.0, 0.5, 0.0);
            e.mesh = Some(MeshComponent {
                mesh_path: "primitives://sphere".into(),
                material_path: "default".into(),
                primitive: MeshPrimitive::Sphere,
            });
        }
    }

    // Logging
    pub fn log(&mut self, level: LogLevel, msg: impl Into<String>) {
        self.console_log.push(ConsoleEntry { level, message: msg.into(), frame: self.frame_count });
    }

    pub fn log_info(&mut self, msg: impl Into<String>) { self.log(LogLevel::Info, msg); }
    pub fn log_warn(&mut self, msg: impl Into<String>) { self.log(LogLevel::Warning, msg); }
    pub fn log_error(&mut self, msg: impl Into<String>) { self.log(LogLevel::Error, msg); }

    // Selection
    pub fn select(&mut self, id: Option<EntityId>) {
        self.selected = id;
        self.multi_selected.clear();
        if let Some(id) = id { self.multi_selected.push(id); }
    }

    pub fn toggle_select(&mut self, id: EntityId) {
        if let Some(pos) = self.multi_selected.iter().position(|&e| e == id) {
            self.multi_selected.remove(pos);
            self.selected = self.multi_selected.last().copied();
        } else {
            self.multi_selected.push(id);
            self.selected = Some(id);
        }
    }

    // Entity operations
    pub fn spawn_entity(&mut self, name: impl Into<String>) -> EntityId {
        let n: String = name.into();
        let id = self.scene.spawn(n.clone());
        self.log_info(format!("Created: {}", n));
        id
    }

    pub fn spawn_cube(&mut self) -> EntityId {
        let id = self.spawn_entity("Cube");
        if let Some(e) = self.scene.get_mut(id) {
            e.mesh = Some(MeshComponent {
                mesh_path: "primitives://cube".into(),
                material_path: "default".into(),
                primitive: MeshPrimitive::Cube,
            });
        }
        id
    }

    pub fn spawn_sphere(&mut self) -> EntityId {
        let id = self.spawn_entity("Sphere");
        if let Some(e) = self.scene.get_mut(id) {
            e.mesh = Some(MeshComponent {
                mesh_path: "primitives://sphere".into(),
                material_path: "default".into(),
                primitive: MeshPrimitive::Sphere,
            });
        }
        id
    }

    pub fn spawn_light(&mut self, lt: LightType) -> EntityId {
        let name = format!("{} Light", lt);
        let id = self.spawn_entity(name);
        if let Some(e) = self.scene.get_mut(id) {
            e.transform.position = Vec3::new(0.0, 3.0, 0.0);
            e.light = Some(LightComponent { light_type: lt, ..Default::default() });
        }
        id
    }

    pub fn spawn_camera(&mut self) -> EntityId {
        let id = self.spawn_entity("Camera");
        if let Some(e) = self.scene.get_mut(id) {
            e.transform.position = Vec3::new(0.0, 2.0, 5.0);
            e.camera = Some(CameraComponent::default());
        }
        id
    }

    pub fn delete_selected(&mut self) {
        if let Some(id) = self.selected.take() {
            if let Some(e) = self.scene.get(id) {
                self.log_info(format!("Deleted: {}", e.name));
            }
            self.scene.remove(id);
            self.multi_selected.retain(|&e| e != id);
        }
    }

    pub fn duplicate_selected(&mut self) {
        if let Some(id) = self.selected {
            if let Some(e) = self.scene.get(id).cloned() {
                let new_id = self.scene.spawn(format!("{} (Copy)", e.name));
                if let Some(new_e) = self.scene.get_mut(new_id) {
                    new_e.transform = e.transform.clone();
                    new_e.transform.position += Vec3::new(1.0, 0.0, 0.0);
                    new_e.mesh = e.mesh.clone();
                    new_e.light = e.light.clone();
                    new_e.camera = e.camera.clone();
                }
                self.select(Some(new_id));
                self.log_info(format!("Duplicated: {}", e.name));
            }
        }
    }

    pub fn focus_selected(&mut self) {
        if let Some(id) = self.selected {
            if let Some(e) = self.scene.get(id) {
                self.camera.focus_on(e.transform.position);
            }
        }
    }

    pub fn update_stats(&mut self, fps: f32, frame_time_ms: f32) {
        self.stats.fps = fps;
        self.stats.frame_time_ms = frame_time_ms;
        self.stats.entity_count = self.scene.entity_count();
        self.frame_count += 1;
    }
}
