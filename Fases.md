<p align="center">
  <img src="image.svg" alt="REACTOR Logo" width="220"/>
</p>

<h1 align="center">REACTOR â€” Fases de ConstrucciÃ³n del SDK</h1>

<p align="center">
  <strong>Roadmap completo para llegar a un SDK de videojuegos AAA en Rust puro + Vulkan 1.3</strong><br/>
  <em>Powered by Salazar-interactive Â· v1.2.0-rust â†’ v2.0.0-sdk</em>
</p>

---

## ðŸ“œ Objetivo

Transformar **REACTOR** desde un framework de rendering Vulkan en un **SDK completo para
producir videojuegos comerciales**, manteniendo:

- ðŸ¦€ **100 % Rust** â€” sin C, sin C++, sin CMake, sin vcpkg.
- âš¡ **Zero-overhead** â€” abstracciones que se compilan a Vulkan puro.
- ðŸ›¡ï¸  **Memory-safe** por construcciÃ³n gracias al sistema de ownership.
- ðŸŽ® **Game-ready** â€” todo lo que necesita un equipo para enviar un juego.

---

## ðŸ—‚ï¸  Resumen de Fases

| Fase | Nombre                         | Objetivo principal                                          | Estado        |
|:----:|--------------------------------|-------------------------------------------------------------|:-------------:|
| **0**| Limpieza y consolidación       | Eliminar C / C++ / CMake. Unificar `src/`.                  | ✅ **Hecho**  |
| **1**| Núcleo Rust + Vulkan estable   | Reescribir core con `Arc<Device>`, `Drop` correcto.         | ✅ **Hecho**  |
| **2**| Pipeline gráfico moderno       | Bindless, dynamic rendering, PSO cache, Mesh Shaders.       | ✅ **Hecho**  |
| **3**| Asset Pipeline                 | glTF 2.0, KTX2, hot-reload, asset DB, loaders.              | 🚧 En curso   |
| **4**| Renderer de producción         | PBR completo, IBL, sombras, GI dinámica.                    | ⌛ Pendiente  |
| **5**| Sistemas de gameplay           | ECS jerárquico, scripting, eventos, navmesh.                | ⌛ Pendiente  |
| **6**| Físicas y colisiones           | Integrar `rapier3d`, character controller, raycast físico.  | 🚧 En curso   |
| **7**| Audio espacial 3D              | Integrar `kira` o backend custom, HRTF, buses.              | ⌛ Pendiente  |
| **8**| Networking                     | Cliente/servidor, replicación, predicción, rollback.        | ⌛ Pendiente  |
| **9**| Editor REACTOR completo        | Viewport, gizmos, scripting visual, play mode in-place.     | ⌛ Pendiente  |
| **10**| Tooling y build pipeline      | CLI `reactor`, plantillas, cooker, packager, shipping.      | ⌛ Pendiente  |
| **11**| Plataformas y portabilidad    | Windows, Linux, macOS (MoltenVK), Android, Steam Deck.      | ⌛ Pendiente  |
| **12**| QA, perf y release v2.0       | Tests, fuzzing, benchmarks, demo AAA y release público.     | ⌛ Pendiente  |

---

## ðŸ§¹ FASE 0 â€” Limpieza y consolidaciÃ³n (Rust puro)

> **Meta:** dejar el repo 100 % Rust, sin rastro de C / C++ / CMake / vcpkg.

### 0.1 Eliminar capas C / C++  âœ…
- [x] Borrado `cpp/reactor_c_api/` (3300 LOC) â€” eliminado del repo.
- [x] Borrado `cpp/reactor_cpp/` (1477 LOC header-only SDK).
- [x] Borrado `cpp/examples/3D/` + carpeta `build/` (~2 GB de artefactos CMake).
- [x] Borrado `vcpkg.json`.
- [x] Borrado `docs/cpp-guide.md` y `docs/cpp_editor_parity_roadmap.md`.
- [x] Reescritos `README.md`, `HOW_BUILD.md`, `docs/manual.md`, `docs/architecture.md` (100 % Rust).
- [x] Bump versiÃ³n `Cargo.toml`: `1.0.5` â†’ **`1.1.0`** + `description` + `license`.

