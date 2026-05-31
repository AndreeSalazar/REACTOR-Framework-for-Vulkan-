// =============================================================================
// REACTOR · shaders/lib/ibl_textures.glsl — IBL desde texturas pre-cocinadas
// =============================================================================
// Drop-in replacement de `ibl.glsl` (procedural) cuando el motor ha cocinado
// un cubemap HDR real con `IblBaker`. Espera que el shader declare:
//
//   layout(set = N, binding = 0) uniform samplerCube  u_ibl_irradiance;
//   layout(set = N, binding = 1) uniform samplerCube  u_ibl_prefiltered;
//   layout(set = N, binding = 2) uniform sampler2D    u_ibl_brdf_lut;
//   layout(set = N, binding = 3) uniform IblParams { float max_mip; } ibl_params;
//
// `max_mip` = nivel máximo del prefiltered cubemap (típicamente 4 para 5 mips).
// =============================================================================
#ifndef REACTOR_LIB_IBL_TEXTURES
#define REACTOR_LIB_IBL_TEXTURES

#include "color.glsl"
#include "pbr.glsl"

// Las declaraciones reales (uniforms / set bindings) las hace el shader que
// nos incluye — aquí solo declaramos forward references documentadas.
// El shader DEBE definir REACTOR_IBL_SET = N antes de incluir esta cabecera.
#ifndef REACTOR_IBL_SET
#define REACTOR_IBL_SET 1
#endif

// ── Sampling helpers ─────────────────────────────────────────────────────────
vec3 ibl_sample_irradiance(vec3 N) {
    return texture(u_ibl_irradiance, N).rgb;
}

vec3 ibl_sample_specular(vec3 R, float roughness, float max_mip) {
    return textureLod(u_ibl_prefiltered, R, roughness * max_mip).rgb;
}

vec2 ibl_sample_brdf(float NoV, float roughness) {
    return texture(u_ibl_brdf_lut, vec2(NoV, roughness)).rg;
}

// ── Evaluación IBL completa (idéntica firma a ibl.glsl::ibl_eval) ────────────
// Diffuse + Specular split-sum + spec AO (Lagarde) + energy compensation
// (Kulla-Conty) — todo con las 3 texturas pre-baked.
vec3 ibl_eval_textured(vec3 N, vec3 V,
                       vec3 albedo, float metallic, float roughness, vec3 f0,
                       float ao, float max_mip) {
    float NoV     = max(dot(N, V), 0.0);
    vec3  R       = reflect(-V, N);

    vec3  irr     = ibl_sample_irradiance(N);
    vec3  preSpec = ibl_sample_specular(R, roughness, max_mip);
    vec2  brdf    = ibl_sample_brdf(NoV, roughness);

    vec3  Fr_ibl  = F_SchlickRoughness(NoV, f0, roughness);
    vec3  kd_ibl  = (1.0 - Fr_ibl) * (1.0 - metallic);

    vec3 spec = preSpec * (Fr_ibl * brdf.x + brdf.y);
    spec *= energy_compensation(f0, brdf);
    spec *= specular_AO(NoV, ao, roughness);

    vec3 diff = kd_ibl * albedo * irr * ao;
    return diff + spec;
}

#endif
