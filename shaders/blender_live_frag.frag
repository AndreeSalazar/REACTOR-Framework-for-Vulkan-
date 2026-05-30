// =============================================================================
// REACTOR ⇄ Blender Live Link — Fragment shader (mini-PBR de estudio)
// =============================================================================
// Reproduce un look estilo Eevee "Material Preview" sin requerir cubemap real:
//
//   • Microfacet Cook-Torrance: GGX (D) · Schlick (F) · Smith (G correlated)
//   • Energy-conserving Lambert diffuse con Fresnel kS/kD
//   • IBL hemisférico procedural (cielo / horizonte / suelo) para ambient
//   • Specular ambiental aproximado por reflexión + horizonte (no cubemap)
//   • Tres luces analíticas: sol direccional + fill + Blender point light
//   • Rim light Fresnel (silueta) — clave del look "Eevee preview"
//   • AO geométrico por curvatura (fwidth de la normal)
//   • Contact shadow suave bajo el objeto cuando hay un "suelo" a Y=0
//   • Micro-variación de roughness con noise procedural (rompe plástico)
//   • Tone mapping ACES + corrección gamma sRGB
//
// El color del material llega vía push.color (RGB albedo, A = alpha). La
// metalicidad y roughness son derivadas heurísticamente del color hasta que
// el protocolo envíe MaterialUpdated PBR real.
// =============================================================================
#version 450

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

// -----------------------------------------------------------------------------
// Constantes
// -----------------------------------------------------------------------------
const float PI       = 3.14159265359;
const float INV_PI   = 0.31830988618;
const float EPS      = 1e-4;

// -----------------------------------------------------------------------------
// Utilidades
// -----------------------------------------------------------------------------
float saturate(float x) { return clamp(x, 0.0, 1.0); }
vec3  saturate3(vec3 x) { return clamp(x, vec3(0.0), vec3(1.0)); }

// Hash determinista para noise procedural (sin texturas).
float hash13(vec3 p) {
    p = fract(p * 0.1031);
    p += dot(p, p.yzx + 19.19);
    return fract((p.x + p.y) * p.z);
}

// Value-noise 3D triplanar — micro-variación de roughness.
float vnoise(vec3 p) {
    vec3 i = floor(p);
    vec3 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float n = mix(
        mix(mix(hash13(i + vec3(0,0,0)), hash13(i + vec3(1,0,0)), f.x),
            mix(hash13(i + vec3(0,1,0)), hash13(i + vec3(1,1,0)), f.x), f.y),
        mix(mix(hash13(i + vec3(0,0,1)), hash13(i + vec3(1,0,1)), f.x),
            mix(hash13(i + vec3(0,1,1)), hash13(i + vec3(1,1,1)), f.x), f.y),
        f.z);
    return n;
}

// ACES Filmic — versión Narkowicz, suficiente para preview.
vec3 aces_filmic(vec3 x) {
    const float a = 2.51, b = 0.03, c = 2.43, d = 0.59, e = 0.14;
    return saturate3((x * (a * x + b)) / (x * (c * x + d) + e));
}

// -----------------------------------------------------------------------------
// BRDF Cook-Torrance
// -----------------------------------------------------------------------------
// D — Trowbridge-Reitz GGX
float D_GGX(float NoH, float a) {
    float a2 = a * a;
    float f  = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / max(PI * f * f, EPS);
}

// G — Smith Schlick-GGX (height-correlated, eficiente)
float V_SmithGGXCorrelated(float NoV, float NoL, float a) {
    float a2 = a * a;
    float ggxV = NoL * sqrt(NoV * NoV * (1.0 - a2) + a2);
    float ggxL = NoV * sqrt(NoL * NoL * (1.0 - a2) + a2);
    return 0.5 / max(ggxV + ggxL, EPS);
}

// F — Fresnel Schlick
vec3 F_Schlick(float VoH, vec3 f0) {
    return f0 + (1.0 - f0) * pow(1.0 - VoH, 5.0);
}

// Fresnel con roughness para IBL (Sébastien Lagarde).
vec3 F_SchlickRoughness(float NoV, vec3 f0, float roughness) {
    return f0 + (max(vec3(1.0 - roughness), f0) - f0) * pow(1.0 - NoV, 5.0);
}

