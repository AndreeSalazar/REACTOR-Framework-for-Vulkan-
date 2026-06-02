// =============================================================================
// REACTOR · shaders/lib/pbr.glsl — Cook-Torrance microfacet BRDF
// =============================================================================
// Implementación física, energía-conservante, lista para producción:
//   • D: Trowbridge-Reitz GGX (Disney parametrization, α = roughness²)
//   • V: Smith height-correlated (Heitz 2014) — ya incluye 1/(4·NoL·NoV)
//   • F: Schlick estándar + variante "with roughness" para IBL (Lagarde)
//   • Diffuse: Burley/Disney con kS/kD energy-conserving
//   • Helpers para horizonte oclusivo y AO especular (Lagarde 2014)
// =============================================================================
#ifndef REACTOR_LIB_PBR
#define REACTOR_LIB_PBR

#include "color.glsl"

const float REACTOR_PI       = 3.14159265359;
const float REACTOR_INV_PI   = 0.31830988618;
const float REACTOR_EPS      = 1e-4;
const vec3  REACTOR_F0_DIEL  = vec3(0.04);     // dielectrico genérico (4%)

// ── D · Trowbridge-Reitz GGX ─────────────────────────────────────────────────
float D_GGX(float NoH, float alpha) {
    float a2 = alpha * alpha;
    float f  = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / max(REACTOR_PI * f * f, REACTOR_EPS);
}

// ── V · Smith height-correlated (incluye 1/(4·NoL·NoV)) ──────────────────────
float V_SmithGGXCorrelated(float NoV, float NoL, float alpha) {
    float a2 = alpha * alpha;
    float gv = NoL * sqrt(NoV * NoV * (1.0 - a2) + a2);
    float gl = NoV * sqrt(NoL * NoL * (1.0 - a2) + a2);
    return 0.5 / max(gv + gl, REACTOR_EPS);
}

// ── F · Schlick ──────────────────────────────────────────────────────────────
vec3 F_Schlick(float VoH, vec3 f0) {
    float f = pow(1.0 - VoH, 5.0);
    return f0 + (1.0 - f0) * f;
}

float F_SchlickScalar(float u, float f0, float f90) {
    return f0 + (f90 - f0) * pow(1.0 - u, 5.0);
}

// Schlick con cap por roughness — preferido para IBL (Lagarde 2014).
vec3 F_SchlickRoughness(float NoV, vec3 f0, float roughness) {
    return f0 + (max(vec3(1.0 - roughness), f0) - f0) * pow(1.0 - NoV, 5.0);
}

// ── Diffuse terms ────────────────────────────────────────────────────────────
vec3 Fd_Lambert(vec3 albedo) {
    return albedo * REACTOR_INV_PI;
}

// Disney/Burley diffuse. Mantiene la respuesta de materiales rugosos mas
// natural que Lambert puro y conserva mejor la energia frente al lobulo GGX.
vec3 Fd_Burley(vec3 albedo, float roughness, float NoV, float NoL, float LoH) {
    float energyBias   = mix(0.0, 0.5, roughness);
    float energyFactor = mix(1.0, 1.0 / 1.51, roughness);
    float f90          = energyBias + 2.0 * roughness * LoH * LoH;
    float lightScatter = F_SchlickScalar(NoL, 1.0, f90);
    float viewScatter  = F_SchlickScalar(NoV, 1.0, f90);
    return albedo * REACTOR_INV_PI * lightScatter * viewScatter * energyFactor;
}

// ── Sample BRDF para una luz analítica ───────────────────────────────────────
// Devuelve la radiancia reflejada en la dirección V para una luz cuya
// dirección incidente es L y cuya radiancia (W/sr/m²) es 'radiance'.
struct BrdfSample {
    vec3 diffuse;
    vec3 specular;
};

BrdfSample brdf_eval(vec3 N, vec3 V, vec3 L,
                     vec3 albedo, float metallic, float roughness, vec3 f0) {
    vec3  H   = normalize(V + L);
    float NoV = max(dot(N, V), 0.0);
    float NoL = max(dot(N, L), 0.0);
    float NoH = max(dot(N, H), 0.0);
    float VoH = max(dot(V, H), 0.0);
    float LoH = max(dot(L, H), 0.0);

    roughness = clamp(roughness, 0.04, 1.0);
    float alpha = roughness * roughness;
    float D  = D_GGX(NoH, alpha);
    float Vi = V_SmithGGXCorrelated(NoV, NoL, alpha);
    vec3  F  = F_Schlick(VoH, f0);

    vec3 spec = D * Vi * F;                        // ya incluye 1/(4·NoL·NoV)
    vec3 kd   = (1.0 - F) * (1.0 - metallic);
    vec3 diff = kd * Fd_Burley(albedo, roughness, NoV, NoL, LoH);

    BrdfSample s;
    s.diffuse  = diff * NoL;
    s.specular = spec * NoL;
    return s;
}

// Versión que devuelve la suma — útil para luces sin attenuación adicional.
vec3 brdf_shade(vec3 N, vec3 V, vec3 L, vec3 radiance,
                vec3 albedo, float metallic, float roughness, vec3 f0) {
    BrdfSample s = brdf_eval(N, V, L, albedo, metallic, roughness, f0);
    return (s.diffuse + s.specular) * radiance;
}

// ── Specular AO (Lagarde 2014, "Moving Frostbite to PBR") ───────────────────
// Atenúa la contribución especular cuando el AO indica oclusión geométrica.
float specular_AO(float NoV, float ao, float roughness) {
    return saturate(pow(NoV + ao, exp2(-16.0 * roughness - 1.0)) - 1.0 + ao);
}

// ── Horizon occlusion — evita lóbulo GGX bajo el horizonte (Heitz 2018) ─────
float horizon_occlusion(vec3 R, vec3 Ng) {
    float horizon = min(1.0 + dot(R, Ng), 1.0);
    return horizon * horizon;
}

// ── Compensación multi-scattering (Kulla-Conty / Turquin) ───────────────────
// Recupera la energía que pierde el modelo single-scattering GGX, sobre todo
// en rugosidades altas. Sólo requiere envBRDF de IBL como tabla.
vec3 energy_compensation(vec3 f0, vec2 envBRDF) {
    return 1.0 + f0 * (1.0 / envBRDF.y - 1.0);
}

#endif
