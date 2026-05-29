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

---

## 🧭 Filosofía — Por qué existe REACTOR

Vulkan es la API gráfica moderna más explícita y veloz del mercado, pero también la
más verbosa: inicializar el contexto, crear un swapchain, compilar shaders, gestionar
descriptors y orquestar barreras de sincronización exige **cientos de líneas** antes
siquiera de pintar el primer triángulo. Cada juego acaba reescribiendo el mismo
boilerplate.

**REACTOR resuelve este problema con un modelo de 3 capas:**

| Capa | Quién la construye | Cuándo | Para qué |
|------|-------------------|--------|---------|
| **1. Vulkan base** | Se construyó **una vez** en `src/core/` y `src/graphics/` | Al inicializar el proceso | Bindings `ash`, instance, device, queues, swapchain, command pools, allocator, descriptors, render passes, pipelines |
| **2. REACTOR cocina** | Se cocina **al arrancar la app** (`init`) y en `build.rs` | En cada ejecución / compilación | FrameGraph, ECS, PBR, post-process, ray tracing, audio, input, partículas, físicas, shadows, VRS — todo listo, sin tocar Vulkan |
| **3. Tu juego encima** | Lo construyes **tú** en cualquier `.rs` | Cada vez que iteras | `impl ReactorApp` — `config / init / update / on_event` y tu lógica de gameplay |

> **Idea fuerza:** *La base Vulkan ya está construida y REACTOR la deja cocinada
> — encima pones cualquier archivo `.rs` para construir tu juego con control total
> y sin renunciar a Vulkan.*

---

## 🏗️ Arquitectura — Las 3 capas

```diagram
╭───────────────────────────────────────────────────────────────╮
│              CAPA 3 — TU JUEGO (cualquier .rs)                │
│   examples/xenofall.rs · examples/cube.rs · src/bin/mi_juego  │
│   ─ Implementas `ReactorApp`: config / init / update / event  │
╰──────────────────────────────┬────────────────────────────────╯
                               │ usa API segura (Vec3, Mat4, spawn_*)
                               ▼
╭───────────────────────────────────────────────────────────────╮
│        CAPA 2 — REACTOR COCINA (src/, todo Rust safe)         │
│  reactor::    Scene · Camera · Lighting · Physics · Audio     │
│  graphics::   FrameGraph · PostProcess · MSAA · Shadows · VRS │
│  resources::  Mesh · Material · Texture · Model (OBJ/glTF)    │
│  systems::    Input · ECS · Animation · Particles · Culling   │
│  build.rs:    GLSL → SPIR-V automático                        │
╰──────────────────────────────┬────────────────────────────────╯
                               │ envuelve con RAII + safe wrappers
                               ▼
╭───────────────────────────────────────────────────────────────╮
│         CAPA 1 — VULKAN BASE (src/core/, único unsafe)        │
│  ash bindings · Device · Allocator · CommandManager · Surface │
│  Swapchain · Pipeline · DescriptorSets · Sync Primitives      │
╰───────────────────────────────────────────────────────────────╯
```

---

## 🧱 CAPA 1 — La base Vulkan (qué se implementó y por qué)

La base Vulkan vive en `src/core/`, `src/graphics/`, `src/raytracing/` y `src/compute/`.
Es **el único lugar donde existe `unsafe`** en todo el framework.

### `src/core/` — Fundamentos del dispositivo

| Archivo | Implementa | Por qué |
|---------|-----------|---------|
| `context.rs` | `VulkanContext` (instance, surface, physical/logical device, queues) | Punto único de creación; centraliza extensiones (`VK_KHR_swapchain`, `VK_KHR_acceleration_structure`, `VK_KHR_fragment_shading_rate`, etc.) y validation layers (`VK_LAYER_KHRONOS_validation`) |
| `device.rs` | Selección de GPU física por features y queue families | Reactor escoge la GPU con mejor soporte (RT, mesh shaders, VRS) automáticamente |
| `allocator.rs` | Integración con `gpu-allocator` (sub-allocation, MemoryLocation::GpuOnly/CpuToGpu) | Reemplaza llamadas crudas a `vkAllocateMemory` con un asignador inteligente que evita fragmentación |
| `command.rs` | `CommandManager`: pools, primary/secondary buffers, one-shot helpers | Encapsula `vkBeginCommandBuffer/vkEndCommandBuffer` y libera pools al drop |
| `surface.rs` | Surface KHR cross-platform (Win32 / Wayland / X11 / MacOS via MoltenVK) | Una sola API para crear la superficie sin escribir branches por OS |
| `debug_utils.rs` | `VK_EXT_debug_utils` callback (errores, warnings, perf) | Logs claros con etiquetas en color en debug; cero overhead en release |
| `error.rs` | `ReactorError` con `thiserror` | Convierte `vk::Result` a errores idiomáticos `Result<T, E>` |
| `arc_handle.rs` | RAII genérico para handles Vulkan (`Arc<ArcHandle<T>>`) | Garantiza destrucción ordenada y comparte handles entre frames sin doble-free |
| `frame_graph.rs` | Render Graph declarativo (passes, recursos, dependencias, auto-barriers) | Sustituye a escribir manualmente cientos de `vkCmdPipelineBarrier` |
| `importance_map.rs` | Mapa de importancia por tile para VRS adaptativo | Alimenta `VK_KHR_fragment_shading_rate` con datos del frame anterior |
| `jobs.rs` | Job System paralelo (rayon) — `parallel_for`, `join`, `scope` | Para sistemas que paralelizan animación, físicas, culling |
| `linear_allocator.rs` | `LinearAllocator` / `BumpArena` (per-frame) | Datos efímeros sin fragmentación; reset O(1) por frame |
| `logging.rs` | `tracing-subscriber` + macros `r_info!/r_warn!/r_error!` con env `REACTOR_LOG` | Logging estructurado JSON o consola, niveles por módulo |
| `memory_budget.rs` | `VK_EXT_memory_budget` queries | Telemetría VRAM real (no estimada) para HUD y debug |
| `profiler.rs` | `profile_scope!`, `CpuTimer`, `PerfCounter` | Profiling jerárquico CPU sin instrumentación externa |
| `vrs.rs` | Pixel Inteligente (VRS = Variable Rate Shading) | Reduce trabajo de fragment shader en zonas de baja importancia; +20-40% FPS sin pérdida visible |

### `src/graphics/` — Pipeline gráfico

| Archivo | Implementa | Por qué |
|---------|-----------|---------|
| `swapchain.rs` | `VK_KHR_swapchain` con vsync dinámico (FIFO/MAILBOX/IMMEDIATE) y recreación on-resize | Cambiar VSync sin recrear toda la app |
| `pipeline.rs` | Graphics Pipeline (shader stages, vertex input, raster, blend, depth, MSAA) | Builder typed-state; imposible olvidar un estado obligatorio |
| `pso_cache.rs` + `pso_hash.rs` | `VkPipelineCache` persistido a disco + hash de descripción | Compilación de pipelines casi instantánea tras la primera ejecución |
| `render_pass.rs` | RenderPass + Subpasses + AttachmentDescription | API segura para forward/deferred; auto-resolve MSAA |
| `framebuffer.rs` | Framebuffer wrapper | Re-crea en resize sin leaks |
| `buffer.rs` | Vertex/Index/Uniform/Storage buffer (`VkBuffer` + `gpu-allocator`) | Una sola API tipada; sub-allocation transparente |
| `image.rs` | `VkImage` + `VkImageView` (color, depth, cube, array) | Transiciones de layout chequeadas por tipo |
| `sampler.rs` | Samplers con LOD bias, anisotropy, address modes, compare | Pool de samplers compartidos para ahorrar handles |
| `descriptors.rs` | DescriptorPool + DescriptorSetLayout + bind helpers | Evita el clásico "out of descriptors" con pools auto-grow |
| `depth.rs` | Z-buffer con formato detectado (D32_SFLOAT / D24_S8 / D16) | Auto-fallback según GPU |
| `msaa.rs` | Multi-Sample Anti-Aliasing 2x/4x/8x/16x (color + depth resolve) | Smooth edges sin shaders extra |
| `shadows.rs` | Shadow maps (directional + spot + omnidireccional con cubemap) | PCF de 3x3/5x5; bias automático |
| `post_process.rs` | Pipeline de post-process con push constants compactos | Vignette, Bloom, ToneMap (ACES), Chromatic, Film Grain, FXAA, Sharpen, SSGI, Volumetric Fog, LUT, SSR, Path-Traced Resolve, Anamorphic Flares, **Pause Overlay** |
| `bindless.rs` | Descriptor indexing (`VK_EXT_descriptor_indexing`) — texturas/buffers por índice | Permite miles de materiales con un solo set; base para ray tracing |
| `indirect.rs` | `vkCmdDrawIndexedIndirect` + GPU-driven rendering | La GPU genera sus propios draw calls |
| `mesh_shader.rs` | Mesh shaders (`VK_EXT_mesh_shader`) opt-in | Sustituye al pipeline tradicional cuando el HW lo soporta |
| `shader_compiler.rs` | Compilación GLSL → SPIR-V vía `glslc` (build.rs) + caché | Sin precompilar a mano |
| `shader_hot_reload.rs` | Watch de archivos `.frag/.vert/.comp` con re-compilación en caliente | Iteración rápida sin reiniciar el juego |
| `uniform_buffer.rs` | Uniform buffers triple-buffered (uno por frame en vuelo) | Sin sincronización extra entre CPU y GPU |
| `debug_renderer.rs` | Líneas, AABB, ejes, frustums, grid | Visual debugging in-game |

