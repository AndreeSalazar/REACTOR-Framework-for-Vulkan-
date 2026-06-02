// =============================================================================
// REACTOR ⇄ Blender Live Link — Fragment shader (mini-PBR AAA)
// =============================================================================
// Pipeline equivalente a Eevee "Material Preview" / UE5 preview:
//
//   1. Material heurístico desde `push.color` (albedo) — placeholder hasta que
//      llegue `MaterialUpdated` real con metallic/roughness/normal.
//   2. Tres luces de estudio (key + fill + back/rim) vía `light_studio_3point`.
//   3. Luz dinámica de Blender (push.light_pos) con falloff físico Frostbite.
//   4. IBL hemisférico procedural con disco solar + halo (envuelve la escena).
//   5. AO geométrico por curvatura + contact shadow heurístico bajo Y≈0.
//   6. Rim Fresnel para silueta.
//   7. Specular AO + energy compensation Kulla-Conty (corrige multi-scattering).
//   8. Tone mapping AgX (más neutro que ACES en altas luces) + gamma sRGB.
//
// Todo el código BRDF/IBL/luces vive en `shaders/lib/*.glsl` y se reutiliza
// desde aquí — el shader concreto se mantiene en ~120 líneas legibles.
// =============================================================================
#version 450
#extension GL_GOOGLE_include_directive : require

#include "color.glsl"
#include "noise.glsl"
#include "pbr.glsl"

// Set 0: Texturas del material
layout(set = 0, binding = 0) uniform sampler2D u_albedo_map;
layout(set = 0, binding = 1) uniform sampler2D u_normal_map;

// Set 1: Texturas IBL (Image-Based Lighting)
#define REACTOR_IBL_SET 1
layout(set = REACTOR_IBL_SET, binding = 0) uniform samplerCube u_ibl_irradiance;
layout(set = REACTOR_IBL_SET, binding = 1) uniform samplerCube u_ibl_prefiltered;
layout(set = REACTOR_IBL_SET, binding = 2) uniform sampler2D   u_ibl_brdf_lut;
layout(set = REACTOR_IBL_SET, binding = 3) uniform IblParams { float max_mip; } ibl_params;

#include "ibl_textures.glsl"
#include "lighting.glsl"
#include "tonemap.glsl"

layout(location = 0) in vec3 vWorldNormal;
layout(location = 1) in vec2 vUV;
layout(location = 2) in vec3 vWorldPos;
layout(location = 3) in vec4 vColor;
layout(location = 4) in vec3 vViewDir;

layout(location = 0) out vec4 outColor;

layout(push_constant) uniform Constants {
    mat4 mvp;
    mat4 model;
    vec4 camera_pos; // camera_pos.w contains packed metallic
    vec4 light_pos;  // light_pos.w contains packed roughness
    vec4 color;
} push;

// AO geométrico por curvatura — convexidades claras, cavidades oscuras.
float curvature_AO(vec3 N) {
    vec3  dx = dFdx(N);
    vec3  dy = dFdy(N);
    float k  = length(dx) + length(dy);
    return saturate(1.0 - k * 1.2);
}

// Sombra de contacto procedural — sólo se nota cerca del "suelo virtual" Y=0.
float contact_shadow(vec3 P, vec3 N) {
    float h    = P.y;
    float NoUp = saturate(dot(N, vec3(0.0, 1.0, 0.0)));
    return mix(1.0, 0.55, saturate(1.0 - h * 1.5) * NoUp);
}

// Reconstrucción del TBN usando derivadas en espacio de pantalla
mat3 calculate_TBN(vec3 p, vec3 n, vec2 uv) {
    vec3 dp1 = dFdx(p);
    vec3 dp2 = dFdy(p);
    vec2 duv1 = dFdx(uv);
    vec2 duv2 = dFdy(uv);
    
    vec3 dp2perp = cross(dp2, n);
    vec3 dp1perp = cross(n, dp1);
    vec3 t = dp2perp * duv1.x + dp1perp * duv2.x;
    vec3 b = dp2perp * duv1.y + dp1perp * duv2.y;
    
    float invmax = inversesqrt(max(dot(t, t), dot(b, b)));
    return mat3(t * invmax, b * invmax, n);
}

void main() {
    // ── Inputs ──────────────────────────────────────────────────────────────
    vec3 P      = vWorldPos;
    vec3 vN     = normalize(vWorldNormal);
    vec3 V      = normalize(vViewDir);
    
    // Mapeo de texturas
    vec4 tex_albedo = texture(u_albedo_map, vUV);
    vec3 albedo     = max(tex_albedo.rgb * push.color.rgb, vec3(0.0));
    
    // Normal mapping
    vec3 normalMap = texture(u_normal_map, vUV).rgb;
    vec3 N = vN;
    // Si la textura de normales contiene datos (es decir, no es el flat color fallback por defecto)
    if (length(normalMap - vec3(0.5, 0.5, 1.0)) > 0.01) {
        mat3 TBN = calculate_TBN(P, vN, vUV);
        vec3 tangentNormal = normalMap * 2.0 - 1.0;
        N = normalize(TBN * tangentNormal);
    }

    // ── Propiedades PBR reales ──────────────────────────────────────────────
    float metallic  = clamp(push.camera_pos.w, 0.0, 1.0);
    float roughness = clamp(push.light_pos.w, 0.04, 1.0);
    vec3  f0        = mix(REACTOR_F0_DIEL, albedo, metallic);

    // ── Oclusión / contact shadow ──────────────────────────────────────────
    float ao = curvature_AO(N);
    float cs = contact_shadow(P, N);

    // ── Lighting analítico: estudio 3-point + luz dinámica de Blender ──────
    vec3 lo = light_studio_3point(N, V, albedo, metallic, roughness, f0);
    lo += light_eval_point(
        N, V, P,
        push.light_pos.xyz,
        /* color     */ vec3(1.0, 1.0, 1.0),
        /* intensity */ 12.0,
        /* range     */ 25.0,
        albedo, metallic, roughness, f0
    );

    // ── IBL (diffuse + specular split-sum from pre-baked HDR cubemap) ────────
    vec3 ambient = ibl_eval_textured(N, V, albedo, metallic, roughness, f0, ao, ibl_params.max_mip);

    // ── Rim light ──────────────────────────────────────────────────────────
    vec3 rim = light_rim(N, V, vec3(0.65, 0.78, 1.0) * 0.5, 4.0, metallic);

    // ── Composición ─────────────────────────────────────────────────────────
    // Salida LINEAR HDR — sin tone mapping aquí.
    // El post-process pipeline aplica bloom (sobre HDR) → AgX → gamma.
    vec3 color = (lo + ambient + rim) * cs;

    outColor = vec4(color, tex_albedo.a * push.color.a);
}
