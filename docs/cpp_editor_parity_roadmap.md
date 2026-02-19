# C++ Productivity Parity Roadmap (Rust Core -> C ABI -> C++ SDK -> Editor)

Objetivo: que el usuario final pueda usar **C++ como capa principal de productividad** sin perder capacidades críticas del core en Rust.

## 1) Estado actual resumido

### Ya disponible en C ABI / C++ SDK (v1.0.5)

- Lifecycle y loop (`reactor_run`, `reactor_run_simple`, init/shutdown, frame begin/end).
- Input/Window/Time/Camera global.
- Scene completa + mesh/material/texture con Vulkan context real.
- Lights completas (directional/point/spot) con parámetros dinámicos.
- **ECS completo**: entity CRUD, transform, mesh renderer, light, camera, rigidbody, queries con bitmask.
- **PBR Materials**: metallic/roughness, instances, emissive, alpha modes.
- **FrameGraph**: passes declarativos, recursos, compilación, forward/deferred presets.
- **Telemetry**: render stats (FPS, draw calls, triangles, VRAM), memory budget, GPU info.
- **PlayMode**: play-in-editor bridge (enter/exit/pause), scene snapshot.
- **Scene Serialization**: export a JSON.
- **Compute**: pipeline stubs (create/dispatch/destroy).
- Post-process toggles simples (bloom/tonemap/vignette/fxaa).
- Utilidades matemáticas + SDF básicos.
- **C++ SDK**: 1477 líneas header-only, RAII wrappers, 9 ejemplos compilando.

### Qué falta para paridad real de productividad C++

## 2) Gaps críticos por capa

### A. Render Graph y orquestación de frame (CRÍTICO) ✅ IMPLEMENTADO
APIs C ABI para exponer el `FrameGraph` del core:
- ✅ Crear/destruir graph de frame por escena o por pipeline (`reactor_frame_graph_create/destroy`).
- ✅ Declarar passes (lectura/escritura de recursos) (`reactor_frame_graph_add_pass`).
- ✅ Recursos transient/persistent con formatos/flags (`reactor_frame_graph_create_resource`).
- ✅ Barreras y sincronización explícita por pass (`reactor_frame_graph_compile`).
- ✅ Métricas y validación del graph (`reactor_frame_graph_get_stats`).
- ✅ Graphs pre-construidos forward/deferred (`reactor_frame_graph_create_forward/deferred`).

### B. Recursos avanzados de GPU (CRÍTICO) — PARCIAL
- ⬜ Descriptor sets/layouts dinámicos.
- ⬜ Uniform/storage buffers con update por frame.
- ⬜ Image/sampler configs avanzadas (formats, mips, usage flags).
- ⬜ AssetManager cache/deduplicación y handles de recursos persistentes.

### C. Materiales modernos y shading (CRÍTICO) ✅ IMPLEMENTADO
- ✅ `PBRMaterial` completo (metallic/roughness/normal/AO/emissive/alpha workflow) (`reactor_pbr_*`).
- ✅ Material instances y parameter blocks (`reactor_pbr_create_instance`).
- ⬜ Variantes de shader (keyword/define system) y compilación controlada.

### D. Ray tracing e híbrido (ALTO) — PARCIAL
- ⬜ BLAS/TLAS lifecycle.
- ⬜ RayTracingPipeline/SBT.
- ✅ Detección de soporte RT (`reactor_is_raytracing_supported` — real query).

### E. Compute y simulación GPU (ALTO) ✅ IMPLEMENTADO (stubs)
- ✅ Create/bind/dispatch de compute pipelines (`reactor_compute_create/dispatch/destroy`).
- ⬜ Gestión de barreras y recursos compute (pendiente Vulkan backend).
- ⬜ Sistemas GPU (ej. partículas) parametrizables desde C++.

### F. Sistemas de mundo completos (ALTO) ✅ IMPLEMENTADO
- ✅ Component CRUD real (transform, mesh renderer, light, camera, physics) (`reactor_entity_*`).
- ✅ Queries con filtros y batches (`reactor_query_entities` con component_mask).
- ✅ Scene serialization estable y versionada (`reactor_scene_serialize`).

### G. Física/Audio/Animación de nivel herramienta (MEDIO-ALTO) — PARCIAL
- ✅ RigidBody component con force/velocity (`reactor_entity_add_rigidbody`, `reactor_entity_apply_force`).
- ✅ Character controller con gravedad y colisión básica.
- ⬜ Physics world handles, colliders constraints y debug data.
- ⬜ Audio buses, spatial tuning, snapshots.
- ⬜ Animation graphs, blend trees, state machines, retargeting básico.

