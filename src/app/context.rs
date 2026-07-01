use std::sync::Arc;
use winit::window::Window;

use crate::app::config::ReactorConfig;
use crate::platform::input::Input;
use crate::platform::time::Time;
use crate::reactor::Reactor;
use crate::resources::{
    AssetDatabase, AssetHotReloadManager, AssetId, AssetLoaderQueue, AssetManager,
    GltfLoader, Handle,
};

pub struct ReactorContext {
    pub window: Arc<Window>,
    pub time: Time,
    pub config: ReactorConfig,
    pub camera: crate::scene::camera::Camera,
    pub scene: crate::systems::scene::Scene,
    pub lighting: crate::systems::lighting::LightingSystem,
    pub physics: crate::systems::physics::PhysicsWorld,
    pub culling: crate::systems::frustum::CullingSystem,
    pub debug: crate::graphics::debug_renderer::DebugRenderer,
    pub asset_manager: AssetManager,
    pub gltf_loader: GltfLoader,
    pub asset_db: AssetDatabase,
    pub asset_hot_reload: Option<AssetHotReloadManager>,
    pub asset_loader_queue: AssetLoaderQueue,
    pub audio: crate::systems::audio::AudioSystem,
    pub event_bus: crate::systems::event_bus::EventBus,
    pub(crate) hot_reload_rx: Option<
        tokio::sync::mpsc::UnboundedReceiver<crate::resources::asset_hot_reload::AssetReloadEvent>,
    >,
    pub(crate) blob_shadow_mesh: Option<Arc<crate::resources::mesh::Mesh>>,
    pub(crate) blob_shadow_material: Option<Arc<crate::resources::material::Material>>,
    pub reactor: Reactor,
    pub(crate) fixed_accumulator: f32,
}

impl Drop for ReactorContext {
    fn drop(&mut self) {
        self.scene.objects.clear();
        self.scene.lights.clear();
        self.blob_shadow_mesh = None;
        self.blob_shadow_material = None;
        self.asset_manager.clear();
        self.asset_hot_reload = None;
        self.hot_reload_rx = None;
    }
}

// ── Context-type structs ─────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct GltfBounds {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
    pub center: glam::Vec3,
    pub size: glam::Vec3,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct GltfSpawn {
    pub position: glam::Vec3,
    pub target_height: Option<f32>,
    pub face_direction: Option<glam::Vec3>,
    pub feet_at_position: bool,
}

impl Default for GltfSpawn {
    fn default() -> Self {
        Self { position: glam::Vec3::ZERO, target_height: None, face_direction: None, feet_at_position: true }
    }
}

impl GltfSpawn {
    pub fn at(position: glam::Vec3) -> Self { Self { position, ..Default::default() } }
    pub fn with_height(mut self, h: f32) -> Self { self.target_height = Some(h); self }
    pub fn facing(mut self, dir: glam::Vec3) -> Self { self.face_direction = Some(dir); self }
    pub fn with_pivot_at_position(mut self) -> Self { self.feet_at_position = false; self }
}

#[derive(Clone, Debug)]
pub struct ModelSpawnInfo {
    pub indices: Vec<usize>,
    pub applied_scale: f32,
    pub applied_rotation: glam::Quat,
    pub native_height: f32,
    pub world_height: f32,
    pub world_bounds_min: glam::Vec3,
    pub world_bounds_max: glam::Vec3,
}

#[derive(Clone, Debug)]
pub struct AssetPipelineStats {
    pub loader_queue: crate::resources::LoaderStats,
    pub hot_reload: Option<crate::resources::HotReloadStats>,
    pub db: crate::resources::AssetDbStats,
    pub gltf_cache: crate::resources::GltfCacheStats,
}

impl ReactorContext {
    pub fn input(&self) -> &Input { &self.reactor.input }

