# C++ Productivity Parity Roadmap (Rust Core -> C ABI -> C++ SDK -> Editor)

Objetivo: que el usuario final pueda usar **C++ como capa principal de productividad** sin perder capacidades críticas del core en Rust.

## 1) Estado actual resumido

### Ya disponible en C ABI / C++ SDK
- Lifecycle y loop (`reactor_run`, `reactor_run_simple`, init/shutdown, frame begin/end).
- Input/Window/Time/Camera global.
- Scene básica + mesh/material/texture base.
- Lights básicas (directional/point/spot).
- ECS mínimo (create/destroy/count), debug draw, animation y audio básicos.
- Post-process toggles simples (bloom/tonemap/vignette/fxaa).
- Utilidades matemáticas + SDF básicos.

### Qué falta para paridad real de productividad C++

## 2) Gaps críticos por capa

### A. Render Graph y orquestación de frame (CRÍTICO)
Faltan APIs C ABI para exponer el `FrameGraph` del core:
- Crear/destruir graph de frame por escena o por pipeline.
- Declarar passes (lectura/escritura de recursos).
- Recursos transient/persistent con formatos/flags.
- Barreras y sincronización explícita por pass.
- Métricas y validación del graph.

Impacto en editor: no puedes heredar pipeline de render configurable por herramienta (viewport, preview, bake, debug overlays) de forma estable en C++.

### B. Recursos avanzados de GPU (CRÍTICO)
Falta exposición completa de:
- Descriptor sets/layouts dinámicos.
- Uniform/storage buffers con update por frame.
- Image/sampler configs avanzadas (formats, mips, usage flags).
- AssetManager cache/deduplicación y handles de recursos persistentes.

Impacto: editor sin flujo sólido para inspección/override de materiales/shaders por instancia.

### C. Materiales modernos y shading (CRÍTICO)
Falta paridad para:
- `PBRMaterial` completo (metallic/roughness/normal/AO/emissive/alpha workflow).
- Material instances y parameter blocks.
- Variantes de shader (keyword/define system) y compilación controlada.

Impacto: el editor no puede ser realmente “autoritativo” para look-dev desde C++.

### D. Ray tracing e híbrido (ALTO)
Faltan endpoints para:
- BLAS/TLAS lifecycle.
- RayTracingPipeline/SBT.
- Alternar forward/deferred/raytracing/hybrid por viewport y por cámara.

Impacto: sin parity visual entre runtime y editor en escenas RTX/hybrid.

### E. Compute y simulación GPU (ALTO)
Falta C ABI para:
- Create/bind/dispatch de compute pipelines.
- Gestión de barreras y recursos compute.
- Sistemas GPU (ej. partículas) parametrizables desde C++.

Impacto: herramientas de VFX/preview limitadas en editor.

### F. Sistemas de mundo completos (ALTO)
ECS actual expone muy poco.
Falta:
- Component CRUD real (transform, mesh renderer, light, camera, physics, audio, custom).
- Queries con filtros y batches.
- Scene serialization estable y versionada.

Impacto: editor no puede operar como DCC completo en C++.

### G. Física/Audio/Animación de nivel herramienta (MEDIO-ALTO)
Hay API base, pero faltan:
- Physics world handles, colliders/rigidbodies constraints y debug data.
- Audio buses, spatial tuning, snapshots.
- Animation graphs, blend trees, state machines, retargeting básico.

### H. Telemetría/diagnóstico para editor (MEDIO)
Falta exponer:
- Stats de GPU/CPU por pass.
- Memory budgets + live allocations.
- Captura de eventos de validación Vulkan por frame.

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
- [ ] Play-in-editor bridge (start/stop/reload deterministic).
- [ ] Undo/redo transaccional conectado al runtime.
- [ ] Deterministic IDs para entidades y recursos.

## 4) Plan de implementación por fases (enfoque C++ first)

### Fase 1 (Base Editor Productivo)
1. Expandir C ABI para scene/components CRUD + resource handles sólidos.
2. PBR material completo + material instances.
3. Uniform/storage buffer API para gizmos/overlays/herramientas.

### Fase 2 (Parity visual)
1. Render graph + passes configurables desde C++.
2. Shadow/postprocess avanzados en C++ SDK.
3. Perfilado de frame y stats en overlay editor.

### Fase 3 (Parity avanzada)
1. Ray tracing y compute pipelines completos.
2. Bridge play-in-editor robusto + hot reload.
3. Serialización versionada + migraciones.

## 5) Criterio de éxito
- El equipo de herramientas puede construir features del editor en C++ sin tocar Rust para tareas comunes.
- Rust queda como capa de rendimiento/seguridad, no como cuello de botella de productividad.
- La UX del editor refleja 1:1 los modos de render y sistemas del runtime.
