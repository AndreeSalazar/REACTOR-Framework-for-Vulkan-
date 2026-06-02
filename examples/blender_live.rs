// =============================================================================
// REACTOR ⇄ Blender Live Link — Ejemplo funcional
// =============================================================================
//
// Este ejemplo arranca REACTOR con un cubo de prueba y el servidor WebSocket
// del bridge (puerto 19840). Cuando Blender se conecta via el addon
// "REACTOR Live Link", cualquier cambio de transformación en Blender se
// refleja aquí EN TIEMPO REAL.
//
// Uso:
//   cargo run --example blender_live
//
// Luego en Blender 4.2+:
//   1. Instala el addon desde reactor-blender-bridge/blender_addon/
//   2. N-Panel → REACTOR → Connect
//   3. Mueve objetos → se mueven aquí
// =============================================================================

use reactor_vulkan::prelude::*;
use reactor_vulkan::systems::scene::SceneObject;
use reactor_vulkan::Vertex;
use std::collections::HashMap;
use std::sync::Arc;
use winit::keyboard::KeyCode;

// Re-use reactor-bridge via dev-dependency
use reactor_bridge::{BridgeConfig, BridgeHandle, Message, TransformUpdated};

#[derive(serde::Deserialize)]
struct LiveConfig {
    host: String,
    port: u16,
}

fn load_live_config() -> LiveConfig {
    if let Ok(content) = std::fs::read_to_string("reactor_live_config.json") {
        if let Ok(cfg) = serde_json::from_str::<LiveConfig>(&content) {
            println!("\x1b[32m  ✓ Cargada configuración desde reactor_live_config.json\x1b[0m");
            return cfg;
        }
    }
    // Fallback
    LiveConfig {
        host: "127.0.0.1".to_string(),
        port: 19840,
    }
}

/// Estado de la app de Live Link.
struct BlenderLive {
    /// Handle del servidor WebSocket (para shutdown limpio).
    bridge_handle: Option<BridgeHandle>,

    /// Canal receptor de mensajes del bridge.
    bridge_rx: Option<tokio::sync::mpsc::UnboundedReceiver<Message>>,

    /// Mapa: id de entidad (string Blender) → índice en ctx.scene.objects
    entity_map: HashMap<String, usize>,

    /// Mapa: id de luz (string Blender) → índice en ctx.lighting.lights
    light_map: HashMap<String, usize>,

    /// Malla de cubo compartida para las nuevas entidades creadas.
    cube_mesh: Option<Arc<reactor_vulkan::Mesh>>,

    /// Material por defecto para los objetos sincronizados desde Blender.
    blender_material: Option<Arc<reactor_vulkan::Material>>,

    /// Runtime de Tokio para el servidor de WebSocket.
    runtime: Option<tokio::runtime::Runtime>,

    /// Texturas de fallback y cache de texturas dinámicas
    fallback_albedo: Option<reactor_vulkan::resources::texture::Texture>,
    fallback_normal: Option<reactor_vulkan::resources::texture::Texture>,
    fallback_metallic: Option<reactor_vulkan::resources::texture::Texture>,
    fallback_roughness: Option<reactor_vulkan::resources::texture::Texture>,
    texture_cache: HashMap<String, reactor_vulkan::resources::texture::Texture>,

    /// Códigos SPIR-V de los shaders
    shader: BaseShaderCookbook,
}

impl BlenderLive {
    fn new() -> Self {
        Self {
            bridge_handle: None,
            bridge_rx: None,
            entity_map: HashMap::new(),
            light_map: HashMap::new(),
            cube_mesh: None,
            blender_material: None,
            runtime: None,
            fallback_albedo: None,
            fallback_normal: None,
            fallback_metallic: None,
            fallback_roughness: None,
            texture_cache: HashMap::new(),
            shader: BaseShaderCookbook::blender_live(),
        }
    }

