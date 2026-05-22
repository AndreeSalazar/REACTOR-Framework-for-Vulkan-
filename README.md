<p align="center">
  <img src="image.svg" alt="REACTOR Logo" width="300"/>
</p>

<h1 align="center">REACTOR Framework for Vulkan</h1>

<p align="center">
  <strong>Zero-overhead Vulkan Game Framework — 100% Rust Puro</strong>
</p>

<p align="center">
  <em>Powered by Salazar-interactive</em>
</p>

<p align="center">
  <a href="https://opensource.org/licenses/TECHNE"><img src="https://img.shields.io/badge/License-TECHNE-yellow.svg" alt="License: TECHNE"/></a>
  <a href="https://www.vulkan.org/"><img src="https://img.shields.io/badge/Vulkan-1.3-red.svg" alt="Vulkan"/></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.70+-orange.svg" alt="Rust"/></a>
  <img src="https://img.shields.io/badge/Version-1.2.0--rust-green.svg" alt="Version 1.2.0-rust"/>
  <img src="https://img.shields.io/badge/Status-Rust%20Only-blueviolet.svg" alt="Status"/>
</p>

---

**REACTOR** es un framework **100% Rust puro** para crear videojuegos con Vulkan 1.3.
Aprovecha el sistema de tipos, ownership y RAII de Rust para garantizar **seguridad de memoria**, **zero-overhead** y **máxima productividad** sin renunciar al control completo de la GPU.

> **Nota importante (v1.1.0):** A partir de esta versión, REACTOR es **exclusivamente Rust**.
> Toda la capa C ABI y el antiguo C++ SDK se han eliminado del roadmap activo para enfocar
> el desarrollo en un único stack idiomático, simple y mantenible.

## 🏗️ Arquitectura

```diagram
╭──────────────╮      ╭──────────────╮     ╭──────────────╮
│  Ash / Vulkan│────▶│ Reactor Core │────▶│  Tu Juego    │
│   (unsafe)   │      │ (safe + RAII)│     │ (ReactorApp) │
╰──────────────╯      ╰──────────────╯     ╰──────────────╯
   Bindings raw       Memory safety           Lógica pura
   a Vulkan 1.3       FrameGraph, ECS,        en Rust
                      PBR, Compute, RT
```

- **Ash**: Bindings directos a Vulkan 1.3 (`unsafe`).
- **Reactor Core**: Abstracciones seguras con RAII automático.
- **ReactorApp**: API declarativa estilo React — heredas un trait y listo.

## ✨ Características

| Módulo | Características |
| ------ | --------------- |
| **Core** | `VulkanContext`, `Device`, `Allocator`, `CommandManager`, `Surface` |
| **Graphics** | Swapchain, Pipeline, RenderPass, Framebuffer, Buffer, Image, Sampler, Descriptors, DepthBuffer, MSAA, UniformBuffers, DebugRenderer, PostProcessing, Shadows |
| **Ray Tracing** | `RayTracingContext`, `AccelerationStructure` (BLAS/TLAS), `RayTracingPipeline`, `ShaderBindingTable` |
| **Compute** | `ComputePipeline`, `ComputeDispatch`, Barriers, Particles compute |
| **Resources** | Mesh, Material, Texture, Vertex, Model (OBJ/glTF), Primitives (Cube, Sphere, Plane, Cylinder, Cone, Torus) |
| **ECS** | Entity CRUD, Transform, MeshRenderer, Light, Camera, RigidBody, Component Queries |
| **PBR** | Metallic/Roughness workflow, Material Instances, Emissive, Alpha modes |
| **FrameGraph** | Render passes declarativos, gestión de recursos, presets Forward/Deferred, auto-barriers |
| **Systems** | Input, Scene, Camera, Transform, Lighting, Physics, FrustumCulling, Animation, Particles, Audio |
| **Telemetry** | RenderStats (FPS, draw calls, triangles, VRAM), MemoryBudget, GPU info |
| **Utils** | GPUDetector, CPUDetector, ResolutionDetector, Time, FixedTimestep |
| **Editor** | Editor visual con `egui` + `egui_dock` (Viewport, Hierarchy, Inspector, Console) |

## 🚀 Quick Start — Patrón `ReactorApp`