### 0.2 Consolidar `src/` (eliminar duplicidad legacy â†” modular)  âœ…
- [x] Borrar mÃ³dulos legacy redundantes en `src/`:
  `vulkan_context.rs`, `swapchain.rs`, `pipeline.rs`, `buffer.rs`, `vertex.rs`,
  `mesh.rs`, `material.rs`, `input.rs`, `ecs.rs`, `ray_tracing.rs`,
  `scene.rs`, `gpu_detector.rs`, `cpu_detector.rs`, `resolution_detector.rs`.
  *(Ejecutar `cleanup.ps1` o `cleanup.sh` para borrar fÃ­sicamente; ya no estÃ¡n declarados en `lib.rs` ni se usan en ningÃºn lugar del codebase)*
- [x] Migrar usos restantes hacia `src/core/`, `src/graphics/`, `src/resources/`, `src/systems/`, `src/utils/`.
  *(Verificado con grep: 0 usos de mÃ³dulos legacy en todo el codebase)*
- [x] Limpiar el `lib.rs` de re-exports `*New` y dejar nombres canÃ³nicos sin sufijos.
  *(Todos los re-exports ahora son canÃ³nicos: `VulkanContext`, `Swapchain`, `Mesh`, `Material`, etc.)*

### 0.3 EstandarizaciÃ³n del workspace  âœ…
- [x] Convertir el repo en un **workspace Cargo** real:
  ```toml
  # Cargo.toml
  [workspace]
  members = [".", "Editor-REACTOR"]
  resolver = "2"
  
  [workspace.dependencies]
  glam = "0.30"
  tracing = "0.1"
  rayon = "1.10"
  parking_lot = "0.12"
  # ... (ver Cargo.toml completo)
  ```
- [x] `Editor-REACTOR/` aÃ±adido como miembro del workspace. *(Mover a `crates/reactor-editor/` pendiente para Fase 0.4)*
- [ ] Mover `src/` â†’ `crates/reactor/src/`. *(Diferido: requiere ajustar todas las rutas de ejemplos; no aporta valor funcional inmediato)*
- [x] AÃ±adir `rust-toolchain.toml` (canal estable + fmt + clippy + rust-analyzer).
- [x] Configurar `clippy.toml` (MSRV 1.70, umbrales engine-grade).
- [x] Configurar `rustfmt.toml` (estilo consistente, imports agrupados por StdExternalCrate).
- [ ] `deny.toml` (cargo-deny) â€” pendiente para Fase 0.4.

### 0.4 CI / CD bÃ¡sico
- [ ] GitHub Actions: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
- [ ] Cache de `target/` y de Vulkan SDK.
- [ ] Build matricial: Windows + Linux + macOS (Vulkan / MoltenVK).

**Entregable F0:** repo limpio, workspace Cargo, CI verde, cero cÃ³digo C/C++.

### 0.5 API corto (facilitar y acortar)  âœ…
- [x] `reactor::quick(title, w, h, |ctx| { â€¦ })` â€” arrancar un juego en una lÃ­nea.
- [x] `reactor::quick_with(config, init, update)` â€” closures init + update.
- [x] Macro `reactor::game! { title, size, vsync, msaa, init, update }` â€” declarativa.
- [x] Ejemplo nuevo `examples/quick.rs` con los 3 modos del API corto.

### 0.6 ErgonomÃ­a del `ReactorContext` (helpers cortos)  âœ…
- [x] `Camera::aim_at(&mut self, eye, target)` y `look_toward` / `set_position` (sin consumir self).
- [x] `ctx.look_at(eye, target)` â€” atajo en 1 lÃ­nea para colocar la cÃ¡mara.
- [x] `ctx.move_camera_to(pos)` â€” mover sin reorientar.
- [x] `ctx.add_sun()` â€” sol direccional con defaults agradables.
- [x] `ctx.add_directional_light(dir, color, intensity)`.
- [x] `ctx.add_point_light(pos, color, intensity, range)`.
- [x] `ctx.add_spot_light(pos, dir, color, intensity, range, angle)`.
- [x] `ctx.spawn(mesh, mat, xf)` â€” aÃ±adir objeto a la escena.
- [x] `ctx.set_transform(index, xf)` â€” actualizar transform por Ã­ndice.
- [x] `ctx.elapsed()` â€” atajo a `time.elapsed()`.