    fn cleanup(&mut self) {
        println!("  ➔ Cerrando servidor bridge y hilos de fondo cleanly...");
        if let Some(handle) = self.bridge_handle.take() {
            if let Some(ref rt) = self.runtime {
                rt.block_on(async {
                    handle.shutdown().await;
                });
            }
        }
        self.runtime = None;
        println!("  ✓ Servidor e hilos cerrados cleanly.");
    }
}

fn create_cube_mesh(ctx: &mut ReactorContext) -> Arc<reactor_vulkan::Mesh> {
    // Cubo con normales **por cara** (24 vértices, 36 índices) y UVs por cara.
    // Esto es indispensable para que el shader PBR pueda iluminar correctamente
    // — un cubo con normales (1,1,1) compartidas se ve "plano" y rompe NdotL.
    let face = |p: [[f32; 3]; 4], n: [f32; 3]| {
        let n = Vec3::from_array(n);
        [
            Vertex::with_normal(Vec3::from_array(p[0]), n, Vec2::new(0.0, 0.0)),
            Vertex::with_normal(Vec3::from_array(p[1]), n, Vec2::new(1.0, 0.0)),
            Vertex::with_normal(Vec3::from_array(p[2]), n, Vec2::new(1.0, 1.0)),
            Vertex::with_normal(Vec3::from_array(p[3]), n, Vec2::new(0.0, 1.0)),
        ]
    };

    let mut vertices: Vec<Vertex> = Vec::with_capacity(24);
    // +Z (front)
    vertices.extend_from_slice(&face(
        [
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ],
        [0.0, 0.0, 1.0],
    ));
    // -Z (back)
    vertices.extend_from_slice(&face(
        [
            [0.5, -0.5, -0.5],
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
        ],
        [0.0, 0.0, -1.0],
    ));
    // +X (right)
    vertices.extend_from_slice(&face(
        [
            [0.5, -0.5, 0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, 0.5, 0.5],
        ],
        [1.0, 0.0, 0.0],
    ));
    // -X (left)
    vertices.extend_from_slice(&face(
        [
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
        ],
        [-1.0, 0.0, 0.0],
    ));
    // +Y (top)
    vertices.extend_from_slice(&face(
        [
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
        ],
        [0.0, 1.0, 0.0],
    ));
    // -Y (bottom)
    vertices.extend_from_slice(&face(
        [
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
        ],
        [0.0, -1.0, 0.0],
    ));

    let mut indices: Vec<u32> = Vec::with_capacity(36);
    for f in 0..6u32 {
        let b = f * 4;
        indices.extend_from_slice(&[b, b + 1, b + 2, b + 2, b + 3, b]);
    }

    Arc::new(
        ctx.create_mesh(&vertices, &indices)
            .expect("failed to create mesh"),
    )
}

impl ReactorApp for BlenderLive {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("REACTOR ⇄ Blender Live Link")
            .with_size(1600, 900)
            .with_vsync(true)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        // -----------------------------------------------------------------
        // 1. Banner de consola
        // -----------------------------------------------------------------
        println!("\x1b[38;2;180;40;40m");
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║                                                               ║");
        println!("║    ██████╗ ███████╗ █████╗  ██████╗████████╗ ██████╗ ██████╗  ║");
        println!("║    ██╔══██╗██╔════╝██╔══██╗██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗ ║");
        println!("║    ██████╔╝█████╗  ███████║██║        ██║   ██║   ██║██████╔╝ ║");
        println!("║    ██╔══██╗██╔══╝  ██╔══██║██║        ██║   ██║   ██║██╔══██╗ ║");
        println!("║    ██║  ██║███████╗██║  ██║╚██████╗   ██║   ╚██████╔╝██║  ██║ ║");
        println!("║    ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝ ║");
        println!("║                                                               ║");
        println!("║           ⇄  B L E N D E R   L I V E   L I N K  ⇄             ║");
        println!("║                                                               ║");
        println!("╚═══════════════════════════════════════════════════════════════╝\x1b[0m");
        println!();
        println!("\x1b[38;2;120;120;120m  WebSocket server: ws://127.0.0.1:19840\x1b[0m");
        println!("\x1b[38;2;120;120;120m  Esperando conexión de Blender...\x1b[0m");
        println!();