// Evaluación de un solo light analítico (dirección L unitaria, intensidad rad).
vec3 shadeAnalytic(vec3 N, vec3 V, vec3 L, vec3 radiance,
                   vec3 albedo, float metallic, float roughness, vec3 f0) {
    vec3 H   = normalize(V + L);
    float NoV = max(dot(N, V), 0.0);
    float NoL = max(dot(N, L), 0.0);
    float NoH = max(dot(N, H), 0.0);
    float VoH = max(dot(V, H), 0.0);

    float a  = roughness * roughness;
    float D  = D_GGX(NoH, a);
    float Vi = V_SmithGGXCorrelated(NoV, NoL, a);
    vec3  F  = F_Schlick(VoH, f0);
    vec3  Fr = D * Vi * F;                              // specular
    vec3  kd = (1.0 - F) * (1.0 - metallic);
    vec3  Fd = kd * albedo * INV_PI;                    // diffuse
    return (Fd + Fr) * radiance * NoL;
}

// -----------------------------------------------------------------------------
// IBL hemisférico procedural — reemplazo barato de cubemap real.
// Devuelve la "radiancia de entorno" en la dirección dir (unitaria).
// -----------------------------------------------------------------------------
vec3 sampleEnv(vec3 dir) {
    // Tres bandas tipo cielo abierto:
    //   zenith  (mirando arriba)   — azul frío suave
    //   horizon (mirando lateral)  — neutro cálido
    //   nadir   (mirando abajo)    — gris/marrón suelo
    const vec3 zenith  = vec3(0.55, 0.70, 0.95);
    const vec3 horizon = vec3(0.90, 0.92, 0.98);
    const vec3 nadir   = vec3(0.10, 0.10, 0.12);

    float up   = dir.y;
    // Compresión hacia el horizonte para que la transición no sea lineal.
    float t    = sign(up) * pow(abs(up), 0.6);
    vec3  sky  = mix(horizon, zenith, saturate(t));
    vec3  gnd  = mix(horizon, nadir,  saturate(-t));
    return mix(gnd, sky, step(0.0, up));
}

// Irradiancia (diffuse IBL) — integral coseno sobre el hemisferio. Aproximada
// con 3 muestras (N, N+tangent, N-tangent) — suficiente para preview.
vec3 envIrradiance(vec3 N) {
    vec3 up    = abs(N.y) < 0.999 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
    vec3 t     = normalize(cross(up, N));
    vec3 b     = cross(N, t);
    vec3 acc   = sampleEnv(N) * 0.5;
    acc       += sampleEnv(normalize(N + 0.5 * t)) * 0.125;
    acc       += sampleEnv(normalize(N - 0.5 * t)) * 0.125;
    acc       += sampleEnv(normalize(N + 0.5 * b)) * 0.125;
    acc       += sampleEnv(normalize(N - 0.5 * b)) * 0.125;
    return acc;
}

// Pre-filtered "specular" IBL — mezcla muestra dura con muestra blureada por
// roughness mirando hacia el horizonte (truco split-sum simplificado).
vec3 envSpecular(vec3 R, float roughness) {
    vec3 sharp = sampleEnv(R);
    vec3 blur  = mix(sampleEnv(R), envIrradiance(R), 0.7);
    return mix(sharp, blur, roughness);
}

// Aproximación analítica del término BRDF integrado (split-sum).
vec2 envBRDF(float NoV, float roughness) {
    // Fórmula Karis 2014 — coeficientes ajustados.
    const vec4 c0 = vec4(-1.0, -0.0275, -0.572,  0.022);
    const vec4 c1 = vec4( 1.0,  0.0425,  1.040, -0.040);
    vec4  r = roughness * c0 + c1;
    float a = min(r.x * r.x, exp2(-9.28 * NoV)) * r.x + r.y;
    return vec2(-1.04, 1.04) * a + r.zw;
}

// -----------------------------------------------------------------------------
// Detalle / micro-variación
// -----------------------------------------------------------------------------
// Curvatura por derivadas — convexidades quedan más claras, cavidades más oscuras.
float curvatureAO(vec3 N) {
    vec3 dx = dFdx(N);
    vec3 dy = dFdy(N);
    float k = length(dx) + length(dy);
    return saturate(1.0 - k * 1.2);
}

// Contact shadow muy barato: si el objeto está cerca del plano Y=0, oscurece
// la parte inferior según la altura. Da peso visual sin shadow map real.
float contactShadow(vec3 P, vec3 N) {
    float groundHeight = 0.0;
    float h = P.y - groundHeight;
    float NoUp = saturate(dot(N, vec3(0.0, 1.0, 0.0)));
    // Sombra fuerte cuando h~0 y la normal mira hacia arriba.
    return mix(1.0, 0.55, saturate(1.0 - h * 1.5) * NoUp);
}

