# REACTOR — Lista de Tareas Pendientes

> **Backlog de desarrollo para la Fase 0.2 y siguientes**
> Actualizado: Mayo 2026

---

## 🎯 FASE 0.2 — Consolidar `src/` (eliminar duplicidad legacy ↔ modular)

### 0.2.1 Eliminar módulos legacy redundantes
- [ ] Borrar `src/vulkan_context.rs` (reemplazado por `src/core/context.rs`)
- [ ] Borrar `src/swapchain.rs` (reemplazado por `src/graphics/swapchain.rs`)
- [ ] Borrar `src/pipeline.rs` (reemplazado por `src/graphics/pipeline.rs`)
- [ ] Borrar `src/buffer.rs` (reemplazado por `src/graphics/buffer.rs`)
- [ ] Borrar `src/vertex.rs` (reemplazado por `src/resources/vertex.rs`)
- [ ] Borrar `src/mesh.rs` (reemplazado por `src/resources/mesh.rs`)
- [ ] Borrar `src/material.rs` (reemplazado por `src/resources/material.rs`)
- [ ] Borrar `src/input.rs` (reemplazado por `src/systems/input.rs`)
- [ ] Borrar `src/ecs.rs` (reemplazado por `src/systems/ecs.rs`)
- [ ] Borrar `src/ray_tracing.rs` (reemplazado por `src/raytracing/`)
- [ ] Borrar `src/scene.rs` (reemplazado por `src/systems/scene.rs`)
- [ ] Borrar `src/gpu_detector.rs` (reemplazado por `src/utils/gpu_detector.rs`)
- [ ] Borrar `src/cpu_detector.rs` (reemplazado por `src/utils/cpu_detector.rs`)
- [ ] Borrar `src/resolution_detector.rs` (reemplazado por `src/utils/resolution_detector.rs`)

### 0.2.2 Migrar usos restantes
- [ ] Actualizar `src/app.rs` para usar módulos nuevos en lugar de legacy
- [ ] Actualizar `src/reactor.rs` para usar módulos nuevos
- [ ] Actualizar todos los ejemplos para usar imports nuevos
- [ ] Verificar que no haya imports rotos con `cargo check`

### 0.2.3 Limpiar `lib.rs`
- [ ] Eliminar todos los `pub use` con sufijo `*New`
- [ ] Dejar nombres canónicos sin sufijos
- [ ] Reorganizar re-exports por categoría (Core, Graphics, Resources, Systems, Utils)
- [ ] Documentar cada re-export con doc comments

---

## 🏗️ FASE 0.3 — Estandarización del workspace

### 0.3.1 Convertir a workspace Cargo
- [ ] Crear `Cargo.toml` raíz como workspace
- [ ] Configurar `members = ["crates/reactor", "crates/reactor-editor", "crates/reactor-cli"]`
- [ ] Configurar `resolver = "2"`
- [ ] Mover `src/` → `crates/reactor/src/`
- [ ] Mover `Cargo.toml` actual → `crates/reactor/Cargo.toml`
- [ ] Mover `examples/` → `crates/reactor/examples/`
- [ ] Mover `shaders/` → `crates/reactor/shaders/`
- [ ] Mover `Editor-REACTOR/` → `crates/reactor-editor/`
- [ ] Crear `crates/reactor-cli/` (placeholder para Fase 10)

### 0.3.2 Toolchain y configuración
- [ ] Añadir `rust-toolchain.toml` con canal estable
- [ ] Configurar `clippy.toml` con reglas estrictas
- [ ] Configurar `rustfmt.toml` con estilo consistente
- [ ] Añadir `deny.toml` para `cargo-deny`
- [ ] Configurar `.editorconfig` para indentación

---

## 🔄 FASE 0.4 — CI / CD básico

### 0.4.1 GitHub Actions
- [ ] Crear `.github/workflows/ci.yml`
- [ ] Configurar `cargo fmt --check`
- [ ] Configurar `cargo clippy -- -D warnings`
- [ ] Configurar `cargo test --all-features`
- [ ] Configurar `cargo build --release`
- [ ] Cache de `target/` con `actions/cache`
- [ ] Cache de Vulkan SDK
- [ ] Build matricial: Windows + Linux + macOS

### 0.4.2 Badges y documentación
- [ ] Añadir badge de CI status al README
- [ ] Añadir badge de coverage (si se integra codecov)
- [ ] Documentar proceso de PR en `CONTRIBUTING.md`

