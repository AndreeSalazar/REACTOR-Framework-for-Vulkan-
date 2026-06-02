#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform sampler2D screenTexture;
layout(binding = 1) uniform sampler2D bloomTexture;
layout(binding = 2) uniform sampler2D depthTexture;

layout(push_constant) uniform PostProcessSettings {
    // Vignette
    float vignette_intensity;
    float vignette_smoothness;

    // Chromatic Aberration
    float chromatic_intensity;

    // Film Grain
    float grain_intensity;
    float grain_speed;

    // Bloom
    float bloom_threshold;
    float bloom_intensity;
    float bloom_blur_size;

    // Tone Mapping
    float exposure;
    float gamma;

    // Sharpen
    float sharpen_intensity;

    // Screen-space lighting
    float ssgi_intensity;
    float ssgi_radius;
    float fog_density;
    float fog_scatter;
    float lut_strength;
    float ssr_strength;
    float pathtrace_intensity;
    float flare_intensity;
    float highlight_recovery;
    float pause_overlay_alpha;
    float pause_page;
    float pause_selected;
    float pause_row_count;

    // General
    float time;
    float depth_near;
    float depth_far;
    uint effect_mask;
    uint _padding;
} settings;

// Effect indices (matching PostProcessEffect enum)
#define EFFECT_GRAYSCALE          (1u << 1)
#define EFFECT_SEPIA              (1u << 2)
#define EFFECT_INVERT             (1u << 3)
#define EFFECT_VIGNETTE           (1u << 4)
#define EFFECT_CHROMATIC          (1u << 5)
#define EFFECT_GRAIN              (1u << 6)
#define EFFECT_SHARPEN            (1u << 7)
#define EFFECT_BLUR               (1u << 8)
#define EFFECT_BLOOM              (1u << 9)
#define EFFECT_TONEMAP            (1u << 10)
#define EFFECT_FXAA               (1u << 11)
#define EFFECT_SSGI               (1u << 14)
#define EFFECT_VOLUMETRIC_FOG     (1u << 15)
#define EFFECT_LUT_COLOR_GRADING  (1u << 16)
#define EFFECT_SSR                (1u << 17)
#define EFFECT_PATH_TRACED_LIGHT  (1u << 18)
#define EFFECT_ANAMORPHIC_FLARES  (1u << 19)

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

// Pseudo-random noise for Film Grain
float rand(vec2 co) {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

float interleaved_gradient_noise(vec2 pixel, float frame) {
    pixel += frame * vec2(5.588238, 5.588238);
    return fract(52.9829189 * fract(0.06711056 * pixel.x + 0.00583715 * pixel.y));
}

// ── AgX Tone Mapping (Troy Sobotka / Filmic Worlds) ─────────────────────────
// Cinematic SDR operator — preserves high-light detail and hue better than ACES.
// Matches Blender's AgX output transform for consistent look between editor and renderer.
vec3 _agx_contrast(vec3 x) {
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
vec3 agx_tonemap(vec3 color) {
    const float min_ev = -12.47393;
    const float max_ev = 4.026069;
    const mat3 agx_mat = mat3(
        0.842479062253094,  0.0423282422610123, 0.0423756549057051,
        0.0784335999999992, 0.878468636469772,  0.0784336,
        0.0792237451477643, 0.0791661274605434, 0.879142973793104
    );
    color = agx_mat * color;
    color = clamp(log2(max(color, vec3(1e-10))), vec3(min_ev), vec3(max_ev));
    color = (color - min_ev) / (max_ev - min_ev);
    color = _agx_contrast(color);
    return clamp(color, 0.0, 1.0);
}

// Legacy ACES Filmic (kept for compatibility / user preference)
vec3 aces_tonemap(vec3 color) {
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), 0.0, 1.0);
}

vec3 sample_screen(vec2 uv) {
    return texture(screenTexture, clamp(uv, vec2(0.001), vec2(0.999))).rgb;
}

float sample_depth(vec2 uv) {
    return texture(depthTexture, clamp(uv, vec2(0.001), vec2(0.999))).r;
}

