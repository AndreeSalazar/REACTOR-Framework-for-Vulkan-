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

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

// Pseudo-random noise for Film Grain
float rand(vec2 co) {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
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

    // 7. Film Grain
    if ((settings.effect_mask & EFFECT_GRAIN) != 0) {
        float noise = rand(uv + settings.time * settings.grain_speed);
        color += (noise - 0.5) * settings.grain_intensity;
        color = max(color, 0.0);
    }

    // 8. Grayscale / Sepia Color Grading
    if ((settings.effect_mask & EFFECT_GRAYSCALE) != 0) {
        float luma = luminance(color);
        color = vec3(luma);
    } else if ((settings.effect_mask & EFFECT_SEPIA) != 0) {
        float r = dot(color, vec3(0.393, 0.769, 0.189));
        float g = dot(color, vec3(0.349, 0.686, 0.168));
        float b = dot(color, vec3(0.272, 0.534, 0.131));
        color = vec3(r, g, b);
    }

    // 9. Invert Color
    if ((settings.effect_mask & EFFECT_INVERT) != 0) {
        color = 1.0 - color;
    }

    // 10. Tone Mapping & Exposure
    if ((settings.effect_mask & EFFECT_TONEMAP) != 0) {
        color *= settings.exposure;
        color = aces_tonemap(color);
    }

    // 11. Gamma Correction
    color = pow(color, vec3(1.0 / settings.gamma));

    outColor = vec4(color, 1.0);
}
