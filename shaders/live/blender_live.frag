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
layout(set = 0, binding = 2) uniform sampler2D u_metallic_map;
layout(set = 0, binding = 3) uniform sampler2D u_roughness_map;

// Set 1: Texturas IBL (Image-Based Lighting)
#define REACTOR_IBL_SET 1
layout(set = REACTOR_IBL_SET, binding = 0) uniform samplerCube u_ibl_irradiance;
layout(set = REACTOR_IBL_SET, binding = 1) uniform samplerCube u_ibl_prefiltered;
layout(set = REACTOR_IBL_SET, binding = 2) uniform sampler2D   u_ibl_brdf_lut;
layout(set = REACTOR_IBL_SET, binding = 3) uniform IblParams { float max_mip; } ibl_params;

// Set 2: Cascaded Shadow Maps
layout(set = 2, binding = 0) uniform sampler2DArray u_shadow_map;
layout(set = 2, binding = 1) uniform ShadowData {
    mat4 cascade_view_proj[4];
    vec4 cascade_splits;
    vec4 light_direction;
    float shadow_bias;
    float normal_bias;
    float pcf_radius;
    uint shadow_enabled;
} shadow;

#include "ibl_textures.glsl"
#include "lighting.glsl"
#include "tonemap.glsl"

// Cálculo de factor de sombra con PCF y Cascaded Shadow Maps (CSM)
float calculate_shadow(vec3 world_pos, vec3 normal) {
    if (shadow.shadow_enabled == 0) {
        return 1.0;
    }

    int cascade_idx = 3;
    vec4 shadow_coord = vec4(0.0);
    
    for (int i = 0; i < 4; i++) {
        vec4 proj_coord = shadow.cascade_view_proj[i] * vec4(world_pos, 1.0);
        vec3 ndc = proj_coord.xyz / proj_coord.w;
        vec3 uv = ndc * 0.5 + 0.5;
        
        if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0 && ndc.z >= 0.0 && ndc.z <= 1.0) {
            cascade_idx = i;
            shadow_coord = vec4(uv.xy, float(cascade_idx), ndc.z);
            break;
        }
    }
    
    if (cascade_idx == 3 && (shadow_coord.x < 0.0 || shadow_coord.x > 1.0 || shadow_coord.y < 0.0 || shadow_coord.y > 1.0)) {
        vec4 proj_coord = shadow.cascade_view_proj[3] * vec4(world_pos, 1.0);
        vec3 ndc = proj_coord.xyz / proj_coord.w;
        vec3 uv = ndc * 0.5 + 0.5;
        shadow_coord = vec4(uv.xy, 3.0, ndc.z);
    }

    float cos_theta = clamp(dot(normal, shadow.light_direction.xyz), 0.0, 1.0);
    float bias = max(shadow.shadow_bias * (1.0 - cos_theta), shadow.shadow_bias * 0.1);
    bias *= (1.0 + float(cascade_idx) * 0.5);

    vec3 biased_pos = world_pos + normal * shadow.normal_bias * (1.0 + float(cascade_idx) * 0.5);
    vec4 proj_coord = shadow.cascade_view_proj[cascade_idx] * vec4(biased_pos, 1.0);
    vec3 ndc = proj_coord.xyz / proj_coord.w;
    vec3 shadow_uv = ndc * 0.5 + 0.5;

    float current_depth = shadow_uv.z - bias;
    
    if (shadow_uv.z > 1.0) {
        return 1.0;
    }

    float shadow_factor = 0.0;
    vec2 texel_size = vec2(shadow.pcf_radius);
    
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            vec2 offset = vec2(x, y) * texel_size;
            float shadow_depth = texture(u_shadow_map, vec3(shadow_uv.xy + offset, float(cascade_idx))).r;
            shadow_factor += current_depth > shadow_depth ? 0.0 : 1.0;
        }
    }
    
    return shadow_factor / 9.0;
}

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
    float tex_metallic  = texture(u_metallic_map, vUV).r;
    float tex_roughness = texture(u_roughness_map, vUV).r;
    float metallic  = clamp(push.camera_pos.w * tex_metallic, 0.0, 1.0);
    float roughness = clamp(push.light_pos.w * tex_roughness, 0.04, 1.0);
    vec3  f0        = mix(REACTOR_F0_DIEL, albedo, metallic);

    // ── Oclusión / contact shadow ──────────────────────────────────────────
    float ao = curvature_AO(N);
    float cs = contact_shadow(P, N);

    // ── Lighting analítico: estudio 3-point + luz dinámica de Blender ──────
    float shadow_factor = 1.0;
    if (shadow.shadow_enabled != 0) {
        shadow_factor = calculate_shadow(P, N);
    }

    vec3 lo = vec3(0.0);
    
    // Evaluate 3-point studio lights, applying shadow to the main key light
    vec3 keyDir  = normalize(vec3(-0.45, 0.85, 0.40));
    vec3 fillDir = normalize(vec3( 0.65, 0.45,-0.30));
    vec3 backDir = normalize(vec3( 0.10, 0.65,-0.95));
    vec3 keyRad  = vec3(3.2, 3.0, 2.7) * shadow_factor;
    vec3 fillRad = vec3(0.55, 0.65, 0.85);
    vec3 backRad = vec3(1.6, 1.55, 1.50);
    lo += light_eval_directional(N, V, keyDir,  keyRad,  albedo, metallic, roughness, f0);
    lo += light_eval_directional(N, V, fillDir, fillRad, albedo, metallic, roughness, f0);
    lo += light_eval_directional(N, V, backDir, backRad, albedo, metallic, roughness, f0);

    // Dynamic point light (shadowed)
    lo += light_eval_point(
        N, V, P,
        push.light_pos.xyz,
        /* color     */ vec3(1.0, 1.0, 1.0),
        /* intensity */ 12.0 * shadow_factor,
        /* range     */ 25.0,
        albedo, metallic, roughness, f0
    );

    // Dynamic Sun directional light
    if (shadow.shadow_enabled != 0) {
        vec3 L_sun = -normalize(shadow.light_direction.xyz);
        vec3 sun_radiance = vec3(2.5, 2.4, 2.2) * shadow_factor;
        lo += light_eval_directional(N, V, L_sun, sun_radiance, albedo, metallic, roughness, f0);
    }

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