float linearize_depth(float depth) {
    float nearPlane = max(settings.depth_near, 0.001);
    float farPlane = max(settings.depth_far, nearPlane + 0.001);
    return (nearPlane * farPlane) / max(farPlane - depth * (farPlane - nearPlane), 0.0001);
}

vec3 reconstruct_view_pos(vec2 uv) {
    vec2 size = vec2(textureSize(depthTexture, 0));
    float aspect = size.x / max(size.y, 1.0);
    float viewZ = linearize_depth(sample_depth(uv));
    vec2 ndc = uv * 2.0 - 1.0;
    return vec3(ndc.x * aspect * viewZ, -ndc.y * viewZ, -viewZ);
}

vec3 estimate_screen_normal(vec2 uv, vec2 texelSize) {
    vec3 p  = reconstruct_view_pos(uv);
    vec3 px = reconstruct_view_pos(uv + vec2(texelSize.x, 0.0));
    vec3 py = reconstruct_view_pos(uv + vec2(0.0, texelSize.y));
    vec3 n = normalize(cross(px - p, py - p));
    return n.z < 0.0 ? -n : n;
}

vec3 screen_space_gi(vec2 uv, vec2 texelSize, vec3 baseColor) {
    float centerDepth = sample_depth(uv);
    if (centerDepth >= 0.9999) {
        return baseColor;
    }

    vec3 normal = estimate_screen_normal(uv, texelSize);
    vec3 centerPos = reconstruct_view_pos(uv);
    vec3 diffuseBounce = vec3(0.0);
    float occlusion = 0.0;
    float totalWeight = 0.0;

    const int DIRS = 8;
    const int STEPS = 3;
    float jitter = interleaved_gradient_noise(gl_FragCoord.xy, settings.time) * 6.2831853;
    float radiusPx = clamp(settings.ssgi_radius, 2.0, 48.0);
    float radiusWorld = max(centerPos.z * -0.018, 0.08) * radiusPx;

    for (int i = 0; i < DIRS; ++i) {
        float a = jitter + (float(i) + 0.5) * 6.2831853 / float(DIRS);
        vec2 dir = vec2(cos(a), sin(a));
        for (int stepId = 1; stepId <= STEPS; ++stepId) {
            float stepT = float(stepId) / float(STEPS);
            vec2 suv = uv + dir * texelSize * radiusPx * stepT;
            float sd = sample_depth(suv);
            if (sd >= 0.9999) {
                continue;
            }

            vec3 samplePos = reconstruct_view_pos(suv);
            vec3 delta = samplePos - centerPos;
            float dist = length(delta);
            vec3 omega = delta / max(dist, 0.0001);
            float facing = max(dot(normal, omega), 0.0);
            float rangeWeight = smoothstep(radiusWorld, 0.0, dist);
            float thickness = smoothstep(0.001, radiusWorld * 0.14, delta.z);
            float w = facing * rangeWeight;

            occlusion += w * thickness;
            diffuseBounce += sample_screen(suv) * w * (1.0 - thickness);
            totalWeight += w;
        }
    }

    float rawAo = 1.0 - clamp(occlusion / max(totalWeight, 0.0001), 0.0, 0.92);

    float aoSum = 0.0;
    float wSum = 0.0;
    for (int y = -1; y <= 1; ++y) {
        for (int x = -1; x <= 1; ++x) {
            vec2 o = vec2(x, y) * texelSize;
            float d = linearize_depth(sample_depth(uv + o));
            float depthWeight = exp(-abs(d + centerPos.z) * 2.5);
            float spatialWeight = (x == 0 && y == 0) ? 1.0 : ((x == 0 || y == 0) ? 0.65 : 0.38);
            float sampleAo = rawAo;
            if (x != 0 || y != 0) {
                vec3 sp = reconstruct_view_pos(uv + o);
                vec3 dd = sp - centerPos;
                sampleAo = 1.0 - clamp(max(dot(normal, normalize(dd)), 0.0) *
                    smoothstep(radiusWorld, 0.0, length(dd)) *
                    smoothstep(0.001, radiusWorld * 0.14, dd.z), 0.0, 0.92);
            }
            float bw = depthWeight * spatialWeight;
            aoSum += sampleAo * bw;
            wSum += bw;
        }
    }

    float ao = clamp(aoSum / max(wSum, 0.0001), 0.08, 1.0);
    vec3 bounce = diffuseBounce / max(totalWeight, 0.0001);
    vec3 gi = baseColor * ao + bounce * (0.08 * settings.ssgi_intensity);
    return mix(baseColor, max(gi, 0.0), settings.ssgi_intensity);
}

