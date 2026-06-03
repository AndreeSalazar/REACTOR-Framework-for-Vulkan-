// =============================================================================
// REACTOR · shaders/lib/ibl.glsl — Image-Based Lighting procedural
// =============================================================================
// Hasta que llegue un cubemap HDR real (KTX2 + mipchain prefilt), esta lib
// fabrica un "estudio" en tiempo de fragment con:
//   • sampleEnv(dir)      — cielo / horizonte / suelo + disco solar
//   • envIrradiance(N)    — integral diffuse (5-tap hemisférico)
//   • envSpecular(R, r)   — prefilt aproximado por roughness
//   • envBRDF(NoV, r)     — tabla split-sum analítica (Karis 2014)
// =============================================================================
#ifndef REACTOR_LIB_IBL
#define REACTOR_LIB_IBL

#include "sky.glsl"

// ── Parámetros de "estudio" procedural ───────────────────────────────────────
// Cambia estas constantes para repaintar el ambiente sin tocar el BRDF.
const vec3  REACTOR_SUN_DIR      = normalize(vec3(-0.45, 0.85, 0.40));

// ── Sky procedural ───────────────────────────────────────────────────────────
vec3 sampleEnv(vec3 dir) {
    if (dir.y < 0.0) {
        float t = saturate(-dir.y * 2.0);
        vec3 ground = vec3(0.04, 0.04, 0.05); // dark ground nadir
        vec3 horizonSky = evaluate_atmosphere(vec3(dir.x, 0.0, dir.z), REACTOR_SUN_DIR, 2.0);
        return mix(horizonSky, ground, t);
    }
    return evaluate_atmosphere(dir, REACTOR_SUN_DIR, 2.0);
}

// ── Diffuse irradiance (5-tap cosine-weighted) ───────────────────────────────
vec3 envIrradiance(vec3 N) {
    vec3 up = abs(N.y) < 0.999 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
    vec3 t  = normalize(cross(up, N));
    vec3 b  = cross(N, t);
    vec3 a  = sampleEnv(N) * 0.500;
    a      += sampleEnv(normalize(N + 0.5 * t)) * 0.125;
    a      += sampleEnv(normalize(N - 0.5 * t)) * 0.125;
    a      += sampleEnv(normalize(N + 0.5 * b)) * 0.125;
    a      += sampleEnv(normalize(N - 0.5 * b)) * 0.125;
    return a;
}

// ── Specular prefilt (truco split-sum reducido) ──────────────────────────────
vec3 envSpecular(vec3 R, float roughness) {
    vec3 sharp = sampleEnv(R);
    vec3 blur  = mix(sharp, envIrradiance(R), 0.7);
    return mix(sharp, blur, roughness);
}

// ── Tabla BRDF integrada (Karis 2014, analítica) ─────────────────────────────
vec2 envBRDF(float NoV, float roughness) {
    const vec4 c0 = vec4(-1.0, -0.0275, -0.572,  0.022);
    const vec4 c1 = vec4( 1.0,  0.0425,  1.040, -0.040);
    vec4  r = roughness * c0 + c1;
    float a = min(r.x * r.x, exp2(-9.28 * NoV)) * r.x + r.y;
    return vec2(-1.04, 1.04) * a + r.zw;
}

// ── Composición IBL completa (diffuse + specular + energy compensation) ──────
vec3 ibl_eval(vec3 N, vec3 V,
              vec3 albedo, float metallic, float roughness, vec3 f0, float ao) {
    float NoV     = max(dot(N, V), 0.0);
    vec3  R       = reflect(-V, N);
    vec3  irr     = envIrradiance(N);
    vec3  preSpec = envSpecular(R, roughness);
    vec2  brdf    = envBRDF(NoV, roughness);
    vec3  Fr_ibl  = F_SchlickRoughness(NoV, f0, roughness);
    vec3  kd_ibl  = (1.0 - Fr_ibl) * (1.0 - metallic);

    vec3 spec = preSpec * (Fr_ibl * brdf.x + brdf.y);
    spec *= energy_compensation(f0, brdf);
    spec *= specular_AO(NoV, ao, roughness);

    vec3 diff = kd_ibl * albedo * irr * ao;
    return diff + spec;
}

#endif
