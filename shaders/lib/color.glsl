// =============================================================================
// REACTOR · shaders/lib/color.glsl — Color space + grading utilities
// =============================================================================
#ifndef REACTOR_LIB_COLOR
#define REACTOR_LIB_COLOR

float saturate(float x) { return clamp(x, 0.0, 1.0); }
vec2  saturate(vec2 v)  { return clamp(v, vec2(0.0), vec2(1.0)); }
vec3  saturate(vec3 v)  { return clamp(v, vec3(0.0), vec3(1.0)); }

// BT.709 luminance.
float luminance(vec3 rgb) {
    return dot(rgb, vec3(0.2126, 0.7152, 0.0722));
}

// sRGB <-> linear (accurate, branchless approximation).
vec3 srgb_to_linear(vec3 c) {
    return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}
vec3 linear_to_srgb(vec3 c) {
    return mix(c * 12.92, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(0.0031308, c));
}

// Saturation around perceptual luma — useful for cinematic grading.
vec3 adjust_saturation(vec3 c, float sat) {
    float y = luminance(c);
    return mix(vec3(y), c, sat);
}

// Contrast pivoting around mid-grey 0.5 in linear space.
vec3 adjust_contrast(vec3 c, float contrast) {
    return (c - 0.5) * contrast + 0.5;
}

// Channel-wise gain * lift * gamma (CDL-style primary grading).
vec3 lift_gamma_gain(vec3 c, vec3 lift, vec3 gamma, vec3 gain) {
    c = c * gain + lift;
    return pow(max(c, vec3(0.0)), 1.0 / max(gamma, vec3(1e-3)));
}

// Simple white-balance shift (warm < 0 < cool on x, magenta/green on y).
vec3 white_balance(vec3 c, vec2 wb) {
    vec3 m = vec3(1.0 + wb.x * 0.2, 1.0 + wb.y * 0.1, 1.0 - wb.x * 0.2);
    return c * m;
}

#endif