---

## ⚙️ FASE 1 — Núcleo Rust + Vulkan estable

### 1.1 Refactor del contexto Vulkan
- [ ] Envolver `Device`, `Instance`, `Surface`, `PhysicalDevice` en `Arc<_>`
- [ ] Implementar `Drop` en orden inverso de creación
- [ ] Centralizar `VulkanError` con `thiserror` (ya añadido)
- [ ] Usar `Result<T, VulkanError>` en TODAS las APIs públicas
- [ ] Eliminar todos los `panic!` innecesarios
- [ ] Soporte completo de `VK_LAYER_KHRONOS_validation` en debug
- [ ] Tests headless con lavapipe / SwiftShader

### 1.2 Allocator GPU
- [ ] Migrar a `gpu-allocator` 0.28+ con `MemoryLocation` explícito
- [ ] Pools de allocación por uso (vertex/index/uniform/storage)
- [ ] Estadísticas de uso (VRAM live + peak) expuestas en `RenderStats`
- [ ] Detectar y reportar memory leaks en tests

### 1.3 Command management
- [ ] `CommandPool` por thread, reusables
- [ ] Submission via colas paralelas (graphics + compute + transfer)
- [ ] Frames-in-flight configurables (1, 2, 3) con semáforos timeline
- [ ] Sincronización correcta con fences y semaphores

### 1.4 Tests del núcleo
- [ ] Tests headless de creación / destrucción de `VulkanContext`
- [ ] Smoke test: crear ventana, abrir swapchain, renderizar 10 frames, cerrar
- [ ] Test de resize (cambiar tamaño de ventana 5 veces)
- [ ] Test de validación: 0 errores de validation layers

---

## 🎨 FASE 2 — Pipeline gráfico moderno

### 2.1 Dynamic Rendering (Vulkan 1.3)
- [ ] Reemplazar `VkRenderPass` + `VkFramebuffer` por `VK_KHR_dynamic_rendering`
- [ ] Eliminar todo el código de `subpasses` heredado
- [ ] Simplificar FrameGraph para usar dynamic rendering

### 2.2 Descriptor Indexing / Bindless
- [ ] Habilitar `VK_EXT_descriptor_indexing`
- [ ] Global texture array bindless (8192 slots)
- [ ] Global buffer array bindless (mesh + material data)
- [ ] `MeshHandle` / `MaterialHandle` / `TextureHandle` como índices u32
- [ ] Actualizar shaders para usar bindless

### 2.3 Pipeline State Object (PSO) cache
- [ ] Cache en disco (`.reactor/pipeline_cache.bin`)
- [ ] Hash de (shader SPIR-V + render state) → PSO
- [ ] Hot-reload de shaders con recompilación incremental
- [ ] Invalidación de cache cuando cambian shaders

### 2.4 Shader system
- [ ] Soporte de WGSL + GLSL + HLSL vía `shaderc` y/o `naga`
- [ ] Reflection automática (descriptor layouts derivados del SPIR-V)
- [ ] Includes y `#define` desde Rust (`shader!` macro)
- [ ] Compilación en background con `tokio`

### 2.5 GPU-driven rendering
- [ ] Indirect draw (`vkCmdDrawIndexedIndirect`)
- [ ] Culling en compute (frustum + occlusion HZB)
- [ ] Mesh shaders opcionales (`VK_EXT_mesh_shader`) cuando disponibles
- [ ] Benchmark: 1M+ objetos a 60 FPS

---

## 📦 FASE 3 — Asset Pipeline

### 3.1 Formatos soportados
- [ ] Modelos: glTF 2.0 (vía `gltf`) — ya básico
- [ ] Modelos: FBX (vía `russimp` o exporter externo)
- [ ] Texturas: PNG, JPG, HDR, EXR — ya básico
- [ ] Texturas: KTX2 + BCn / ASTC (vía `ktx2` + `basis-universal`)
- [ ] Audio: OGG, WAV, FLAC, MP3
- [ ] Fuentes: TTF / OTF (vía `fontdue` o `cosmic-text`)

### 3.2 Asset Database
- [ ] `AssetId(u64)` estable (hash del path + contenido)
- [ ] Metadata en `.reactor/assets.db` (sqlite o sled)
- [ ] Carga lazy + reference counting (`Handle<T>`)
- [ ] Streaming asíncrono con `tokio` o `async-std`

