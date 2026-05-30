#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform sampler2D screenTexture;

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
    uint effect_mask;
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

// ACES Filmic Tone Mapping curve
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

vec3 estimate_screen_normal(vec2 uv, vec2 texelSize) {
    float l = luminance(sample_screen(uv - vec2(texelSize.x, 0.0)));
    float r = luminance(sample_screen(uv + vec2(texelSize.x, 0.0)));
    float u = luminance(sample_screen(uv - vec2(0.0, texelSize.y)));
    float d = luminance(sample_screen(uv + vec2(0.0, texelSize.y)));
    vec2 g = vec2(r - l, d - u);
    return normalize(vec3(-g * 2.4, 1.0));
}

vec3 screen_space_gi(vec2 uv, vec2 texelSize, vec3 baseColor) {
    vec3 normal = estimate_screen_normal(uv, texelSize);
    float baseLum = luminance(baseColor);
    vec3 diffuseBounce = vec3(0.0);
    vec3 directionalOcclusion = vec3(0.0);
    float totalWeight = 0.0;

    const int RINGS = 3;
    const int DIRS = 8;
    float jitter = interleaved_gradient_noise(gl_FragCoord.xy, 0.0) * 0.35;

    for (int ring = 1; ring <= RINGS; ++ring) {
        float ringRadius = settings.ssgi_radius * float(ring) / float(RINGS);
        for (int i = 0; i < DIRS; ++i) {
            float a = (float(i) + jitter) * 6.2831853 / float(DIRS);
            vec2 dir = vec2(cos(a), sin(a));
            vec2 suv = uv + dir * texelSize * ringRadius;
            vec3 s = sample_screen(suv);
            float sLum = luminance(s);

            float edge = abs(sLum - baseLum);
            float edgeWeight = exp(-edge * 5.0);
            float hemisphere = max(dot(normal, normalize(vec3(dir, 0.55))), 0.0);
            float distanceWeight = 1.0 / (1.0 + float(ring) * float(ring));
            float w = edgeWeight * hemisphere * distanceWeight;

            diffuseBounce += s * w;
            directionalOcclusion += vec3(sLum < baseLum ? w : 0.0);
            totalWeight += w;
        }
    }

    vec3 bounce = diffuseBounce / max(totalWeight, 0.0001);
    float ao = 1.0 - clamp(dot(directionalOcclusion, vec3(0.333)) / max(totalWeight, 0.0001), 0.0, 0.75);
    vec3 gi = mix(baseColor * ao, baseColor + bounce * 0.28, settings.ssgi_intensity);
    return max(gi, 0.0);
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
        float active = i == page ? 1.0 : 0.25;
        float tab = rect_mask(uv, vec2(x, 0.255), vec2(0.052, 0.012), 0.003);
        outc = mix(outc, accent, tab * alpha * active);
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

    // 4. Bloom (simulated light bleed from bright areas)
    if ((settings.effect_mask & EFFECT_BLOOM) != 0) {
        // Sample surrounding bright areas
        vec3 bloomAccum = vec3(0.0);
        float samplesCount = 0.0;
        
        // Dynamic search area based on blur size setting
        int radius = int(clamp(settings.bloom_blur_size, 1.0, 6.0));
        for (int y = -radius; y <= radius; y += 2) {
            for (int x = -radius; x <= radius; x += 2) {
                vec3 sampleColor = texture(screenTexture, uv + vec2(x, y) * texelSize).rgb;
                float luma = luminance(sampleColor);
                if (luma > settings.bloom_threshold) {
                    float weight = 1.0 - (length(vec2(x, y)) / (float(radius) + 1.0));
                    bloomAccum += sampleColor * weight;
                    samplesCount += weight;
                }
            }
        }
        
        if (samplesCount > 0.0) {
            color += (bloomAccum / samplesCount) * settings.bloom_intensity;
        }
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

    // 16. Tone Mapping & Exposure
    if ((settings.effect_mask & EFFECT_TONEMAP) != 0) {
        color *= settings.exposure;
        color = aces_tonemap(color);
    }

    // 17. Gamma Correction
    color = pow(color, vec3(1.0 / settings.gamma));
    color = draw_pause_overlay(uv, color);

    outColor = vec4(color, 1.0);
}