// -----------------------------------------------------------------------------
// Main
// -----------------------------------------------------------------------------
void main() {
    // ── Inputs ──────────────────────────────────────────────────────────────
    vec3  N  = normalize(vWorldNormal);
    vec3  V  = normalize(vViewDir);
    vec3  P  = vWorldPos;
    vec3  albedo = max(vColor.rgb, vec3(0.0));

    // ── Heurística de material (placeholder hasta MaterialUpdated real) ────
    // Color saturado y oscuro → más plástico (roughness ~0.5)
    // Color claro y desaturado → más metálico/limpio (roughness ~0.3)
    float luma  = dot(albedo, vec3(0.2126, 0.7152, 0.0722));
    float satur = length(albedo - vec3(luma));
    float metallic  = 0.0;
    float baseRough = mix(0.55, 0.30, saturate(luma) * (1.0 - saturate(satur * 1.2)));

    // Micro-variación procedural — rompe la sensación "shader plano".
    float n      = vnoise(P * 3.5);
    float rough  = clamp(baseRough + (n - 0.5) * 0.12, 0.04, 1.0);
    vec3  f0     = mix(vec3(0.04), albedo, metallic);

    // ── Direcciones de luz (estudio 3-point + Blender point light) ─────────
    // Sol: key light cálido desde arriba-derecha-delante.
    vec3  sunDir   = normalize(vec3(-0.45, 0.85, 0.40));
    vec3  sunRad   = vec3(3.2, 3.0, 2.7);
    // Fill: luz fría contraria, suave.
    vec3  fillDir  = normalize(vec3(0.65, 0.45, -0.30));
    vec3  fillRad  = vec3(0.55, 0.65, 0.85);
    // Back/rim: luz blanca por detrás-arriba para silueta.
    vec3  backDir  = normalize(vec3(0.10, 0.65, -0.95));
    vec3  backRad  = vec3(1.6, 1.55, 1.50);

    // ── Lighting analítico ─────────────────────────────────────────────────
    vec3 lo = vec3(0.0);
    lo += shadeAnalytic(N, V, sunDir,  sunRad,  albedo, metallic, rough, f0);
    lo += shadeAnalytic(N, V, fillDir, fillRad, albedo, metallic, rough, f0);
    lo += shadeAnalytic(N, V, backDir, backRad, albedo, metallic, rough, f0);

    // Blender point light (push.light_pos): atenuación cuadrática física-like.
    vec3  Lv  = push.light_pos.xyz - P;
    float d2  = dot(Lv, Lv);
    if (d2 > 1e-4 && d2 < 60.0 * 60.0) {
        float d  = sqrt(d2);
        vec3  L  = Lv / d;
        // 1/d^2 con smooth cutoff a 30u (evita pop al límite).
        float a  = 1.0 / (1.0 + d2 * 0.10);
        float w  = 1.0 - smoothstep(20.0, 30.0, d);
        vec3  rad = vec3(8.0) * a * w; // intensidad blanca
        lo += shadeAnalytic(N, V, L, rad, albedo, metallic, rough, f0);
    }

    // ── IBL ambiental (split-sum aproximado) ───────────────────────────────
    float NoV     = max(dot(N, V), 0.0);
    vec3  R       = reflect(-V, N);
    vec3  irr     = envIrradiance(N);
    vec3  preSpec = envSpecular(R, rough);
    vec2  brdf    = envBRDF(NoV, rough);
    vec3  Fr_ibl  = F_SchlickRoughness(NoV, f0, rough);
    vec3  kd_ibl  = (1.0 - Fr_ibl) * (1.0 - metallic);

    vec3 ambient = kd_ibl * albedo * irr
                 + preSpec * (Fr_ibl * brdf.x + brdf.y);

    // ── Rim light Fresnel (silueta tipo Eevee preview) ─────────────────────
    float rim = pow(1.0 - NoV, 4.0);
    vec3  rimColor = vec3(0.65, 0.78, 1.0) * 0.5;
    vec3  rimContrib = rimColor * rim * (1.0 - metallic);

    // ── Oclusión geométrica + contact shadow ───────────────────────────────
    float ao = curvatureAO(N);
    float cs = contactShadow(P, N);

    // ── Composición final ──────────────────────────────────────────────────
    vec3 color = (lo + ambient + rimContrib) * ao * cs;

    // Saturación cinematográfica sutil (+5%).
    float Y = dot(color, vec3(0.2126, 0.7152, 0.0722));
    color = mix(vec3(Y), color, 1.05);

    // Tonemap + gamma.
    color = aces_filmic(color);
    color = pow(color, vec3(1.0 / 2.2));

    outColor = vec4(color, vColor.a);
}