### Requisitos
- [Rust](https://rustup.rs/) (1.70+)
- Vulkan SDK (1.3+)
- (Opcional) `glslangValidator` para recompilar shaders manualmente — los shaders se compilan automáticamente vía `build.rs`.

### Ejecutar Ejemplos

```bash
cargo run --example cube              # Cubo 3D con controles
cargo run --example textured_cube     # Cubo con textura
cargo run --example sandbox           # Sandbox experimental
cargo run --example physics_camera    # Cámara con física
cargo run --example obj_loader_demo   # Carga de modelos OBJ
```

### 🎯 ReactorApp — El Patrón Principal

REACTOR usa un patrón **declarativo estilo React** donde implementas un trait y configuras
todo desde **un solo archivo**:

```rust
use reactor::prelude::*;

struct MiJuego {
    rotacion: f32,
}

impl ReactorApp for MiJuego {
    // ── CONFIG ─────────────────────────────────────────────────────────
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Mi Juego")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_msaa(4)
            .with_renderer(RendererMode::Forward)
    }

    // ── INIT ───────────────────────────────────────────────────────────
    fn init(&mut self, ctx: &mut ReactorContext) {
        ctx.camera.position = Vec3::new(0.0, 2.0, 4.0);
        ctx.lighting.add_light(Light::directional(Vec3::NEG_Y, Vec3::ONE, 1.0));
        // Agregar objetos a la escena…
    }

    // ── UPDATE ─────────────────────────────────────────────────────────
    fn update(&mut self, ctx: &mut ReactorContext) {
        self.rotacion += ctx.time.delta();
        ctx.scene.objects[0].transform = Mat4::from_rotation_y(self.rotacion);
    }

    // render() es AUTOMÁTICO — el framework se encarga.
}

fn main() {
    reactor::run(MiJuego { rotacion: 0.0 });
}
```

## 📁 Estructura del Proyecto

```text
REACTOR-Framework-for-Vulkan-/
├── image.svg                   # Logo REACTOR (Salazar-interactive)
├── README.md                   # Este archivo
├── Fases.md                    # Roadmap completo de construcción del SDK
├── HOW_BUILD.md                # Guía de compilación
├── VERSION.txt                 # Versión actual
├── Cargo.toml                  # Crate principal (reactor)
├── build.rs                    # Auto-compilación de shaders GLSL → SPIR-V
│
├── src/                        # Rust Core
│   ├── lib.rs                  # Exports + Prelude
│   ├── app.rs                  # Trait ReactorApp + run()
│   ├── reactor.rs              # Fachada principal de rendering
│   ├── core/                   # VulkanContext, Device, Allocator, FrameGraph, ImportanceMap
│   ├── graphics/               # Swapchain, Pipeline, RenderPass, MSAA, Depth, Shadows, PostFX
│   ├── raytracing/             # RT Context, BLAS/TLAS, RT Pipeline, SBT
│   ├── compute/                # ComputePipeline, Dispatch, Particles
│   ├── resources/              # Mesh, Material (PBR), Texture, Vertex, Model, Primitives
│   ├── systems/                # Input, ECS, Scene, Camera, Transform, Lighting,
│   │                           # Physics, Frustum, Animation, Particles, Audio
│   ├── platform/               # Window + Config
│   └── utils/                  # GPUDetector, CPUDetector, ResolutionDetector, Time
│
├── examples/                   # Ejemplos Rust
│   ├── cube.rs
│   ├── textured_cube.rs
│   ├── sandbox.rs
│   ├── physics_camera.rs
│   └── obj_loader_demo.rs
│
├── shaders/                    # GLSL + SPIR-V compilados
│   ├── cube/                   # Shaders de cubo básico
│   ├── simple/                 # Shaders básicos
│   ├── sdf/                    # Shaders SDF
│   ├── isr/                    # Shaders ISR
│   └── aa/                     # Shaders Anti-Aliasing
│
├── assets/                     # Modelos y texturas de ejemplo
│   ├── models/
│   └── textures/
│
├── docs/                       # Documentación
│   ├── manual.md               # Manual general
│   ├── rust-guide.md           # Guía Rust
│   ├── architecture.md         # Diagrama de arquitectura
│   └── Tareas.md               # Lista de tareas pendientes
│
└── Editor-REACTOR/             # Editor visual (egui + egui_dock)
    └── src/                    # Viewport, Hierarchy, Inspector, Console
```

## 🎮 Uso Avanzado

### Prelude
```rust
use reactor::prelude::*;
```

### Sistema de Iluminación
```rust
let mut lighting = LightingSystem::with_sun();

// Luz puntual
lighting.add_light(Light::point(
    Vec3::new(0.0, 5.0, 0.0),  // posición
    Vec3::new(1.0, 0.8, 0.6),  // color
    2.0,                        // intensidad
    20.0,                       // rango
));

// Spotlight
lighting.add_light(Light::spot(
    Vec3::new(0.0, 10.0, 0.0), // posición
    Vec3::NEG_Y,               // dirección
    Vec3::ONE,                 // color
    5.0,                       // intensidad
    30.0,                      // rango
    45.0,                      // ángulo
));
```

### Sistema de Partículas
```rust
// Presets
let mut fire = ParticleSystem::fire();
let mut explosion = ParticleSystem::explosion();
explosion.play();

// Personalizado
let config = ParticleSystemConfig {
    emission_rate: 100.0,
    lifetime: RandomRange::new(1.0, 2.0),
    start_color: Vec4::new(0.0, 0.5, 1.0, 1.0),
    ..Default::default()
};
let custom = ParticleSystem::new(config);
```

### Animaciones y Tweens
```rust
// Tween
let mut tween = Tween::new(0.0, 100.0, 2.0)
    .with_easing(EasingFunction::EaseOutElastic);

let value = tween.update(delta_time);

// AnimationPlayer
let mut player = AnimationPlayer::new();
player.add_clip(walk_animation);
player.play("walk");
let sample = player.update(delta_time);
sample.apply_to_transform(&mut transform);
```

### Física y Colisiones
```rust
let mut physics = PhysicsWorld::new();
physics.gravity = Vec3::new(0.0, -9.81, 0.0);

let mut body = RigidBody::default();
body.add_force(Vec3::new(100.0, 0.0, 0.0));

// Raycasting
let ray = Ray::from_screen(mouse_x, mouse_y, width, height, inv_vp);
if let Some(t) = ray.intersects_aabb(&aabb) {
    let hit_point = ray.point_at(t);
}

// Frustum Culling
let mut culling = CullingSystem::new();
culling.update_frustum(view_projection);
for object in &scene.objects {
    if culling.is_visible_aabb(&object.bounds) { /* renderizar */ }
}
```

### Post-Processing
```rust
let post = PostProcessPipeline::with_preset(PostProcessPreset::Cinematic);

let mut settings = PostProcessSettings::default();
settings.enable_effect(PostProcessEffect::Bloom);
settings.enable_effect(PostProcessEffect::Vignette);
settings.bloom_intensity = 0.5;
settings.vignette_intensity = 0.3;
```

### Debug Renderer
```rust
let mut debug = DebugRenderer::new();
debug.line(start, end, Vec4::new(1.0, 0.0, 0.0, 1.0));
debug.aabb(&DebugAABB { min, max }, Vec4::ONE);
debug.axes(origin, 1.0);
debug.grid(Vec3::ZERO, 10.0, 10, Vec4::new(0.5, 0.5, 0.5, 1.0));
debug.frustum(inv_view_proj, Vec4::new(1.0, 1.0, 0.0, 1.0));
```

### Primitivas Geométricas
```rust
let (vertices, indices) = Primitives::cube();
let (vertices, indices) = Primitives::sphere(32, 16);
let (vertices, indices) = Primitives::plane(10);
let (vertices, indices) = Primitives::cylinder(32, 2.0, 0.5);
let (vertices, indices) = Primitives::cone(32, 2.0, 0.5);
let (vertices, indices) = Primitives::torus(32, 16, 1.0, 0.3);
```

### Cámara 3D
```rust
let camera = Camera::perspective(45.0, aspect, 0.1, 1000.0)
    .look_at(eye, target, Vec3::Y);
let vp = camera.view_projection_matrix();

// Controles FPS
camera.rotate_yaw(mouse_delta.x * sensitivity);
camera.rotate_pitch(mouse_delta.y * sensitivity);
camera.move_forward(speed * delta);
```

## 📊 Comparación: Vulkan puro vs REACTOR

| Tarea               | Vulkan puro (Rust) | REACTOR     |
|---------------------|--------------------|-------------|
| Inicializar Vulkan  | ~300 líneas        | 1 línea     |
| Crear Buffer        | ~50 líneas         | 1 línea     |
| Crear Pipeline      | ~200 líneas        | 1 línea     |
| Renderizar Escena   | ~100 líneas        | 1 línea     |
| **Total típico**    | **800–1500 líneas**| **~50 líneas** |

---

---

## 📖 Documentación

| Documento                                  | Descripción                                       |
|--------------------------------------------|---------------------------------------------------|
| [Manual General](docs/manual.md)           | Manual de uso general                              |
| [Guía Rust](docs/rust-guide.md)            | Desarrollo de juegos con REACTOR en Rust           |
| [Arquitectura](docs/architecture.md)       | Diagrama de sistema, ownership, ABI interna       |
| [Cómo Compilar](HOW_BUILD.md)              | Guía paso a paso para compilar todo                |
| [**Fases del SDK**](Fases.md)              | **Roadmap completo para llegar a SDK v2.0**       |
| [Tareas pendientes](docs/Tareas.md)        | Backlog detallado de tareas                        |

### Ejemplos Rust

```bash
cargo run --example cube              # Cubo 3D con controles
cargo run --example textured_cube     # Cubo con textura
cargo run --example sandbox           # Sandbox experimental
cargo run --example physics_camera    # Cámara con física
cargo run --example obj_loader_demo   # Carga de modelos OBJ
```

---

## 🔄 Changelog

### v1.2.0 — UE5-style Core (Mayo 2026)
- Workspace Cargo real (reactor-vulkan + reactor-editor).
- Sistema de profiling jerárquico (`profile_scope!`, `CpuTimer`, `PerfCounter`).
- Logging estructurado (`tracing-subscriber`, `REACTOR_LOG` env var, `r_info!`/`r_warn!`/`r_error!`).
- Job System paralelo (rayon-backed: `parallel_for`, `join`, `scope`, `par_iter_mut`).
- Linear Allocator para datos por-frame (`LinearAllocator`, `BumpArena`, zero-fragmentation).
- Limpiado `lib.rs` de legacy y `*New` suffix — todos los exports ahora canónicos.
- Configuración de calidad: `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.
- Script `cleanup.ps1` / `cleanup.sh` para eliminar 14 archivos legacy dead-code.

### v1.1.0 — Rust Only (Mayo 2026)

**Migración a Rust puro:**

- ❌ **Eliminado:** C ABI (`cpp/reactor_c_api/`) del roadmap activo.
- ❌ **Eliminado:** C++ SDK (`cpp/reactor_cpp/`) del roadmap activo.
- ❌ **Eliminado:** Ejemplos C++ y dependencia de CMake / vcpkg.
- ✅ **Nuevo:** [`Fases.md`](Fases.md) — Roadmap completo para SDK v2.0.
- ✅ **Enfoque:** Un único stack, idiomático, mantenible y seguro.

### v1.0.5 — Febrero 2026

- FrameGraph declarativo (forward / deferred presets, auto-barriers).
- ECS con queries de componentes (Transform, MeshRenderer, Light, Camera, RigidBody).
- PBR (metallic / roughness + material instances + emissive + alpha modes).
- Telemetría (FPS, draw calls, triangles, VRAM, memory budget).
- Compute pipeline (dispatch + barriers).
- Play-mode bridge para futuro editor.
- Serialización de escena a JSON.
- Auto-compilación de shaders vía `build.rs`.
- MSAA 4x por defecto, Ray Tracing auto-detectado.
- 3000+ FPS en RTX 3060.

### v1.0.0 — v1.0.4

- Lifecycle base, input, camera, lighting, scene.
- Shaders SPIR-V embebidos.
- Editor REACTOR (`egui` + `egui_dock`): Viewport, Hierarchy, Inspector, Console, Asset Browser.

### v0.4.x

- Versión inicial en Rust.
- Vulkan 1.3 base.

---

## 📄 Licencia

TECHNE License — **Powered by Salazar-interactive**
