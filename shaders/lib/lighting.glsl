// =============================================================================
// REACTOR · shaders/lib/lighting.glsl — Modelos de luces analíticas
// =============================================================================
// Funciones de atenuación / cono según Frostbite "Moving to PBR" + UE5:
//   • smooth_distance_attenuation — 1/d² con falloff cuadrático suave
//   • spot_angle_attenuation      — cono inner/outer físicamente correcto
//   • light_eval_directional/point/spot — wrappers + composición Cook-Torrance
// =============================================================================
#ifndef REACTOR_LIB_LIGHTING
#define REACTOR_LIB_LIGHTING

#include "pbr.glsl"

// ── Falloff de distancia físico (Frostbite §5.1) ─────────────────────────────
float smooth_distance_attenuation(float d2, float range) {
    float r2     = range * range;
    float factor = d2 / max(r2, REACTOR_EPS);
    float smooth = saturate(1.0 - factor * factor);
    return (smooth * smooth) / max(d2, 0.01 * 0.01);
}

// ── Falloff de cono (spot light) ─────────────────────────────────────────────
float spot_angle_attenuation(vec3 L, vec3 spotDir,
                             float cosInner, float cosOuter) {
    float cd = dot(-spotDir, L);
    float t  = saturate((cd - cosOuter) / max(cosInner - cosOuter, REACTOR_EPS));
    return t * t;
}

// ── Directional ──────────────────────────────────────────────────────────────
vec3 light_eval_directional(vec3 N, vec3 V, vec3 L, vec3 radiance,
                            vec3 albedo, float metallic, float roughness, vec3 f0) {
    return brdf_shade(N, V, L, radiance, albedo, metallic, roughness, f0);
}

// ── Point ────────────────────────────────────────────────────────────────────
vec3 light_eval_point(vec3 N, vec3 V, vec3 P,
                      vec3 lightPos, vec3 lightColor,
                      float intensity, float range,
                      vec3 albedo, float metallic, float roughness, vec3 f0) {
    vec3  Lv  = lightPos - P;
    float d2  = dot(Lv, Lv);
    if (d2 < REACTOR_EPS) return vec3(0.0);
    float d   = sqrt(d2);
    vec3  L   = Lv / d;
    float att = smooth_distance_attenuation(d2, range);
    vec3  rad = lightColor * intensity * att;
    return brdf_shade(N, V, L, rad, albedo, metallic, roughness, f0);
}

// ── Spot ─────────────────────────────────────────────────────────────────────
vec3 light_eval_spot(vec3 N, vec3 V, vec3 P,
                     vec3 lightPos, vec3 lightDir, vec3 lightColor,
                     float intensity, float range,
                     float cosInner, float cosOuter,
                     vec3 albedo, float metallic, float roughness, vec3 f0) {
    vec3  Lv  = lightPos - P;
    float d2  = dot(Lv, Lv);
    if (d2 < REACTOR_EPS) return vec3(0.0);
    float d   = sqrt(d2);
    vec3  L   = Lv / d;
    float att = smooth_distance_attenuation(d2, range)
              * spot_angle_attenuation(L, lightDir, cosInner, cosOuter);
    vec3  rad = lightColor * intensity * att;
    return brdf_shade(N, V, L, rad, albedo, metallic, roughness, f0);
}

// ── Setup de estudio 3-point (key + fill + back/rim) ─────────────────────────
// Devuelve el aporte combinado — útil cuando aún no hay luces en escena.
vec3 light_studio_3point(vec3 N, vec3 V,
                         vec3 albedo, float metallic, float roughness, vec3 f0) {
    vec3 keyDir  = normalize(vec3(-0.45, 0.85, 0.40));
    vec3 fillDir = normalize(vec3( 0.65, 0.45,-0.30));
    vec3 backDir = normalize(vec3( 0.10, 0.65,-0.95));
    vec3 keyRad  = vec3(3.2, 3.0, 2.7);
    vec3 fillRad = vec3(0.55, 0.65, 0.85);
    vec3 backRad = vec3(1.6, 1.55, 1.50);
    vec3 lo = vec3(0.0);
    lo += light_eval_directional(N, V, keyDir,  keyRad,  albedo, metallic, roughness, f0);
    lo += light_eval_directional(N, V, fillDir, fillRad, albedo, metallic, roughness, f0);
    lo += light_eval_directional(N, V, backDir, backRad, albedo, metallic, roughness, f0);
    return lo;
}

// ── Rim light Fresnel (silueta tipo Eevee preview) ───────────────────────────
vec3 light_rim(vec3 N, vec3 V, vec3 rimColor, float power, float metallic) {
    float NoV = max(dot(N, V), 0.0);
    float rim = pow(1.0 - NoV, power);
    return rimColor * rim * (1.0 - metallic);
}

#endif