        // -----------------------------------------------------------------
        // 2. Iluminación de estudio (referencia: Eevee "studio" preset)
        // -----------------------------------------------------------------
        // Ambient bajo + sol cálido suave; el shader PBR añade encima un IBL
        // hemisférico procedural (cielo/horizonte/suelo) + rim light.
        ctx.scene.set_ambient(Vec3::new(0.045, 0.05, 0.06));
        ctx.scene.set_sun(
            Vec3::new(-0.4, -1.0, -0.6).normalize(),
            Vec3::new(1.0, 0.96, 0.88),
        );

        // Activar la cocina base profesional. Los juegos pueden mutar
        // `self.shader` antes de crear materiales si necesitan otro look.
        ctx.apply_base_shader(&self.shader);

        // Bake the procedural studio sky cubemap at startup
        let ibl = reactor_vulkan::graphics::IblBaker::bake_procedural(
            &ctx.reactor.context,
            ctx.reactor.allocator.clone(),
        )
        .expect("Failed to bake procedural IBL");
        let ibl_layout = ibl.descriptor_set_layout;
        ctx.reactor.ibl_textures = Some(ibl);

        // Crear texturas de fallback sólidas
        let fallback_albedo = ctx
            .reactor
            .create_solid_texture(255, 255, 255, 255)
            .expect("Failed to create fallback albedo texture");
        let fallback_normal = ctx
            .reactor
            .create_solid_texture(128, 128, 255, 255)
            .expect("Failed to create fallback normal texture");
        let fallback_metallic = ctx
            .reactor
            .create_solid_texture(255, 255, 255, 255)
            .expect("Failed to create fallback metallic texture");
        let fallback_roughness = ctx
            .reactor
            .create_solid_texture(255, 255, 255, 255)
            .expect("Failed to create fallback roughness texture");
        self.fallback_albedo = Some(fallback_albedo);
        self.fallback_normal = Some(fallback_normal);
        self.fallback_metallic = Some(fallback_metallic);
        self.fallback_roughness = Some(fallback_roughness);

        // Crear materiales con IBL y texturas
        let albedo_ref = self.fallback_albedo.as_ref().unwrap();
        let normal_ref = self.fallback_normal.as_ref().unwrap();
        let metallic_ref = self.fallback_metallic.as_ref().unwrap();
        let roughness_ref = self.fallback_roughness.as_ref().unwrap();
        let blender_mat = Arc::new(
            ctx.create_base_pbr_material(
                &self.shader,
                ibl_layout,
                albedo_ref,
                normal_ref,
                metallic_ref,
                roughness_ref,
            )
            .expect("Failed to create PBR material"),
        );
        self.blender_material = Some(blender_mat);

        // Crear malla
        let mesh = create_cube_mesh(ctx);
        self.cube_mesh = Some(mesh);

        // Cámara
        ctx.camera.position = Vec3::new(3.0, 3.0, 5.0);
        ctx.camera.set_rotation(-0.5, -0.6);

        // Inicializar Cascade Shadow Maps
        ctx.reactor
            .init_shadows()
            .expect("Failed to initialize shadows");

        // -----------------------------------------------------------------
        // 3. Arrancar el servidor WebSocket del bridge
        // -----------------------------------------------------------------
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
        self.bridge_rx = Some(rx);

        let live_cfg = load_live_config();
        let cfg = BridgeConfig { host: live_cfg.host, port: live_cfg.port };
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("failed to build tokio runtime for bridge");

        let handle = rt.block_on(async { reactor_bridge::server::spawn(cfg, Some(tx)).await });