### 0.7 Higiene del build  âœ…
- [x] `cargo check`: **0 warnings**, 0 errors.
- [x] `cargo build --examples`: **0 warnings**, 0 errors (6 ejemplos OK).
- [x] `env_logger::try_init()` para no panicar si se inicializa dos veces.

---

## âš™ï¸ FASE 1 â€” NÃºcleo Rust + Vulkan estable

> **Meta:** un `VulkanContext` idiomÃ¡tico con `Arc<Device>` y `Drop` correcto.

### 1.1 Refactor del contexto Vulkan
- [x] âœ… `Device`, `Instance`, `Surface`, `PhysicalDevice` envueltos en `Arc<_>`.
- [x] âœ… Implementar `Drop` en orden inverso de creaciÃ³n (sin leaks de validation layers).
- [x] Centralizar `VulkanError` con `thiserror`.
  *(Ya existe `core::error::ReactorError` con `From<ash::vk::Result>`, `From<std::io::Error>`, `ReactorResult<T>` alias)*
- [x] âœ… Usar `Result<T, ReactorError>` en APIs pÃºblicas (eliminados `panic!` en mÃ³dulos grÃ¡ficos).
  - [x] âœ… `src/graphics/swapchain.rs` â€” migrado a `ReactorResult<Self>`
  - [x] âœ… `src/graphics/buffer.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/graphics/pipeline.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/graphics/render_pass.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/graphics/image.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/graphics/framebuffer.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/graphics/depth.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/graphics/descriptors.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/graphics/msaa.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/graphics/sampler.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/graphics/uniform_buffer.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/core/command.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/core/allocator.rs` â€” ArcDevice fix + migrado a `ReactorResult`
  - [x] âœ… `src/raytracing/pipeline.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/compute/pipeline.rs` â€” migrado a `ReactorResult`
  - [x] âœ… `src/resources/model.rs` â€” `From<ParseFloatError>`, `From<ParseIntError>`, `From<gltf::Error>`
  - [x] âœ… `src/resources/texture.rs` â€” `borrow of moved value` fix
  - [x] âœ… `src/resources/material.rs` â€” imports limpios
  - [x] âœ… `src/resources/mesh.rs` â€” imports limpios
  - [x] âœ… `src/resources/asset_manager.rs` â€” imports limpios
  - [x] âœ… `src/platform/window.rs` â€” `From<OsError>` implementado
  - [ ] `src/reactor.rs` â€” pendiente (monolito, ~20 usos de `Box<dyn Error>`)
- [ ] Soporte completo de `VK_LAYER_KHRONOS_validation` en debug.

### 1.2 Allocator GPU
- [x] Migrar a `gpu-allocator` 0.28+ con `MemoryLocation` explÃ­cito.
  *(Ya en `core::allocator::MemoryAllocator`, wrapper `Arc<Mutex<Allocator>>`)*
- [x] âœ… Pools de allocaciÃ³n por uso (vertex/index/uniform/storage).
- [x] âœ… EstadÃ­sticas de uso (VRAM live + peak) expuestas en `RenderStats`.

### 1.3 Command management
- [x] âœ… `CommandPool` por thread, reusables.
- [x] âœ… Submission via colas paralelas (graphics + compute + transfer).
- [ ] Frames-in-flight configurables (1, 2, 3) con semÃ¡foros timeline.

### 1.4 Subsystems UE5-style (NUEVO â€” v1.2.0)  âœ…
- [x] `core::profiler` â€” `profile_scope!` macro, `CpuTimer`, `PerfCounter`, Tracy-ready.
- [x] `core::logging` â€” `tracing-subscriber` + `REACTOR_LOG` env var + `r_info!`/`r_warn!`/`r_error!` macros.
- [x] `core::jobs` â€” JobSystem con rayon: `parallel_for`, `join`, `scope`, `par_iter_mut`, `parallel_reduce`.
- [x] `core::linear_allocator` â€” `LinearAllocator` (bump allocator) + `BumpArena` tipado para datos por-frame.

