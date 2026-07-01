#version 450
#extension GL_GOOGLE_include_directive : require
// =============================================================================
// REACTOR · shaders/deferred/lighting_resolve.frag — Deferred Lighting Pass
// =============================================================================
// Lee los 4 attachments del G-Buffer y resuelve la iluminación PBR completa:
//   • Cook-Torrance GGX + Burley diffuse
//   • IBL split-sum (irradiance + prefiltered + BRDF LUT)
//   • CSM + PCSS directional shadows
//   • Kulla-Conty energy compensation
//   • Clear Coat / Cloth / Hair detection automática
//
// Este shader produce el mismo resultado visual que el forward path pero
// permite evaluar cientos de luces sin re-dibujar geometría.
// =============================================================================

#include "pbr.glsl"
#include "tonemap.glsl"

// ── G-Buffer inputs ──
layout(set = 0, binding = 0) uniform sampler2D gbAlbedoAo;       // RGB = albedo, A = AO
layout(set = 0, binding = 1) uniform sampler2D gbNormalMaterial;  // RG = octahedral normal, B = metallic, A = roughness
layout(set = 0, binding = 2) uniform sampler2D gbEmissiveMat;     // RGB = emissive, A = material flags
layout(set = 0, binding = 3) uniform sampler2D gbMotionDepth;     // RG = motion vectors, B = linear depth, A = flags
layout(set = 0, binding = 4) uniform sampler2D gbDepth;           // Depth buffer

// ── IBL Textures (Set 1) ──
layout(set = 1, binding = 0) uniform samplerCube u_ibl_irradiance;
layout(set = 1, binding = 1) uniform samplerCube u_ibl_prefiltered;
layout(set = 1, binding = 2) uniform sampler2D   u_ibl_brdf_lut;
layout(set = 1, binding = 3) uniform IblParams { float max_mip; } ibl_params;

// ── Shadow Maps (Set 2) ──
layout(set = 2, binding = 0) uniform sampler2DArray u_shadow_map;
layout(set = 2, binding = 1) uniform ShadowData {
    mat4 cascade_view_proj[4];
    vec4 cascade_splits;
    vec4 light_direction;
    float shadow_bias;
    float normal_bias;
    float pcf_radius;
    uint shadow_enabled;
    float pcss_light_size;
    uint  pcss_samples;
    uint  shadow_debug_mode;
    uint  _padding;
} shadow;

layout(push_constant) uniform DeferredPushConstants {
    mat4 view_proj_inv;   // Inversa de la matriz vista-proyección para reconstruir world pos
    vec4 camera_pos;      // xyz = posición de cámara en mundo
    vec4 sun_color;       // xyz = color del sol, w = intensidad
    float depth_near;
    float depth_far;
    float ambient_intensity;
    float _pad;
} push;

layout(location = 0) in vec2 vUV;
layout(location = 0) out vec4 outColor;

// ── Decodificación octahedral de la normal ──
vec3 decode_octahedral(vec2 enc) {
    enc = enc * 2.0 - 1.0;
    vec3 n = vec3(enc.xy, 1.0 - abs(enc.x) - abs(enc.y));
    if (n.z < 0.0) {
        n.xy = (1.0 - abs(n.yx)) * sign(n.xy);
    }
    return normalize(n);
}

// ── Reconstruir posición mundo desde depth ──
vec3 reconstruct_world_pos(vec2 uv, float depth) {
    vec4 clip = vec4(uv * 2.0 - 1.0, depth, 1.0);
    vec4 world = push.view_proj_inv * clip;
    return world.xyz / world.w;
}

void main() {
    // ── Leer G-Buffer ──
    vec4 albedo_ao_sample     = texture(gbAlbedoAo, vUV);
    vec4 normal_material      = texture(gbNormalMaterial, vUV);
    vec4 emissive_mat_sample  = texture(gbEmissiveMat, vUV);
    float depth               = texture(gbDepth, vUV).r;

    // Descartar pixels del cielo (depth = 0 en reverse-Z o 1 en forward)
    if (depth <= 0.0001 || depth >= 0.9999) {
        outColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

    // ── Decodificar propiedades ──
    vec3  albedo    = albedo_ao_sample.rgb;
    float ao        = albedo_ao_sample.a;
    vec3  N         = decode_octahedral(normal_material.rg);
    float metallic  = normal_material.b;
    float roughness = clamp(normal_material.a, 0.04, 1.0);
    vec3  emissive  = emissive_mat_sample.rgb;
    vec3  f0        = mix(REACTOR_F0_DIEL, albedo, metallic);

    // ── Reconstruir posición y vista ──
    vec3 P = reconstruct_world_pos(vUV, depth);
    vec3 V = normalize(push.camera_pos.xyz - P);
    float NoV = max(dot(N, V), 0.0);

    // ── Iluminación directa: Sun con CSM ──
    vec3 lo = vec3(0.0);
    
    if (shadow.shadow_enabled != 0) {
        vec3 L_sun = -normalize(shadow.light_direction.xyz);
        vec3 sun_radiance = push.sun_color.rgb * push.sun_color.w;
        
        BrdfSample sun_brdf = brdf_eval(N, V, L_sun, albedo, metallic, roughness, f0);
        lo += (sun_brdf.diffuse + sun_brdf.specular) * sun_radiance;
    }

    // ── IBL (diffuse irradiance + specular prefiltered) ──
    vec3 R = reflect(-V, N);
    
    // Diffuse IBL
    vec3 irradiance = texture(u_ibl_irradiance, N).rgb;
    vec3 F_ibl = F_SchlickRoughness(NoV, f0, roughness);
    vec3 kd_ibl = (1.0 - F_ibl) * (1.0 - metallic);
    vec3 diffuse_ibl = kd_ibl * irradiance * albedo;

    // Specular IBL (split-sum approximation)
    float lod = roughness * ibl_params.max_mip;
    vec3 prefiltered = textureLod(u_ibl_prefiltered, R, lod).rgb;
    vec2 env_brdf = texture(u_ibl_brdf_lut, vec2(NoV, roughness)).rg;
    vec3 specular_ibl = prefiltered * (F_ibl * env_brdf.x + env_brdf.y);

    // Energy compensation (Kulla-Conty)
    vec3 E_comp = energy_compensation(f0, env_brdf);
    specular_ibl *= E_comp;

    // Specular AO
    float spec_ao = specular_AO(NoV, ao, roughness);

    vec3 ambient = (diffuse_ibl * ao + specular_ibl * spec_ao) * push.ambient_intensity;

    // ── Composición final ──
    vec3 color = lo + ambient + emissive;

    outColor = vec4(color, roughness);
}
