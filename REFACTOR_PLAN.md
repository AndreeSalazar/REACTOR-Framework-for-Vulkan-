# REACTOR — Plan de Refactor y Completado

**Fecha:** 18-Jun-2026
**Estado del proyecto:** 32K LOC, 119 archivos, deferred+forward render, 21 features AAA, ~3.2K LOC huérfanos
**Objetivo:** Cerrar brechas AAA + modularizar para mantenibilidad

---

## Diagnóstico consolidado

### Lo que funciona ✅
- **Base arquitectónica sólida** — `Arc<Inner<T>>` para Vulkan, RAII con Drop, Result propagación
- **Pipeline de render maduro** — Forward PBR con IBL + CSM PCSS + SSDS + TAA + 12 efectos post-proc
- **Subsistemas completos** — IBL baker, asset pipeline con tokio, hot-reload, frame graph, profiler
- **Validación Vulkan** — Habilitada y reportando errores correctamente
- **BUILD CLEAN** — `cargo build --example xenofall` sin errores

### Las 4 brechas técnicas CRÍTICAS (gaps de funcionalidad)

| # | Brecha | Impacto | Archivos |
|---|--------|---------|----------|
| **B1** | **G-Buffer creado pero nunca escrito** — `core/shader.frag` (forward) escribe a offscreen; `gbuffer.frag`+MRT no se ejecuta. `lighting_resolve.frag` existe pero no se llama | Alto: imposibilita deferred real, GI por G-Buffer, decals MRT no producen efecto | `reactor/draw.rs:531-687`, `gbuffer.frag`, `lighting_resolve.frag` |
| **B2** | **Hi-Z pyramid fantasma** — 3 shaders lo esperan (`cull.comp`, `ssgi_hiz.comp`) pero ningún compute lo genera. Falta `hiz_build.comp` y el recurso | Alto: SSGI Hi-Z + GPU culling bloqueados | `shaders/post/hiz_build.comp` (NO EXISTE), `src/graphics/hiz.rs` (NO EXISTE) |
| **B3** | **Light culling con count=0** — `light_cull.comp` listo, `post_process.dispatch_light_cull` se llama, pero `draw.rs:818` pasa `light_count: 0` | Alto: 360 LOC de compute inútiles, no hay clustered lighting real | `reactor/draw.rs:818`, `scene/lighting.rs` |
| **B4** | **Volumetric clouds con shader completo, sin dispatch** — `volumetric_clouds.comp` 197 LOC, no instanciado, faltan texturas 3D noise | Alto: feature AAA listada pero inerte | `shaders/post/volumetric_clouds.comp` (existe), falta `dispatch_volumetric_clouds` + 3D noise generation |

### Las 6 brechas GRAVES de robustez (producen panics o fugas)

| # | Brecha | Archivos | Fix |
|---|--------|----------|-----|
| **R1** | **2 `panic!()` recuperables** | `core/frame_graph.rs:336`, `systems/particles.rs:82` | Convertir a `Result` o default |
| **R2** | **60+ `unwrap()` tras `is_some()` check** en `draw.rs` y `post_process.rs` | `reactor/draw.rs` (26), `post_process.rs` (28) | Refactor a `if let Some` propagation |
| **R3** | **17 `Mutex::lock().unwrap()` en hot path** (poisoning propagation) | `core/allocator.rs`, `graphics/pso_cache.rs`, `resources/asset_loader_queue.rs` | Migrar a `parking_lot::Mutex` |
| **R4** | **Solo 7 `// SAFETY:` de 416 `unsafe`** — el 99% del unsafe no documenta invariantes | `reactor/init.rs:480-587`, `core/context.rs`, `core/arc_handle.rs` | Añadir SAFETY comments en bloques críticos |
| **R5** | **Magic numbers hardcoded** (4 cascades, 2048×2048 shadow) | `reactor/draw.rs:195,208,227,818` | Mover a `ShadowConfig` |
| **R6** | **Drop silencioso con `eprintln!`** bypasea tracing | `graphics/buffer.rs:172`, `graphics/image.rs:282` | Usar `tracing::error!` |

### Las 5 brechas de modularización (deuda arquitectónica)

| # | Brecha | LOC | Plan |
|---|--------|-----|------|
| **M1** | `graphics/post_process.rs` monolítico | 3212 (9.1%) | Dividir en `bloom.rs`, `taa.rs`, `gtao.rs`, `fog.rs`, `flare.rs`, `light_cull.rs`, `ssgi.rs`, `effects.rs` |
| **M2** | `reactor/draw.rs` mega-función | 1744 | Dividir por pass: `draw_geometry.rs`, `draw_post.rs`, `draw_shadow.rs`, `cull.rs` |
| **M3** | `reactor/init.rs` 696 líneas, 7 sub-responsabilidades | 696 | Extraer `init_context.rs`, `init_resources.rs`, `init_shadows.rs` (separar de init.rs) |
| **M4** | `graphics/ibl.rs` 1472 líneas | 1472 | Dividir en `ibl/equirect.rs`, `ibl/irradiance.rs`, `ibl/prefilter.rs`, `ibl/lut.rs`, `ibl/pipeline.rs` |
| **M5** | ~3.2K LOC huérfanos | 3200 | Mover `_legacy_backup/`, `renderer/forward.rs`, `renderer/bindless_forward.rs` (rehacer o archivar) |