### 1.5 Tests del nÃºcleo
- [ ] Tests headless de creaciÃ³n / destrucciÃ³n de `VulkanContext` (lavapipe / SwiftShader).
- [ ] Smoke test: crear ventana, abrir swapchain, renderizar 10 frames, cerrar.

**Entregable F1:** nÃºcleo sin leaks, validation 0 errores, tests en CI.

---

## ðŸŽ¨ FASE 2 â€” Pipeline grÃ¡fico moderno

> **Meta:** rendering moderno con bindless, dynamic rendering y PSO cache.

### 2.1 Dynamic Rendering (Vulkan 1.3)
- [x] âœ… Reemplazar `VkRenderPass` + `VkFramebuffer` por `VK_KHR_dynamic_rendering`.
- [x] âœ… Eliminar todo el cÃ³digo de `subpasses` heredado.

### 2.2 Descriptor Indexing / Bindless
- [x] ✅ Habilitar `VK_EXT_descriptor_indexing` con feature chain completo.
- [x] ✅ Global texture array bindless (8192 slots UE5-style).
- [x] ✅ Global buffer array bindless (mesh + material data, 4096 slots).
- [x] âœ… `MeshHandle` / `MaterialHandle` / `TextureHandle` como Ã­ndices u32.

### 2.3 Pipeline State Object (PSO) cache
- [x] âœ… Cache en disco (`.reactor/pipeline_cache.bin`).
- [x] âœ… Hash de (shader SPIR-V + render state) â†’ PSO.
- [x] âœ… Hot-reload de shaders con recompilaciÃ³n incremental.

### 2.4 Shader system
- [x] âœ… Soporte de **WGSL** + **GLSL** + **HLSL** vÃ­a `naga` (integrado, compila 18 shaders).
- [x] âœ… Reflection automÃ¡tica (descriptor layouts derivados del SPIR-V).
- [x] ✅ `ShaderCompiler` con macro `shader!` declarativa.

### 2.5 GPU-driven rendering
- [x] ✅ Indirect draw (`vkCmdDrawIndexedIndirect`) con `IndirectDrawBuffer`.
- [x] ✅ Culling en compute (frustum AABB, 64 threads por workgroup).
- [x] Mesh shaders opcionales (`VK_EXT_mesh_shader`) cuando disponibles.

**Entregable F2:** renderer que sostiene 1 M+ objetos a 60 FPS con culling GPU.

---

## 📦 FASE 3 — Asset Pipeline (🚧 En curso / Integración Pendiente)

> **Meta:** importar, optimizar, cachear y hot-reload de todos los assets de un juego.

### 3.1 Formatos soportados
- [x] **Modelos:** glTF 2.0 (vía `gltf` e importador custom en `gltf_loader.rs`), OBJ (cargador custom en `model.rs`).
- [ ] **Texturas:** PNG, JPG, HDR, EXR (mediante `image` crate), **KTX2 + BCn / ASTC** (vía `ktx2` + `basis-universal`).
- [ ] **Audio:** OGG, WAV, FLAC, MP3.
- [ ] **Fuentes:** TTF / OTF (vía `fontdue` o `cosmic-text`).

### 3.2 Asset Database
- [x] `AssetId(u64)` estable (hash del path + contenido).
- [x] Metadata en `.reactor/assets.db` vía `sled` (implementado en `asset_database.rs`).
- [x] Carga lazy + reference counting (`Handle<T>` en `handle.rs`).
- [x] Streaming asíncrono con cola de tareas (`asset_loader_queue.rs`).

### 3.3 Asset cooker
- [ ] Pre-procesa assets RAW → formato runtime optimizado:
  - Texturas → BC7 / ASTC + mipmaps.
  - Meshes → meshlets (vía `meshopt`).
  - Audio → OGG comprimido + bus tags.
- [ ] CLI: `reactor cook --input assets/ --output cooked/`.

### 3.4 Hot-reload
- [x] Watcher con `notify` que recook + reupload en vivo (`asset_hot_reload.rs`).
- [ ] Notificación al editor / juego vía `EventBus`.

