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
#include "ibl.glsl"
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
    vec4 camera_pos;
    vec4 light_pos;
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

void main() {
    // ── Inputs ──────────────────────────────────────────────────────────────
    vec3 N      = normalize(vWorldNormal);
    vec3 V      = normalize(vViewDir);
    vec3 P      = vWorldPos;
    vec3 albedo = max(vColor.rgb, vec3(0.0));

    // ── Material heurístico (placeholder hasta MaterialUpdated PBR real) ───
    float y     = luminance(albedo);
    float sat   = length(albedo - vec3(y));
    float metallic   = 0.0;
    float baseRough  = mix(0.55, 0.30, saturate(y) * (1.0 - saturate(sat * 1.2)));
    float n          = value_noise(P * 3.5);
    float roughness  = clamp(baseRough + (n - 0.5) * 0.12, 0.04, 1.0);
    vec3  f0         = mix(REACTOR_F0_DIEL, albedo, metallic);

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

    // ── IBL (diffuse + specular split-sum + energy compensation) ───────────
    vec3 ambient = ibl_eval(N, V, albedo, metallic, roughness, f0, ao);

    // ── Rim light ──────────────────────────────────────────────────────────
    vec3 rim = light_rim(N, V, vec3(0.65, 0.78, 1.0) * 0.5, 4.0, metallic);

    // ── Composición y grading ──────────────────────────────────────────────
    vec3 color = (lo + ambient + rim) * cs;
    color = adjust_saturation(color, 1.05);

    // AgX da altas luces más limpias que ACES en preview de materiales.
    color = agx_default(color);

    // AgX ya devuelve sRGB; sólo aplicar dithering anti-banding.
    color += bayer_dither(gl_FragCoord.xy);

    outColor = vec4(color, vColor.a);
}