### `src/raytracing/` — RT Hardware (Vulkan 1.3 + KHR)

| Archivo | Implementa | Por qué |
|---------|-----------|---------|
| `context.rs` | `RayTracingContext` (auto-detección de `VK_KHR_ray_tracing_pipeline`, `VK_KHR_acceleration_structure`, `VK_KHR_ray_query`) | Carga el SDK RT solo si la GPU lo soporta |
| `acceleration_structure.rs` | BLAS + TLAS con build/refit | Acelera trazado a O(log n) sobre la geometría |
| `pipeline.rs` | Ray Tracing Pipeline (raygen/miss/closest-hit/any-hit/intersection) | Trazado completo de rayos en GPU |
| `shader_binding_table.rs` | SBT con stride respetado por device | Sin esto, RT pipeline crashea silenciosamente |

### `src/compute/` — Compute shaders

| Archivo | Implementa | Por qué |
|---------|-----------|---------|
| `pipeline.rs` | `ComputePipeline` (descriptor + shader) | Para frustum cull, partículas, post-process compute |
| `dispatch.rs` | Helpers `dispatch_1d/2d/3d` con local-size auto-calculado | Olvida calcular `(n + ls - 1) / ls` en cada call |
| `particles.rs` | Sistema de partículas GPU-driven (1M+ partículas) | Demo de compute + indirect draw |

---

## ⚙️ CAPA 2 — REACTOR cocina (qué deja listo y para qué)

Todo lo que sigue es **Rust 100% safe** y consume los wrappers de la Capa 1.
Lo que pasa "automáticamente" entre `init()` y tu primer frame:

1. Detección de GPU/CPU/resolución óptima → `utils/gpu_detector.rs`, `cpu_detector.rs`, `resolution_detector.rs`.
2. Compilación / carga de shaders SPIR-V → `graphics/shader_compiler.rs` + `builtin_shaders.rs` (shaders embebidos).
3. Construcción del FrameGraph según `RendererMode` (Forward / BindlessForward) → `renderer/`.
4. Setup del `Scene`, cámara, iluminación, audio, input, físicas → `reactor/init.rs`.
5. Carga asíncrona de assets (modelos, texturas, audio) → `resources/asset_loader_queue.rs`.

### `src/reactor/` — Fachada principal

| Archivo | Implementa | Por qué |
|---------|-----------|---------|
| `mod.rs` | Struct `Reactor` (singleton de rendering) | Tu punto único de acceso a la GPU desde gameplay |
| `init.rs` | Bootstrap completo (Vulkan + post-process + RT) | Una sola llamada `Reactor::new(config)` |
| `draw.rs` | Loop de un frame (acquire, record, submit, present) | Maneja triple buffering y MAILBOX |
| `swapchain_recreate.rs` | Recreación segura on-resize y on-vsync-toggle | Sin glitches al cambiar de ventana a fullscreen |
| `render_pass.rs` | Selección dinámica de pass según features activas | PBR + MSAA + Shadows en una pasada cuando es posible |
| `events.rs` | Re-export y filtrado de WindowEvent | Para que tu app solo vea eventos relevantes |
| `resources.rs` | Helpers `spawn_cube`, `spawn_plane`, `spawn_gltf_smart`, `set_transform`, `move_blob_shadow` | Una línea = un objeto en escena |
| `depth.rs`, `msaa.rs` | Helpers de configuración runtime | Cambiar MSAA sin recrear todo |

### `src/app/` — El trait `ReactorApp`

| Archivo | Implementa | Por qué |
|---------|-----------|---------|
| `app.rs` | Trait `ReactorApp` + `ReactorContext` | Tu juego sólo implementa 4 métodos |
| `config.rs` | `ReactorConfig` (size, vsync, msaa, renderer, physics_hz, fullscreen) | Configuración declarativa |
| `runner.rs` | `reactor::run(app)` — main loop con winit | Una línea y arrancas |
| `pause_config.rs` | `PauseConfig` con páginas (Display/Lighting/Color/Performance/Presets) | HUD de pausa con overlay dibujado por el fragment shader |

### `src/systems/` — Sistemas de gameplay

| Sistema | Capacidades clave |
|---------|------------------|
| `audio.rs` | Backend `rodio` con `AudioListener`, SFX 3D atenuado, música streaming, `play_sfx(clip, position, volume)` |
| `lighting.rs` | `LightingSystem`, `Light::directional/point/spot`, sombras dinámicas, IBL, sun preset |
| `particles.rs` | `ParticleSystem` con presets `fire/explosion/smoke/sparkle`, `RandomRange`, color over lifetime |
| `physics.rs` | `PhysicsWorld` con gravedad, `RigidBody`, `Ray`, AABB, raycasting, esfera-vs-AABB, fixed timestep |
| `animation.rs` | `Tween` (29 easings), `AnimationPlayer`, `AnimationClip`, sampling, glTF skeletal opcional |
| `frustum.rs` | `CullingSystem` con frustum 6-plane test (AABB/Sphere) |
| `fps_controller.rs` | Controlador FPS first-person reutilizable (WASD + mouse) |
| `scene.rs` | Wrapper alto nivel: añadir/quitar objetos, queries por componente |
| `event_bus.rs` | Pub/sub interno para desacoplar sistemas |
| `console.rs` | Consola tipo Quake con comandos y autocompletado |

### `src/scene/` — ECS y entidades

| Archivo | Implementa | Por qué |
|---------|-----------|---------|
| `ecs.rs` | Entity CRUD, queries por componente | Sin frameworks externos pesados |
| `transform.rs` | `Transform` jerárquico (parent/child) con matrices cacheadas | Mover un padre mueve los hijos |
| `camera.rs` | `Camera::perspective/orthographic`, `look_at`, FPS controls, view/projection cacheadas | Una API para todas las cámaras |
| `light.rs` | Componente `Light` (tipo, color, intensidad, rango, ángulo, shadow caster) | Adjunta luces a entidades |

### `src/resources/` — Assets

| Archivo | Implementa |
|---------|-----------|
| `vertex.rs` | `Vertex` (pos/normal/uv/tangent), `BindingDescription`, `AttributeDescription` |
| `mesh.rs` | `Mesh` (vertex + index buffer), bounding box auto-calculado |
| `material.rs` + `pbr_material.rs` | Material PBR (metallic/roughness/normal/occlusion/emissive), alpha modes |
| `texture.rs` | Carga JPG/PNG/HDR + mipmaps generados en GPU |
| `model.rs` + `gltf_loader.rs` | Carga OBJ y glTF 2.0 (escenas multi-malla, jerarquía, materiales, animaciones) |
| `primitives.rs` | `Primitives::cube/sphere/plane/cylinder/cone/torus` |
| `font.rs` | Carga de fuentes TTF para HUD |
| `asset_cooker.rs` + `asset_database.rs` | "Cocción" de assets (precompilados a `.reactor`) para arranque instantáneo |
| `asset_hot_reload.rs` | Watch de `assets/` con recarga en caliente |
| `asset_loader_queue.rs` | Carga asíncrona con workers (rayon) |
| `asset_id.rs` + `handle.rs` | IDs estables + `Handle<T>` genérico |

### `src/platform/` — Ventana, input, tiempo

| Archivo | Implementa |
|---------|-----------|
| `window.rs` | Window winit cross-platform, fullscreen borderless dinámico |
| `input.rs` | Teclado/ratón con `is_key_just_pressed`, `is_mouse_button_down`, posición/delta |
| `gamepad.rs` | Gamepad multiplataforma (XInput / DualSense / Switch Pro) |
| `time.rs` | `Time`, `FixedTimestep`, FPS calc |
| `config.rs` | Carga config desde `.reactor/` |

### `src/renderer/` — Renderers selectables

| Archivo | Implementa | Por qué |
|---------|-----------|---------|
| `forward.rs` | Forward renderer clásico (geometría + luces por píxel) | Mejor con MSAA, pocas luces |
| `bindless_forward.rs` | Forward con descriptor indexing (cientos de materiales) | Escala bien con muchos materiales únicos |

---

## 🎮 CAPA 3 — Tu juego encima (1 archivo, control total)

