# REACTOR — Ideas para Delta Rendering Inteligente

Este documento captura las ideas actuales para XENOFALL/REACTOR y las convierte en una lista de diseño a considerar. No es una promesa cerrada de implementación: es una guía para decidir qué vale la pena prototipar, medir y luego integrar.

## Observación clave

En una escena típica de XENOFALL, gran parte de la imagen no cambia entre frames:

- Corredores, paredes, suelo y props estáticos pueden permanecer iguales durante muchos frames.
- Personajes, enemigos, balas, partículas, puertas, luces dinámicas y sombras móviles cambian constantemente.
- El render tradicional suele pagar mucho trabajo cada frame aunque solo una parte pequeña de la imagen haya cambiado.

La idea central de REACTOR es tratar la VRAM como memoria activa y persistente:

> RenderMan calcula el universo. REACTOR calcula lo que importa.

## Delta Rendering

### Idea base

En vez de recalcular el 100% del frame, REACTOR podría recalcular solo las regiones afectadas por cambios visibles y reutilizar el resto desde history/cache.

```text
Frame anterior + mapa de cambios + acumulación temporal = frame actual
```

Objetivo conceptual:

- 95% de pixels estables: copiar/reusar/validar desde history.
- 5% de pixels sucios: recalcular con raster/path tracing/lighting dinámico.
- Resultado visual equivalente o suficientemente indistinguible para tiempo real.

### Implementación propuesta

1. **Dirty Buffer**
   - Mapa por tile/pixel/región que indica qué cambió.
   - Cada objeto dinámico marca su bounding rect, su sombra, sus reflejos aproximados y su zona de influencia como dirty.
   - Idealmente trabajar por tiles, no por pixel individual, para reducir overhead.

2. **Static Cache en VRAM**
   - Datos estáticos calculados una vez o actualizados muy lentamente.
   - Iluminación estática, GI acumulada, sombras fijas, probes, lightmaps dinámicas persistentes, depth estático, materiales y texturas.
   - En una RTX 3060 de 12GB, reservar una zona grande para datos persistentes tiene sentido si se mide y controla.

3. **Delta Pass**
   - Los tiles sucios se recalculan.
   - Los tiles limpios se copian o reconstruyen desde history/static cache.
   - Motion vectors y depth history validan si el pixel todavía representa la misma superficie.

4. **Blend final**
   - Combinar cache estático + contribución dinámica + acumulación temporal.
   - Aplicar filtros de estabilidad para evitar ghosting, bleeding, lag visual o sombras congeladas.

### Importante: invalidación correcta

El reto real no es copiar pixels; el reto es saber cuándo ya no son válidos.

Un pixel aparentemente estático puede invalidarse por:

- Movimiento de cámara.
- Disocclusion: aparece una zona antes tapada.
- Cambios de exposición/tonemapping.
- Sombras de objetos dinámicos.
- Reflejos/specular de objetos que se movieron fuera del pixel directo.
- Luces dinámicas o flickering.
- Partículas transparentes delante.
- Materiales animados.
- GI indirecta afectada por objetos dinámicos.

Por eso el sistema debe cachear con metadatos, no solo con color:

- Depth.
- Normal.
- Material ID.
- Object/instance ID.
- Motion vector.
- Roughness/metalness.
- Confidence/history length.
- Último frame actualizado.

## VRAM inteligente: “la VRAM que se acuerda”

### Problema histórico

Muchos pipelines tratan la VRAM como una mula de transporte:

```text
Cargar datos → procesar → descartar/olvidar → repetir
```

REACTOR debería tratar la VRAM como memoria persistente:

```text
Calcular → guardar → validar → mejorar → reutilizar
```

### División conceptual de VRAM para RTX 3060 12GB

La división exacta debe ser dinámica y medida, pero como punto de partida:

```text
Zona permanente / persistente (~5–6GB)
├── Iluminación estática calculada
├── GI acumulada de escena
├── Texturas y materiales residentes
├── Shadow cache estático
├── Probes / irradiance volumes
└── Cache de geometría estática

Zona dinámica (~3–4GB)
├── Personajes y enemigos
├── Partículas y efectos
├── Dynamic shadows
├── History buffers temporales
├── Motion vectors
└── Buffers para ray queries / compute

Zona sistema / working set (~2GB)
├── Pipeline state
├── Descriptors
├── Staging/transient buffers
├── Swapchain/depth/G-buffer
└── Reservas de seguridad
```