**Entregable F3:** carga de una escena glTF con 200 materiales / 1 GB de texturas en <2 s.

---

## ðŸŒŸ FASE 4 â€” Renderer de producciÃ³n

> **Meta:** calidad visual comparable a Unity HDRP o Unreal en escenas medianas.

### 4.1 PBR completo
- [ ] Cook-Torrance (GGX + Schlick + Smith).
- [ ] Clear-coat, sheen, anisotropy, subsurface scattering (KHR_materials_*).
- [ ] Soporte `KHR_materials_transmission` (vidrio fÃ­sico).

### 4.2 Image-Based Lighting (IBL)
- [ ] Equirectangular â†’ cubemap â†’ prefiltered + irradiance.
- [ ] BRDF LUT precomputada.
- [ ] Tone mapping ACES / AGX.

### 4.3 Sombras
- [ ] CSM (Cascaded Shadow Maps) para luces direccionales.
- [ ] Point light shadows (cube maps).
- [ ] Spot light shadows (2D).
- [ ] **VSM / PCSS** para sombras suaves.
- [ ] Sombras de ray tracing cuando RT estÃ© disponible.

### 4.4 IluminaciÃ³n global
- [ ] **Light probes** (esfÃ©ricos armÃ³nicos L2).
- [ ] **Reflection probes** (cubemaps locales).
- [ ] **Screen-Space GI (SSGI)** como fallback.
- [ ] **Voxel-cone GI** opcional para escenas dinÃ¡micas.

### 4.5 Post-processing AAA
- [ ] Bloom (mipmap chain con upsample dual).
- [ ] Motion blur (per-object + camera).
- [ ] Depth of field (Bokeh + circular).
- [ ] Auto-exposure (histograma compute).
- [ ] Color grading (LUT 3D).
- [ ] TAA / SMAA / FXAA / **ADead-AA** seleccionable.

### 4.6 Decals y particles GPU
- [ ] Decal system (deferred).
- [ ] GPU particles vÃ­a compute (millones de partÃ­culas).
- [ ] VFX graph bÃ¡sico (nodos).

**Entregable F4:** demo "Sponza PBR" a 4K / 60 FPS en RTX 3060 con GI + sombras + TAA.

---

## ðŸŽ® FASE 5 â€” Sistemas de gameplay

> **Meta:** todo lo que necesita el cÃ³digo de gameplay sin reinventar la rueda.

### 5.1 ECS jerÃ¡rquico
- [ ] Migrar a o integrar `hecs` / `bevy_ecs` (o reforzar el ECS propio).
- [ ] `Parent` / `Children` con propagaciÃ³n de transform.
- [ ] World queries con `With<T>` / `Without<T>` / `Changed<T>`.
- [ ] Systems schedule con dependencias.

### 5.2 Scripting
- [ ] Embed de **Rhai** o **Lua** (mlua) para scripting de gameplay.
- [ ] Hot-reload de scripts en editor.
- [ ] Bindings auto-generados desde tipos `#[reactor::reflect]`.

### 5.3 Event bus
- [ ] `EventBus<T>` global + locales por escena.
- [ ] `Observer<T>` para reaccionar (UI, audio, animaciones).

### 5.4 AI y navegaciÃ³n
- [ ] **NavMesh** vÃ­a `recast-rs` o port propio.
- [ ] Pathfinding A* + jump links.
- [ ] Behaviour trees (`bonsai-bt`).
- [ ] State machines.

### 5.5 Input avanzado
- [ ] Soporte gamepad (XInput / DInput / SDL2 / `gilrs`).
- [ ] Input mapping configurable (`input.toml`).
- [ ] Acciones, axes, gestures.

**Entregable F5:** demo "FPS arena" con bots IA, navmesh y scripting en Rhai.

---

## 🧱 FASE 6 — Físicas y colisiones (🚧 En curso)

> **Meta:** física rígida y de personaje a nivel comercial.

### 6.1 Integración Rapier
- [ ] `rapier3d` (CPU) como backend por defecto (Pendiente).
- [x] Wrapper `PhysicsWorld` básico propio en `physics.rs` (Euler integration, gravedad, fricción).
- [ ] Sincronización ECS ↔ Rapier vía `Transform` y `RigidBody` component.

