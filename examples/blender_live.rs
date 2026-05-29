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
    let vertices = [
        // Front face (z = 0.5)
        Vertex::new(Vec3::new(-0.5, -0.5, 0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(0.5, -0.5, 0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        // Back face (z = -0.5)
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(0.5, -0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(0.5, 0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5, 0.5, -0.5), Vec3::new(1.0, 1.0, 1.0), Vec2::ZERO),
    ];
    let indices = [
        // Front face
        0, 1, 2, 2, 3, 0,
        // Right face
        1, 5, 6, 6, 2, 1,
        // Back face
        5, 4, 7, 7, 6, 5,
        // Left face
        4, 0, 3, 3, 7, 4,
        // Top face
        3, 2, 6, 6, 7, 3,
        // Bottom face
        4, 5, 1, 1, 0, 4,
    ];
    Arc::new(ctx.create_mesh(&vertices, &indices).expect("failed to create mesh"))
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
        // 2. Escena de prueba: cubo + suelo + iluminación
        // -----------------------------------------------------------------
        ctx.scene.set_ambient(Vec3::new(0.08, 0.06, 0.10));
        ctx.scene.set_sun(
            Vec3::new(-0.4, -1.0, -0.6).normalize(),
            Vec3::new(1.0, 0.95, 0.85),
        );

        // Cargar shaders base profesionales y neutros para Live Link
        let vert = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../shaders/blender_live_vert.spv"
        )))
        .expect("Failed to load vertex shader");

        let frag = ash::util::read_spv(&mut std::io::Cursor::new(include_bytes!(
            "../shaders/blender_live_frag.spv"
        )))
        .expect("Failed to load fragment shader");

        // Crear materiales
        let blender_mat = Arc::new(ctx.create_material(&vert, &frag).expect("Failed to create material"));
        self.blender_material = Some(blender_mat);

        // Crear malla
        let mesh = create_cube_mesh(ctx);
        self.cube_mesh = Some(mesh);

        // Cámara
        ctx.camera.position = Vec3::new(3.0, 3.0, 5.0);
        ctx.camera.set_rotation(-0.5, -0.6);

        // -----------------------------------------------------------------
        // 3. Arrancar el servidor WebSocket del bridge
        // -----------------------------------------------------------------
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
        self.bridge_rx = Some(rx);

        let live_cfg = load_live_config();
        let cfg = BridgeConfig {
            host: live_cfg.host,
            port: live_cfg.port,
        };
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("failed to build tokio runtime for bridge");

        let handle = rt.block_on(async {
            reactor_bridge::server::spawn(cfg, Some(tx)).await
        });

        match handle {
            Ok(h) => {
                println!(
                    "\x1b[32m  ✓ Bridge server arrancado en {}\x1b[0m",
                    h.addr
                );
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
                println!(
                    "\x1b[33m  ← Goodbye: {}\x1b[0m",
                    g.reason
                );
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
                let light_idx = ctx.lighting.add_light(reactor_vulkan::prelude::Light::point(
                    pos.truncate(),
                    Vec3::new(1.0, 1.0, 1.0), // Neutral white light
                    15.0, // High intensity
                    30.0, // Range
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
            let mat = self.blender_material.clone().expect("blender material not initialized");
            let m = t.matrix;
            let transform = Mat4::from_cols_array(&m);
            let mut obj = SceneObject::new(mesh, mat, transform);
            
            // Si viene un color de material, aplicarlo al objeto de la escena
            if let Some(col) = t.color {
                obj.color = glam::Vec4::from_slice(&col);
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