### H. Telemetría/diagnóstico para editor (MEDIO) ✅ IMPLEMENTADO
- ✅ Stats de GPU/CPU por pass (`reactor_get_render_stats`).
- ✅ Memory budgets + live allocations (`reactor_get_memory_budget`).
- ✅ VRAM real desde Vulkan (`reactor_get_vram_mb`).
- ⬜ Captura de eventos de validación Vulkan por frame.

## 3) Backlog Vulkan “largo” recomendado para que el Editor herede todo

### Core Vulkan
- [ ] Device feature negotiation reportable por C ABI.
- [ ] Queue families + async compute/transfer control.
- [ ] Timeline semaphores y fences expuestos.
- [ ] Descriptor indexing / bindless policy configurable.
- [ ] Dynamic rendering path configurable por pipeline.

### Rendering
- [ ] RenderPass abstraction completa en C ABI.
- [ ] Framebuffer/MSAA/depth API completa.
- [ ] Shadow cascades API (splits, bias, PCF params).
- [ ] PostProcess stack ordenable por passes.
- [ ] Debug renderer con canales/capas y lifetime de primitivas.

### Resources
- [ ] Texture import pipeline (sRGB/HDR/compression/mips policy).
- [ ] Model pipeline (gltf/material bindings/skin data).
- [ ] Hot-reload de shaders/materiales/texturas.
- [ ] Asset dependency graph + invalidación incremental.

### Raytracing/Compute
- [ ] AS build/update/refit API.
- [ ] RT shader groups + SBT records API.
- [ ] Compute graph para simulaciones de editor.
- [ ] Shared buffers entre viewport/editor tools.

### Runtime-Editor bridge
- [ ] Scene snapshot diff/patch.
- [x] Play-in-editor bridge (start/stop/pause) — `reactor_play_enter/exit/pause` ✅
- [ ] Undo/redo transaccional conectado al runtime.
- [ ] Deterministic IDs para entidades y recursos.

## 4) Plan de implementación por fases (enfoque C++ first)

### Fase 1 (Base Editor Productivo) ✅ COMPLETADA
1. ✅ Expandir C ABI para scene/components CRUD + resource handles sólidos.
2. ✅ PBR material completo + material instances.
3. ✅ `reactor_create_mesh` y `reactor_create_material` funcionales con Vulkan context.

### Fase 2 (Parity visual) ✅ COMPLETADA
1. ✅ Render graph + passes configurables desde C++.
2. ✅ Perfilado de frame y stats (`reactor_get_render_stats`, `reactor_get_memory_budget`).
3. ✅ Scene serialization (`reactor_scene_serialize`).

### Fase 3 (Parity avanzada) — PARCIAL
1. ✅ Compute pipeline stubs (`reactor_compute_create/dispatch/destroy`).
2. ✅ Bridge play-in-editor (`reactor_play_enter/exit/pause`).
3. ⬜ Ray tracing pipelines completos (BLAS/TLAS/SBT).
4. ⬜ Hot reload de shaders/materiales/texturas.
5. ⬜ Serialización versionada + migraciones.

## 5) C++ Examples Implementados (9 total)

| Ejemplo | Carpeta | Qué demuestra |
| ------- | ------- | ------------- |
| reactor_3d | `main_basic.cpp` | Lifecycle básico, cubo con material |
| reactor_ecs_scene | `ecs_scene/` | Entity CRUD, components, queries |
| reactor_pbr_materials | `pbr_materials/` | PBR metallic/roughness, instances, emissive |
| reactor_frame_graph | `frame_graph/` | Custom render passes, forward/deferred |
| reactor_fps_controller | `fps_controller/` | WASD + mouse look + jump + gravity |
| reactor_lighting | `lighting_showcase/` | Directional, point, spot lights animados |
| reactor_telemetry | `telemetry_stats/` | GPU stats, memory budget, serialización |
| reactor_play_mode | `play_mode/` | Enter/exit/pause play mode |
| reactor_multi_object | `multi_object/` | 225 objetos, wave, visibility, queries |

CMakeLists.txt builds all 9 with `add_reactor_example()` helper function.

## 6) Criterio de éxito

- El equipo de herramientas puede construir features del editor en C++ sin tocar Rust para tareas comunes.
- Rust queda como capa de rendimiento/seguridad, no como cuello de botella de productividad.
- La UX del editor refleja 1:1 los modos de render y sistemas del runtime.
- Los 9 ejemplos C++ demuestran todas las APIs expuestas vía C ABI.