### 6.2 Character Controller
- [x] Character Controller custom básico en `physics.rs` (Kinematic capsule, salto, fricción de suelo/aire).
- [ ] Slope handling, crouch.
- [ ] Networking-friendly (deterministic).

### 6.3 Queries físicas
- [x] Raycast, AABB intersections, Sphere intersections en `physics.rs`.
- [x] Colisión y empuje en AABB (`collide_aabb`).
- [ ] Capa de colisión avanzada (groups + filters).

### 6.4 Vehículos y joints
- [ ] Wheel collider (raycast vehicle).
- [ ] Hinge, ball, prismatic, fixed joints.

### 6.5 GPU physics (opcional, largo plazo)
- [ ] Cloth (compute shader).
- [ ] Fluids (FLIP / SPH compute).

**Entregable F6:** demo "Physics playground" con 1 000 cuerpos + ragdoll.

---

## ðŸ”Š FASE 7 â€” Audio espacial 3D

> **Meta:** audio AAA con mezclador, espacializaciÃ³n y oclusiÃ³n.

### 7.1 Backend
- [ ] Integrar `kira` o `cpal` + `oddio` para 3D.
- [ ] Buses (master, music, sfx, voice, ambience).
- [ ] VolÃºmenes y EQ por bus.

### 7.2 EspacializaciÃ³n
- [ ] HRTF estÃ©reo + binaural.
- [ ] AtenuaciÃ³n distancia (lineal / log / custom).
- [ ] Doppler.
- [ ] Reverb por zona (zones triggers).

### 7.3 OclusiÃ³n y obstrucciÃ³n
- [ ] Raycast fÃ­sico + filtros low-pass.
- [ ] Portal-based propagation.

### 7.4 MÃºsica dinÃ¡mica
- [ ] Stems con transiciones por estado.
- [ ] SincronizaciÃ³n a tempo (BPM events).

**Entregable F7:** demo "audio walk" con HRTF + reverb por zona + mÃºsica dinÃ¡mica.

---

## ðŸŒ FASE 8 â€” Networking

> **Meta:** soporte cliente/servidor para shooters y juegos cooperativos.

### 8.1 Transporte
- [ ] **QUIC** vÃ­a `quinn` (TCP/UDP-friendly).
- [ ] WebSocket + WebTransport para web (futuro Fase 11).
- [ ] Mensajes confiables y no-confiables.

### 8.2 ReplicaciÃ³n
- [ ] Snapshot de entidades + delta compression.
- [ ] Interest management (relevancia por distancia).
- [ ] Replicated components con `#[reactor::replicate]`.

### 8.3 PredicciÃ³n y reconciliaciÃ³n
- [ ] Client-side prediction.
- [ ] Server reconciliation.
- [ ] Lag compensation (rewind & replay).

### 8.4 Lobby & matchmaking
- [ ] Server browser bÃ¡sico.
- [ ] IntegraciÃ³n Steam (Steamworks vÃ­a `steamworks-rs`).

**Entregable F8:** demo "deathmatch 4 jugadores" con predicciÃ³n y rollback.

---

## ðŸ› ï¸ FASE 9 â€” Editor REACTOR completo

> **Meta:** un editor visual estilo Unity / Godot / Unreal.

### 9.1 UI base
- [ ] Mantener `egui` + `egui_dock`.
- [ ] Tema oscuro REACTOR + tema claro.
- [ ] Layouts guardables.

### 9.2 Paneles obligatorios
- [ ] **Viewport 3D** con cÃ¡mara editor (FPS + orbit).
- [ ] **Hierarchy** (Ã¡rbol de entidades, drag & drop, parent / unparent).
- [ ] **Inspector** (auto-generado vÃ­a reflection).
- [ ] **Console** (logs + filtros + cargo errors).
- [ ] **Asset Browser** (thumbnails, drag & drop al viewport).
- [ ] **Scene panel** (lista de escenas, build settings).
- [ ] **Profiler** (frame time, GPU stats, memory).