### 3.3 Asset cooker
- [ ] Pre-procesa assets RAW → formato runtime optimizado
- [ ] Texturas → BC7 / ASTC + mipmaps
- [ ] Meshes → meshlets (vía `meshopt`)
- [ ] Audio → OGG comprimido + bus tags
- [ ] CLI: `reactor cook --input assets/ --output cooked/`

### 3.4 Hot-reload
- [ ] Watcher con `notify` que recook + reupload en vivo
- [ ] Notificación al editor / juego vía `EventBus`
- [ ] UI de notificación de cambios en editor

---

## 🌟 FASE 4 — Renderer de producción

### 4.1 PBR completo
- [ ] Cook-Torrance (GGX + Schlick + Smith)
- [ ] Clear-coat, sheen, anisotropy, subsurface scattering
- [ ] Soporte `KHR_materials_transmission` (vidrio físico)
- [ ] Material instances con parámetros customizables

### 4.2 Image-Based Lighting (IBL)
- [ ] Equirectangular → cubemap → prefiltered + irradiance
- [ ] BRDF LUT precomputada
- [ ] Tone mapping ACES / AGX
- [ ] Exposure control automático

### 4.3 Sombras
- [ ] CSM (Cascaded Shadow Maps) para luces direccionales
- [ ] Point light shadows (cube maps)
- [ ] Spot light shadows (2D)
- [ ] VSM / PCSS para sombras suaves
- [ ] Sombras de ray tracing cuando RT esté disponible

### 4.4 Iluminación global
- [ ] Light probes (esféricos armónicos L2)
- [ ] Reflection probes (cubemaps locales)
- [ ] Screen-Space GI (SSGI) como fallback
- [ ] Voxel-cone GI opcional para escenas dinámicas

### 4.5 Post-processing AAA
- [ ] Bloom (mipmap chain con upsample dual)
- [ ] Motion blur (per-object + camera)
- [ ] Depth of field (Bokeh + circular)
- [ ] Auto-exposure (histograma compute)
- [ ] Color grading (LUT 3D)
- [ ] TAA / SMAA / FXAA seleccionable

### 4.6 Decals y particles GPU
- [ ] Decal system (deferred)
- [ ] GPU particles vía compute (millones de partículas)
- [ ] VFX graph básico (nodos)
- [ ] Particle collision con escena

---

## 🎮 FASE 5 — Sistemas de gameplay

### 5.1 ECS jerárquico
- [ ] Migrar a o integrar `hecs` / `bevy_ecs`
- [ ] `Parent` / `Children` con propagación de transform
- [ ] World queries con `With<T>` / `Without<T>` / `Changed<T>`
- [ ] Systems schedule con dependencias

### 5.2 Scripting
- [ ] Embed de Rhai o Lua (mlua) para scripting de gameplay
- [ ] Hot-reload de scripts en editor
- [ ] Bindings auto-generados desde tipos `#[reactor::reflect]`
- [ ] Sandbox de seguridad para scripts

### 5.3 Event bus
- [ ] `EventBus<T>` global + locales por escena
- [ ] `Observer<T>` para reaccionar (UI, audio, animaciones)
- [ ] Event priority y ordering

### 5.4 AI y navegación
- [ ] NavMesh vía `recast-rs` o port propio
- [ ] Pathfinding A* + jump links
- [ ] Behaviour trees (`bonsai-bt`)
- [ ] State machines (FSM + HSM)

### 5.5 Input avanzado
- [ ] Soporte gamepad (XInput / DInput / SDL2 / `gilrs`)
- [ ] Input mapping configurable (`input.toml`)
- [ ] Acciones, axes, gestures
- [ ] Haptic feedback

---

## 🧱 FASE 6 — Físicas y colisiones

### 6.1 Integración Rapier
- [ ] `rapier3d` (CPU) como backend por defecto
- [ ] Wrapper `PhysicsWorld` idiomático
- [ ] Sincronización ECS ↔ Rapier vía `Transform` y `RigidBody` component
- [ ] Debug visualization de colliders

### 6.2 Character Controller
- [ ] Kinematic capsule + step offset
- [ ] Slope handling, jump, crouch
- [ ] Networking-friendly (deterministic)
- [ ] Coyote time y jump buffering

