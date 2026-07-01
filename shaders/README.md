# REACTOR · `shaders/`

Librería de shaders del motor, organizada en **2 categorías**:

| Categoría | Directorio | Propósito |
|-----------|------------|-----------|
| **REACTOR base** | `reactor/` | Shaders del núcleo del motor (built-in, deferred, compute, IBL, post-process, partículas) |
| **Live Link** | `live_link/` | Shaders profesionales para reflejar Blender en tiempo real (mini-PBR estudio, sombras) |

Ambas categorías comparten la biblioteca de headers GLSL en [`lib/`](lib/).

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
│   ├── ibl.glsl             ─ IBL procedural (sky + 5-tap diffuse)
│   ├── ibl_textures.glsl    ─ IBL desde texturas pre-baked
│   ├── lighting.glsl        ─ Directional / Point / Spot + studio 3-point
│   ├── tonemap.glsl         ─ ACES Narkowicz · ACES fitted · AgX
│   ├── parallax.glsl        ─ Parallax mapping routines
│   └── sky.glsl             ─ Sky rendering routines
│
├── reactor/                 ← REACTOR base engine shaders
│   ├── core/                ─ Pipelines built-in (vert.spv, frag.spv, texture)
│   ├── deferred/            ─ Geometry pass G-Buffer profesional
│   ├── compute/             ─ Frustum culling, light culling
│   ├── ibl/                 ─ Compute shaders para cocinar IBL en GPU
│   ├── particles/           ─ Sistema de partículas
│   └── post/                ─ Post-process chain (13 efectos: TAA, SSGI, GTAO, bloom, etc.)
│
└── live_link/               ← Blender Live Link (render profesional)
    ├── blender_live.vert    ─ Vertex shader Live Link
    ├── blender_live.frag    ─ Fragment shader mini-PBR estudio
    ├── shadow.vert          ─ Shadow map (depth-only)
    └── shadow.frag          ─ Shadow map (trivial)
```

Las rutas canónicas de salida (`.spv`) viven en `shaders/` (root, `post/`, `deferred/`,
`compute/`, `ibl/`, `particles/`) y son referenciadas por `include_bytes!` en el runtime.
El `build.rs` resuelve automáticamente la compilación desde los fuentes en `reactor/` y
`live_link/` hacia esas rutas canónicas.

---

## 📚 Cómo usar la lib desde un shader

Cualquier `.vert` / `.frag` / `.comp` de REACTOR puede consumir la librería:

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
preprocesador resuelve los `#include` automáticamente.

---

## 🛠️ Recompilar

`cargo build` recompila los shaders que tengan source más nuevo que el .spv.
Para forzar recompilación:

```bash
Remove-Item shaders/*.spv,shaders/post/*.spv,shaders/deferred/*.spv,shaders/compute/*.spv,shaders/ibl/*.spv,shaders/particles/*.spv -Force
cargo build
```

Para inspeccionar SPIR-V generado:

```bash
spirv-dis shaders/blender_live_frag.spv | less
```
