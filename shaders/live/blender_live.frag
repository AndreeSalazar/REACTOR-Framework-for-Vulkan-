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
    // PCSS — blocker search + penumbra variable.
    // light_size_uv en UV-space (0 = PCF puro; 0.005-0.025 = soft realista).
    float pcss_light_size;
    uint  pcss_samples;          // muestras de blocker search (8/16/24)
    // Debug visualization: 0=off, 1=cascade tint, 2=texel density, 3=penumbra heat
    uint  shadow_debug_mode;
    uint  _padding;
} shadow;

#include "ibl_textures.glsl"
#include "lighting.glsl"
#include "tonemap.glsl"

const vec2 CSM_POISSON_12[12] = vec2[](
    vec2(-0.326, -0.406), vec2(-0.840, -0.074), vec2(-0.696,  0.457),
    vec2(-0.203,  0.621), vec2( 0.962, -0.195), vec2( 0.473, -0.480),
    vec2( 0.519,  0.767), vec2( 0.185, -0.893), vec2( 0.507,  0.064),
    vec2( 0.896,  0.412), vec2(-0.322, -0.933), vec2(-0.792, -0.598)
);

mat2 rotate2d(float a) {
    float s = sin(a);
    float c = cos(a);
    return mat2(c, -s, s, c);
}

float sample_shadow_compare(vec2 uv, int cascade_idx, float current_depth) {
    float shadow_depth = texture(u_shadow_map, vec3(uv, float(cascade_idx))).r;
    return current_depth > shadow_depth ? 0.0 : 1.0;
}

float cascade_edge_fade(vec2 uv) {
    vec2 edge = min(uv, 1.0 - uv);
    float min_edge = min(edge.x, edge.y);
    return smoothstep(0.003, 0.035, min_edge);
}

// ── PCSS: blocker search ──────────────────────────────────────────────────────
// Devuelve la profundidad media de los ocluyentes encontrados en un disco de
// radio `search_radius_uv` alrededor del receptor. Si no hay ocluyentes (toda
// la zona está iluminada) devuelve -1 para indicar "skip PCF, totalmente lit".
float pcss_blocker_search(vec2 uv, int cascade_idx, float z_receiver,
                          float search_radius_uv, int sample_count,
                          mat2 rot, float cascade_scale)
{
    float blocker_sum = 0.0;
    float blocker_cnt = 0.0;
    // Reusamos los 12 taps de Poisson pero permitimos hasta `sample_count`.
    int n = min(sample_count, 12);
    for (int i = 0; i < n; ++i) {
        vec2 offset = rot * CSM_POISSON_12[i] * search_radius_uv * cascade_scale;
        vec2 s_uv = uv + offset;
        if (any(lessThan(s_uv, vec2(0.0))) || any(greaterThan(s_uv, vec2(1.0)))) continue;
        float depth = texture(u_shadow_map, vec3(s_uv, float(cascade_idx))).r;
        if (depth < z_receiver) {
            blocker_sum += depth;
            blocker_cnt += 1.0;
        }
    }
    if (blocker_cnt < 0.5) return -1.0;
    return blocker_sum / blocker_cnt;
}