vec3 screen_space_reflection(vec2 uv, vec2 texelSize, vec3 color) {
    vec3 normal = estimate_screen_normal(uv, texelSize);
    float floorMask = smoothstep(0.42, 0.96, uv.y);
    float wetMask = smoothstep(0.18, 0.62, luminance(color)) * floorMask;

    vec2 view = normalize(uv - vec2(0.5, 1.15));
    vec2 refl = reflect(view, normalize(normal.xy + vec2(0.0, 0.35)));
    refl.y = -abs(refl.y);

    vec3 hit = vec3(0.0);
    float hitWeight = 0.0;
    for (int i = 1; i <= 14; ++i) {
        float t = float(i) / 14.0;
        vec2 suv = uv + refl * texelSize * mix(6.0, 90.0, t);
        if (any(lessThan(suv, vec2(0.0))) || any(greaterThan(suv, vec2(1.0)))) {
            break;
        }

        vec3 s = sample_screen(suv);
        float candidate = smoothstep(0.35, 1.2, luminance(s));
        float fade = (1.0 - t) * smoothstep(1.0, 0.45, abs(suv.y - uv.y));
        hit += s * candidate * fade;
        hitWeight += candidate * fade;
    }

    vec3 reflection = hit / max(hitWeight, 0.0001);
    reflection *= vec3(0.75, 0.9, 1.08);
    float strength = wetMask * settings.ssr_strength;
    return mix(color, color + reflection * 0.55, strength);
}

vec3 volumetric_fog(vec2 uv, vec3 color) {
    vec2 p = uv - 0.5;
    float radialDepth = smoothstep(0.1, 0.92, length(p));
    float horizon = smoothstep(0.20, 1.0, uv.y);
    float noise = rand(floor(gl_FragCoord.xy * 0.5) + settings.time * 19.0);

    vec2 lightUvA = vec2(0.18, 0.22);
    vec2 lightUvB = vec2(0.82, 0.24);
    float shaftA = pow(max(0.0, 1.0 - length((uv - lightUvA) * vec2(1.0, 1.6))), 4.0);
    float shaftB = pow(max(0.0, 1.0 - length((uv - lightUvB) * vec2(1.0, 1.6))), 4.0);
    vec3 shaftColor = vec3(0.55, 0.08, 0.35) * shaftA + vec3(0.03, 0.45, 0.65) * shaftB;

    float density = settings.fog_density * (0.45 + horizon * 0.95 + radialDepth * 0.55);
    density *= 0.9 + noise * 0.2;

    vec3 fogColor = vec3(0.018, 0.016, 0.032) + shaftColor * settings.fog_scatter;
    float fogAmount = 1.0 - exp(-density);
    return mix(color, fogColor + color * 0.12, clamp(fogAmount, 0.0, 0.82));
}

vec3 lut_color_grade(vec3 color) {
    float luma = luminance(color);
    vec3 shadows = vec3(0.78, 0.88, 1.12);
    vec3 mids = vec3(0.96, 1.02, 1.03);
    vec3 highs = vec3(1.14, 1.02, 0.88);

    vec3 grade = mix(shadows, mids, smoothstep(0.05, 0.45, luma));
    grade = mix(grade, highs, smoothstep(0.45, 0.95, luma));

    vec3 graded = color * grade;
    float gradedLuma = luminance(graded);
    graded = mix(vec3(gradedLuma), graded, 1.10);
    graded = (graded - 0.5) * 1.08 + 0.5;
    graded += vec3(0.012, -0.004, 0.006);
    return mix(color, max(graded, 0.0), settings.lut_strength);
}