### 6.3 Queries físicas
- [ ] Raycast, shape-cast, overlap, sweep
- [ ] Capa de colisión (groups + filters)
- [ ] Hit callbacks (on_hit, on_stay, on_exit)

### 6.4 Vehículos y joints
- [ ] Wheel collider (raycast vehicle)
- [ ] Hinge, ball, prismatic, fixed joints
- [ ] Breakable joints

### 6.5 GPU physics (opcional, largo plazo)
- [ ] Cloth (compute shader)
- [ ] Fluids (FLIP / SPH compute)
- [ ] Soft bodies

---

## 🔊 FASE 7 — Audio espacial 3D

### 7.1 Backend
- [ ] Integrar `kira` o `cpal` + `oddio` para 3D
- [ ] Buses (master, music, sfx, voice, ambience)
- [ ] Volúmenes y EQ por bus
- [ ] Ducking automático

### 7.2 Espacialización
- [ ] HRTF estéreo + binaural
- [ ] Atenuación distancia (lineal / log / custom)
- [ ] Doppler
- [ ] Reverb por zona (zones triggers)

### 7.3 Oclusión y obstrucción
- [ ] Raycast físico + filtros low-pass
- [ ] Portal-based propagation
- [ ] Diffraction simulation

### 7.4 Música dinámica
- [ ] Stems con transiciones por estado
- [ ] Sincronización a tempo (BPM events)
- [ ] Adaptive music system

---

## 🌐 FASE 8 — Networking

### 8.1 Transporte
- [ ] QUIC vía `quinn` (TCP/UDP-friendly)
- [ ] WebSocket + WebTransport para web
- [ ] Mensajes confiables y no-confiables
- [ ] Connection management

### 8.2 Replicación
- [ ] Snapshot de entidades + delta compression
- [ ] Interest management (relevancia por distancia)
- [ ] Replicated components con `#[reactor::replicate]`
- [ ] State synchronization

### 8.3 Predicción y reconciliación
- [ ] Client-side prediction
- [ ] Server reconciliation
- [ ] Lag compensation (rewind & replay)
- [ ] Interpolation

### 8.4 Lobby & matchmaking
- [ ] Server browser básico
- [ ] Integración Steam (Steamworks vía `steamworks-rs`)
- [ ] Matchmaking system
- [ ] Party system

---

## 🛠️ FASE 9 — Editor REACTOR completo

### 9.1 UI base
- [ ] Mantener `egui` + `egui_dock`
- [ ] Tema oscuro REACTOR + tema claro
- [ ] Layouts guardables
- [ ] Keyboard shortcuts

### 9.2 Paneles obligatorios
- [ ] Viewport 3D con cámara editor (FPS + orbit)
- [ ] Hierarchy (árbol de entidades, drag & drop, parent / unparent)
- [ ] Inspector (auto-generado vía reflection)
- [ ] Console (logs + filtros + cargo errors)
- [ ] Asset Browser (thumbnails, drag & drop al viewport)
- [ ] Scene panel (lista de escenas, build settings)
- [ ] Profiler (frame time, GPU stats, memory)

### 9.3 Gizmos
- [ ] Translate / Rotate / Scale (clicables)
- [ ] Snap a grid / vértices / ángulos
- [ ] Multi-selección
- [ ] Local / World / Parent space

### 9.4 Play mode in-place
- [ ] Play / Pause / Stop con snapshot reversible
- [ ] Edit-in-play (cambios no destructivos)
- [ ] Live debug del ECS
- [ ] Frame stepping

### 9.5 Scripting visual
- [ ] Sistema de nodos estilo Blueprints
- [ ] Eventos, branches, variables, llamadas a funciones Rust
- [ ] Custom node types
- [ ] Macro nodes

### 9.6 Reflection / Serialización
- [ ] `#[derive(Reflect)]` proc-macro para auto-inspector
- [ ] Serialización a JSON + RON + binary (`bincode`)
- [ ] Scene diffing
- [ ] Prefab system

---

## 📦 FASE 10 — Tooling y build pipeline

### 10.1 CLI `reactor`
- [ ] `reactor new <nombre>` — plantillas (FPS, 2D-platformer, racing, sandbox)
- [ ] `reactor run` — compila + lanza el juego
- [ ] `reactor cook` — cook assets a formato runtime
- [ ] `reactor pack` — empaqueta build (PAK virtual o `.reactor` archive)
- [ ] `reactor ship --platform windows|linux|macos|android` — build final

