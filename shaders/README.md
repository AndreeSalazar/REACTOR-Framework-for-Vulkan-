# REACTOR · `shaders/`

Librería de shaders del motor. Organizada por **propósito**, no por tipo. Toda
fuente GLSL se compila automáticamente a SPIR-V en `cargo build` vía
[`build.rs`](../build.rs).

---

## 📁 Estructura

```text
shaders/
├── README.md                ← este archivo
│
├── lib/                     ← snippets reutilizables (`#include` only)
│   ├── color.glsl           ─ sRGB ⇄ linear, luminance, grading
│   ├── noise.glsl           ─ hash, value/perlin noise, fbm, dither
│   ├── pbr.glsl             ─ Cook-Torrance: D_GGX · V_Smith · F_Schlick · Burley diffuse
│   ├── ibl.glsl             ─ IBL **procedural** (sky cielo/horizonte + 5-tap diffuse)
│   ├── ibl_textures.glsl    ─ IBL **desde texturas pre-baked** (cubemap HDR + LUT)
│   ├── lighting.glsl        ─ Directional / Point / Spot + studio 3-point
│   └── tonemap.glsl         ─ ACES Narkowicz · ACES fitted · AgX
│
├── core/                    ← pipelines built-in del framework
│   ├── shader.vert/.frag    → alias  vert.spv  /  frag.spv
│   └── texture.vert/.frag   → alias  texture_vert.spv / texture_frag.spv
│
├── post/                    ← post-process chain (13 efectos)
│   └── post_process.vert/.frag → alias post_process_{vert,frag}.spv
│
├── live/                    ← Blender Live Link (mini-PBR estudio)
│   └── blender_live.vert/.frag → alias blender_live_{vert,frag}.spv
│
└── ibl/                     ← Compute shaders para cocinar IBL en GPU
    ├── equirect_to_cube.comp    ─ HDR equirect 2D → cubemap RGBA16F
    ├── irradiance.comp          ─ cubemap → irradiance difusa (32×32)
    ├── prefilter.comp           ─ cubemap → specular prefilt (Karis GGX importance)
    └── brdf_lut.comp            ─ split-sum LUT 2D (RG16F)
```

Las **8 rutas canónicas de salida** (`vert.spv`, `frag.spv`,
`texture_{vert,frag}.spv`, `post_process_{vert,frag}.spv`,
`blender_live_{vert,frag}.spv`) viven directamente en `shaders/` porque
`include_bytes!` del runtime las referencia con esa ruta exacta. Toda otra
fuente compila junto a sí misma.

---

## 📚 Cómo usar la lib desde un shader

Cualquier `.vert` / `.frag` de REACTOR puede consumir la librería:

```glsl
#version 450
#extension GL_GOOGLE_include_directive : require

#include "pbr.glsl"
#include "ibl.glsl"
#include "lighting.glsl"
#include "tonemap.glsl"

void main() {
    // ... usa light_studio_3point, ibl_eval, agx_default, ...
}
```

`build.rs` invoca `glslc -I shaders/lib --target-env=vulkan1.3 -O` así que el
preprocesador resuelve los `#include` automáticamente. Las cabeceras usan
guardas `#ifndef … #define … #endif` para tolerar inclusión múltiple.

---

## 🧪 Recetas rápidas

### Shadear un objeto con lighting de estudio + IBL

```glsl
#include "pbr.glsl"
#include "ibl.glsl"
#include "lighting.glsl"

vec3 lo  = light_studio_3point(N, V, albedo, metallic, roughness, f0);
vec3 amb = ibl_eval(N, V, albedo, metallic, roughness, f0, ao);
vec3 color = lo + amb;
```

### Añadir luces dinámicas Frostbite-style

```glsl
color += light_eval_point(N, V, P, lightPos, lightColor, /*intensity*/ 12.0,
                          /*range*/ 25.0, albedo, metallic, roughness, f0);
color += light_eval_spot (N, V, P, spotPos, spotDir, spotColor, 15.0, 20.0,
                          cosInner, cosOuter, albedo, metallic, roughness, f0);
```

### Tone mapping de calidad

```glsl
#include "tonemap.glsl"

color = agx_default(color);            // moderno, alto rango dinámico
// alternativas:
// color = aces_filmic_fitted(color);  // ACES RRT+ODT, look "cinema"
// color = aces_filmic_narkowicz(color); // ACES rápido
```

### Micro-detalle anti-plástico

```glsl
#include "noise.glsl"

float n = value_noise(P * 4.0);
roughness = clamp(roughness + (n - 0.5) * 0.12, 0.04, 1.0);

// dithering 1/255 (anti-banding en gradientes)
color += bayer_dither(gl_FragCoord.xy);
```

---

## 🔭 Qué falta para subir al nivel UE5 / Unity HDRP

| Pieza                                          | Estado | Cómo subir nivel |
|------------------------------------------------|--------|------------------|
| **Cubemap HDR real** (KTX2 prefilt)            | ❌ (procedural) | Cargar IBL desde `assets/`, prefilt en compute shader, exponer 1 cubemap + 1 BRDF LUT 2D al frag shader vía descriptors. Sustituye `sampleEnv`. |
| **Normal mapping**                             | ❌ | Añadir slot tangent al vértice (`VertexPBR`) + sampler de normal map, TBN matrix correcto. |
| **Shadow maps cascaded (CSM)**                 | ✅ v1.6 | 4 cascadas depth-only + PCF Poisson rotado en `blender_live.frag`; falta debug view, PCSS/contact-hardening y blend temporal. |
| **Screen-Space Reflections** (SSR)             | ❌ | Compute shader sobre depth + hierarchical Hi-Z marcha. |
| **Screen-Space AO** (GTAO)                     | ❌ | Reemplaza `curvature_AO` por compute pass sobre depth + normal G-buffer. |
| **Volumetric fog froxels**                     | ❌ | Compute shader poblando un volumen 3D + ray-march en la composición. |
| **TAA con motion vectors**                     | ❌ | Sub-pixel jitter en projection + clipping en el composite. |
| **Bloom mip-chain (13 niveles, Karis avg)**    | ❌ | Sustituir bloom single-pass de `post_process.frag` por downsample/upsample (COD AW 2014). |
| **Path-traced reference** (verifier)           | ❌ (RT base existe) | Cocinar pipeline RT en `src/raytracing/` + denoiser SVGF. |
| **Material capas** (clearcoat, sheen, aniso)   | ❌ | Añadir lóbulos extra al BRDF — el slot existe en `pbr.glsl`. |
| **HDR display output** (HDR10 / scRGB)         | ❌ | `VK_EXT_hdr_metadata` + swapchain en formato RGBA16F. |

Cada item es **incremental** sobre la lib actual: la API de funciones
(`brdf_eval`, `ibl_eval`, `light_eval_point`, etc.) ya está modelada para
extenderse, no requiere reescribirse.

---

## 🛠️ Recompilar

`cargo build` recompila los shaders que tengan source más nuevo que el .spv.
Para forzar recompilación:

```bash
Remove-Item shaders/*.spv -Force
cargo build
```

Para inspeccionar SPIR-V generado:

```bash
spirv-dis shaders/blender_live_frag.spv | less
```
