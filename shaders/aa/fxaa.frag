// =============================================================================
// ADead-AA: FXAA (Fast Approximate Anti-Aliasing) Fragment Shader
// =============================================================================
// Implementación de FXAA 3.11 optimizada para REACTOR
// Elimina bordes dentados en post-proceso
// =============================================================================

#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform sampler2D screenTexture;

layout(push_constant) uniform FXAAParams {
    vec2 texelSize;      // 1.0 / screenResolution
    float edgeThreshold; // 0.125 default
    float edgeThresholdMin; // 0.0625 default
} params;

// =============================================================================
// FUNCIONES DE LUMINANCIA
// =============================================================================

float luminance(vec3 color) {
    return dot(color, vec3(0.299, 0.587, 0.114));
}

float luminanceLinear(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

// =============================================================================
// FXAA CORE
// =============================================================================

vec4 fxaa(sampler2D tex, vec2 uv, vec2 texelSize) {
    // Muestrear píxeles vecinos
    vec3 rgbNW = texture(tex, uv + vec2(-1.0, -1.0) * texelSize).rgb;
    vec3 rgbNE = texture(tex, uv + vec2( 1.0, -1.0) * texelSize).rgb;
    vec3 rgbSW = texture(tex, uv + vec2(-1.0,  1.0) * texelSize).rgb;
    vec3 rgbSE = texture(tex, uv + vec2( 1.0,  1.0) * texelSize).rgb;
    vec3 rgbM  = texture(tex, uv).rgb;
    
    // Calcular luminancia
    float lumaNW = luminance(rgbNW);
    float lumaNE = luminance(rgbNE);
    float lumaSW = luminance(rgbSW);
    float lumaSE = luminance(rgbSE);
    float lumaM  = luminance(rgbM);
    
    // Encontrar rango de luminancia
    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));
    float lumaRange = lumaMax - lumaMin;
    
    // Si el contraste es bajo, no aplicar AA
    if (lumaRange < max(params.edgeThresholdMin, lumaMax * params.edgeThreshold)) {
        return vec4(rgbM, 1.0);
    }
    
    // Calcular dirección del borde
    float lumaNS = lumaNW + lumaNE - lumaSW - lumaSE;
    float lumaWE = lumaNW - lumaNE + lumaSW - lumaSE;
    
    vec2 dir;
    dir.x = -lumaNS;
    dir.y = lumaWE;
    
    // Normalizar dirección
    float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * 0.25 * 0.25, 0.0001);
    float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
    
    dir = clamp(dir * rcpDirMin, vec2(-8.0), vec2(8.0)) * texelSize;
    
    // Muestrear a lo largo del borde
    vec3 rgbA = 0.5 * (
        texture(tex, uv + dir * (1.0/3.0 - 0.5)).rgb +
        texture(tex, uv + dir * (2.0/3.0 - 0.5)).rgb
    );
    
    vec3 rgbB = rgbA * 0.5 + 0.25 * (
        texture(tex, uv + dir * -0.5).rgb +
        texture(tex, uv + dir *  0.5).rgb
    );
    
    float lumaB = luminance(rgbB);
    
    // Elegir resultado final
    if (lumaB < lumaMin || lumaB > lumaMax) {
        return vec4(rgbA, 1.0);
    } else {
        return vec4(rgbB, 1.0);
    }
}

// =============================================================================
// MAIN
// =============================================================================

void main() {
    outColor = fxaa(screenTexture, fragTexCoord, params.texelSize);
}