### 10.2 Plantillas (`reactor-templates/`)
- [ ] `template-fps/`
- [ ] `template-2d-platformer/`
- [ ] `template-racing/`
- [ ] `template-sandbox-minimal/`

### 10.3 Packer y archive format
- [ ] `.reactor` archive (chunked + zstd + mmap-friendly)
- [ ] Encriptación opcional (AES-GCM)
- [ ] Resource patching (DLC)
- [ ] Compression levels configurables

### 10.4 Installer / Distribution
- [ ] Bundle Windows (MSI / portable zip)
- [ ] AppImage / .deb / .rpm (Linux)
- [ ] .app + .dmg (macOS)
- [ ] APK (Android, Fase 11)

---

## 🖥️ FASE 11 — Plataformas y portabilidad

### 11.1 Desktop
- [ ] Windows 10/11 (x64 + ARM64)
- [ ] Linux (Ubuntu, Arch, Steam Deck / SteamOS)
- [ ] macOS vía MoltenVK (Vulkan → Metal)

### 11.2 Mobile
- [ ] Android (Vulkan 1.3 nativo, NDK)
- [ ] iOS vía MoltenVK + bindings (largo plazo)

### 11.3 Web
- [ ] WebGPU backend alternativo (vía `wgpu`)
- [ ] Build con `wasm-pack`
- [ ] Streaming de assets via fetch

### 11.4 Consolas (largo plazo, requiere licencias)
- [ ] Nintendo Switch (NVN)
- [ ] PS5 / Xbox (vía SDK propietario)

### 11.5 VR / XR
- [ ] OpenXR (`openxr-rs`)
- [ ] Foveated rendering
- [ ] Soporte Quest, Index, Vive, PSVR2

---

## ✅ FASE 12 — QA, performance y release v2.0

### 12.1 Testing
- [ ] >80 % cobertura en `crates/reactor/`
- [ ] Tests visuales (golden images con tolerancia perceptual)
- [ ] Fuzzing de loaders (gltf, ktx2, ogg)
- [ ] Integration tests

### 12.2 Performance
- [ ] Benchmarks `criterion` para core paths (draw, ECS, physics)
- [ ] Profiling integrado (Tracy via `tracy-client`)
- [ ] Memory profiling (`dhat`, `valgrind` en Linux CI)
- [ ] Performance regression tests

### 12.3 Documentación
- [ ] `cargo doc` completo + ejemplos en cada item público
- [ ] Libro REACTOR (`mdbook`) con tutoriales paso a paso
- [ ] Video-tutoriales oficiales
- [ ] API reference completa

### 12.4 Demo AAA
- [ ] Construir un juego demo de 30 min usando 100 % REACTOR
- [ ] Publicarlo gratuito en Steam / itch.io
- [ ] Source code disponible
- [ ] Post-mortem técnico

### 12.5 Release v2.0
- [ ] SemVer estable
- [ ] LTS 24 meses
- [ ] Anuncio público + landing en reactor.salazar-interactive.dev
- [ ] Migration guide desde v1.x

---

## 📊 Métricas de éxito (KPIs)

| KPI | Objetivo v2.0 |
|-----|---------------|
| Líneas de código C / C++ | 0 |
| Cobertura de tests | ≥ 80 % |
| Errores de validation layer en demo AAA | 0 |
| FPS demo "Sponza PBR" en RTX 3060 @ 4K | ≥ 60 FPS |
| Tiempo de compilación full (workspace, release) | ≤ 90 s |
| Crash-free sessions del demo Steam | ≥ 99,5 % |
| Plataformas soportadas con CI | ≥ 5 |
| Tamaño del runtime base (sin assets) | ≤ 25 MB |

---

## 🤝 Cómo contribuir

1. Elegir un ítem `[ ]` no marcado de la fase activa
2. Abrir un issue con prefijo `[Fxx]` (ej. `[F2] PSO cache en disco`)
3. Crear rama `fXX/<slug>` (ej. `f2/pso-cache`)
4. PR con tests + `cargo clippy --all -- -D warnings` limpio
5. Marcar el checkbox en este archivo al hacer merge

---

**Última actualización:** Mayo 2026
**Versión actual:** 1.1.0-rust