vec3 path_traced_lighting_resolve(vec2 uv, vec2 texelSize, vec3 color) {
    vec3 normal = estimate_screen_normal(uv, texelSize);
    float centerLum = luminance(color);
    vec3 diffuseTransport = vec3(0.0);
    vec3 specularTransport = vec3(0.0);
    float occlusion = 0.0;
    float totalWeight = 0.0001;

    const int DIRS = 12;
    for (int i = 0; i < DIRS; ++i) {
        float a = (float(i) + 0.5) * 6.2831853 / float(DIRS);
        vec2 dir = vec2(cos(a), sin(a));
        float hemi = max(dot(normal, normalize(vec3(dir, 0.7))), 0.0);

        for (int stepId = 1; stepId <= 3; ++stepId) {
            float travel = float(stepId * stepId) * 9.0;
            vec2 suv = uv + dir * texelSize * travel;
            vec3 s = sample_screen(suv);
            float sLum = luminance(s);
            float edgeAware = exp(-abs(sLum - centerLum) * 3.5);
            float distanceWeight = 1.0 / float(stepId * stepId + 1);
            float w = hemi * edgeAware * distanceWeight;

            diffuseTransport += s * w;
            specularTransport += max(s - vec3(settings.bloom_threshold), vec3(0.0)) * w;
            occlusion += (sLum < centerLum ? w : 0.0);
            totalWeight += w;
        }
    }

    vec3 bouncedLight = diffuseTransport / totalWeight;
    vec3 glossyLight = specularTransport / totalWeight;
    float ao = 1.0 - clamp(occlusion / totalWeight, 0.0, 0.55);
    vec3 resolved = color * ao + bouncedLight * 0.20 + glossyLight * 0.75;
    return mix(color, max(resolved, 0.0), settings.pathtrace_intensity);
}

vec3 anamorphic_flares(vec2 uv, vec2 texelSize, vec3 color) {
    vec3 flare = vec3(0.0);

    for (int i = -10; i <= 10; ++i) {
        if (i == 0) {
            continue;
        }

        float t = abs(float(i)) / 10.0;
        vec2 horizontal = vec2(float(i) * 7.5, 0.0) * texelSize;
        vec3 h = sample_screen(uv + horizontal);
        vec3 brightH = max(h - vec3(settings.bloom_threshold), vec3(0.0));
        flare += brightH * (1.0 - t) * vec3(0.80, 0.95, 1.25);

        vec2 diagonal = vec2(float(i) * 3.0, float(i) * 0.55) * texelSize;
        vec3 d = sample_screen(uv + diagonal);
        vec3 brightD = max(d - vec3(settings.bloom_threshold + 0.08), vec3(0.0));
        flare += brightD * (1.0 - t) * vec3(1.20, 0.55, 1.05) * 0.35;
    }

    return color + flare * settings.flare_intensity * 0.18;
}

vec3 recover_highlights(vec3 color) {
    float luma = luminance(color);
    vec3 compressed = color / (1.0 + color * settings.highlight_recovery);
    float mask = smoothstep(0.72, 2.4, luma);
    return mix(color, compressed * (1.0 + luma * 0.18), mask);
}

float rect_mask(vec2 uv, vec2 center, vec2 halfSize, float softness) {
    vec2 d = abs(uv - center) - halfSize;
    float outside = length(max(d, 0.0));
    float inside = min(max(d.x, d.y), 0.0);
    return 1.0 - smoothstep(0.0, softness, outside + inside);
}

