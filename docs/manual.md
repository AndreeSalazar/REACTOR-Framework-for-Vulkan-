# REACTOR Framework — Manual General

**Version 1.1.0-rust** | Vulkan 1.3 | 100 % Rust puro

## ¿Qué es REACTOR?

REACTOR es un framework de desarrollo de **videojuegos en Rust puro** sobre Vulkan 1.3.
Aprovecha el sistema de ownership y RAII de Rust para **seguridad de memoria** y
**zero-overhead**, exponiendo una API declarativa estilo `ReactorApp`.

## Instalación

### Requisitos

- **Rust 1.70+** ([rustup](https://rustup.rs/))
- **Vulkan SDK 1.3+** ([vulkan.lunarg.com](https://vulkan.lunarg.com/))
- **GPU compatible con Vulkan**

### Clonar y compilar

```bash
git clone https://github.com/user/REACTOR-Framework-for-Vulkan.git
cd REACTOR-Framework-for-Vulkan
cargo build --release
```

## Uso Básico

### Patrón `ReactorApp`

```
1. Crear struct con tu estado de juego.
2. Implementar `ReactorApp` (config / init / update).
3. Llamar `reactor::run(MiJuego { ... })`.
```

### Ejemplo Mínimo

```rust
use reactor::prelude::*;

struct MiJuego;

impl ReactorApp for MiJuego {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Mi Juego").with_size(1280, 720)
    }
    fn init(&mut self, _ctx: &mut ReactorContext) {}
    fn update(&mut self, _ctx: &mut ReactorContext) {}
}

fn main() { reactor::run(MiJuego); }
```

### Forma SUPER corta (`reactor::quick`)

```rust
use reactor::prelude::*;

fn main() {
    reactor::quick("Mi Juego", 1280, 720, |ctx| {
        ctx.camera.position.x = ctx.time.elapsed().sin() * 5.0;
    });
}
```

### Macro `reactor::game!`

```rust
reactor::game! {
    title: "Mi Juego",
    size: (1280, 720),
    update: |ctx| {
        ctx.camera.position.x = ctx.time.elapsed().sin() * 5.0;
    }
}
```

## Funciones Principales

### Cámara

```rust
ctx.camera.position = Vec3::new(0.0, 2.0, 5.0);
ctx.camera.look_at(Vec3::ZERO, Vec3::Y);
```

### Iluminación

```rust
ctx.lighting.add_light(Light::directional(Vec3::NEG_Y, Vec3::ONE, 1.0));
ctx.lighting.add_light(Light::point(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 2.0, 20.0));
```

### Escena

```rust
let mesh = Arc::new(reactor.create_mesh(&vertices, &indices)?);
let mat  = Arc::new(reactor.create_material()?);
ctx.scene.add_object(mesh, mat, Mat4::IDENTITY);
```

### Input

```rust
if ctx.reactor.input.key_pressed(KeyCode::Escape) {
    // cerrar...
}
```

### Telemetría

```rust
let fps = ctx.time.fps();
let dt  = ctx.time.delta();
```

## Arquitectura

```diagram
╭──────────────╮     ╭──────────────╮     ╭──────────────╮
│  Ash / Vulkan│────▶│ Reactor Core │────▶│  Tu Juego    │
│   (unsafe)   │     │ (safe + RAII)│     │ (ReactorApp) │
╰──────────────╯     ╰──────────────╯     ╰──────────────╯
```

## Características

| Módulo        | Descripción |
|---------------|-------------|
| Core          | `VulkanContext`, `Device`, `Allocator`, `CommandManager` |
| Graphics      | Swapchain, Pipeline, MSAA, Depth, PostProcessing, Shadows |
| Ray Tracing   | RTX en GPUs compatibles (auto-detectado) |
| Compute       | `ComputePipeline`, `ComputeDispatch`, Barriers |
| Resources     | Mesh, Material, Texture, Primitives, glTF/OBJ |
| ECS           | Entity CRUD, Transform, MeshRenderer, Light, Camera, RigidBody, Queries |
| PBR           | Metallic/Roughness, Material Instances, Emissive |
| FrameGraph    | Render passes declarativos, presets Forward/Deferred |
| Systems       | Input, Camera, Lighting, Physics, Animation, Audio, Particles |
| Telemetry     | `RenderStats`, `MemoryBudget`, GPU info, VRAM |
| ADead-GPU     | ISR, SDF, Ray Marching, Anti-Aliasing, Hybrid Rendering |
| Editor        | `egui` + `egui_dock` (Viewport, Hierarchy, Inspector, Console) |

## Actualizaciones

### v1.1.0-rust (Mayo 2026)

- Migración a **Rust puro** — eliminado C ABI y C++ SDK.
- Nueva API corta: `reactor::quick(...)` y macro `reactor::game!`.
- [`Fases.md`](../Fases.md): roadmap completo hasta SDK v2.0.

### v1.0.5 (Febrero 2026)

- ECS con queries (bitmask), PBR Materials, FrameGraph, Telemetría, PlayMode.
- Editor REACTOR (`egui` + `egui_dock`).
- Shaders SPIR-V embebidos, Ray Tracing auto-detectado, MSAA 4x.
- 3000+ FPS en RTX 3060.

### v0.4.x

- Versión inicial Rust, Vulkan 1.3 base, sistema ADead-GPU.

## Soporte

- **Docs:** `/docs/` (manual, rust-guide, architecture, Tareas).
- **Ejemplos:** `/examples/` (5 demos).
- **Editor:** `/Editor-REACTOR/` (`egui`).

## Licencia

MIT — **Powered by Salazar-interactive**
