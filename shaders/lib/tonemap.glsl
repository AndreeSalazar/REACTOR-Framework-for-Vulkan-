// =============================================================================
// REACTOR · shaders/lib/tonemap.glsl — Operadores de tone mapping
// =============================================================================
// Tres operadores cubren la mayoría de casos:
//   • aces_filmic_narkowicz   — barato, look "películo de fotos"
//   • aces_filmic_fitted      — ACES RRT+ODT más fiel (sRGB output)
//   • agx_default             — operador AgX neutro (más moderno, mantiene
//                                detalle en altas luces sin saturar)
// =============================================================================
#ifndef REACTOR_LIB_TONEMAP
#define REACTOR_LIB_TONEMAP

#include "color.glsl"

// ── ACES Narkowicz 2015 (rápido) ─────────────────────────────────────────────
vec3 aces_filmic_narkowicz(vec3 x) {
    const float a = 2.51, b = 0.03, c = 2.43, d = 0.59, e = 0.14;
    return saturate((x * (a * x + b)) / (x * (c * x + d) + e));
}

// ── ACES "fitted" Hill 2017 (más fiel al RRT) ─────────────────────────────────
mat3 _ACES_INPUT = mat3(
    0.59719, 0.07600, 0.02840,
    0.35458, 0.90834, 0.13383,
    0.04823, 0.01566, 0.83777
);
mat3 _ACES_OUTPUT = mat3(
     1.60475, -0.10208, -0.00327,
    -0.53108,  1.10813, -0.07276,
    -0.07367, -0.00605,  1.07602
);
vec3 _rrt_odt_fit(vec3 v) {
    vec3 a = v * (v + 0.0245786) - 0.000090537;
    vec3 b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return a / b;
}
vec3 aces_filmic_fitted(vec3 color) {
    color = _ACES_INPUT * color;
    color = _rrt_odt_fit(color);
    color = _ACES_OUTPUT * color;
    return saturate(color);
}

// ── AgX (Filmic Worlds, Troy Sobotka) — versión "default" simplificada ───────
// Aproximación polinómica del LUT 3D AgX, suficiente para preview en tiempo
// real. Devuelve color sRGB display-ready.
vec3 _agx_default_contrast(vec3 x) {
    vec3 x2  = x * x;
    vec3 x4  = x2 * x2;
    return + 15.5     * x4 * x2
           - 40.14    * x4 * x
           + 31.96    * x4
           - 6.868    * x2 * x
           + 0.4298   * x2
           + 0.1191   * x
           - 0.00232;
}
vec3 agx_default(vec3 color) {
    // Min / max EV range del operador AgX (referencia Sobotka).
    const float min_ev = -12.47393;
    const float max_ev = 4.026069;
    // Matriz de transformación AgX (input lineal sRGB → AgX log).
    const mat3 agx_mat = mat3(
        0.842479062253094,  0.0423282422610123, 0.0423756549057051,
        0.0784335999999992, 0.878468636469772,  0.0784336,
        0.0792237451477643, 0.0791661274605434, 0.879142973793104
    );
    color = agx_mat * color;
    color = clamp(log2(max(color, vec3(1e-10))), vec3(min_ev), vec3(max_ev));
    color = (color - min_ev) / (max_ev - min_ev);
    color = _agx_default_contrast(color);
    return saturate(color);
}

// ── Reinhard extendido (whitepoint-aware) — para HDR muy dinámico ────────────
vec3 reinhard_extended(vec3 x, float whitepoint) {
    return (x * (1.0 + x / (whitepoint * whitepoint))) / (1.0 + x);
}

// ── Conversión final a sRGB (display) ────────────────────────────────────────
vec3 to_display(vec3 linear_color) {
    return linear_to_srgb(linear_color);
}

#endif