float pause_row_value(int page, int row) {
    if (page == 0) {
        if (row == 0) return 1.0;
        if (row == 1) return clamp(settings.pause_overlay_alpha, 0.0, 1.0);
        if (row == 2) return float(settings.effect_mask != 0u);
        if (row == 3) return 1.0;
        if (row == 4) return clamp((settings.exposure - 0.4) / 2.1, 0.0, 1.0);
        if (row == 5) return clamp((settings.gamma - 1.6) / 1.2, 0.0, 1.0);
        if (row == 6) return clamp(settings.bloom_intensity / 2.0, 0.0, 1.0);
        if (row == 7) return clamp((settings.bloom_threshold - 0.3) / 1.3, 0.0, 1.0);
        if (row == 8) return clamp(settings.grain_intensity / 0.12, 0.0, 1.0);
        if (row == 9) return clamp(settings.chromatic_intensity / 0.015, 0.0, 1.0);
        if (row == 10) return clamp(settings.vignette_intensity / 0.9, 0.0, 1.0);
        return clamp(settings.sharpen_intensity / 1.5, 0.0, 1.0);
    }

    if (page == 1) {
        if (row == 0) return ((settings.effect_mask & EFFECT_SSGI) != 0u) ? 1.0 : 0.0;
        if (row == 1) return clamp(settings.ssgi_intensity / 1.5, 0.0, 1.0);
        if (row == 2) return clamp(settings.ssgi_radius / 40.0, 0.0, 1.0);
        if (row == 3) return ((settings.effect_mask & EFFECT_PATH_TRACED_LIGHT) != 0u) ? 1.0 : 0.0;
        if (row == 4) return clamp(settings.pathtrace_intensity / 1.5, 0.0, 1.0);
        if (row == 5) return ((settings.effect_mask & EFFECT_SSR) != 0u) ? 1.0 : 0.0;
        if (row == 6) return clamp(settings.ssr_strength / 1.5, 0.0, 1.0);
        if (row == 7) return ((settings.effect_mask & EFFECT_VOLUMETRIC_FOG) != 0u) ? 1.0 : 0.0;
        if (row == 8) return clamp(settings.fog_density / 1.2, 0.0, 1.0);
        if (row == 9) return clamp(settings.fog_scatter / 2.0, 0.0, 1.0);
        if (row == 10) return ((settings.effect_mask & EFFECT_ANAMORPHIC_FLARES) != 0u) ? 1.0 : 0.0;
        if (row == 11) return clamp(settings.flare_intensity / 2.0, 0.0, 1.0);
        return clamp(settings.highlight_recovery / 2.0, 0.0, 1.0);
    }

    if (page == 2) {
        if (row == 0) return ((settings.effect_mask & EFFECT_LUT_COLOR_GRADING) != 0u) ? 1.0 : 0.0;
        if (row == 1) return clamp(settings.lut_strength / 1.5, 0.0, 1.0);
        if (row == 2) return ((settings.effect_mask & EFFECT_TONEMAP) != 0u) ? 1.0 : 0.0;
        if (row == 3) return ((settings.effect_mask & EFFECT_BLOOM) != 0u) ? 1.0 : 0.0;
        if (row == 4) return ((settings.effect_mask & EFFECT_VIGNETTE) != 0u) ? 1.0 : 0.0;
        if (row == 5) return ((settings.effect_mask & EFFECT_CHROMATIC) != 0u) ? 1.0 : 0.0;
        if (row == 6) return ((settings.effect_mask & EFFECT_GRAIN) != 0u) ? 1.0 : 0.0;
        if (row == 7) return ((settings.effect_mask & EFFECT_FXAA) != 0u) ? 1.0 : 0.0;
        if (row == 8) return ((settings.effect_mask & EFFECT_SHARPEN) != 0u) ? 1.0 : 0.0;
        if (row == 9) return ((settings.effect_mask & EFFECT_GRAYSCALE) != 0u) ? 1.0 : 0.0;
        if (row == 10) return ((settings.effect_mask & EFFECT_SEPIA) != 0u) ? 1.0 : 0.0;
        if (row == 11) return ((settings.effect_mask & EFFECT_INVERT) != 0u) ? 1.0 : 0.0;
        return ((settings.effect_mask & EFFECT_BLUR) != 0u) ? 1.0 : 0.0;
    }

    if (page == 3) {
        if (row == 0) return 0.65;
        if (row == 1) return 1.0;
        if (row == 2) return float(settings.effect_mask != 0u);
        if (row == 3) return ((settings.effect_mask & EFFECT_FXAA) != 0u) ? 1.0 : 0.0;
        if (row == 4) return ((settings.effect_mask & EFFECT_PATH_TRACED_LIGHT) != 0u) ? 1.0 : 0.0;
        if (row == 5) return ((settings.effect_mask & EFFECT_SSR) != 0u) ? 1.0 : 0.0;
        if (row == 6) return ((settings.effect_mask & EFFECT_VOLUMETRIC_FOG) != 0u) ? 1.0 : 0.0;
        if (row == 7) return ((settings.effect_mask & EFFECT_ANAMORPHIC_FLARES) != 0u) ? 1.0 : 0.0;
        return ((settings.effect_mask & EFFECT_GRAIN) != 0u) ? 1.0 : 0.0;
    }

    return row == int(settings.pause_selected) ? 1.0 : 0.35;
}