### 9.3 Gizmos
- [ ] Translate / Rotate / Scale (clicables).
- [ ] Snap a grid / vÃ©rtices / Ã¡ngulos.
- [ ] Multi-selecciÃ³n.

### 9.4 Play mode in-place
- [ ] Play / Pause / Stop con snapshot reversible.
- [ ] Edit-in-play (cambios no destructivos).
- [ ] Live debug del ECS.

### 9.5 Scripting visual
- [ ] Sistema de **nodos** estilo Blueprints (mediano plazo).
- [ ] Eventos, branches, variables, llamadas a funciones Rust.

### 9.6 Reflection / SerializaciÃ³n
- [ ] `#[derive(Reflect)]` proc-macro para auto-inspector.
- [ ] SerializaciÃ³n a JSON + RON + binary (`bincode`).

**Entregable F9:** editor capaz de construir un nivel completo sin tocar cÃ³digo Rust.

---

## ðŸ“¦ FASE 10 â€” Tooling y build pipeline

> **Meta:** la experiencia "Unity Hub" / `cargo new` para juegos REACTOR.

### 10.1 CLI `reactor`
- [ ] `reactor new <nombre>` â€” plantillas (FPS, 2D-platformer, racing, sandbox).
- [ ] `reactor run` â€” compila + lanza el juego.
- [ ] `reactor cook` â€” cook assets a formato runtime.
- [ ] `reactor pack` â€” empaqueta build (PAK virtual o `.reactor` archive).
- [ ] `reactor ship --platform windows|linux|macos|android` â€” build final.

### 10.2 Plantillas (`reactor-templates/`)
- [ ] `template-fps/`
- [ ] `template-2d-platformer/`
- [ ] `template-racing/`
- [ ] `template-sandbox-minimal/`

### 10.3 Packer y archive format
- [ ] `.reactor` archive (chunked + zstd + mmap-friendly).
- [ ] EncriptaciÃ³n opcional (AES-GCM).
- [ ] Resource patching (DLC).

### 10.4 Installer / Distribution
- [ ] Bundle Windows (MSI / portable zip).
- [ ] AppImage / .deb / .rpm (Linux).
- [ ] .app + .dmg (macOS).
- [ ] APK (Android, Fase 11).

**Entregable F10:** `reactor new mi_juego && reactor ship` produce un ejecutable listo para Steam.

---

## ðŸ–¥ï¸ FASE 11 â€” Plataformas y portabilidad

> **Meta:** "build once, ship everywhere".

### 11.1 Desktop
- [ ] **Windows 10/11** (x64 + ARM64).
- [ ] **Linux** (Ubuntu, Arch, Steam Deck / SteamOS).
- [ ] **macOS** vÃ­a **MoltenVK** (Vulkan â†’ Metal).

### 11.2 Mobile
- [ ] **Android** (Vulkan 1.3 nativo, NDK).
- [ ] **iOS** vÃ­a MoltenVK + bindings (largo plazo).

### 11.3 Web
- [ ] **WebGPU backend** alternativo (vÃ­a `wgpu`).
- [ ] Build con `wasm-pack`.
- [ ] Streaming de assets via fetch.

### 11.4 Consolas (largo plazo, requiere licencias)
- [ ] **Nintendo Switch** (NVN).
- [ ] **PS5 / Xbox** (vÃ­a SDK propietario).

### 11.5 VR / XR
- [ ] **OpenXR** (`openxr-rs`).
- [ ] Foveated rendering vÃ­a ADead-ISR.
- [ ] Soporte Quest, Index, Vive, PSVR2.

**Entregable F11:** demo corriendo en Windows + Linux + macOS + Steam Deck con un solo `cargo build`.

---

## âœ… FASE 12 â€” QA, performance y release v2.0

> **Meta:** REACTOR v2.0 como SDK comercial estable.

### 12.1 Testing
- [ ] >80 % cobertura en `crates/reactor/`.
- [ ] Tests visuales (golden images con tolerancia perceptual).
- [ ] Fuzzing de loaders (gltf, ktx2, ogg).

### 12.2 Performance
- [ ] Benchmarks `criterion` para core paths (draw, ECS, physics).
- [ ] Profiling integrado (Tracy via `tracy-client`).
- [ ] Memory profiling (`dhat`, `valgrind` en Linux CI).