        match handle {
            Ok(h) => {
                println!("\x1b[32m  ✓ Bridge server arrancado en {}\x1b[0m", h.addr);
                self.bridge_handle = Some(h);
                self.runtime = Some(rt);
            }
            Err(e) => {
                eprintln!("\x1b[31m  ✗ Error arrancando bridge: {e}\x1b[0m");
            }
        }
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        // -----------------------------------------------------------------
        // 1. Procesar mensajes del bridge (no-blocking)
        // -----------------------------------------------------------------
        if let Some(mut rx) = self.bridge_rx.take() {
            // Drain all pending messages this frame
            loop {
                match rx.try_recv() {
                    Ok(msg) => self.handle_bridge_message(ctx, msg),
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        eprintln!("\x1b[33m  ⚠ Bridge channel disconnected\x1b[0m");
                        break;
                    }
                }
            }
            self.bridge_rx = Some(rx);
        }

        // -----------------------------------------------------------------
        // 2. Salir con ESC
        // -----------------------------------------------------------------
        if ctx.input().is_key_just_pressed(KeyCode::Escape) {
            ctx.reactor.exit_requested = true;
        }
    }

    fn on_exit(&mut self, _ctx: &mut ReactorContext) {
        self.cleanup();
    }
}

impl BlenderLive {
    fn handle_bridge_message(&mut self, ctx: &mut ReactorContext, msg: Message) {
        match msg {
            Message::Hello(h) => {
                println!(
                    "\x1b[36m  ← Hello de '{}' (protocol v{})\x1b[0m",
                    h.client, h.version
                );
            }
            Message::TransformUpdated(t) => {
                self.apply_transform(ctx, t);
            }
            Message::Ping(_) | Message::Pong(_) | Message::HelloAck(_) => {
                // Handled by the server internally
            }
            Message::Error(e) => {
                eprintln!(
                    "\x1b[31m  ← Error del cliente: [{}] {}\x1b[0m",
                    e.code, e.message
                );
            }
            Message::Goodbye(g) => {
                println!("\x1b[33m  ← Goodbye: {}\x1b[0m", g.reason);
            }
        }
    }