### Tiers de memoria

1. **Tier 1 — Permanente**
   - Iluminación estática, GI estable, sombras fijas, probes y datos de nivel.
   - Nunca se borra durante el nivel salvo cambio fuerte de escena.

2. **Tier 2 — Semi-permanente**
   - Nubes, vegetación, ambiente, efectos lentos.
   - Actualización cada N frames o por presupuesto.

3. **Tier 3 — Dinámico**
   - Personajes, balas, enemigos, puertas, físicas, luces móviles.
   - Actualización cada frame.

4. **Tier 4 — Temporal**
   - Buffers intermedios de cálculo.
   - Se descartan o reciclan al final del frame.

### Ventaja de Vulkan

Vulkan permite control explícito de memoria, sincronización, descriptors y residency. REACTOR debería usar eso para decidir:

- Qué datos viven permanentemente.
- Qué datos se actualizan por presupuesto.
- Qué datos se pueden desalojar.
- Qué tareas corren en graphics/compute/transfer queues.
- Qué recursos son bindless/descriptoreados para reducir cambios de estado.

## Acumulación temporal estilo RenderMan, pero en tiempo real

### Insight

RenderMan/Cycles logran calidad alta acumulando muchas muestras. El problema es que offline suelen recalcular muchísimo por frame y no tienen las mismas restricciones de tiempo real.

REACTOR puede invertir la filosofía:

```text
Pocas muestras por frame
+ memoria persistente en VRAM
+ validación temporal
+ acumulación inteligente
= calidad que converge con el tiempo
```

Ejemplo conceptual:

```text
Frame 1:    calidad baja pero usable
Frame 10:   menos ruido
Frame 30:   calidad estable
Frame 60+:  acumulación casi indistinguible si la escena es estable
```

La matemática simple de la idea:

```text
Offline bruto:
1000 rayos × 12 rebotes = 12,000 cálculos/pixel/frame

REACTOR temporal:
16 rayos × 3 rebotes = 48 cálculos/pixel/frame
+ acumulación en 200 frames
= muchas muestras acumuladas donde la imagen permanece válida
```

### Técnicas relacionadas a estudiar

- TAA con rejection robusto.
- SVGF / A-Trous denoising temporal.
- ReSTIR DI/GI para reusar muestras de iluminación.
- Reservoir sampling por pixel/tile.
- Checkerboard rendering + temporal resolve.
- Variable Rate Shading por importancia.
- Foveated rendering si hay soporte o modo experimental.
- Ray queries selectivas para sombras/reflejos importantes.

## Importance Sampling visual

La idea no es calcular todos los rayos posibles, sino los que más afectan el resultado visual.

Prioridades:

- Luces con mayor contribución estimada.
- Superficies visibles y cercanas.
- Bordes, siluetas y regiones con alto contraste.
- Materiales brillantes donde el error se nota más.
- Movimiento rápido, porque history es menos confiable.
- Zonas donde el jugador mira o apunta.

Se puede ignorar o aproximar contribución menor al umbral visual, por ejemplo contribuciones bajo ~0.1%, siempre que el error no se acumule de forma visible.

## “La cámara es la realidad”

### Principio

En tiempo real, lo que no afecta la cámara no debe consumir el mismo costo que lo visible.

```text
Si la cámara no lo ve y no afecta lo visible, no existe para este frame.
```

Esto no significa ignorar toda luz fuera de cámara, porque puede afectar rebotes o sombras visibles. Significa aproximarla con estructuras baratas cuando su contribución indirecta no justifica trazado completo.

### Técnicas base

- Frustum culling agresivo.
- Occlusion culling con Hi-Z depth pyramid.
- GPU culling por AABB/meshlet.
- Clustered/Forward+ light culling.
- Portal rendering para corredores y habitaciones.
- BVH/Octree para agrupar estáticos.
- LOD y HLOD por distancia, tamaño en pantalla y relevancia.
- Impostors o cards para elementos lejanos.

### Para XENOFALL

