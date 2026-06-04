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

// ── Anisotropic GGX (Disney / RenderMan / Cyberpunk 2077 Style) ──────────────

float Lambda_GGX_Anisotropic(float NoV, float ToV, float BoV, float ax, float ay) {
    float v_x = ax * ToV;
    float v_y = ay * BoV;
    return sqrt(v_x * v_x + v_y * v_y + NoV * NoV);
}

float V_SmithGGXCorrelated_Anisotropic(float NoV, float NoL, float ToV, float ToL, float BoV, float BoL, float ax, float ay) {
    float lambdaV = NoL * Lambda_GGX_Anisotropic(NoV, ToV, BoV, ax, ay);
    float lambdaL = NoV * Lambda_GGX_Anisotropic(NoL, ToL, BoL, ax, ay);
    return 0.5 / max(lambdaV + lambdaL, REACTOR_EPS);
}

float D_GGX_Anisotropic(float NoH, float ToH, float BoH, float ax, float ay) {
    float d = ToH * ToH / (ax * ax) + BoH * BoH / (ay * ay) + NoH * NoH;
    return 1.0 / max(REACTOR_PI * ax * ay * d * d, REACTOR_EPS);
}

BrdfSample brdf_eval_anisotropic(vec3 N, vec3 V, vec3 L, vec3 T, vec3 B,
                                 vec3 albedo, float metallic, float roughness, float anisotropy, vec3 f0) {
    vec3  H   = normalize(V + L);
    float NoV = max(dot(N, V), 0.0);
    float NoL = max(dot(N, L), 0.0);
    float NoH = max(dot(N, H), 0.0);
    float VoH = max(dot(V, H), 0.0);
    float LoH = max(dot(L, H), 0.0);

    float ToV = dot(T, V);
    float ToL = dot(T, L);
    float ToH = dot(T, H);
    float BoV = dot(B, V);
    float BoL = dot(B, L);
    float BoH = dot(B, H);

    roughness = clamp(roughness, 0.04, 1.0);
    float alpha = roughness * roughness;
    
    // Anisotropy mapping from Disney model
    float aspect = sqrt(1.0 - anisotropy * 0.9);
    float ax = max(alpha / aspect, 0.001);
    float ay = max(alpha * aspect, 0.001);

    float D  = D_GGX_Anisotropic(NoH, ToH, BoH, ax, ay);
    float Vi = V_SmithGGXCorrelated_Anisotropic(NoV, NoL, ToV, ToL, BoV, BoL, ax, ay);
    vec3  F  = F_Schlick(VoH, f0);

    vec3 spec = D * Vi * F;
    vec3 kd   = (1.0 - F) * (1.0 - metallic);
    vec3 diff = kd * Fd_Burley(albedo, roughness, NoV, NoL, LoH);

    BrdfSample s;
    s.diffuse  = diff * NoL;
    s.specular = spec * NoL;
    return s;
}

vec3 brdf_shade_anisotropic(vec3 N, vec3 V, vec3 L, vec3 T, vec3 B, vec3 radiance,
                            vec3 albedo, float metallic, float roughness, float anisotropy, vec3 f0) {
    BrdfSample s = brdf_eval_anisotropic(N, V, L, T, B, albedo, metallic, roughness, anisotropy, f0);
    return (s.diffuse + s.specular) * radiance;
}