// Cálculo de factor de sombra con PCSS (blocker search + penumbra variable)
// o PCF Poisson rotado clásico si pcss_light_size == 0.
// Devuelve sombra en .x, cascade index en .y, penumbra width en .z (para debug).
vec3 calculate_shadow_debug(vec3 world_pos, vec3 normal) {
    if (shadow.shadow_enabled == 0) {
        return vec3(1.0, 0.0, 0.0);
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

    if (shadow_uv.z > 1.0 || shadow_uv.z < 0.0 ||
        any(lessThan(shadow_uv.xy, vec2(0.0))) ||
        any(greaterThan(shadow_uv.xy, vec2(1.0)))) {
        return vec3(1.0, float(cascade_idx), 0.0);
    }

    float current_depth = shadow_uv.z - bias;
    vec2 texel_size = 1.0 / vec2(textureSize(u_shadow_map, 0).xy);
    float cascade_scale = mix(1.35, 2.65, float(cascade_idx) / 3.0);
    mat2 rot = rotate2d(hash12(gl_FragCoord.xy + vec2(float(cascade_idx) * 37.0, shadow_uv.z * 8192.0)) * 6.2831853);

    // ── PCSS path: blocker search → penumbra variable → PCF escalado ──────────
    float pcf_radius_uv = max(shadow.pcf_radius, texel_size.x);
    float penumbra_uv = pcf_radius_uv;
    bool pcss_enabled = shadow.pcss_light_size > 0.0001;

    if (pcss_enabled) {
        // 1. Blocker search en un disco de tamaño proporcional al light_size.
        float search_radius = shadow.pcss_light_size * (1.0 + float(cascade_idx) * 0.5);
        int sample_count = int(max(shadow.pcss_samples, 4u));
        float avg_blocker = pcss_blocker_search(
            shadow_uv.xy, cascade_idx, current_depth,
            search_radius, sample_count, rot, cascade_scale
        );

        if (avg_blocker < 0.0) {
            // No hay ocluyentes — totalmente iluminado, salimos pronto.
            return vec3(1.0, float(cascade_idx), 0.0);
        }

        // 2. Penumbra (PCSS classic):
        //    penumbra = (z_receiver - z_blocker) / z_blocker * light_size
        float penumbra_ratio = (current_depth - avg_blocker) / max(avg_blocker, 1e-5);
        penumbra_uv = clamp(penumbra_ratio * shadow.pcss_light_size, texel_size.x, pcf_radius_uv * 8.0);
    }

    // ── PCF final con radio = penumbra ────────────────────────────────────────
    float shadow_factor = 0.0;
    for (int i = 0; i < 12; ++i) {
        vec2 offset = rot * CSM_POISSON_12[i] * penumbra_uv * cascade_scale;
        shadow_factor += sample_shadow_compare(shadow_uv.xy + offset, cascade_idx, current_depth);
    }

    float pcf = shadow_factor / 12.0;
    float final_shadow = mix(1.0, pcf, cascade_edge_fade(shadow_uv.xy));
    return vec3(final_shadow, float(cascade_idx), penumbra_uv);
}

// Wrapper compatible con la API anterior (un solo float).
float calculate_shadow(vec3 world_pos, vec3 normal) {
    return calculate_shadow_debug(world_pos, normal).x;
}

// ── Debug view de cascadas ────────────────────────────────────────────────────
// Modos:
//   0 = off
//   1 = tinte de cascada (rojo/verde/azul/amarillo)
//   2 = texel density (gradient — escala log del tamaño de texel en world)
//   3 = penumbra heat (PCSS — azul=duro, rojo=blando)
vec3 apply_shadow_debug_tint(vec3 color, vec3 shadow_dbg, vec3 world_pos) {
    if (shadow.shadow_debug_mode == 0u) return color;

    int cascade_idx = int(shadow_dbg.y + 0.5);
    if (shadow.shadow_debug_mode == 1u) {
        vec3 tints[4] = vec3[](
            vec3(1.0, 0.35, 0.35),
            vec3(0.35, 1.0, 0.35),
            vec3(0.35, 0.55, 1.0),
            vec3(1.0, 0.95, 0.35)
        );
        vec3 tint = tints[clamp(cascade_idx, 0, 3)];
        return mix(color, color * tint, 0.55);
    }
    if (shadow.shadow_debug_mode == 2u) {
        // Texel density: estima cuántos texels world-space caben en un fragment.
        vec4 proj = shadow.cascade_view_proj[cascade_idx] * vec4(world_pos, 1.0);
        vec2 uv = (proj.xy / proj.w) * 0.5 + 0.5;
        vec2 duvdx = dFdx(uv);
        vec2 duvdy = dFdy(uv);
        float density = max(length(duvdx), length(duvdy)) *
                        float(textureSize(u_shadow_map, 0).x);
        // density < 1 = sobre-sample (verde); >1 = sub-sample / aliasing (rojo).
        float t = clamp(log2(max(density, 1e-3)) * 0.25 + 0.5, 0.0, 1.0);
        vec3 heat = mix(vec3(0.1, 0.95, 0.2), vec3(1.0, 0.15, 0.1), t);
        return mix(color, heat, 0.6);
    }
    if (shadow.shadow_debug_mode == 3u) {
        // Penumbra heat (sólo significativo con PCSS).
        float penumbra = shadow_dbg.z;
        float t = clamp(penumbra * 35.0, 0.0, 1.0);
        vec3 heat = mix(vec3(0.1, 0.25, 1.0), vec3(1.0, 0.3, 0.15), t);
        return mix(color, heat, 0.55);
    }
    return color;
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
    vec4 emission;   // emission.xyz = color, emission.w = strength/translucency
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
// Subsurface Scattering (SSS) / Translucency backlit approximation
vec3 evaluate_sss(vec3 N, vec3 V, vec3 L, vec3 lightColor, vec3 sssColor, float weight) {
    if (weight <= 0.001) return vec3(0.0);
    // Backlit scattering: light coming from behind the surface
    vec3 SSS_L = L + N * 0.42; // wrap normal to allow SSS slightly to the front
    float backlit = max(dot(V, -SSS_L), 0.0);
    // Exponential falloff for realistic translucent dispersion
    float sssFactor = pow(backlit, 8.0) * weight;
    return lightColor * sssColor * sssFactor * 0.45;
}

void main() {
    // ── Inputs ──────────────────────────────────────────────────────────────
    vec3 P      = vWorldPos;
    vec3 vN     = normalize(vWorldNormal);
    vec3 V      = normalize(vViewDir);
    
    // Mapeo de texturas
    vec4 tex_albedo = texture(u_albedo_map, vUV);
    vec3 albedo     = max(tex_albedo.rgb * push.color.rgb, vec3(0.0));
    
    // Reconstrucción del TBN
    mat3 TBN = calculate_TBN(P, vN, vUV);
    
    // Normal mapping
    vec3 normalMap = texture(u_normal_map, vUV).rgb;
    vec3 N = vN;
    if (length(normalMap - vec3(0.5, 0.5, 1.0)) > 0.01) {
        vec3 tangentNormal = normalMap * 2.0 - 1.0;
        N = normalize(TBN * tangentNormal);
    }

    // Re-ortogonalizar base tangente contra la normal modificada
    vec3 T = normalize(TBN[0] - N * dot(TBN[0], N));
    vec3 B = cross(N, T);

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

    // Parámetros de anisotropía de push.color.a
    float anisotropy = clamp(push.color.a, -1.0, 1.0);
    bool use_aniso = abs(anisotropy) > 0.01;

    vec3 lo = vec3(0.0);
    
    // Configuración de luces analíticas
    vec3 keyDir  = normalize(vec3(-0.45, 0.85, 0.40));
    vec3 fillDir = normalize(vec3( 0.65, 0.45,-0.30));
    vec3 backDir = normalize(vec3( 0.10, 0.65,-0.95));
    vec3 keyRad  = vec3(3.2, 3.0, 2.7) * shadow_factor;
    vec3 fillRad = vec3(0.55, 0.65, 0.85);
    vec3 backRad = vec3(1.6, 1.55, 1.50);

    if (use_aniso) {
        lo += light_eval_directional_anisotropic(N, V, keyDir, T, B, keyRad, albedo, metallic, roughness, anisotropy, f0);
        lo += light_eval_directional_anisotropic(N, V, fillDir, T, B, fillRad, albedo, metallic, roughness, anisotropy, f0);
        lo += light_eval_directional_anisotropic(N, V, backDir, T, B, backRad, albedo, metallic, roughness, anisotropy, f0);
        
        // Point light dinámica
        lo += light_eval_point_anisotropic(
            N, V, P, T, B,
            push.light_pos.xyz,
            vec3(1.0, 1.0, 1.0),
            12.0 * shadow_factor,
            25.0,
            albedo, metallic, roughness, anisotropy, f0
        );

        // Sun light dinámica
        if (shadow.shadow_enabled != 0) {
            vec3 L_sun = -normalize(shadow.light_direction.xyz);
            vec3 sun_radiance = vec3(2.5, 2.4, 2.2) * shadow_factor;
            lo += light_eval_directional_anisotropic(N, V, L_sun, T, B, sun_radiance, albedo, metallic, roughness, anisotropy, f0);
        }
    } else {
        lo += light_eval_directional(N, V, keyDir,  keyRad,  albedo, metallic, roughness, f0);
        lo += light_eval_directional(N, V, fillDir, fillRad, albedo, metallic, roughness, f0);
        lo += light_eval_directional(N, V, backDir, backRad, albedo, metallic, roughness, f0);
        
        lo += light_eval_point(
            N, V, P,
            push.light_pos.xyz,
            vec3(1.0, 1.0, 1.0),
            12.0 * shadow_factor,
            25.0,
            albedo, metallic, roughness, f0
        );

        if (shadow.shadow_enabled != 0) {
            vec3 L_sun = -normalize(shadow.light_direction.xyz);
            vec3 sun_radiance = vec3(2.5, 2.4, 2.2) * shadow_factor;
            lo += light_eval_directional(N, V, L_sun, sun_radiance, albedo, metallic, roughness, f0);
        }
    }

    // ── IBL (diffuse + specular split-sum) ──────────────────────────────────
    vec3 ambient;
    if (use_aniso) {
        ambient = ibl_eval_textured_anisotropic(N, V, T, B, albedo, metallic, roughness, anisotropy, f0, ao, ibl_params.max_mip);
    } else {
        ambient = ibl_eval_textured(N, V, albedo, metallic, roughness, f0, ao, ibl_params.max_mip);
    }

    // ── Rim light ──────────────────────────────────────────────────────────
    vec3 rim = light_rim(N, V, vec3(0.65, 0.78, 1.0) * 0.5, 4.0, metallic);

    // ── Composición y Subsurface Scattering Avanzado (SSS) ──────────────────
    float emission_strength = 0.0;
    float sss_weight = 0.0;
    if (length(push.emission.rgb) > 0.01) {
        emission_strength = push.emission.w;
    } else {
        sss_weight = push.emission.w;
    }

    vec3 sss_accum = vec3(0.0);
    if (sss_weight > 0.001) {
        vec3 sss_color = albedo * vec3(1.0, 0.35, 0.22);
        float thickness = clamp(1.0 - sss_weight, 0.01, 1.0);

        // Key light
        sss_accum += evaluate_sss(N, V, keyDir, keyRad, sss_color, sss_weight);
        sss_accum += evaluate_transmittance(N, V, keyDir, thickness, sss_color, keyRad);

        // Sun light
        if (shadow.shadow_enabled != 0) {
            vec3 L_sun = -normalize(shadow.light_direction.xyz);
            vec3 sun_radiance = vec3(2.5, 2.4, 2.2) * shadow_factor;
            sss_accum += evaluate_sss(N, V, L_sun, sun_radiance, sss_color, sss_weight);
            sss_accum += evaluate_transmittance(N, V, L_sun, thickness, sss_color, sun_radiance);
        }

        // Point light
        vec3 pointL = normalize(push.light_pos.xyz - P);
        float dist = length(push.light_pos.xyz - P);
        float atten = 1.0 / (dist * dist + 1.0);
        vec3 point_radiance = vec3(1.0, 1.0, 1.0) * 12.0 * shadow_factor * atten;
        sss_accum += evaluate_sss(N, V, pointL, point_radiance, sss_color, sss_weight);
        sss_accum += evaluate_transmittance(N, V, pointL, thickness, sss_color, point_radiance);
    }

    // Salida LINEAR HDR
    vec3 emissive = push.emission.rgb * emission_strength;
    vec3 color = (lo + ambient + rim) * cs + sss_accum + emissive;

    outColor = vec4(color, roughness);
}