### Quick start mínimo

```rust
use reactor_vulkan::prelude::*;

struct MiJuego { rotacion: f32 }

impl ReactorApp for MiJuego {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Mi Juego")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_msaa(4)
            .with_renderer(RendererMode::Forward)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        ctx.camera.position = Vec3::new(0.0, 2.0, 4.0);
        ctx.add_sun();
        let _cube = ctx.spawn_cube(Vec3::ZERO).unwrap();
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        self.rotacion += ctx.delta();
        ctx.set_transform(0, Mat4::from_rotation_y(self.rotacion));
    }
}

fn main() { reactor_vulkan::run(MiJuego { rotacion: 0.0 }); }
```

### Ejemplo completo de gameplay — `examples/xenofall.rs`

Un **Rail Shooter Roguelite** de 2.197 líneas que demuestra que **todo el stack
funciona**: glTF con esqueleto, audio 3D, post-process, VRS, sombras, partículas,
sistema de cartas roguelite, 8 oleadas, HUD overlay dibujado por shader, pausa con
configuración tactical en tiempo real.

| Mecánica | Subsistema REACTOR usado |
|----------|-------------------------|
| Cámara sobre rieles + bob | `Camera` + `Time` |
| Apuntado con mouse (ray-cast) | `Input` + `Camera` ray inverse |
| TAP/HOLD/Normal damage | `Input::is_mouse_button_down` con timing |
| Combo x10 + score cap | Lógica propia + `Time` |
| Recarga R, 8 balas | `Input::is_key_just_pressed` |
| Pausa P/Esc con overlay shader | `PauseConfig` + `post_process.frag::draw_pause_overlay` |
| 8 oleadas crecientes | `WaveDef` propio + spawn helpers |
| Cartas roguelite (9 tipos) | `Build` propio |
| Enemigos zombie glTF (1.8m UE5-scale) | `ctx.spawn_gltf_smart()` |
| Audio (disparos, recargas, impactos, groans) | `audio::play_sfx(clip, pos, vol)` |
| Tracers + impacts pooled | `spawn_cube` reutilizados |
| Blob shadows por enemigo | `ctx.spawn_blob_shadow / move_blob_shadow` |
| HUD en título de ventana | `ctx.set_title(...)` |
| **Sin un solo `unsafe`** | Toda Capa 1 está encapsulada |

Y todo eso **sin tocar Vulkan jamás**.

---

## ✨ Tabla resumen de módulos

| Módulo | Características |
| ------ | --------------- |
| **Core** | `VulkanContext`, `Device`, `Allocator`, `CommandManager`, `Surface`, `FrameGraph`, `Jobs`, `Profiler`, `LinearAllocator`, `MemoryBudget`, `VRS` |
| **Graphics** | Swapchain, Pipeline (+PSO cache), RenderPass, Framebuffer, Buffer, Image, Sampler, Descriptors, Depth, MSAA 2-16x, UniformBuffers, DebugRenderer, PostProcessing (13 efectos), Shadows (dir/spot/omni), Bindless, Indirect, Mesh shaders, Shader hot-reload |
| **Ray Tracing** | `RayTracingContext`, BLAS/TLAS, RT Pipeline, SBT |
| **Compute** | `ComputePipeline`, `ComputeDispatch`, Particles GPU |
| **Resources** | Mesh, Material (PBR), Texture (JPG/PNG/HDR), Vertex, Model (OBJ/glTF), Primitives, Asset Cooker, Hot-reload, Async loader |
| **ECS** | Entity CRUD, Transform jerárquico, MeshRenderer, Light, Camera, RigidBody, Queries |
| **PBR** | Metallic/Roughness, instances, emissive, alpha modes (Opaque/Mask/Blend) |
| **FrameGraph** | Render passes declarativos, gestión de recursos, presets Forward/BindlessForward, auto-barriers |
| **Systems** | Input, Scene, Camera, Transform, Lighting, Physics, FrustumCulling, Animation (29 easings), Particles, Audio 3D (rodio) |
| **Telemetry** | RenderStats (FPS, draw calls, triangles, VRAM), MemoryBudget, GPU info, CPU info |
| **Utils** | GPUDetector, CPUDetector, ResolutionDetector, Time, FixedTimestep, Math helpers, Hash |
| **Editor** *(opcional)* | Editor visual con `egui` + `egui_dock` (Viewport, Hierarchy, Inspector, Console) |

---

## 🚀 Quick Start