XENOFALL se beneficia mucho porque su estética de corredores, puertas, salas y líneas de visión permite:

- Portal rendering por puertas/pasillos.
- Room/sector visibility.
- Precomputed Potentially Visible Sets (PVS).
- Cache estático por sector.
- Activación dinámica solo de enemigos/efectos del sector visible o cercano.

## Mejoras adicionales para considerar

### 1. Dirty tiles en vez de dirty pixels

Usar tiles de 8×8, 16×16 o 32×32 permite marcar regiones sucias con menos overhead. Cada tile puede tener:

- Dirty directo.
- Dirty por sombra.
- Dirty por reflexión.
- Dirty por GI.
- Dirty por transparencia.
- Nivel de confianza temporal.

### 2. Cache con confidence score

Cada pixel/tile debería saber cuánto confiar en su history:

- History largo y estable: reutilizar más.
- Cámara movida: reducir confianza.
- Cambio de normal/depth/material: resetear.
- Movimiento rápido: acumular menos.
- Disocclusion: recalcular completo.

### 3. Separar caches por tipo de contribución

No todo debe vivir en un solo color buffer. Separar permite invalidar menos:

- Direct lighting cache.
- Indirect diffuse cache.
- Specular/reflection cache.
- Shadow cache.
- Ambient/probe cache.
- Albedo/material cache.

Si solo cambia una sombra dinámica, no debería invalidar todo el shading estático.

### 4. Static shadow atlas persistente

Sombras de objetos estáticos y luces estáticas pueden vivir en atlas persistente. Solo las sombras de actores dinámicos entran en un overlay dinámico.

### 5. GI por probes persistentes

Para corredores, una red de probes por sector puede acumular GI durante muchos frames. Las probes cercanas a cambios dinámicos se actualizan con menor frecuencia o bajo presupuesto.

### 6. Portal + sector cache

Cada sala/corredor puede tener su propio paquete persistente:

- Static geometry cache.
- Light cache.
- Probe cache.
- Visibility list.
- Occluders principales.

Cuando el jugador cruza una puerta, REACTOR precalienta el sector siguiente antes de que sea dominante en pantalla.

### 7. Render budget fijo por frame

En vez de intentar terminar todo cada frame:

- Presupuesto fijo de ms para GI.
- Presupuesto fijo de ms para sombras.
- Presupuesto fijo de ms para denoise.
- Cola de trabajos pendientes con prioridad visual.

Esto evita spikes y mantiene frame pacing.

### 8. Async compute

Usar compute para tareas que pueden solaparse:

- Construcción de Hi-Z.
- Culling GPU.
- Clasificación de tiles.
- Denoising.
- Actualización de probes.
- Compactación de listas visibles.

### 9. Meshlets y GPU-driven rendering

Convertir geometría a meshlets/clusteres permite:

- Culling más fino.
- LOD más controlado.
- Menos draw calls.
- Mejor compatibilidad con indirect draws.

### 10. Sistema de “reprojection primero”

Antes de renderizar caro:

1. Reproyectar frame anterior con motion vectors.
2. Validar depth/normal/material.
3. Marcar fallos como dirty.
4. Renderizar solo lo que falló o cambió.

### 11. Denoiser consciente de gameplay

El denoiser no debe borrar información importante:

- Siluetas de enemigos.
- Balas/proyectiles.
- UI diegética.
- Luces de amenaza.
- Sangre/impactos/feedback de combate.

### 12. Métricas obligatorias

Para evitar autoengaño, cada prototipo debe mostrar métricas:

- Porcentaje de tiles reused vs recalculated.
- Tiempo GPU por pass.
- VRAM persistente usada.
- Número de invalidaciones por causa.
- Ghosting/disocclusion failures.
- Latencia de convergencia visual.
- Spikes por cambio de sector.

## Riesgos técnicos

Estas ideas son fuertes, pero hay que tratarlas con cuidado.

- Reusar pixels no siempre da el mismo resultado si la cámara, luces o materiales cambian.
- La acumulación temporal puede crear ghosting.
- Cache persistente puede consumir demasiada VRAM si no hay política de eviction.
- Dirty tracking incorrecto produce artefactos difíciles de depurar.
- Transparencias, partículas y reflejos son casos duros.
- Path tracing parcial necesita denoising excelente para no verse inestable.
- “0% costo” para estáticos es ideal conceptual; en la práctica siempre queda costo de composición, validación, reproyección y bandwidth.

