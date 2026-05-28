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