    fn apply_transform(&mut self, ctx: &mut ReactorContext, t: TransformUpdated) {
        let name_lower = t.id.to_lowercase();

        // ── CASO A: Sincronización de Cámara ──
        if name_lower.contains("camera") {
            let m = Mat4::from_cols_array(&t.matrix);
            let pos = m.w_axis.truncate();
            let forward = -m.z_axis.truncate().normalize();

            let yaw = forward.x.atan2(-forward.z);
            let pitch = forward.y.asin();

            ctx.camera.position = pos;
            ctx.camera.set_rotation(yaw, pitch);

            println!(
                "\x1b[35m  🎥 Cámara Sincronizada → pos: (x: {:.2}, y: {:.2}, z: {:.2}), yaw: {:.2}, pitch: {:.2}\x1b[0m",
                pos.x, pos.y, pos.z, yaw, pitch
            );
            return;
        }

        // ── CASO B: Sincronización de Luces Dinámicas ──
        if name_lower.contains("light") {
            let m = Mat4::from_cols_array(&t.matrix);
            let pos = m.w_axis;

            // Actualizar la posición de la luz en el reactor para los push constants
            ctx.reactor.light_pos = pos;

            if let Some(&light_idx) = self.light_map.get(&t.id) {
                if let Some(light) = ctx.lighting.get_light_mut(light_idx) {
                    light.position = pos.truncate();
                    println!(
                        "\x1b[33m  💡 Luz Actualizada '{}' → pos: (x: {:.2}, y: {:.2}, z: {:.2})\x1b[0m",
                        t.id, pos.x, pos.y, pos.z
                    );
                }
            } else {
                let light_idx = ctx
                    .lighting
                    .add_light(reactor_vulkan::prelude::Light::point(
                        pos.truncate(),
                        Vec3::new(1.0, 1.0, 1.0), // Neutral white light
                        15.0,                     // High intensity
                        30.0,                     // Range
                    ));
                self.light_map.insert(t.id.clone(), light_idx);
                println!(
                    "\x1b[32m  + Nueva Luz Sincronizada '{}' → pos: (x: {:.2}, y: {:.2}, z: {:.2})\x1b[0m",
                    t.id, pos.x, pos.y, pos.z
                );
            }
            return;
        }

        // ── CASO C: Sincronización de Geometría / Objetos MESH ──
        // Buscar si ya tenemos esta entidad mapeada
        if let Some(&idx) = self.entity_map.get(&t.id) {
            // Aplicar la nueva matriz de transformación
            if let Some(obj) = ctx.scene.get_mut(idx) {
                let m = t.matrix;
                obj.transform = Mat4::from_cols_array(&m);

                // Si viene un color de material, aplicarlo al objeto de la escena
                if let Some(col) = t.color {
                    obj.color = glam::Vec4::from_slice(&col);
                    println!(
                        "\x1b[36m  🎨 Color de Material Actualizado para '{}' → (r: {:.2}, g: {:.2}, b: {:.2}, a: {:.2})\x1b[0m",
                        t.id, col[0], col[1], col[2], col[3]
                    );
                }

                // Sincronizar metálico y rugosidad
                if let Some(met) = t.metallic {
                    obj.metallic = met;
                }
                if let Some(rough) = t.roughness {
                    obj.roughness = rough;
                }

                // Cargar/Actualizar texturas si cambian
                let mut albedo_updated = false;
                let mut normal_updated = false;
                let mut metallic_updated = false;
                let mut roughness_updated = false;

                if let Some(ref path) = t.albedo_path {
                    if !self.texture_cache.contains_key(path) {
                        if let Ok(tex) = ctx.reactor.load_texture(path) {
                            self.texture_cache.insert(path.clone(), tex);
                        }
                    }
                    albedo_updated = true;
                }

                if let Some(ref path) = t.normal_path {
                    if !self.texture_cache.contains_key(path) {
                        if let Ok(tex) = ctx.reactor.load_texture(path) {
                            self.texture_cache.insert(path.clone(), tex);
                        }
                    }
                    normal_updated = true;
                }

                if let Some(ref path) = t.metallic_path {
                    if !self.texture_cache.contains_key(path) {
                        if let Ok(tex) = ctx.reactor.load_texture_linear(path) {
                            self.texture_cache.insert(path.clone(), tex);
                        }
                    }
                    metallic_updated = true;
                }

                if let Some(ref path) = t.roughness_path {
                    if !self.texture_cache.contains_key(path) {
                        if let Ok(tex) = ctx.reactor.load_texture_linear(path) {
                            self.texture_cache.insert(path.clone(), tex);
                        }
                    }
                    roughness_updated = true;
                }

                if albedo_updated || normal_updated || metallic_updated || roughness_updated {
                    if let Some(descriptor_set) = obj.material.descriptor_set {
                        let albedo_tex = t
                            .albedo_path
                            .as_ref()
                            .and_then(|p| self.texture_cache.get(p))
                            .unwrap_or(self.fallback_albedo.as_ref().unwrap());

                        let normal_tex = t
                            .normal_path
                            .as_ref()
                            .and_then(|p| self.texture_cache.get(p))
                            .unwrap_or(self.fallback_normal.as_ref().unwrap());

                        let metallic_tex = t
                            .metallic_path
                            .as_ref()
                            .and_then(|p| self.texture_cache.get(p))
                            .unwrap_or(self.fallback_metallic.as_ref().unwrap());

                        let roughness_tex = t
                            .roughness_path
                            .as_ref()
                            .and_then(|p| self.texture_cache.get(p))
                            .unwrap_or(self.fallback_roughness.as_ref().unwrap());

                        let albedo_info = ash::vk::DescriptorImageInfo::default()
                            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .image_view(albedo_tex.view())
                            .sampler(albedo_tex.sampler_handle());

                        let normal_info = ash::vk::DescriptorImageInfo::default()
                            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .image_view(normal_tex.view())
                            .sampler(normal_tex.sampler_handle());

                        let metallic_info = ash::vk::DescriptorImageInfo::default()
                            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .image_view(metallic_tex.view())
                            .sampler(metallic_tex.sampler_handle());

                        let roughness_info = ash::vk::DescriptorImageInfo::default()
                            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .image_view(roughness_tex.view())
                            .sampler(roughness_tex.sampler_handle());

                        let write_albedo = ash::vk::WriteDescriptorSet::default()
                            .dst_set(descriptor_set)
                            .dst_binding(0)
                            .dst_array_element(0)
                            .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .image_info(std::slice::from_ref(&albedo_info));

                        let write_normal = ash::vk::WriteDescriptorSet::default()
                            .dst_set(descriptor_set)
                            .dst_binding(1)
                            .dst_array_element(0)
                            .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .image_info(std::slice::from_ref(&normal_info));

                        let write_metallic = ash::vk::WriteDescriptorSet::default()
                            .dst_set(descriptor_set)
                            .dst_binding(2)
                            .dst_array_element(0)
                            .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .image_info(std::slice::from_ref(&metallic_info));

                        let write_roughness = ash::vk::WriteDescriptorSet::default()
                            .dst_set(descriptor_set)
                            .dst_binding(3)
                            .dst_array_element(0)
                            .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .image_info(std::slice::from_ref(&roughness_info));

                        unsafe {
                            ctx.reactor.context.device.update_descriptor_sets(
                                &[write_albedo, write_normal, write_metallic, write_roughness],
                                &[],
                            );
                        }
                        println!(
                            "\x1b[35m  📝 Texturas dinámicas actualizadas para '{}'\x1b[0m",
                            t.id
                        );
                    }
                }

                // Mostrar log visual de la sincronización en tiempo real
                let translation = obj.transform.w_axis;
                println!(
                    "\x1b[36m  ↺ Sincronizado '{}' → pos: (x: {:.2}, y: {:.2}, z: {:.2})\x1b[0m",
                    t.id, translation.x, translation.y, translation.z
                );
            }
        } else {
            // Entidad nueva — crear un cubo placeholder
            let mesh = self.cube_mesh.clone().expect("cube mesh not initialized");

            // Crear un material único para esta entidad
            let ibl_layout = ctx
                .reactor
                .ibl_textures
                .as_ref()
                .unwrap()
                .descriptor_set_layout;
            let albedo_ref = self.fallback_albedo.as_ref().unwrap();
            let normal_ref = self.fallback_normal.as_ref().unwrap();
            let metallic_ref = self.fallback_metallic.as_ref().unwrap();
            let roughness_ref = self.fallback_roughness.as_ref().unwrap();

            let mat = Arc::new(
                ctx.create_base_pbr_material(
                    &self.shader,
                    ibl_layout,
                    albedo_ref,
                    normal_ref,
                    metallic_ref,
                    roughness_ref,
                )
                .expect("Failed to create entity material"),
            );

            let m = t.matrix;
            let transform = Mat4::from_cols_array(&m);
            let mut obj = SceneObject::new(mesh, mat, transform);

            // Si viene un color de material, aplicarlo al objeto de la escena
            if let Some(col) = t.color {
                obj.color = glam::Vec4::from_slice(&col);
            }
            obj.metallic = t.metallic.unwrap_or(0.0);
            obj.roughness = t.roughness.unwrap_or(0.5);

            // Cargar texturas al inicio si vienen indicadas
            let mut albedo_updated = false;
            let mut normal_updated = false;
            let mut metallic_updated = false;
            let mut roughness_updated = false;

            if let Some(ref path) = t.albedo_path {
                if !self.texture_cache.contains_key(path) {
                    if let Ok(tex) = ctx.reactor.load_texture(path) {
                        self.texture_cache.insert(path.clone(), tex);
                    }
                }
                albedo_updated = true;
            }

            if let Some(ref path) = t.normal_path {
                if !self.texture_cache.contains_key(path) {
                    if let Ok(tex) = ctx.reactor.load_texture(path) {
                        self.texture_cache.insert(path.clone(), tex);
                    }
                }
                normal_updated = true;
            }

            if let Some(ref path) = t.metallic_path {
                if !self.texture_cache.contains_key(path) {
                    if let Ok(tex) = ctx.reactor.load_texture_linear(path) {
                        self.texture_cache.insert(path.clone(), tex);
                    }
                }
                metallic_updated = true;
            }

            if let Some(ref path) = t.roughness_path {
                if !self.texture_cache.contains_key(path) {
                    if let Ok(tex) = ctx.reactor.load_texture_linear(path) {
                        self.texture_cache.insert(path.clone(), tex);
                    }
                }
                roughness_updated = true;
            }

            if albedo_updated || normal_updated || metallic_updated || roughness_updated {
                if let Some(descriptor_set) = obj.material.descriptor_set {
                    let albedo_tex = t
                        .albedo_path
                        .as_ref()
                        .and_then(|p| self.texture_cache.get(p))
                        .unwrap_or(self.fallback_albedo.as_ref().unwrap());

                    let normal_tex = t
                        .normal_path
                        .as_ref()
                        .and_then(|p| self.texture_cache.get(p))
                        .unwrap_or(self.fallback_normal.as_ref().unwrap());

                    let metallic_tex = t
                        .metallic_path
                        .as_ref()
                        .and_then(|p| self.texture_cache.get(p))
                        .unwrap_or(self.fallback_metallic.as_ref().unwrap());

                    let roughness_tex = t
                        .roughness_path
                        .as_ref()
                        .and_then(|p| self.texture_cache.get(p))
                        .unwrap_or(self.fallback_roughness.as_ref().unwrap());

                    let albedo_info = ash::vk::DescriptorImageInfo::default()
                        .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(albedo_tex.view())
                        .sampler(albedo_tex.sampler_handle());

                    let normal_info = ash::vk::DescriptorImageInfo::default()
                        .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(normal_tex.view())
                        .sampler(normal_tex.sampler_handle());

                    let metallic_info = ash::vk::DescriptorImageInfo::default()
                        .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(metallic_tex.view())
                        .sampler(metallic_tex.sampler_handle());

                    let roughness_info = ash::vk::DescriptorImageInfo::default()
                        .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(roughness_tex.view())
                        .sampler(roughness_tex.sampler_handle());

                    let write_albedo = ash::vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(0)
                        .dst_array_element(0)
                        .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&albedo_info));

                    let write_normal = ash::vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(1)
                        .dst_array_element(0)
                        .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&normal_info));

                    let write_metallic = ash::vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(2)
                        .dst_array_element(0)
                        .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&metallic_info));

                    let write_roughness = ash::vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(3)
                        .dst_array_element(0)
                        .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&roughness_info));

                    unsafe {
                        ctx.reactor.context.device.update_descriptor_sets(
                            &[write_albedo, write_normal, write_metallic, write_roughness],
                            &[],
                        );
                    }
                }
            }

            let idx = ctx.scene.objects.len();
            ctx.scene.objects.push(obj);
            self.entity_map.insert(t.id.clone(), idx);

            let translation = transform.w_axis;
            println!(
                "\x1b[32m  + Nueva Entidad Sincronizada '{}' → scene[{}] pos: (x: {:.2}, y: {:.2}, z: {:.2})\x1b[0m",
                t.id, idx, translation.x, translation.y, translation.z
            );
        }
    }
}

fn main() {
    reactor_vulkan::run(BlenderLive::new());
}