### 12.3 DocumentaciÃ³n
- [ ] `cargo doc` completo + ejemplos en cada item pÃºblico.
- [ ] **Libro REACTOR** (`mdbook`) con tutoriales paso a paso.
- [ ] Video-tutoriales oficiales.

### 12.4 Demo AAA
- [ ] Construir un juego demo de 30 min usando 100 % REACTOR.
- [ ] Publicarlo gratuito en Steam / itch.io.

### 12.5 Release v2.0
- [ ] SemVer estable.
- [ ] LTS 24 meses.
- [ ] Anuncio pÃºblico + landing en reactor.salazar-interactive.dev.

**Entregable F12:** **REACTOR v2.0.0 â€” el primer SDK AAA en Rust puro.**

---

## ðŸ§­ Hoja de ruta visual

```diagram
                  â•­â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•®
                  â”‚   F0  Limpieza (Rust puro)           â”‚
                  â•°â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•¯
                                   â–¼
       â•­â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•®
       â”‚  F1 NÃºcleo Vulkan  â†’  F2 Pipeline moderno  â†’  F3 Assetsâ”‚
       â•°â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â•¯
                       â–¼               â–¼                â–¼
              â•­â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•®   â•­â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•®  â•­â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•®
              â”‚ F4 Renderer â”‚   â”‚ F5 Gameplay â”‚  â”‚ F6 Physics â”‚
              â”‚   AAA       â”‚   â”‚  + AI       â”‚  â”‚  + char    â”‚
              â•°â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â•¯   â•°â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â•¯  â•°â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â•¯
                     â–¼                 â–¼                â–¼
                  â•­â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•®
                  â”‚ F7 Audio Â· F8 Net Â· F9 Editor     â”‚
                  â•°â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•¯
                                   â–¼
            â•­â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•®
            â”‚  F10 Tooling/CLI  â†’  F11 Plataformas         â”‚
            â•°â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•¯
                                     â–¼
                       â•­â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•®
                       â”‚  F12 QA + Release v2.0   â”‚
                       â•°â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â•¯
```

---

## ðŸ“Š MÃ©tricas de Ã©xito (KPIs)

| KPI                                            | Objetivo v2.0          |
|------------------------------------------------|------------------------|
| LÃ­neas de cÃ³digo C / C++                       | **0**                  |
| Cobertura de tests                             | **â‰¥ 80 %**             |
| Errores de validation layer en demo AAA        | **0**                  |
| FPS demo "Sponza PBR" en RTX 3060 @ 4K         | **â‰¥ 60 FPS**           |
| Tiempo de compilaciÃ³n full (workspace, release)| **â‰¤ 90 s**             |
| Crash-free sessions del demo Steam             | **â‰¥ 99,5 %**           |
| Plataformas soportadas con CI                  | **â‰¥ 5**                |
| TamaÃ±o del runtime base (sin assets)           | **â‰¤ 25 MB**            |

---

## ðŸ¤ CÃ³mo contribuir a cada fase

1. Elegir un Ã­tem `[ ]` no marcado de la fase activa.
2. Abrir un issue con prefijo `[Fxx]` (ej. `[F2] PSO cache en disco`).
3. Crear rama `fXX/<slug>` (ej. `f2/pso-cache`).
4. PR con tests + `cargo clippy --all -- -D warnings` limpio.
5. Marcar el checkbox en este archivo al hacer merge.

---

## ðŸ“Œ Notas finales

- **Sin C / C++:** cualquier dependencia que requiera C-bindings debe estar justificada
  (ej. Vulkan / Rapier). Preferir crates 100 % Rust siempre que sea viable.
- **Sin GC:** cero garbage collector, cero `Rc<RefCell<_>>` salvo en el editor.
- **Sin macros mÃ¡gicas innecesarias:** preferir `Builder` patterns explÃ­citos.
- **API estable desde v2.0:** romper compatibilidad solo en mayores (SemVer estricto).

---

<p align="center">
  <strong>REACTOR â€” Rust + Vulkan, sin compromisos.</strong><br/>
  <em>Powered by Salazar-interactive</em>
</p>