// ── Sombreado de Cabello Kajiya-Kay (RenderMan / Scheuermann Style) ──────────
vec3 brdf_eval_hair(vec3 T, vec3 N, vec3 V, vec3 L, vec3 albedo, float roughness,
                     float shift, float secondary_roughness, vec3 secondary_color) {
    vec3 H = normalize(V + L);
    float NoL = max(dot(N, L), 0.0);
    
    // Desplazamiento de la tangente en dirección de la normal para simular
    // la orientación de las escamas del cabello (Marschner-lite/Scheuermann)
    vec3 t1 = normalize(T + N * shift);
    vec3 t2 = normalize(T - N * shift);
    
    // Especular Primario (brillo en la superficie exterior, color de luz blanca)
    float dotTH1 = dot(t1, H);
    float sinTH1 = sqrt(max(0.0, 1.0 - dotTH1 * dotTH1));
    float alpha1 = roughness * roughness;
    float spec_power1 = 2.0 / max(alpha1 * alpha1, 0.001) - 2.0;
    float spec1 = pow(sinTH1, max(1.0, spec_power1));
    
    // Especular Secundario (luz transmitida y reflejada internamente, coloreada por el cabello)
    float dotTH2 = dot(t2, H);
    float sinTH2 = sqrt(max(0.0, 1.0 - dotTH2 * dotTH2));
    float alpha2 = secondary_roughness * secondary_roughness;
    float spec_power2 = 2.0 / max(alpha2 * alpha2, 0.001) - 2.0;
    float spec2 = pow(sinTH2, max(1.0, spec_power2));
    
    // Difusa Wrap de Kajiya-Kay
    float dotTL = dot(T, L);
    float sinTL = sqrt(max(0.0, 1.0 - dotTL * dotTL));
    vec3 diffuse = albedo * sinTL * REACTOR_INV_PI;
    
    // Atenuar por sombra normalizada
    vec3 specular = vec3(spec1) + secondary_color * spec2;
    return (diffuse + specular) * NoL;
}

// ── Clear Coat BRDF (Forza Horizon / Cyberpunk 2077 / Disney Principled) ─────
// Simula una capa transparente de barniz sobre la superficie base:
//   • Pintura de coches (la más icónica)
//   • Suelos mojados (charcos, lluvia)
//   • Ojos (córnea), pantallas de teléfono, madera lacada
//
// Modelo: segundo lóbulo GGX independiente con IOR del coat (típ. 1.5 = F0 ~0.04).
// La energía que refleja el coat se resta de la capa base (conservación).
//
// Parámetros:
//   coat_weight    [0..1] — intensidad del clear coat (0 = sin coat, 1 = coat completo)
//   coat_roughness [0..1] — rugosidad del coat (típ. 0.0-0.1 para coches, 0.3 para mojado)

struct ClearCoatSample {
    vec3 base_diffuse;
    vec3 base_specular;
    vec3 coat_specular;
};

// Fresnel del coat usando IOR fijo de 1.5 (F0 = 0.04)
float F_Schlick_Coat(float VoH) {
    const float F0_COAT = 0.04;
    return F0_COAT + (1.0 - F0_COAT) * pow(1.0 - VoH, 5.0);
}

ClearCoatSample brdf_eval_clearcoat(vec3 N, vec3 V, vec3 L,
                                     vec3 albedo, float metallic, float roughness, vec3 f0,
                                     float coat_weight, float coat_roughness) {
    vec3  H   = normalize(V + L);
    float NoV = max(dot(N, V), 0.0);
    float NoL = max(dot(N, L), 0.0);
    float NoH = max(dot(N, H), 0.0);
    float VoH = max(dot(V, H), 0.0);
    float LoH = max(dot(L, H), 0.0);

    // ── Base layer (standard Cook-Torrance) ──
    roughness = clamp(roughness, 0.04, 1.0);
    float alpha = roughness * roughness;
    float D_base  = D_GGX(NoH, alpha);
    float V_base  = V_SmithGGXCorrelated(NoV, NoL, alpha);
    vec3  F_base  = F_Schlick(VoH, f0);

    vec3 spec_base = D_base * V_base * F_base;
    vec3 kd        = (1.0 - F_base) * (1.0 - metallic);
    vec3 diff_base = kd * Fd_Burley(albedo, roughness, NoV, NoL, LoH);

    // ── Coat layer (second GGX lobe, dielectric IOR 1.5) ──
    float coat_alpha = clamp(coat_roughness, 0.002, 1.0);
    coat_alpha *= coat_alpha;
    float D_coat = D_GGX(NoH, coat_alpha);
    float V_coat = V_SmithGGXCorrelated(NoV, NoL, coat_alpha);
    float F_coat = F_Schlick_Coat(VoH) * coat_weight;

    vec3 spec_coat = vec3(D_coat * V_coat * F_coat);

    // ── Energy conservation: coat absorbs energy from the base layer ──
    // Lo que refleja el coat, no llega a la base
    float coat_attenuation = 1.0 - F_coat;

    ClearCoatSample s;
    s.base_diffuse  = diff_base * coat_attenuation * NoL;
    s.base_specular  = spec_base * coat_attenuation * NoL;
    s.coat_specular  = spec_coat * NoL;
    return s;
}