vec3 draw_pause_overlay(vec2 uv, vec3 color) {
    float alpha = clamp(settings.pause_overlay_alpha, 0.0, 1.0);
    if (alpha <= 0.001) {
        return color;
    }

    vec3 outc = mix(color, vec3(0.006, 0.008, 0.014), alpha * 0.70);

    vec2 panelCenter = vec2(0.50, 0.52);
    vec2 panelHalf = vec2(0.38, 0.38);
    float panel = rect_mask(uv, panelCenter, panelHalf, 0.012);
    outc = mix(outc, vec3(0.018, 0.020, 0.034), panel * alpha * 0.88);

    float border = rect_mask(uv, panelCenter, panelHalf + vec2(0.004), 0.004)
        - rect_mask(uv, panelCenter, panelHalf - vec2(0.004), 0.004);
    vec3 accent = mix(vec3(0.0, 0.85, 1.0), vec3(1.0, 0.05, 0.58), fract(settings.pause_page * 0.37));
    outc += accent * border * alpha * 0.65;

    // Block-letter PAUSA hint: five luminous vertical glyph zones.
    vec2 titleBase = vec2(0.305, 0.175);
    for (int i = 0; i < 5; ++i) {
        vec2 c = titleBase + vec2(float(i) * 0.047, 0.0);
        float stemL = rect_mask(uv, c + vec2(-0.014, 0.0), vec2(0.004, 0.035), 0.002);
        float stemR = rect_mask(uv, c + vec2(0.014, 0.0), vec2(0.004, 0.035), 0.002);
        float top = rect_mask(uv, c + vec2(0.0, -0.030), vec2(0.017, 0.004), 0.002);
        float mid = rect_mask(uv, c + vec2(0.0, 0.000), vec2(0.017, 0.004), 0.002);
        float bot = rect_mask(uv, c + vec2(0.0, 0.030), vec2(0.017, 0.004), 0.002);
        float glyph = max(max(stemL, stemR), max(top, max(mid, bot)));
        if (i == 1 || i == 3) glyph = max(stemL, max(stemR, bot)); // A/U-ish
        if (i == 2) glyph = max(stemL, max(top, max(mid, bot))); // S-ish
        outc += glyph * accent * alpha * 0.95;
    }

    int page = int(clamp(settings.pause_page, 0.0, 4.0));
    for (int i = 0; i < 5; ++i) {
        float x = 0.22 + float(i) * 0.14;
        float is_active = i == page ? 1.0 : 0.25;
        float tab = rect_mask(uv, vec2(x, 0.255), vec2(0.052, 0.012), 0.003);
        outc = mix(outc, accent, tab * alpha * is_active);
    }

    int rows = int(clamp(settings.pause_row_count, 1.0, 13.0));
    int selected = int(clamp(settings.pause_selected, 0.0, float(rows - 1)));
    for (int row = 0; row < 13; ++row) {
        if (row >= rows) continue;
        float y = 0.315 + float(row) * 0.043;
        float selectedMask = row == selected ? 1.0 : 0.0;
        float rail = rect_mask(uv, vec2(0.50, y), vec2(0.255, 0.010), 0.003);
        float value = pause_row_value(page, row);
        float fillCenter = 0.245 + value * 0.255;
        float fill = rect_mask(uv, vec2(fillCenter, y), vec2(value * 0.255, 0.008), 0.003);
        float knob = rect_mask(uv, vec2(0.245 + value * 0.510, y), vec2(0.006, 0.017), 0.003);
        float leftTick = rect_mask(uv, vec2(0.185, y), vec2(0.010, 0.010), 0.002);

        outc = mix(outc, vec3(0.10, 0.12, 0.18), rail * alpha * (0.82 + selectedMask * 0.18));
        outc = mix(outc, accent, fill * alpha * (0.78 + selectedMask * 0.18));
        outc += knob * vec3(0.95, 0.98, 1.0) * alpha * (0.55 + selectedMask * 0.35);
        outc += leftTick * accent * alpha * (0.30 + value * 0.45);

        float rowFrame = rect_mask(uv, vec2(0.50, y), vec2(0.305, 0.018), 0.003)
            - rect_mask(uv, vec2(0.50, y), vec2(0.300, 0.014), 0.003);
        outc += rowFrame * accent * alpha * selectedMask * 0.55;
    }

    return clamp(outc, 0.0, 8.0);
}

