# REACTOR Framework — Architecture (100 % Rust)

## System Diagram

```diagram
╭─────────────────────────────────────────────────────────────╮
│                Tu Juego (impl ReactorApp)                   │
│                                                             │
│  reactor::run(MiJuego { … })                                │
│  reactor::quick("Mi Juego", w, h, |ctx| { … })              │
│  reactor::game! { title: "...", update: |ctx| { … } }       │
╰─────────────────────────────────────────────────────────────╯
                          │
                          ▼
╭─────────────────────────────────────────────────────────────╮
│                  REACTOR Core (Rust)                        │
│                                                             │
│  ReactorContext (engine + game systems en un struct):       │
│    • Reactor (Vulkan context)                               │
│    • Scene, Camera, Lighting, Physics, Culling, Debug       │
│    • Input, Time                                            │
│                                                             │
│  Subsistemas:                                               │
│    • core/        — VulkanContext, Device, Allocator        │
│    • graphics/    — Swapchain, Pipeline, MSAA, PostFX       │
│    • raytracing/  — RT Context, BLAS/TLAS, SBT              │
│    • compute/     — Pipeline, Dispatch, Particles GPU       │
│    • resources/   — Mesh, Material, Texture, PBR, Model     │
│    • systems/     — ECS, Camera, Lighting, Physics, Audio   │
│    • adead/       — ISR, SDF, RayMarching, AA, Hybrid       │
│    • utils/       — GPU/CPU detector, Time                  │
╰─────────────────────────────────────────────────────────────╯
                          │
                          ▼
╭─────────────────────────────────────────────────────────────╮
│              ash 0.38 (bindings Vulkan unsafe)              │
╰─────────────────────────────────────────────────────────────╯
                          │
                          ▼
╭─────────────────────────────────────────────────────────────╮
│            Vulkan 1.3 Driver  →  GPU Hardware               │
╰─────────────────────────────────────────────────────────────╯
```

## Lifecycle (resumen)

```
main()
  │
  └── reactor::run(app)
        │
        ├── EventLoop::new()                  // winit
        ├── Window::create()                  // OS window
        ├── Reactor::init(window)             // Vulkan setup
        ├── app.init(&mut ctx)                // user setup
        │
        └── [loop]
              ├── handle_event(event)         // input, resize, close
              ├── time.update()               // delta time
              ├── app.fixed_update(ctx, dt)   // física fija
              ├── app.update(&mut ctx)        // lógica de juego
              ├── app.render(&mut ctx)        // dibujado
              └── request_redraw()            // próximo frame
        │
        └── device_wait_idle()                // GPU sync antes de salir
```

## Ownership Model (Rust-only)

```
Regla: todos los recursos GPU viven en Rust con RAII.

    Tu juego                          Reactor / ash
    --------                          -------------
    let mesh = reactor.create_mesh(...)?;  → ManuallyDrop<Buffer> + Drop
    let mesh = Arc::new(mesh);             → shared ownership
    scene.add_object(mesh, mat, xf);       → Scene<Arc<Mesh>>
    ⇒ Al hacer drop de la Scene se liberan
       todos los buffers Vulkan en orden.
```

## Error Model

```rust
// Sin paneles en producción: Result<T, E> idiomático.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReactorError {
    #[error("Vulkan init failed: {0}")]
    VulkanInit(String),
    #[error("Shader compilation failed: {0}")]
    Shader(String),
    #[error("GPU out of memory")]
    OutOfMemory,
    #[error("Invalid argument: {0}")]
    InvalidArgument(&'static str),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type ReactorResult<T> = Result<T, ReactorError>;
```

## Frame Lifecycle (modo manual)

```rust
let mut reactor = Reactor::init(&window)?;

while !should_close {
    reactor.begin_frame()?;       // tiempo, input, sincronización
    reactor.draw_scene(&scene, &vp)?;
    reactor.end_frame()?;         // submit + present
}
```

## Reglas del Core

1. **Cero `unsafe` filtrado al usuario** — todo lo `unsafe` queda bajo abstracciones seguras.
2. **`Result<T, ReactorError>`** en todas las APIs públicas (sin `panic!`).
3. **`Drop` correcto** — en orden inverso de creación; los handles Vulkan jamás se filtran.
4. **`Arc<Device>`** para compartir el dispositivo entre subsistemas.
5. **Frames-in-flight** configurables (1, 2, 3) con semáforos timeline.
6. **Validación** activada automáticamente en `cfg(debug_assertions)`.

## Layer Summary

| Capa     | Lenguaje    | Responsabilidad                              |
|----------|-------------|----------------------------------------------|
| Game     | Rust        | Lógica del juego, implementa `ReactorApp`    |
| Core     | Rust        | Vulkan, memoria, sistemas (`reactor` crate)  |
| Editor   | Rust        | UI visual con `egui` (opcional)              |
| Bindings | Rust unsafe | `ash` 0.38 (Vulkan 1.3 raw)                  |
| Driver   | Vulkan 1.3  | Comandos GPU                                 |
| Hardware | GPU         | Renderizado real                             |

> **Sin C, sin C++, sin FFI hacia el exterior.** Todo el stack es Rust.