---

## Plan de trabajo en 4 fases

### **FASE 1 — Cerrar brechas críticas de funcionalidad (Quick wins AAA)**
**Duración:** 1 sesión · **Impacto:** Alto · **Riesgo:** Bajo

1. **B3** Enchufar `light_cull` con luces reales (S)
   - Modificar `scene/lighting.rs` para serializar luces a `LightBuffer` cada frame
   - Pasar `light_count` real en `draw.rs:818`
2. **B2** Crear `hiz_build.comp` + `src/graphics/hiz.rs` (M)
   - Compute shader que reduce depth a mip-chain
   - Recurso Image con mip-levels
   - Dispatch en draw.rs después de depth_resolve
3. **B2'** Activar `ssgi_hiz.comp` ahora que Hi-Z existe (S)
   - Añadir `dispatch_ssgi_hiz` en draw.rs
4. **B4** Integrar `volumetric_clouds.comp` (L)
   - Crear `src/graphics/noise_3d.rs` con Worley + Perlin generation
   - Subir a Image3D vía staging buffer
   - `init_volumetric_clouds`, `dispatch_volumetric_clouds`
   - Añadir binding 9 (cloudTexture) a post_process.frag
   - Sampling en fragment shader
5. **GPU culling** wired (M) — conectar `cull.comp` ya integrado con Hi-Z

**Resultado esperado:** REACTOR pasa de "Indie premium" a **"AAA feature-complete"** en GI, sombras, nubes, culling.

### **FASE 2 — Robustez y eliminación de panics (correctness)**
**Duración:** 1 sesión · **Impacto:** Alto · **Riesgo:** Bajo

1. **R1** Convertir 2 `panic!` a Result/default
2. **R2** Refactorizar 60+ `unwrap()` en `draw.rs` y `post_process.rs` a `if let Some` propagation
3. **R3** Migrar allocator/pso_cache/asset_loader_queue a `parking_lot::Mutex`
4. **R4** Añadir `// SAFETY:` a 30 bloques unsafe más críticos
5. **R5** Mover magic numbers a `ShadowConfig`
6. **R6** Reemplazar `eprintln!` con `tracing::error!`

**Resultado esperado:** Score de robustez 5.8 → 8.5/10. Cero panics recuperables.

### **FASE 3 — Modularización de archivos monolíticos**
**Duración:** 1-2 sesiones · **Impacto:** Medio · **Riesgo:** Medio (refactor invasivo)

1. **M1** Dividir `post_process.rs` (3212 LOC) en submódulos
2. **M2** Dividir `draw.rs` (1744 LOC) por pass
3. **M3** Dividir `init.rs` (696 LOC) por subsistema
4. **M4** Dividir `ibl.rs` (1472 LOC) por componente

**Resultado esperado:** Archivos más pequeños y enfocados. Mejor build times incrementales. Mejor DX.

### **FASE 4 — Cleanup y features avanzadas**
**Duración:** múltiples sesiones

1. Eliminar `renderer/forward.rs` (stub) y `renderer/bindless_forward.rs` (rehacer o archivar)
2. Decidir qué hacer con `core/importance_map.rs` (430 LOC huérfanos)
3. Mover `_legacy_backup/` fuera del repo (ya no se compila)
4. Implementar Ray Tracing shaders reales (XL)
5. Implementar Skinning/Skeletal (L)
6. **FASE 5 (futuro)**: DDGI, ReSTIR GI, FSR 2, Virtual geometry

---

## Orden de ejecución recomendado

```
Fase 1 (AAA features) ─► Fase 2 (Robustez) ─► Fase 3 (Modular) ─► Fase 4 (Cleanup+Avanzado)
        ↓                       ↓                       ↓
  Cierra G-Buffer,         Cierra panics,        Archivos más         Eliminar huérfanos,
  Hi-Z, Clouds,            unwraps, magic        pequeños y           implementar RT,
  LightCull,               numbers, SAFETY       enfocados            skinning, FSR
  GPU cull                 comments
```

**Reglas durante la ejecución:**
- Cada cambio compila limpio
- Cada cambio preserva el comportamiento existente (no breaking changes)
- Commit por cada feature cerrada (B1, B2, B3, B4 individualmente)
- Validar con `cargo build --example xenofall` después de cada sub-cambio

---

## Métricas objetivo

| Métrica | Actual | Después Fase 1 | Después Fase 2 | Después Fase 3 |
|---------|--------|----------------|----------------|----------------|
| Features AAA activas | 14/40 | **20/40** | 20/40 | 20/40 |
| LOC huérfanos | 3.200 | 3.200 | 3.200 | <500 |
| `panic!` recuperables | 2 | 2 | **0** | 0 |
| `unwrap()` en hot path | 60+ | 60+ | **<10** | <10 |
| `// SAFETY:` coverage | 1.7% | 1.7% | **15%** | 15% |
| Archivo más grande | 3.212 LOC | 3.212 | 3.212 | **<1.000** |
| Robustez score | 5.8/10 | 5.8 | **8.5** | 8.5 |
| Clasificación | Indie premium | **AAA feature-complete** | AAA robust | AAA maintainable |