void main() {
    vec2 uv = fragTexCoord;
    vec2 texelSize = 1.0 / textureSize(screenTexture, 0);

    vec3 color = vec3(0.0);

    // 1. Chromatic Aberration (Radial dispersion)
    if ((settings.effect_mask & EFFECT_CHROMATIC) != 0) {
        vec2 distVec = uv - 0.5;
        float dist = length(distVec);
        vec2 offset = distVec * dist * settings.chromatic_intensity;
        
        color.r = texture(screenTexture, uv - offset).r;
        color.g = texture(screenTexture, uv).g;
        color.b = texture(screenTexture, uv + offset).b;
    } else {
        color = texture(screenTexture, uv).rgb;
    }

    // 2. Sharpen (3x3 Laplacian edge enhancement)
    if ((settings.effect_mask & EFFECT_SHARPEN) != 0) {
        vec3 center = color;
        vec3 left   = texture(screenTexture, uv - vec2(texelSize.x, 0.0)).rgb;
        vec3 right  = texture(screenTexture, uv + vec2(texelSize.x, 0.0)).rgb;
        vec3 up     = texture(screenTexture, uv - vec2(0.0, texelSize.y)).rgb;
        vec3 down   = texture(screenTexture, uv + vec2(0.0, texelSize.y)).rgb;

        vec3 laplacian = 4.0 * center - (left + right + up + down);
        color = clamp(color + settings.sharpen_intensity * laplacian, 0.0, 10.0);
    }

    // 3. Simple Box Blur (3x3 average)
    if ((settings.effect_mask & EFFECT_BLUR) != 0) {
        vec3 accum = vec3(0.0);
        for (int y = -1; y <= 1; ++y) {
            for (int x = -1; x <= 1; ++x) {
                accum += texture(screenTexture, uv + vec2(x, y) * texelSize).rgb;
            }
        }
        color = accum / 9.0;
    }

    // 4. Bloom (GPU Compute mip-chain — Karis average + progressive upsample)
    if ((settings.effect_mask & EFFECT_BLOOM) != 0) {
        vec3 bloomColor = texture(bloomTexture, uv).rgb;
        color += bloomColor * settings.bloom_intensity;
    }

    // 5. FXAA (Fast Approximate Anti-Aliasing)
    if ((settings.effect_mask & EFFECT_FXAA) != 0) {
        vec3 rgbNW = texture(screenTexture, uv + vec2(-1.0, -1.0) * texelSize).rgb;
        vec3 rgbNE = texture(screenTexture, uv + vec2( 1.0, -1.0) * texelSize).rgb;
        vec3 rgbSW = texture(screenTexture, uv + vec2(-1.0,  1.0) * texelSize).rgb;
        vec3 rgbSE = texture(screenTexture, uv + vec2( 1.0,  1.0) * texelSize).rgb;
        vec3 rgbM  = color;
        
        float lumaNW = luminance(rgbNW);
        float lumaNE = luminance(rgbNE);
        float lumaSW = luminance(rgbSW);
        float lumaSE = luminance(rgbSE);
        float lumaM  = luminance(rgbM);
        
        float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
        float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));
        float lumaRange = lumaMax - lumaMin;
        
        // Edge check
        float edgeThreshold = 0.125;
        float edgeThresholdMin = 0.0625;
        if (lumaRange >= max(edgeThresholdMin, lumaMax * edgeThreshold)) {
            float lumaNS = lumaNW + lumaNE - lumaSW - lumaSE;
            float lumaWE = lumaNW - lumaNE + lumaSW - lumaSE;
            
            vec2 dir = vec2(-lumaNS, lumaWE);
            float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * 0.0625, 0.0001);
            float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
            
            dir = clamp(dir * rcpDirMin, vec2(-8.0), vec2(8.0)) * texelSize;
            
            vec3 rgbA = 0.5 * (
                texture(screenTexture, uv + dir * (1.0/3.0 - 0.5)).rgb +
                texture(screenTexture, uv + dir * (2.0/3.0 - 0.5)).rgb
            );
            
            vec3 rgbB = rgbA * 0.5 + 0.25 * (
                texture(screenTexture, uv + dir * -0.5).rgb +
                texture(screenTexture, uv + dir *  0.5).rgb
            );
            
            float lumaB = luminance(rgbB);
            if (lumaB < lumaMin || lumaB > lumaMax) {
                color = rgbA;
            } else {
                color = rgbB;
            }
        }
    }

    // 6. Vignette (Dark edges)
    if ((settings.effect_mask & EFFECT_VIGNETTE) != 0) {
        vec2 distVec = uv - 0.5;
        float dist = length(distVec);
        float vignette = smoothstep(settings.vignette_intensity, settings.vignette_intensity - settings.vignette_smoothness, dist);
        color *= vignette;
    }

    // 7. Screen-Space Global Illumination / GTAO
    if ((settings.effect_mask & EFFECT_SSGI) != 0) {
        color = screen_space_gi(uv, texelSize, color);
    }

    // 8. Path-tracing style multi-bounce resolve
    if ((settings.effect_mask & EFFECT_PATH_TRACED_LIGHT) != 0) {
        color = path_traced_lighting_resolve(uv, texelSize, color);
    }

    // 9. Screen-Space Reflection for wet/highlighted surfaces
    if ((settings.effect_mask & EFFECT_SSR) != 0) {
        color = screen_space_reflection(uv, texelSize, color);
    }

    // 10. Volumetric Fog and light shafts
    if ((settings.effect_mask & EFFECT_VOLUMETRIC_FOG) != 0) {
        color = volumetric_fog(uv, color);
    }

    // 11. Anamorphic neon flares
    if ((settings.effect_mask & EFFECT_ANAMORPHIC_FLARES) != 0) {
        color = anamorphic_flares(uv, texelSize, color);
    }

    // 12. Film Grain
    if ((settings.effect_mask & EFFECT_GRAIN) != 0) {
        float noise = rand(uv + settings.time * settings.grain_speed);
        color += (noise - 0.5) * settings.grain_intensity;
        color = max(color, 0.0);
    }

    // 13. LUT-style Color Grading
    if ((settings.effect_mask & EFFECT_LUT_COLOR_GRADING) != 0) {
        color = lut_color_grade(color);
    }

    color = recover_highlights(color);

    // 14. Grayscale / Sepia Color Grading
    if ((settings.effect_mask & EFFECT_GRAYSCALE) != 0) {
        float luma = luminance(color);
        color = vec3(luma);
    } else if ((settings.effect_mask & EFFECT_SEPIA) != 0) {
        float r = dot(color, vec3(0.393, 0.769, 0.189));
        float g = dot(color, vec3(0.349, 0.686, 0.168));
        float b = dot(color, vec3(0.272, 0.534, 0.131));
        color = vec3(r, g, b);
    }

    // 15. Invert Color
    if ((settings.effect_mask & EFFECT_INVERT) != 0) {
        color = 1.0 - color;
    }

    // 16. Tone Mapping & Exposure (AgX — cinematic SDR, matches Blender output)
    if ((settings.effect_mask & EFFECT_TONEMAP) != 0) {
        color *= settings.exposure;
        color = agx_tonemap(color);
        // Subtle saturation recovery post-tonemap (AgX is deliberately neutral)
        float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
        color = mix(vec3(luma), color, 1.08);
    }

    // 17. Gamma Correction
    color = pow(color, vec3(1.0 / settings.gamma));

    // 18. Anti-banding dither (reduces 8-bit quantization artifacts on SDR)
    float dither = (fract(sin(dot(gl_FragCoord.xy, vec2(12.9898, 78.233))) * 43758.5453) - 0.5) / 255.0;
    color += dither;

    color = draw_pause_overlay(uv, color);

    outColor = vec4(color, 1.0);
}