    pub fn aspect_ratio(&self) -> f32 {
        let size = self.window.inner_size();
        if size.height == 0 { return 1.0; }
        size.width as f32 / size.height as f32
    }
    pub fn window_size(&self) -> (u32, u32) { let s = self.window.inner_size(); (s.width, s.height) }
    pub fn set_title(&self, title: &str) { self.window.set_title(title); }

    pub fn create_mesh(&self, vertices: &[crate::resources::vertex::Vertex], indices: &[u32])
        -> crate::core::error::ReactorResult<crate::resources::mesh::Mesh> {
        self.reactor.create_mesh(vertices, indices).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }
    pub fn create_material(&self, vert_code: &[u32], frag_code: &[u32])
        -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        self.reactor.create_material(vert_code, frag_code).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }
    pub fn load_texture(&self, path: &str)
        -> crate::core::error::ReactorResult<crate::resources::texture::Texture> {
        self.reactor.load_texture(path).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }
    pub fn load_texture_bytes(&self, bytes: &[u8])
        -> crate::core::error::ReactorResult<crate::resources::texture::Texture> {
        self.reactor.load_texture_bytes(bytes).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }
    pub fn create_solid_texture(&self, r: u8, g: u8, b: u8, a: u8)
        -> crate::core::error::ReactorResult<crate::resources::texture::Texture> {
        self.reactor.create_solid_texture(r, g, b, a).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }
    pub fn create_textured_material(&self, vert_code: &[u32], frag_code: &[u32], texture: &crate::resources::texture::Texture)
        -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        self.reactor.create_textured_material(vert_code, frag_code, texture).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }
    pub fn base_shader_cookbook(&self) -> crate::base_shader::BaseShaderCookbook {
        crate::base_shader::BaseShaderCookbook::default()
    }
    pub fn apply_base_shader(&mut self, cookbook: &crate::base_shader::BaseShaderCookbook) {
        cookbook.apply_to_post_process(&mut self.reactor.post_process);
    }
    pub fn create_base_material(&self, cookbook: &crate::base_shader::BaseShaderCookbook)
        -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        self.create_material(&cookbook.forward.vertex, &cookbook.forward.fragment)
    }
    pub fn create_base_textured_material(&self, cookbook: &crate::base_shader::BaseShaderCookbook, texture: &crate::resources::texture::Texture)
        -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        self.create_textured_material(&cookbook.textured.vertex, &cookbook.textured.fragment, texture)
    }
    pub fn create_base_pbr_material(&self, cookbook: &crate::base_shader::BaseShaderCookbook, ibl_set_layout: ash::vk::DescriptorSetLayout,
        albedo: &crate::resources::texture::Texture, normal: &crate::resources::texture::Texture,
        metallic: &crate::resources::texture::Texture, roughness: &crate::resources::texture::Texture)
        -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        self.reactor.create_pbr_material(&cookbook.blender_live_pbr.vertex, &cookbook.blender_live_pbr.fragment,
            ibl_set_layout, albedo, normal, metallic, roughness)
            .map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }

    pub fn load_obj(&self, path: &str) -> crate::core::error::ReactorResult<crate::resources::mesh::Mesh> {
        use crate::resources::model::ObjData;
        let obj = ObjData::load(path).map_err(|_e| crate::core::error::ReactorError::file_not_found(path))?;
        if obj.vertices.is_empty() { return Err(crate::core::error::ReactorError::invalid_format("OBJ file contains no vertices")); }
        println!("📦 Loaded OBJ: {} vertices, {} triangles", obj.vertex_count(), obj.triangle_count());
        self.reactor.create_mesh(&obj.vertices, &obj.indices).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }
    pub fn load_obj_with_material(&mut self, path: &str, material: Arc<crate::resources::material::Material>)
        -> crate::core::error::ReactorResult<u32> {
        let mesh = Arc::new(self.load_obj(path)?);
        let index = self.scene.objects.len() as u32;
        self.scene.add_object(mesh, material, glam::Mat4::IDENTITY);
        Ok(index)
    }

    pub fn render_scene(&mut self) {
        self.camera.set_aspect_ratio(self.window.inner_size().width as f32, self.window.inner_size().height as f32);
        let vp = self.camera.view_projection_matrix();
        self.reactor.camera_pos = self.camera.position;
        self.reactor.camera_view = self.camera.view_matrix();
        self.reactor.camera_proj = self.camera.projection_matrix();
        self.reactor.camera_near = self.camera.near;
        self.reactor.camera_far = self.camera.far;
        self.reactor.post_process.update_time(self.time.elapsed());
        if let Err(e) = self.reactor.draw_scene(&self.scene, &vp) { eprintln!("REACTOR draw error: {}", e); }
    }
    pub fn draw_scene_with_vp(&mut self, view_projection: &glam::Mat4) {
        self.reactor.camera_pos = self.camera.position;
        self.reactor.camera_view = self.camera.view_matrix();
        self.reactor.camera_proj = self.camera.projection_matrix();
        self.reactor.camera_near = self.camera.near;
        self.reactor.camera_far = self.camera.far;
        self.reactor.post_process.update_time(self.time.elapsed());
        if let Err(e) = self.reactor.draw_scene(&self.scene, view_projection) { eprintln!("REACTOR draw error: {}", e); }
    }
    pub fn draw_scene(&mut self, scene: &crate::systems::scene::Scene, view_projection: &glam::Mat4) {
        self.reactor.camera_pos = self.camera.position;
        self.reactor.camera_view = self.camera.view_matrix();
        self.reactor.camera_proj = self.camera.projection_matrix();
        self.reactor.camera_near = self.camera.near;
        self.reactor.camera_far = self.camera.far;
        self.reactor.post_process.update_time(self.time.elapsed());
        if let Err(e) = self.reactor.draw_scene(scene, view_projection) { eprintln!("REACTOR draw error: {}", e); }
    }
    pub fn draw(&mut self, mesh: &crate::resources::mesh::Mesh, material: &crate::resources::material::Material, transform: &glam::Mat4) {
        if let Err(e) = self.reactor.draw_frame(mesh, material, transform) { eprintln!("REACTOR draw error: {}", e); }
    }

    pub fn delta(&self) -> f32 { self.time.delta() }
    pub fn fps(&self) -> f32 { self.time.fps() }
    pub fn elapsed(&self) -> f32 { self.time.elapsed() }

    pub fn look_at(&mut self, eye: glam::Vec3, target: glam::Vec3) -> &mut Self { self.camera.aim_at(eye, target); self }
    pub fn move_camera_to(&mut self, position: glam::Vec3) -> &mut Self { self.camera.position = position; self }

    pub fn add_sun(&mut self) -> usize { self.lighting.add_light(crate::systems::lighting::Light::sun()) }
    pub fn add_directional_light(&mut self, direction: glam::Vec3, color: glam::Vec3, intensity: f32) -> usize {
        self.lighting.add_light(crate::systems::lighting::Light::directional(direction, color, intensity))
    }
    pub fn add_point_light(&mut self, position: glam::Vec3, color: glam::Vec3, intensity: f32, range: f32) -> usize {
        self.lighting.add_light(crate::systems::lighting::Light::point(position, color, intensity, range))
    }
    pub fn add_spot_light(&mut self, position: glam::Vec3, direction: glam::Vec3, color: glam::Vec3, intensity: f32, range: f32, angle_degrees: f32) -> usize {
        self.lighting.add_light(crate::systems::lighting::Light::spot(position, direction, color, intensity, range, angle_degrees))
    }

    pub fn spawn(&mut self, mesh: Arc<crate::resources::mesh::Mesh>, material: Arc<crate::resources::material::Material>, transform: glam::Mat4) -> usize {
        self.scene.add_object(mesh, material, transform)
    }
    pub fn set_transform(&mut self, index: usize, transform: glam::Mat4) {
        if let Some(obj) = self.scene.objects.get_mut(index) { obj.transform = transform; }
    }
    pub fn get_transform(&self, index: usize) -> Option<glam::Mat4> {
        self.scene.objects.get(index).map(|obj| obj.transform)
    }

    pub fn default_material(&self) -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        self.create_base_material(&self.base_shader_cookbook())
    }
    pub fn spawn_cube(&mut self, position: glam::Vec3) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::cube();
        self.spawn_primitive(&v, &i, glam::Mat4::from_translation(position))
    }
    pub fn spawn_sphere(&mut self, position: glam::Vec3, _radius: f32) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::sphere(32, 16);
        let xf = glam::Mat4::from_scale_rotation_translation(glam::Vec3::splat(_radius.max(0.001)), glam::Quat::IDENTITY, position);
        self.spawn_primitive(&v, &i, xf)
    }
    pub fn spawn_plane(&mut self, position: glam::Vec3, size: f32) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::plane(1);
        let xf = glam::Mat4::from_scale_rotation_translation(glam::Vec3::new(size, 1.0, size), glam::Quat::IDENTITY, position);
        self.spawn_primitive(&v, &i, xf)
    }
    pub fn spawn_blob_shadow(&mut self, position: glam::Vec3, radius: f32) -> crate::core::error::ReactorResult<usize> {
        use crate::resources::primitives::Primitives;
        if self.blob_shadow_mesh.is_none() {
            let (v, i) = Primitives::sphere(12, 6);
            let mesh = self.reactor.create_mesh(&v, &i).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?;
            self.blob_shadow_mesh = Some(Arc::new(mesh));
        }
        if self.blob_shadow_material.is_none() {
            let dark_tex = self.reactor.create_solid_texture(8, 8, 10, 200).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?;
            let mat = self.create_base_textured_material(&self.base_shader_cookbook(), &dark_tex)?.with_kept_texture(dark_tex);
            self.blob_shadow_material = Some(Arc::new(mat));
        }
        let mesh = self.blob_shadow_mesh.clone().unwrap();
        let mat = self.blob_shadow_material.clone().unwrap();
        Ok(self.scene.add_object(mesh, mat, Self::blob_xf(position, radius)))
    }
    pub fn move_blob_shadow(&mut self, index: usize, position: glam::Vec3, radius: f32) { self.set_transform(index, Self::blob_xf(position, radius)); }
    pub fn hide_blob_shadow(&mut self, index: usize) { self.set_transform(index, glam::Mat4::from_translation(glam::Vec3::new(0.0, -1000.0, 0.0))); }
    fn blob_xf(position: glam::Vec3, radius: f32) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(glam::Vec3::new(radius, 0.02, radius), glam::Quat::IDENTITY, glam::Vec3::new(position.x, 0.02, position.z))
    }
    fn spawn_primitive(&mut self, vertices: &[crate::resources::vertex::Vertex], indices: &[u32], transform: glam::Mat4) -> crate::core::error::ReactorResult<usize> {
        let legacy: &[crate::resources::vertex::Vertex] = bytemuck::cast_slice(vertices);
        let mesh = Arc::new(self.reactor.create_mesh(legacy, indices).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?);
        let material = Arc::new(self.default_material()?);
        Ok(self.scene.add_object(mesh, material, transform))
    }

    pub fn spawn_colored_sphere(&mut self, position: glam::Vec3, radius: f32, r: u8, g: u8, b: u8, a: u8) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::sphere(16, 8);
        let mesh = Arc::new(self.reactor.create_mesh(&v, &i).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?);
        let mat = Arc::new(self.create_colored_material(r, g, b, a)?);
        Ok(self.scene.add_object(mesh, mat, glam::Mat4::from_scale_rotation_translation(glam::Vec3::splat(radius.max(0.001)), glam::Quat::IDENTITY, position)))
    }
    pub fn spawn_textured_quad(&mut self, texture_path: &str, transform: glam::Mat4) -> crate::core::error::ReactorResult<usize> {
        let (v, i) = crate::resources::primitives::Primitives::quad();
        let mesh = Arc::new(self.reactor.create_mesh(&v, &i).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?);
        let texture = self.load_texture(texture_path)?;
        let mat = Arc::new(self.create_base_textured_material(&self.base_shader_cookbook(), &texture)?.with_kept_texture(texture));
        Ok(self.scene.add_object(mesh, mat, transform))
    }
    pub fn create_colored_material(&self, r: u8, g: u8, b: u8, a: u8) -> crate::core::error::ReactorResult<crate::resources::material::Material> {
        let texture = self.create_solid_texture(r, g, b, a)?;
        self.create_base_textured_material(&self.base_shader_cookbook(), &texture).map(|m| m.with_kept_texture(texture))
    }

    pub fn load_gltf<P: AsRef<std::path::Path>>(&mut self, path: P) -> crate::core::error::ReactorResult<crate::resources::GltfModel> {
        self.gltf_loader.load(path).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))
    }
    pub async fn load_gltf_async<P: AsRef<std::path::Path>>(&mut self, path: P) -> crate::core::error::ReactorResult<crate::resources::GltfModel> {
        let path_buf = path.as_ref().to_path_buf();
        let mut loader = self.gltf_loader.clone();
        tokio::task::spawn_blocking(move || loader.load(path_buf)).await
            .map_err(|e| crate::core::error::ReactorError::internal(format!("Blocking task failed: {}", e)))?
    }
    pub fn load_gltf_queued<P: AsRef<std::path::Path>>(&mut self, path: P, priority: crate::resources::LoadPriority)
        -> tokio::sync::oneshot::Receiver<crate::core::error::ReactorResult<Handle<crate::resources::GltfModel>>> {
        let path_buf = path.as_ref().to_path_buf();
        let id = AssetId::from_path(&path_buf);
        self.asset_loader_queue.enqueue_gltf(id, path_buf, priority)
    }
    pub fn spawn_gltf<P: AsRef<std::path::Path>>(&mut self, path: P, transform: glam::Mat4) -> crate::core::error::ReactorResult<Vec<usize>> {
        let model = self.load_gltf(path)?;
        self.spawn_gltf_model(&model, transform)
    }
    pub fn gltf_bounds<P: AsRef<std::path::Path>>(&mut self, path: P) -> crate::core::error::ReactorResult<GltfBounds> {
        let model = self.load_gltf(path)?;
        let (min, max) = model.bounds().ok_or_else(|| crate::core::error::ReactorError::asset_load("glTF model has no meshes"))?;
        Ok(GltfBounds { min, max, center: (min + max) * 0.5, size: max - min, height: max.y - min.y })
    }
    pub fn spawn_gltf_smart<P: AsRef<std::path::Path>>(&mut self, path: P, spawn: GltfSpawn) -> crate::core::error::ReactorResult<ModelSpawnInfo> {
        let model = self.load_gltf(path)?;
        let (min, max) = model.bounds().ok_or_else(|| crate::core::error::ReactorError::asset_load("glTF model has no meshes"))?;
        let native_height = max.y - min.y;
        let native_center = (min + max) * 0.5;
        let scale = match spawn.target_height { Some(h) if native_height > 1e-6 => h / native_height, _ => 1.0 };
        let rotation = match spawn.face_direction {
            Some(dir) => {
                let flat = glam::Vec3::new(dir.x, 0.0, dir.z);
                if flat.length_squared() > 1e-6 { let n = flat.normalize(); glam::Quat::from_rotation_y((-n.x).atan2(-n.z)) }
                else { glam::Quat::IDENTITY }
            }
            None => glam::Quat::IDENTITY,
        };
        let feet_offset = if spawn.feet_at_position { glam::Vec3::new(0.0, -min.y * scale, 0.0) } else { glam::Vec3::ZERO };
        let final_pos = spawn.position + feet_offset;
        let transform = glam::Mat4::from_scale_rotation_translation(glam::Vec3::splat(scale), rotation, final_pos);
        let indices = self.spawn_gltf_model(&model, transform)?;
        let world_min = (min - native_center) * scale + spawn.position + glam::Vec3::Y * (native_center.y * scale - min.y * scale);
        let world_max = (max - native_center) * scale + spawn.position + glam::Vec3::Y * (native_center.y * scale - min.y * scale);
        Ok(ModelSpawnInfo { indices, applied_scale: scale, applied_rotation: rotation, native_height, world_height: native_height * scale, world_bounds_min: world_min, world_bounds_max: world_max })
    }
    pub fn spawn_gltf_model(&mut self, model: &crate::resources::GltfModel, parent_transform: glam::Mat4) -> crate::core::error::ReactorResult<Vec<usize>> {
        let mut indices = Vec::new();
        self.spawn_gltf_node_recursive(&model.root_node, model, parent_transform, &mut indices)?;
        Ok(indices)
    }
    fn spawn_gltf_node_recursive(&mut self, node: &crate::resources::GltfNode, model: &crate::resources::GltfModel, parent_transform: glam::Mat4, indices: &mut Vec<usize>) -> crate::core::error::ReactorResult<()> {
        let world_transform = parent_transform * node.transform;
        if let Some(mesh_idx) = node.mesh_index {
            if let Some(mesh_data) = model.meshes.get(mesh_idx) {
                let vulkan_mesh = crate::resources::mesh::Mesh::new(&self.reactor.context, &self.reactor.allocator, &mesh_data.vertices, &mesh_data.indices)?;
                let mesh_arc = Arc::new(vulkan_mesh);
                let material_arc = {
                    let mut tex_to_use = None;
                    if let Some(mat_idx) = mesh_data.material_index {
                        if let Some(mat_data) = model.materials.get(mat_idx) { tex_to_use = mat_data.base_color_texture_index; }
                    }
                    if tex_to_use.is_none() && !model.textures.is_empty() { tex_to_use = Some(0); }
                    match tex_to_use {
                        Some(tex_idx) if let Some(tex_data) = model.textures.get(tex_idx) => {
                            let texture = crate::resources::texture::Texture::from_rgba(&self.reactor.context, self.reactor.allocator.clone(), &tex_data.pixels, tex_data.width, tex_data.height, true)?;
                            let cookbook = self.base_shader_cookbook();
                            let mat = self.create_base_textured_material(&cookbook, &texture).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?.with_kept_texture(texture);
                            Arc::new(mat)
                        }
                        _ => Arc::new(self.default_material()?),
                    }
                };
                indices.push(self.scene.add_object(mesh_arc, material_arc, world_transform));
            }
        }
        for child in &node.children { self.spawn_gltf_node_recursive(child, model, world_transform, indices)?; }
        Ok(())
    }
    pub fn track_asset_for_reload<P: AsRef<std::path::Path>>(&mut self, path: P, asset_type: crate::resources::AssetType) -> crate::core::error::ReactorResult<AssetId> {
        let path = path.as_ref();
        let id = AssetId::from_path(path);
        if let Some(ref mut hot_reload) = self.asset_hot_reload { hot_reload.track_asset(id, path, asset_type).map_err(|e| crate::core::error::ReactorError::internal(e.to_string()))?; }
        Ok(id)
    }
    pub fn asset_stats(&self) -> AssetPipelineStats {
        AssetPipelineStats {
            loader_queue: self.asset_loader_queue.stats(),
            hot_reload: self.asset_hot_reload.as_ref().map(|hr| hr.stats()),
            db: self.asset_db.stats(),
            gltf_cache: self.gltf_loader.cache_stats(),
        }
    }
}