vec3 brdf_shade_clearcoat(vec3 N, vec3 V, vec3 L, vec3 radiance,
                           vec3 albedo, float metallic, float roughness, vec3 f0,
                           float coat_weight, float coat_roughness) {
    ClearCoatSample s = brdf_eval_clearcoat(N, V, L, albedo, metallic, roughness, f0,
                                             coat_weight, coat_roughness);
    return (s.base_diffuse + s.base_specular + s.coat_specular) * radiance;
}

// ── Cloth / Sheen BRDF (Horizon Zero Dawn / RenderMan / Filament) ────────────
// Modelo dedicado para tela y tejidos:
//   • Terciopelo, satén, algodón, lana, cuero suave
//   • La tela NO tiene Fresnel metálico — tiene un halo suave difuso
//   • Usa la distribución Charlie (Ashikhmin invertida) en vez de GGX
//   • Visibilidad Neubelt (uniform cylinder approximation)
//
// Parámetros:
//   sheen_color     — color del halo de sheen (típ. blanco para algodón, coloreado para terciopelo)
//   sheen_roughness — rugosidad del sheen [0..1] (típ. 0.5-0.8)

// D · Charlie / Ashikhmin inverted Gaussian — distribución suave para cloth
float D_Charlie(float NoH, float roughness) {
    float alpha = roughness * roughness;
    float inv_alpha = 1.0 / alpha;
    float cos2h = NoH * NoH;
    float sin2h = max(1.0 - cos2h, REACTOR_EPS);
    // Ashikhmin inverted Gaussian: (2 + 1/α) / (2π) * sin(θ)^(1/α)
    return (2.0 + inv_alpha) * pow(sin2h, inv_alpha * 0.5) / (2.0 * REACTOR_PI);
}

// V · Neubelt — aproximación analítica para la visibilidad de cloth
// Uniform cylinder model: V = 1 / (4 * (NoL + NoV - NoL*NoV))
float V_Neubelt(float NoV, float NoL) {
    return 1.0 / max(4.0 * (NoL + NoV - NoL * NoV), REACTOR_EPS);
}

BrdfSample brdf_eval_cloth(vec3 N, vec3 V, vec3 L,
                            vec3 albedo, float roughness,
                            vec3 sheen_color, float sheen_roughness) {
    vec3  H   = normalize(V + L);
    float NoV = max(dot(N, V), 0.0);
    float NoL = max(dot(N, L), 0.0);
    float NoH = max(dot(N, H), 0.0);
    float LoH = max(dot(L, H), 0.0);

    // Sheen specular: Charlie distribution + Neubelt visibility
    // No Fresnel — cloth has diffuse-like sheen instead
    float D_sheen = D_Charlie(NoH, clamp(sheen_roughness, 0.07, 1.0));
    float V_sheen = V_Neubelt(NoV, NoL);
    vec3  spec    = sheen_color * D_sheen * V_sheen;

    // Diffuse: Burley wrap — cloth is primarily diffuse
    // Cloth energy compensation: subtract sheen contribution from diffuse
    float sheen_energy = max(max(sheen_color.r, sheen_color.g), sheen_color.b);
    vec3 diff = (1.0 - sheen_energy * 0.5) * Fd_Burley(albedo, roughness, NoV, NoL, LoH);

    BrdfSample s;
    s.diffuse  = diff * NoL;
    s.specular = spec * NoL;
    return s;
}

vec3 brdf_shade_cloth(vec3 N, vec3 V, vec3 L, vec3 radiance,
                       vec3 albedo, float roughness,
                       vec3 sheen_color, float sheen_roughness) {
    BrdfSample s = brdf_eval_cloth(N, V, L, albedo, roughness, sheen_color, sheen_roughness);
    return (s.diffuse + s.specular) * radiance;
}

#endif

