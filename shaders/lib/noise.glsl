// =============================================================================
// REACTOR · shaders/lib/noise.glsl — Noise procedural sin texturas
// =============================================================================
// Para micro-variación de roughness, dithering, displacement de polvo, etc.
// Todo determinista (mismas coords → mismo valor).
// =============================================================================
#ifndef REACTOR_LIB_NOISE
#define REACTOR_LIB_NOISE

// ── Hashes (Dave Hoskins, single + integer) ──────────────────────────────────
float hash11(float p) {
    p = fract(p * 0.1031);
    p *= p + 33.33;
    p *= p + p;
    return fract(p);
}
float hash12(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
float hash13(vec3 p) {
    p = fract(p * 0.1031);
    p += dot(p, p.yzx + 19.19);
    return fract((p.x + p.y) * p.z);
}
vec3 hash33(vec3 p) {
    p = vec3(dot(p, vec3(127.1, 311.7, 74.7)),
             dot(p, vec3(269.5, 183.3, 246.1)),
             dot(p, vec3(113.5, 271.9, 124.6)));
    return -1.0 + 2.0 * fract(sin(p) * 43758.5453);
}

// ── Value noise 3D ───────────────────────────────────────────────────────────
float value_noise(vec3 p) {
    vec3 i = floor(p);
    vec3 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float n000 = hash13(i + vec3(0,0,0));
    float n100 = hash13(i + vec3(1,0,0));
    float n010 = hash13(i + vec3(0,1,0));
    float n110 = hash13(i + vec3(1,1,0));
    float n001 = hash13(i + vec3(0,0,1));
    float n101 = hash13(i + vec3(1,0,1));
    float n011 = hash13(i + vec3(0,1,1));
    float n111 = hash13(i + vec3(1,1,1));
    return mix(mix(mix(n000, n100, f.x), mix(n010, n110, f.x), f.y),
               mix(mix(n001, n101, f.x), mix(n011, n111, f.x), f.y),
               f.z);
}

// ── Perlin gradient noise 3D (-1..1) ─────────────────────────────────────────
float perlin_noise(vec3 p) {
    vec3 i = floor(p);
    vec3 f = fract(p);
    vec3 u = f * f * (3.0 - 2.0 * f);
    float g000 = dot(hash33(i + vec3(0,0,0)), f - vec3(0,0,0));
    float g100 = dot(hash33(i + vec3(1,0,0)), f - vec3(1,0,0));
    float g010 = dot(hash33(i + vec3(0,1,0)), f - vec3(0,1,0));
    float g110 = dot(hash33(i + vec3(1,1,0)), f - vec3(1,1,0));
    float g001 = dot(hash33(i + vec3(0,0,1)), f - vec3(0,0,1));
    float g101 = dot(hash33(i + vec3(1,0,1)), f - vec3(1,0,1));
    float g011 = dot(hash33(i + vec3(0,1,1)), f - vec3(0,1,1));
    float g111 = dot(hash33(i + vec3(1,1,1)), f - vec3(1,1,1));
    return mix(mix(mix(g000, g100, u.x), mix(g010, g110, u.x), u.y),
               mix(mix(g001, g101, u.x), mix(g011, g111, u.x), u.y),
               u.z);
}

// ── fBm (4 octavas) sobre value noise — buena base para detalle de superficie ─
float fbm(vec3 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 4; ++i) {
        v += a * value_noise(p);
        p *= 2.02;
        a *= 0.5;
    }
    return v;
}

// ── Dithering 1/255 — quita banding en gradientes pequeños ───────────────────
float bayer_dither(vec2 coord) {
    return (hash12(coord) - 0.5) / 255.0;
}

#endif