La regla: primero prototipar simple, medir, luego complicar.

## Plan de prototipos sugerido

### Fase 2 — XENOFALL como laboratorio visual profesional

Antes de implementar delta rendering completo, XENOFALL debe servir como escena de estrés controlada para shaders, agua ligera, VFX y métricas. Lo más recomendable:

1. **Materiales profesionales primero**
   - Corredor mojado con roughness baja, normal maps, grime masks y zonas metálicas.
   - Esto es barato, estático y cacheable en VRAM persistente.

2. **Agua simple antes de agua física**
   - Charcos pequeños, no simulación oceánica.
   - SSR + normales animadas + variación de roughness.
   - Ripples reactivos solo después de tener puddles estables.

3. **VFX de gameplay antes de VFX decorativos**
   - Muzzle flash, trazadores, impactos, sangre, polvo y chispas.
   - Deben marcar dirty tiles siempre porque comunican combate.

4. **Luces dinámicas medidas**
   - Flicker de emergencia, luces de disparo, luces rojas de alarma.
   - Buen test para invalidación de sombras, bloom e iluminación.

5. **Fog/volumetría con presupuesto fijo**
   - Low-res, temporal, bilateral upsample.
   - Nunca debe causar spikes.

6. **Decals semi-persistentes**
   - Bullet holes, sangre, leaks, quemaduras.
   - Ideales para probar memoria persistente de mundo.

7. **Debug visual obligatorio**
   - Overlay de dirty/reused tiles.
   - Métricas de history accepted/rejected.
   - VRAM por tier.

Esta fase queda representada en código por `xenofall/visual_features.rs`.

### Prototipo A — Dirty tile renderer

- Dividir pantalla en tiles.
- Marcar tiles afectados por objetos dinámicos.
- Reusar color del frame anterior en tiles limpios.
- Visualizar overlay de tiles dirty/reused.

### Prototipo B — History validation

- Añadir depth/normal/object ID history.
- Rechazar history cuando no coincide.
- Medir ghosting y disocclusion.

### Prototipo C — Static cache por sector

- Elegir un corredor/sala de XENOFALL.
- Cachear lighting estático.
- Recalcular solo actor dinámico + sombras.
- Comparar frame time contra render completo.

### Prototipo D — Portal visibility

- Definir sectores y portales.
- Renderizar solo sectores visibles desde puertas/cámara.
- Precalentar sector siguiente.

### Prototipo E — Temporal GI/probes

- Colocar probes persistentes.
- Actualizarlas por presupuesto.
- Reusar GI acumulada entre frames.

## Nota de arquitectura: XENOFALL deja atrás el monolito

El directorio `xenofall/` hereda el diseño y datos que nacieron en `examples/xenofall.rs`.

`examples/xenofall.rs` fue útil como prueba monolítica para construir XENOFALL sobre REACTOR, pero no debe ser la arquitectura final.

Decisión a considerar:

- Migrar comportamiento del monolito hacia módulos dentro de `xenofall/`.
- Mantener `examples/xenofall.rs` solo mientras sirva como referencia temporal.
- Eliminar `examples/xenofall.rs` cuando `xenofall/` tenga equivalencia funcional suficiente.
- Evitar seguir agregando features grandes al monolito.

Módulos deseables a futuro:

```text
xenofall/
├── audio.rs
├── cards.rs
├── constants.rs
├── helpers.rs
├── types.rs
├── world.rs              # sectores, salas, portales
├── player.rs             # cámara, input, estado jugador
├── enemies.rs            # IA, spawns, animación
├── weapons.rs            # disparos, balas, daño
├── rendering.rs          # integración con REACTOR
├── visibility.rs         # culling, portals, PVS
├── delta_rendering.rs    # dirty tiles/history/cache
├── lighting.rs           # probes, shadows, GI
└── game.rs               # loop y estado principal
```

## Frase guía

> REACTOR no intenta simular todo el universo cada frame. REACTOR recuerda lo estable, recalcula lo importante y usa la cámara como definición de realidad visual.