### Requisitos
- [Rust](https://rustup.rs/) 1.70+
- [Vulkan SDK](https://vulkan.lunarg.com/) 1.3+
- (Opcional) `glslangValidator` / `glslc` — los shaders se compilan automáticamente vía `build.rs`.

### Ejecutar ejemplos

```bash
cargo run --example cube              # Cubo 3D con controles
cargo run --example textured_cube     # Cubo con textura PBR
cargo run --example physics_camera    # Cámara con física + raycast
cargo run --example obj_loader_demo   # Carga de modelos OBJ
cargo run --example inspect_model     # Inspector de glTF (jerarquía, materiales)
cargo run --example generate_audio    # Generador de SFX procedurales
cargo run --release --example xenofall # ⚡ Demo completa Rail Shooter
```

> Para Xenofall recomendado `--release` por su carga de assets y VRS.

---

## 🧪 Demostraciones — Todos los elementos funcionan

Cada ejemplo prueba un sub-sistema:

| Ejemplo | Demuestra que funciona |
|---------|----------------------|
| `cube.rs` | Capa 1 entera (Vulkan init, swapchain, pipeline, drawing) |
| `textured_cube.rs` | Texturas + samplers + descriptors |
| `physics_camera.rs` | `PhysicsWorld`, `Ray::intersects_aabb`, `Camera` FPS |
| `obj_loader_demo.rs` | `Model::load_obj`, materiales, normales |
| `inspect_model.rs` | `gltf_loader.rs`, escenas jerárquicas, animaciones skeletal |
| `generate_audio.rs` | `Audio::play_sfx`, listener 3D, atenuación por distancia |
| `xenofall.rs` | TODO junto: input, audio 3D, glTF, post-process 13 efectos, VRS, sombras, pausa overlay, HUD, partículas, lógica de juego |

---

## 🛠️ Uso avanzado — Recetas

### Sistema de iluminación

```rust
let mut lighting = LightingSystem::with_sun();
lighting.add_light(Light::point(Vec3::new(0.0, 5.0, 0.0), Vec3::new(1.0, 0.8, 0.6), 2.0, 20.0));
lighting.add_light(Light::spot(Vec3::new(0.0, 10.0, 0.0), Vec3::NEG_Y, Vec3::ONE, 5.0, 30.0, 45.0));
```

### Sistema de partículas

```rust
let mut fire = ParticleSystem::fire();
let mut explosion = ParticleSystem::explosion();
explosion.play();

let custom = ParticleSystem::new(ParticleSystemConfig {
    emission_rate: 100.0,
    lifetime: RandomRange::new(1.0, 2.0),
    start_color: Vec4::new(0.0, 0.5, 1.0, 1.0),
    ..Default::default()
});
```

### Animaciones y tweens

```rust
let mut tween = Tween::new(0.0, 100.0, 2.0)
    .with_easing(EasingFunction::EaseOutElastic);
let value = tween.update(dt);

let mut player = AnimationPlayer::new();
player.add_clip(walk_animation);
player.play("walk");
let sample = player.update(dt);
sample.apply_to_transform(&mut transform);
```

### Física y colisiones

```rust
let mut physics = PhysicsWorld::new();
physics.gravity = Vec3::new(0.0, -9.81, 0.0);

let mut body = RigidBody::default();
body.add_force(Vec3::new(100.0, 0.0, 0.0));

// Raycast
let ray = Ray::from_screen(mouse_x, mouse_y, width, height, inv_vp);
if let Some(t) = ray.intersects_aabb(&aabb) {
    let hit_point = ray.point_at(t);
}

// Frustum culling
let mut culling = CullingSystem::new();
culling.update_frustum(view_projection);
for object in &scene.objects {
    if culling.is_visible_aabb(&object.bounds) { /* render */ }
}
```

### Post-processing (13 efectos)

```rust
let post = PostProcessPipeline::with_preset(PostProcessPreset::Cinematic);
let s = &mut ctx.reactor.post_process.settings;
s.enable_effect(PostProcessEffect::Bloom);
s.enable_effect(PostProcessEffect::Vignette);
s.enable_effect(PostProcessEffect::SSGI);
s.enable_effect(PostProcessEffect::PathTracedLighting);
s.bloom_intensity = 0.5;
s.vignette_intensity = 0.3;
```

### PauseConfig in-game (UI dibujada por el fragment shader)

```rust
let mut pause = PauseConfig::new().with_title("MI JUEGO — CONFIG");
// Al presionar P o Esc:
pause.show(ctx);                  // overlay_alpha = 0.78
// Cada frame en estado Paused:
let result = pause.update(ctx);   // teclas F1-F5, ↑↓←→, V, F, etc.
if result.requested_resume { /* volver a jugar */ }
if result.requested_quit { std::process::exit(0); }
```

### Debug renderer

```rust
let mut debug = DebugRenderer::new();
debug.line(start, end, Vec4::new(1.0, 0.0, 0.0, 1.0));
debug.aabb(&DebugAABB { min, max }, Vec4::ONE);
debug.axes(origin, 1.0);
debug.grid(Vec3::ZERO, 10.0, 10, Vec4::new(0.5, 0.5, 0.5, 1.0));
debug.frustum(inv_view_proj, Vec4::new(1.0, 1.0, 0.0, 1.0));
```

### Pixel Inteligente (VRS)

```rust
use reactor_vulkan::core::PixelIntelligentProfile;
ctx.reactor.set_pixel_intelligent_profile(PixelIntelligentProfile::Performance);
// Off · Quality · Balanced · Performance · UltraPerformance
let rate = ctx.reactor.pixel_intelligent_rate(); // e.g. 2x2
```

---

## 📊 Comparación: Vulkan puro vs REACTOR

| Tarea               | Vulkan puro (Rust) | REACTOR     |
|---------------------|--------------------|-------------|
| Inicializar Vulkan  | ~300 líneas        | 1 línea     |
| Crear Buffer        | ~50 líneas         | 1 línea     |
| Crear Pipeline      | ~200 líneas        | 1 línea     |
| Renderizar escena   | ~100 líneas        | 1 línea     |
| Cargar glTF + skin  | ~600 líneas        | 1 línea     |
| Post-process stack  | ~400 líneas        | 1 línea     |
| Ray Tracing setup   | ~800 líneas        | ~10 líneas  |
| **Total típico**    | **2.000+ líneas**  | **~50 líneas** |

---

## 📁 Estructura del proyecto

```text
REACTOR-Framework-for-Vulkan-/
├── image.svg                   # Logo (Salazar-interactive)
├── README.md                   # Este archivo
├── Cargo.toml                  # Workspace + crate principal
├── build.rs                    # Auto-compilación GLSL → SPIR-V
│
├── src/                        # ──────── REACTOR (todas las capas)
│   ├── lib.rs                  # Exports + prelude
│   ├── builtin_shaders.rs      # SPIR-V embebido (no requiere SDK runtime)
│   │
│   ├── app/                    # CAPA 3: trait `ReactorApp`, runner, config, PauseConfig
│   ├── reactor/                # Fachada de rendering — un frame completo
│   ├── core/                   # CAPA 1: Vulkan base (único `unsafe`)
│   ├── graphics/               # CAPA 1: Pipeline gráfico
│   ├── raytracing/             # CAPA 1: RT Hardware
│   ├── compute/                # CAPA 1: Compute shaders
│   ├── renderer/               # CAPA 2: Renderers (Forward / BindlessForward)
│   ├── resources/              # CAPA 2: Mesh, Material, Texture, Model, Cooker
│   ├── scene/                  # CAPA 2: ECS, Transform, Camera, Light
│   ├── systems/                # CAPA 2: Input, Audio, Lighting, Physics, …
│   ├── platform/               # CAPA 2: Window, Input, Gamepad, Time
│   ├── utils/                  # Detectores y math helpers
│   └── bin/                    # Binarios auxiliares (editor, herramientas)
│
├── examples/                   # CAPA 3: demos
│   ├── cube.rs                 # Cubo 3D básico
│   ├── textured_cube.rs        # Cubo con textura
│   ├── physics_camera.rs       # Física + raycast
│   ├── obj_loader_demo.rs      # OBJ loader
│   ├── inspect_model.rs        # glTF inspector
│   ├── generate_audio.rs       # SFX procedural
│   └── xenofall.rs             # ⚡ Rail Shooter Roguelite completo
│
├── shaders/                    # GLSL + SPIR-V compilados
│   ├── post_process.frag       # 13 efectos + pause overlay
│   ├── bindless_pbr.frag       # PBR bindless
│   ├── frustum_cull.comp       # Frustum culling en GPU
│   ├── cube/ simple/ texture/  # Shaders básicos
│   ├── sdf/                    # Shaders SDF (signed distance fields)
│   ├── isr/                    # ISR (Image Super Resolution)
│   └── aa/                     # Anti-Aliasing
│
├── assets/                     # Modelos, texturas y audio fuente
├── cooked_assets/              # Assets cocinados (.reactor) — arranque rápido
├── .reactor/                   # Config persistente del usuario
├── target/                     # Build artifacts (cargo)
│
└── docs/
    ├── manual.md
    ├── rust-guide.md
    └── architecture.md
```

---

## 🔭 Más ideas — Qué puede implementar/reconstruir REACTOR

REACTOR está diseñado para **crecer**. La Capa 1 expone toda la API oficial de
Vulkan 1.3, así que cualquier técnica documentada en
[vulkan.org](https://www.vulkan.org/) puede subir a Capa 2 como wrapper safe.
Roadmap abierto:

### Renderizado avanzado

- **Visibility Buffer / Nanite-style virtual geometry** (cluster culling + soft rasterization en compute)
- **GPU-driven scene** completo (todo el frame se construye en GPU con `vkCmdDrawIndexedIndirectCount`)
- **Path Tracing en tiempo real** con denoiser (RTX 4000+/AMD 7000+)
- **Restir DI/GI** (Spatiotemporal reservoir resampling)
- **DLSS / FSR 3 / XeSS** vía cross-vendor abstraction sobre VRS + temporal upscaling
- **Frame Generation** propia con `VK_KHR_present_wait` + motion vectors
- **Hi-Z occlusion culling** en compute (depth pyramid)
- **Tiled / Clustered Forward+ Lighting** (cientos de luces dinámicas)
- **Volumetric Clouds + GI temporales** (Schneider/Wronski)
- **Subsurface Scattering** (skin, wax, foliage)
- **Hair & Fur** con tessellation o mesh shaders
- **Mesh Shading** completo (sustituir vertex/index pipeline en HW compatible)
- **Variable Rate Shading 4x4** + foveated rendering para VR

### Ray Tracing avanzado

- **Inline ray queries** (`VK_KHR_ray_query`) en rasterización para shadows blandas y AO
- **DDGI (Dynamic Diffuse Global Illumination)** con probes RT
- **Reflection probes RT** filtradas por roughness
- **Material LOD** para BLAS (geometría procedural)

### Compute & GPGPU

- **Cloth simulation** (PBD / XPBD) en compute
- **Fluid sim 2D/3D** (FLIP/PIC, SPH, MPM)
- **Soft-body** (mass-spring + Verlet)
- **Skeletal animation en GPU** con dual-quaternion blending
- **Neural networks inference** (Cooperative Matrix `VK_KHR_cooperative_matrix`)

### Plataforma y herramientas

- **WebGPU backend** secundario (wgpu) para distribución web
- **Vulkan Video** (`VK_KHR_video_*`) para playback H.264/H.265/AV1 a GPU
- **OpenXR** para VR/AR (Quest, Index, Vision Pro)
- **Bevy-style scheduler** opcional para usuarios que prefieran ECS estricto
- **Hot-reload de Rust** vía `dlopen` + `wasm` para scripting de gameplay
- **Editor REACTOR** maduro con timeline de animación, blueprint visual, profiler GPU
- **Asset pipeline distribuido** (CDN + delta patching)
- **Networking** (rollback, prediction, lockstep)
- **Save system** con `serde` + versionado de esquema

### Audio

- **Convolución HRTF** para audio binaural
- **Reverb / occlusion** por raycast en geometría
- **Music stems** con crossfade dinámico (FMOD/Wwise-style)
- **Procedural SFX** (síntesis granular y subtractiva)

> Cada idea de arriba **se puede prototipar en horas** porque la Capa 1 ya expone
> el handle Vulkan crudo si lo necesitas, mientras que la Capa 2 te da la red de
> seguridad para no romper el resto del frame.

---

## 🎬 Hacia iluminación AAA — Qué tenemos y qué falta

Esta es la auditoría honesta del stack de iluminación actual y el camino para
llegar a calidad **Triple-A profesional** (UE5 Lumen / Frostbite / Decima / RED).

### ✅ Lo que ya está cocinado en la base

| Pieza | Dónde vive | Estado |
|-------|-----------|--------|
| ACES Filmic Tone Mapping | `shaders/shader.frag` + `post_process.frag` | Funcional |
| Hemispherical Ambient | `shader.frag` | Funcional |
| Schlick Fresnel + GGX-inspired specular | `shader.frag` | **Simplificado** (no es Cook-Torrance completo) |
| Soft shadows analíticos (ray-cylinder) | `shader.frag::getPillarShadow` | **Ad-hoc**: hardcoded para corredor de Xenofall |
| Contact AO geométrico | `shader.frag::getContactShadow` | **Ad-hoc**: distancias hardcoded |
| SSGI aproximado (rebote de luces neón) | `shader.frag::sceneSpaceGlobalIllumination` | **Ad-hoc**: posiciones hardcoded |
| SSS aproximado | `shader.frag::subsurfaceScatter` | Funcional pero sin albedo difuso real |
| Volumetric fog + Mie phase | `shader.frag` | Funcional |
| Cinematic LUT grading | `shader.frag::cinematicLutGrade` | Funcional |
| Post-process 13 efectos | `shaders/post_process.frag` | Funcional |
| Pause overlay UI | `post_process.frag::draw_pause_overlay` | Funcional |
| `ShadowConfig` + `ShadowCascade` + `ShadowMap` structs | `src/graphics/shadows.rs` | **Andamio**: `update()` ignora `camera_view/proj`; `calculate_shadow_factor` comenta "_real implementation would sample shadow map_" — falta wiring |
| Luces múltiples por uniform buffer | `src/graphics/uniform_buffer.rs` (`MAX_LIGHTS = 16`) | Funcional pero **techo bajo** para AAA |
| `bindless_pbr.frag` | `shaders/` | **Stub**: solo devuelve `albedo × base_color`, sin lighting |

### ❌ Lo que falta para iluminación Triple-A

Cada item de abajo es una **caja de cocina** que se construye sobre la Capa 1
existente (Vulkan + descriptors + compute + RT). No hay que tocar `unsafe`.

#### 1. Material PBR completo (Cook-Torrance real)

- **D**isney/GGX Trowbridge-Reitz **D**(h) microfacet normal distribution
- **G**eometry term Smith Schlick-GGX (correlated)
- **F**resnel Schlick con ior y f90
- **Energy compensation** para multi-scattering (Kulla-Conty / Turquin)
- **Anisotropy** (T/B tangent frame) para pelo, metales cepillados
- **Clear coat** (capa transparente sobre coches, plásticos)
- **Sheen** (ropa, terciopelo) — Charlie distribution
- **Transmission + thickness** (vidrio, hielo, agua)
- **Iridescence** (capas finas, jabón, mariposas)
- **Burley/Disney diffusion SSS** (piel, mármol, hojas) — reemplazo del hack actual

> **Por qué**: el `bindless_pbr.frag` actual no ilumina. Sin BRDF correcto,
> materiales se ven "planos" y reaccionan mal a IBL/lights dinámicas.

#### 2. Image-Based Lighting (IBL) completo

- **Pre-filtered specular cubemap** (mipmaps por roughness, split-sum approx.)
- **Irradiance cubemap** convolucionado (diffuse ambient direccional)
- **BRDF LUT 2D** (split-sum integration, RG16F texture)
- **Box/Sphere parallax-corrected reflection probes** colocables en escena
- **Sky cubemap dinámico** (atmosphere precomputed o tiempo real)

> **Por qué**: el ambient hemisférico actual es sólo 2 colores constantes. IBL
> hace que cualquier material reaccione al entorno real.

#### 3. Cascaded Shadow Maps reales (wiring completo)

El struct existe — falta:

- Render passes a 4 shadow textures (1 por cascada) con `vkCmdBeginRenderPass`
- **PSSM** (Practical Split Scheme) con `cascade_splits` ya configurados
- Sampling en fragment shader con bias por cascada + slope-scaled depth bias
- **PCF Poisson disk** o **PCSS real** (blocker search + penumbra)
- **Variance / Exponential / Moment Shadow Maps** para soft shadows sin artefactos
- **Contact-hardening shadows** (PCSS con kernel adaptativo)
- **Shadow caching** para luces estáticas (no re-render cada frame)
- **Distance Field Shadows** (UE5 Lumen-style) sobre SDFs precalculados

#### 4. Renderer Deferred + G-Buffer

Actualmente solo Forward. Para AAA con cientos de luces:

- **G-Buffer layout** (R11G11B10F + RGBA8 + RGBA8 + RG16F):
  - `gbuffer0`: Base color (RGB) + AO (A)
  - `gbuffer1`: World normal octahedral (RG) + metallic + roughness
  - `gbuffer2`: Emissive (RGB) + material ID (A)
  - `gbuffer3`: Motion vectors (RG) + linear depth (B) + flags (A)
- **Lighting pass** en compute shader sobre G-Buffer
- **Decals** proyectados en G-Buffer entre geometry y lighting

#### 5. Clustered/Tiled Forward+ Lighting

- **Light culling en compute**: dividir frustum en tiles 16×16 o froxels 16×16×32
- Cada tile/cluster sólo itera luces relevantes (`MAX_LIGHTS = 16` → ilimitado)
- Estructura jerárquica BVH para 10.000+ luces dinámicas

#### 6. Global Illumination en tiempo real

Una o varias de estas estrategias (de menos a más AAA):

- **SSGI real en compute** (raymarch del depth/color buffer, no hardcoded)
- **DDGI (Dynamic Diffuse GI)** con probes en grid 3D + relighting RT
- **Surfel GI** (EA SEED) — partículas de irradiancia sobre superficies
- **Voxel Cone Tracing (VXGI)** — voxelizar escena + cone trace en compute
- **SDFGI** (Godot 4 / Distance Field GI software)
- **Lumen** (UE5 software + hardware RT, screen-trace + mesh SDF)
- **ReSTIR DI/GI** — Spatiotemporal reservoir resampling (state of the art 2024)

#### 7. Screen-Space Effects "reales" en compute

- **GTAO / HBAO+** sobre depth + normal G-buffer (no hardcoded geometry)
- **SSR (Screen-Space Reflections)** con BVH hierarchical ray-march
- **Contact shadows** SSCS (Screen-Space Contact Shadows)
- **Bent Normals** para AO direccional

#### 8. Anti-aliasing temporal + upscaling

- **TAA real** con motion vectors, neighborhood clipping, history rectification
- **DLSS 3.5** (NVIDIA Streamline SDK)
- **FSR 3.1** (AMD FidelityFX)
- **XeSS** (Intel)
- **Sub-pixel jitter** en projection matrix
- **TSR** (UE5 Temporal Super Resolution) como fallback open-source

#### 9. Bloom "AAA"

El bloom actual es single-pass blur. AAA usa:

- **Downsample/upsample mip chain** (13 mips a 1/2, 1/4, … COD Advanced Warfare paper)
- **Karis average** anti-firefly en el downsample
- **Tent filter** en upsample para difusión suave
- **Lens dirt + chromatic bloom** con texture overlay

#### 10. Auto-exposure HDR

- **Histograma de luminancia** en compute (256 bins)
- **Eye adaptation** temporal (smooth EV adjustment)
- **HDR display output**: `VK_EXT_hdr_metadata` + scRGB / HDR10 / Dolby Vision
- **Color spaces**: BT.709 (SDR) ↔ BT.2020 (HDR) ↔ ACEScg (work)

#### 11. Volumetric lighting AAA

- **Froxel volumetrics** (Frostbite: 160×90×128 froxels)
- **Volumetric shadows** desde CSM en cada froxel
- **Participating media**: humo, niebla, polvo con scattering anisotrópico
- **God rays** desde sol/spot lights con CSM sampling
- **Volumetric clouds** (Schneider/Wronski raymarch + curl noise)

#### 12. Motion blur + Depth of Field

- **Motion blur per-object** sampling motion vectors
- **DoF bokeh** con circle-of-confusion buffer + hex/circle aperture
- **Lens flares anamórficos** (ya hay flag, falta implementación real)

#### 13. Atmosphere & Sky

- **Bruneton precomputed scattering** (Rayleigh + Mie LUTs)
- **Time-of-day** dinámico (sol/luna/estrellas)
- **Aerial perspective** desde atmosphere LUT
- **Realtime clouds** integrados con sky

#### 14. Ray Tracing Hardware (DXR equivalent vía VK_KHR_ray_*)

La base existe en `src/raytracing/`. Falta cocinarla con:

- **RT Shadows** (área luces, soft shadows perfectos)
- **RT Reflections** (espejos, agua, metales — sin SSR artifacts)
- **RT GI** (DDGI o ReSTIR como arriba)
- **RT AO** (alta calidad, sin screen-space leaks)
- **RT Translucency / caustics**
- **Hybrid Rendering**: raster G-Buffer + RT para reflexiones/sombras/GI

#### 15. Especialidades de personaje

- **Skin shader doble-lobe SSS** (Burley + textura de SSS thickness)
- **Eye shader** con cornea refractiva + scleral SSS
- **Hair Marschner / Kajiya-Kay** BRDF + alpha-to-coverage
- **Cloth Charlie sheen** + anisotropic
- **Wetness** dinámica (lluvia → roughness/metallic)

#### 16. Naturaleza / mundo abierto

- **Vegetation wind** (vertex shader perlin)
- **Tree LOD + imposters**
- **Terrain**: virtual texturing, blend de N capas, displacement
- **Water**: Gerstner waves + foam + refraction + caustics + SSR
- **Atmosphere fog** integrado con water y volumetrics

---

### 🗺️ Hoja de ruta sugerida (orden recomendado)

```diagram
╭─────────────────────────────────────────────────────╮
│ FASE 1 — Cimientos PBR + Shadows reales             │
│ ▸ Cook-Torrance completo en bindless_pbr.frag       │
│ ▸ CSM wiring (4 cascadas, PCF Poisson)              │
│ ▸ G-Buffer + Deferred opcional                      │
│ ▸ MAX_LIGHTS dinámico vía SSBO                      │
╰──────────────────────┬──────────────────────────────╯
                       ▼
╭─────────────────────────────────────────────────────╮
│ FASE 2 — Ambient + Reflections                      │
│ ▸ IBL completo (cubemap pre-filt + irrad + BRDF LUT)│
│ ▸ Reflection probes parallax-corrected              │
│ ▸ SSR + SSAO/GTAO reales en compute                 │
╰──────────────────────┬──────────────────────────────╯
                       ▼
╭─────────────────────────────────────────────────────╮
│ FASE 3 — Temporal + HDR                             │
│ ▸ Motion vectors + TAA                              │
│ ▸ Auto-exposure histograma                          │
│ ▸ Bloom mip chain + lens dirt                       │
│ ▸ Tonemap pipeline (ACES → AgX → OCIO)              │
│ ▸ HDR10 output                                      │
╰──────────────────────┬──────────────────────────────╯
                       ▼
╭─────────────────────────────────────────────────────╮
│ FASE 4 — GI dinámica                                │
│ ▸ Clustered Forward+ light culling                  │
│ ▸ DDGI o Surfel GI con probes RT                    │
│ ▸ Volumetric froxels + god-rays                     │
│ ▸ Atmosphere Bruneton + clouds                      │
╰──────────────────────┬──────────────────────────────╯
                       ▼
╭─────────────────────────────────────────────────────╮
│ FASE 5 — Ray Tracing hardware (el "wow")            │
│ ▸ RT Shadows / Reflections / AO híbridos            │
│ ▸ ReSTIR GI                                         │
│ ▸ Path tracing ground truth opcional + SVGF denoiser│
│ ▸ DLSS/FSR/XeSS para 4K-8K                          │
╰─────────────────────────────────────────────────────╯
```

**Tiempo estimado por fase**: 2-4 semanas cada una para un developer con la
base actual. Cada fase **deja un demo visible y vendible** (no es un waterfall
de 6 meses sin output).

> 💡 La base actual ([src/](file:///c:/Users/andre/OneDrive/Desktop/REACTOR-Framework-for-Vulkan-/src)) ya expone todos los handles Vulkan necesarios:
> descriptors bindless, compute pipelines, RT context con auto-detection,
> uniform/storage buffers, mipmaps, samplers anisotrópicos, MSAA resolve.
> **Falta cocinar shaders y wiring** — no falta infraestructura.

---

## 🔗 REACTOR ⇄ Blender Live Link — Construir el juego en tiempo real

**Visión:** abrir Blender, modelar / iluminar / animar, y que **REACTOR muestre
el resultado al instante** en una ventana paralela — sin exportar, sin reiniciar,
sin tocar archivos. Cuando guardas en Blender, el juego se actualiza vivo.
El binomio **Blender + REACTOR = un nuevo juego construido más rápido**.

```diagram
╭─────────────────────────╮   delta sync (TCP/WS)  ╭──────────────────────────╮
│  BLENDER 4.x            │ ◀────────────────────▶ │  REACTOR runtime         │
│  ─ Addon Python         │   JSON / MessagePack   │  ─ Bridge server (Rust)  │
│  ─ Panel "REACTOR Live" │   60 Hz tick           │  ─ Live scene mutator    │
│  ─ Operadores Bake/Push │                        │  ─ Hot-reload assets     │
│  ─ Preview camera link  │   eventos bidireccional│  ─ Picking back-channel  │
╰─────────┬───────────────╯                        ╰─────────┬────────────────╯
          │                                                  │
          ▼                                                  ▼
   tu .blend (verdad)                              tu juego corriendo
          │                                                  │
          ╰─────────────── un solo .reactor project ─────────╯
```

> **Carpeta nueva propuesta:** [`reactor-blender-bridge/`](file:///c:/Users/andre/OneDrive/Desktop/REACTOR-Framework-for-Vulkan-/reactor-blender-bridge)
> en la raíz del repo, separada de `src/` para que sea instalable como add-on
> de Blender independiente y cocinable por el motor.

---

### 📁 Estructura propuesta de la carpeta

```text
reactor-blender-bridge/
├── README.md                        # Guía de instalación + flujo de uso
├── blender_addon/                   # ── LADO BLENDER (Python) ───────────
│   ├── __init__.py                  # bl_info + register_module
│   ├── manifest.toml                # Manifest Blender 4.2+ (extensions)
│   ├── prefs.py                     # Preferencias addon (puerto, host, modo)
│   ├── panel.py                     # Panel "REACTOR Live" en N-panel del 3D View
│   ├── operators/
│   │   ├── connect.py               # Conectar / desconectar al runtime
│   │   ├── push_scene.py            # Empujar escena entera (cold start)
│   │   ├── bake_assets.py           # Cocinar texturas/meshes a cooked_assets/
│   │   ├── play_in_reactor.py       # Lanzar REACTOR runtime y empezar live
│   │   ├── pick_entity.py           # Click en viewport REACTOR → seleccionar en Blender
│   │   └── snapshot.py              # Capturar frame REACTOR a Image Editor
│   ├── handlers/
│   │   ├── depsgraph.py             # depsgraph_update_post → delta
│   │   ├── frame_change.py          # frame_change_post → animation tick
│   │   ├── save.py                  # save_post → cocinar assets cambiados
│   │   └── undo_redo.py             # rebuild state al deshacer
│   ├── encoders/
│   │   ├── mesh.py                  # bpy.Mesh → vertices/indices/uv/tangent
│   │   ├── material.py              # bpy.Material PBR → REACTOR material
│   │   ├── light.py                 # bpy.Light → Light
│   │   ├── camera.py                # bpy.Camera → Camera
│   │   ├── transform.py             # matrix_world → Mat4 (Z-up → Y-up convert)
│   │   ├── animation.py             # NLA tracks / actions → AnimationClip
│   │   ├── armature.py              # bones → Skeleton + skin weights
│   │   ├── shape_keys.py            # blendshapes / morph targets
│   │   ├── texture.py               # imágenes → DDS/KTX2/PNG (cooked)
│   │   └── collection.py            # bpy.Collection → ECS hierarchy
│   ├── transport/
│   │   ├── websocket.py             # cliente WS (asyncio)
│   │   ├── tcp_socket.py            # cliente TCP raw (fallback)
│   │   ├── shared_memory.py         # mmap para mesh data grande
│   │   └── protocol.py              # serialización delta (msgpack)
│   └── ui/
│       ├── icons.py                 # iconos REACTOR
│       └── status_bar.py            # estado conexión en status bar
│
├── reactor_bridge/                  # ── LADO REACTOR (Rust crate) ───────
│   ├── Cargo.toml                   # Crate `reactor-bridge`
│   └── src/
│       ├── lib.rs                   # Re-exports + ReactorBridge plugin
│       ├── server.rs                # tokio TCP/WS server (puerto 19840)
│       ├── protocol.rs              # tipos compartidos (serde + msgpack)
│       ├── live_scene.rs            # Apply delta → ctx.scene mutator
│       ├── asset_pipe.rs            # Recibir texturas/meshes, hot-swap
│       ├── picking.rs               # Mouse pick → enviar entity_id de vuelta
│       ├── preview_camera.rs        # Sync cámara Blender ↔ REACTOR
│       ├── recorder.rs              # Grabar timeline para "scrub" en Blender
│       └── coord_convert.rs         # Z-up ⇄ Y-up, handedness
│
├── proto/                           # ── PROTOCOLO COMÚN (cross-lang) ────
│   ├── messages.fbs                 # FlatBuffers schema (opcional)
│   ├── messages.proto               # Protobuf alternativa
│   └── README.md                    # Especificación de mensajes
│
├── examples/
│   ├── basic_sync.blend             # Escena demo (cubo + luz + cámara)
│   ├── animated_character.blend     # Char con armature + actions
│   ├── pbr_materials.blend          # Demo materiales
│   └── live_lighting.blend          # Demo iluminación en vivo
│
└── tests/
    ├── test_mesh_encoding.py        # PyTest del lado Blender
    └── test_live_scene.rs           # cargo test del lado Rust
```

---

### 🎯 Hoja de ruta — 8 fases con tareas detalladas

#### 🔧 FASE 0 — Cimientos del protocolo (semana 1)

- [ ] Diseñar el **transport** definitivo (recomendado: WebSocket sobre `tokio-tungstenite` + msgpack-rs en Rust; `websockets` + `msgpack` en Python).
- [ ] Definir el **enum de mensajes** en `proto/messages.fbs` o `protocol.rs`:
   - `Handshake { version, capabilities[] }`
   - `Heartbeat { ts }`
   - `EntityCreated { id, kind, parent }`
   - `EntityRemoved { id }`
   - `TransformUpdated { id, matrix4x4 }`
   - `MeshUploaded { id, vertices, indices, attributes[] }`
   - `MaterialUpdated { id, pbr_params }`
   - `LightUpdated { id, light_data }`
   - `CameraUpdated { id, projection, view }`
   - `TextureUploaded { id, format, mip_chain }`
   - `AnimationKeyframe { rig_id, time, pose }`
   - `PickRequest { screen_x, screen_y }` → `PickResponse { entity_id }`
   - `ScenePushFull` / `ScenePushDelta`
- [ ] Versionado del protocolo (`PROTOCOL_VERSION = 1`) con backward compat.
- [ ] Logs estructurados a ambos lados (tracing en Rust, `bpy.app.debug`).
- [ ] **Test de ping/pong** Blender ↔ REACTOR midiendo latencia.

#### 🐍 FASE 1 — Addon Blender mínimo (semana 2)

- [ ] Esqueleto del addon Blender 4.2+ con `manifest.toml` (extensions).
- [ ] Panel "REACTOR Live" en N-panel del 3D Viewport con: botón Connect, IP, puerto, estado.
- [ ] Operador `reactor.connect` que abre WebSocket con handshake.
- [ ] Operador `reactor.push_scene` que serializa la escena actual (mesh + transform + cámara) y la empuja.
- [ ] Operador `reactor.disconnect` con cleanup limpio.
- [ ] Status bar global mostrando "REACTOR: connected · 60 Hz · 12 entities".

#### 🦀 FASE 2 — Bridge server REACTOR (semana 2-3)

- [ ] Crate `reactor-bridge` en `reactor-blender-bridge/reactor_bridge/`.
- [ ] `ReactorBridge` como **plugin opcional**: si está activo, añade un `tokio` runtime y server WS al lado del main loop Vulkan.
- [ ] **`live_scene.rs`**: aplica un mensaje delta a `ReactorContext::scene` de forma thread-safe (canal `mpsc` consumido en `update()`).
- [ ] Mensaje `EntityCreated → spawn_cube/spawn_gltf_smart`.
- [ ] Mensaje `TransformUpdated → ctx.set_transform`.
- [ ] Mensaje `LightUpdated → lighting.add_light / mutate`.
- [ ] Manejo de errores: si Blender se desconecta, REACTOR sigue corriendo.

#### 🎨 FASE 3 — Sincronización de geometría y materiales (semana 3-4)

- [ ] **Encoder de mesh** Python: extraer `vertices, normals, uvs, tangents, indices` de `bpy.Mesh.calc_loop_triangles()`.
- [ ] Compresión **Draco** opcional para meshes pesadas.
- [ ] **Encoder de material PBR**: leer el shader Principled BSDF de Blender → base_color, metallic, roughness, emissive, normal map, alpha mode.
- [ ] **Encoder de imagen**: detectar texturas, exportar a PNG/KTX2 en directorio temporal, enviar handle.
- [ ] **Hot-swap de assets** en REACTOR: cuando llega `TextureUploaded`, re-bind descriptor sin recrear pipeline.
- [ ] **Conversión de coordenadas**: Blender es Z-up RH, REACTOR es Y-up RH → matriz de cambio de base en `coord_convert.rs`.
- [ ] **Auto-bake de normal maps** si el material usa nodos procedurales no PBR.
- [ ] Test E2E: modificar color de un material en Blender → se ve en REACTOR en < 100 ms.

#### 🦴 FASE 4 — Animaciones y rigs (semana 4-5)

- [ ] **Encoder de armature**: extraer jerarquía de huesos + bind pose + skin weights por vértice (4 huesos/vert).
- [ ] **Encoder de actions**: muestrear cada F-Curve a 30/60 fps → keyframes Vec3/Quat.
- [ ] Mensaje `AnimationClipUploaded` con todas las pose tracks.
- [ ] **Shape keys / blendshapes**: enviar los morph targets como `MorphTarget`.
- [ ] Sync de **frame_current**: cuando el usuario hace scrub en el timeline de Blender, REACTOR pinta ese frame exacto.
- [ ] Botón "**Play in REACTOR**" que lanza la animación a velocidad real.
- [ ] Soporte de **NLA tracks** (stripes de animaciones combinables).

#### 💡 FASE 5 — Iluminación y cámaras en vivo (semana 5)

- [ ] **Encoder de luces** Blender (Sun / Point / Spot / Area) → `Light::*` REACTOR.
- [ ] Mover una luz en Blender → REACTOR re-ilumina en el siguiente frame.
- [ ] **Sync de world background**: HDRI / sky color de Blender → IBL en REACTOR.
- [ ] **Encoder de cámaras**: Blender camera → `Camera::perspective` con FOV/clip planes.
- [ ] **Preview camera lock**: tu cámara de Blender = cámara de REACTOR en tiempo real.
- [ ] **Toggle "Game Camera vs Editor Camera"** en el panel.

#### 🖱️ FASE 6 — Bidireccional: REACTOR → Blender (semana 6)

- [ ] **Picking back-channel**: click en la ventana de REACTOR → REACTOR raycast → envía `PickResponse { entity_id }` → Blender selecciona el objeto correspondiente.
- [ ] **Gizmos en runtime**: arrastrar un objeto en el viewport de REACTOR con un gizmo → enviar nueva transform a Blender → `bpy.object.matrix_world = ...`.
- [ ] **Recorder de gameplay**: grabar las transforms del jugador en REACTOR → importar a Blender como F-Curve (cinemática automática).
- [ ] **Screenshot operator**: capturar frame REACTOR → enviar como imagen al Image Editor de Blender.
- [ ] **Runtime stats panel**: FPS, draw calls, VRAM de REACTOR visibles en Blender.

#### ⚙️ FASE 7 — Asset cooker integrado (semana 7)

- [ ] Operador "**Cook & Push**" en Blender:
  1. Aplica modificadores destructivamente.
  2. Triangula meshes.
  3. Genera mipmaps de texturas.
  4. Empaqueta todo en `cooked_assets/<scene_name>.reactor`.
  5. Empuja al runtime.
- [ ] Watcher de `.blend` → re-cook automático al guardar (opcional).
- [ ] **Cache inteligente**: solo re-cook lo que cambió (hash de meshes/materials).
- [ ] **Progress bar** en Blender durante el cook.
- [ ] **Deploy mode**: cook final + empaquetado para distribución.

#### 🚀 FASE 8 — Productividad y QoL (semana 8+)

- [ ] **Live scripting**: editar `gameplay.lua` o `gameplay.rhai` en el text editor de Blender, recargar en REACTOR sin reiniciar.
- [ ] **Console interactiva**: enviar comandos REPL desde Blender al runtime ("teletransport jugador a 0,5,0").
- [ ] **Profiler integrado**: ver el GPU/CPU profiler de REACTOR como gráfico dentro de Blender.
- [ ] **Multi-cliente**: dos artistas conectados al mismo runtime para playtesting cooperativo.
- [ ] **Record / replay** de sesiones de live sync (debuggable).
- [ ] **VR preview**: si el usuario tiene VR, ver el juego en HMD desde Blender directamente.
- [ ] **AI assist**: usar IA local para auto-rig, auto-UV, auto-LOD durante el cook.
- [ ] **Export final**: botón "Build Standalone" que compila el juego cocinado a `.exe` / `.app` / `.AppImage`.

---

### 🧪 Sub-sistemas con tareas atómicas

#### Transport layer (escoger UNO o varios)

- [ ] **WebSocket** (`tokio-tungstenite` + `websockets`): cross-platform, debug fácil con DevTools, latencia ~1 ms en localhost. **← RECOMENDADO**.
- [ ] **TCP raw** (`std::net::TcpStream` + `socket`): mínimo overhead, requiere framing manual.
- [ ] **Named pipes / Unix sockets**: latencia más baja, sólo localhost.
- [ ] **Shared memory** (`mmap` + ringbuffer): para meshes pesadas (> 10 MB).
- [ ] **gRPC** (`tonic`): si quieres tipado fuerte y streaming oficial.
- [ ] **Cap'n Proto / FlatBuffers**: zero-copy parsing.

#### Encoder reglas (Blender → REACTOR)

- [ ] Axis convention: Blender `Z-up RH` → REACTOR `Y-up RH` (mat4 swap).
- [ ] Units: Blender metros → REACTOR metros (1:1).
- [ ] Color space: Blender Linear → REACTOR Linear (sRGB conv en shader).
- [ ] Mesh: triangulate, generar tangents si faltan (MikkTSpace).
- [ ] UVs: flip V si tu texture loader lo necesita.
- [ ] Animation: tiempo Blender (frames @ scene.fps) → segundos REACTOR.
- [ ] Bone weights: normalizar suma=1, top-4 weights, descartar resto.

#### Conflict resolution

- [ ] Estrategia "Blender es la verdad" (cambios en REACTOR no persisten).
- [ ] Estrategia "Last writer wins" con timestamps.
- [ ] Estrategia "CRDT" para edición multi-usuario (avanzado).

#### Performance budgets

- [ ] Delta push **< 16 ms** end-to-end para mantener 60 fps en Blender.
- [ ] Mesh upload máx **5 MB/frame** (chunked si más).
- [ ] Texture upload **async background** (no bloquea el frame).
- [ ] Throttle: si Blender está editando un slider, agrupar updates a 30 Hz.
- [ ] Bandwidth target: **< 100 KB/s** en idle, **< 10 MB/s** en edición activa.

#### Seguridad y robustez

- [ ] Sólo `localhost` por defecto (sin exposición red).
- [ ] Token opcional `REACTOR_BRIDGE_TOKEN` env var para autenticar.
- [ ] Reconnect automático si REACTOR muere y se relanza.
- [ ] Graceful degradation: si llega un mensaje desconocido, log y continuar.
- [ ] Validación de IDs (evitar `unwrap()` con entity_id ajeno).

#### Testing

- [ ] **Unit Python**: encoders de mesh/material (PyTest + fixtures `.blend`).
- [ ] **Unit Rust**: `cargo test` del `live_scene::apply_delta`.
- [ ] **Integration**: spawn Blender headless (`blender -b -P test.py`) + REACTOR runtime + script de assertions.
- [ ] **Latency benchmark**: medir tiempo entre `obj.location.x = 5` y `set_transform` en REACTOR.
- [ ] **Fuzz protocol**: enviar mensajes malformados, REACTOR no debe panic.

#### Documentación

- [ ] [README de instalación](file:///c:/Users/andre/OneDrive/Desktop/REACTOR-Framework-for-Vulkan-/reactor-blender-bridge) con GIFs.
- [ ] Tutorial "**De Blender vacío a juego corriendo en 5 minutos**".
- [ ] Referencia de cada mensaje del protocolo.
- [ ] Recetas: "Sincronizar luces", "Animar un personaje", "Sustituir material en vivo".
- [ ] Vídeo de 3 min mostrando el flujo completo.

---

### 💡 Ideas extra (stretch goals para diferenciarse)

- [ ] **REACTOR Geometry Nodes**: poder usar nodos de Blender que generan geometría procedural y se evaluen en GPU vía compute shader REACTOR.
- [ ] **Shader Graph bidireccional**: el editor de nodos Shading de Blender genera GLSL/SPIR-V para REACTOR (Principled BSDF → bindless PBR material).
- [ ] **Bake de iluminación a probes**: usar Cycles para bakear DDGI probes y enviarlos a REACTOR como cubemaps.
- [ ] **Live debugging visual**: render gizmos REACTOR (colliders, raycast lines, ECS bounds) en el viewport de Blender.
- [ ] **Asset Browser bridge**: el Asset Browser de Blender muestra los `cooked_assets/` con thumbnails generados por REACTOR.
- [ ] **VSE (Video Sequence Editor) cinemáticas**: grabar gameplay en REACTOR → traer la timeline al VSE para edición no-lineal.
- [ ] **Storyboard mode**: definir secuencias en Blender Grease Pencil → REACTOR las anima.
- [ ] **Networking de prueba**: dos instancias REACTOR conectadas para playtesting LAN, controladas desde una sola Blender.
- [ ] **Live scripting con Rhai/Lua**: escribir gameplay en el text editor de Blender, REACTOR lo hot-reload.
- [ ] **Profile presets**: Cinematic / Mobile / VR / Lowend que reconfiguran REACTOR vía un dropdown.
- [ ] **DCC bridge generalizado**: misma arquitectura para integrar Maya, Houdini, ZBrush, Substance Painter.
- [ ] **Web preview**: además de la ventana nativa, un viewer WebGPU en el navegador que recibe el mismo stream.
- [ ] **Recorder → glTF**: exportar todo lo enviado en una sesión como un glTF estático para distribución.
- [ ] **AI co-pilot in Blender**: panel con LLM que sugiere mejoras de gameplay/iluminación viendo el stream.
- [ ] **Marketplace de presets**: comunidad sube presets de iluminación / shaders / setups que se aplican con un click.

---

### 🏁 Definition of Done — Cuándo está listo

✅ **MVP listo si:** crear un cubo en Blender + asignarle material rojo + añadir un sun light → aparecen en REACTOR con menos de 100 ms de latencia, sin reiniciar nada.

✅ **v1.0 listo si:** un usuario abre Blender, sigue el tutorial de 5 minutos, y termina con un personaje 3D iluminado y animado corriendo en REACTOR sin tocar la línea de comandos.

✅ **v2.0 listo si:** un equipo de 2 artistas + 1 programador edita simultáneamente desde Blender + VS Code, juega en REACTOR, y publica un build standalone con el botón "Deploy".

---

## 📖 Documentación

| Documento                                  | Descripción                                       |
|--------------------------------------------|---------------------------------------------------|
| [Manual general](docs/manual.md)           | Manual de uso general                              |
| [Guía Rust](docs/rust-guide.md)            | Desarrollo de juegos con REACTOR en Rust           |
| [Arquitectura](docs/architecture.md)       | Diagrama de sistema, ownership, ABI interna       |
| [Cómo compilar](HOW_BUILD.md)              | Guía paso a paso para compilar todo                |
| [**Fases del SDK**](Fases.md)              | **Roadmap completo para llegar a SDK v2.0**       |
| [Tareas pendientes](docs/Tareas.md)        | Backlog detallado de tareas                        |
| [Mejoras aplicadas](MEJORAS_APLICADAS.md)  | Historial de mejoras técnicas                      |

---

## 🔄 Changelog

### v1.2.0 — UE5-style Core (Mayo 2026)
- Workspace Cargo real (reactor-vulkan + reactor-editor).
- Sistema de profiling jerárquico (`profile_scope!`, `CpuTimer`, `PerfCounter`).
- Logging estructurado (`tracing-subscriber`, `REACTOR_LOG`, `r_info!`/`r_warn!`/`r_error!`).
- Job System paralelo (rayon: `parallel_for`, `join`, `scope`, `par_iter_mut`).
- Linear Allocator per-frame (`LinearAllocator`, `BumpArena`, zero-fragmentation).
- `lib.rs` limpio de legacy y sufijos `*New` — todos los exports canónicos.
- Configuración de calidad: `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.
- Pause overlay dibujado por fragment shader (`pause_overlay_alpha/page/selected/row_count`).
- PSO cache persistente, hot-reload de shaders, asset cooker.
- Pixel Inteligente (VRS) con perfiles Off/Quality/Balanced/Performance/Ultra.

### v1.1.0 — Rust Only (Mayo 2026)

**Migración a Rust puro:**

- ❌ Eliminado: C ABI (`cpp/reactor_c_api/`) del roadmap activo.
- ❌ Eliminado: C++ SDK (`cpp/reactor_cpp/`) del roadmap activo.
- ❌ Eliminado: ejemplos C++ y dependencia de CMake / vcpkg.
- ✅ Nuevo: [`Fases.md`](Fases.md) — Roadmap completo para SDK v2.0.
- ✅ Enfoque: un único stack idiomático, mantenible y seguro.

### v1.0.5 — Febrero 2026

- FrameGraph declarativo (forward / deferred presets, auto-barriers).
- ECS con queries de componentes (Transform, MeshRenderer, Light, Camera, RigidBody).
- PBR (metallic / roughness + material instances + emissive + alpha modes).
- Telemetría (FPS, draw calls, triangles, VRAM, memory budget).
- Compute pipeline (dispatch + barriers).
- Serialización de escena a JSON.
- Auto-compilación de shaders vía `build.rs`.
- MSAA 4x por defecto, Ray Tracing auto-detectado.
- 3000+ FPS en RTX 3060.

### v1.0.0 – v1.0.4

- Lifecycle base, input, camera, lighting, scene.
- Shaders SPIR-V embebidos.
- Editor REACTOR (`egui` + `egui_dock`): Viewport, Hierarchy, Inspector, Console, Asset Browser.

### v0.4.x

- Versión inicial en Rust.
- Vulkan 1.3 base.

---

## 📄 Licencia

TECHNE License — **Powered by Salazar-interactive**
