// =============================================================================
// ADead-AA: SMAA Neighborhood Blending Fragment Shader
// =============================================================================
// Paso final de SMAA: Mezclar colores basado en pesos calculados
// Elimina bordes dentados con suavizado de alta calidad
// =============================================================================

#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform sampler2D colorTex;
layout(binding = 1) uniform sampler2D blendTex;

layout(push_constant) uniform SMAAParams {
    vec2 texelSize;
    float blendFactor;
    float padding;
} params;

// =============================================================================
// FUNCIONES DE SUAVIZADO ULTRA
// =============================================================================

// Smootherstep (quintic) - más suave que smoothstep
float smootherstep(float edge0, float edge1, float x) {
    float t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

// Ultra smoothstep (septic) - máxima suavidad
float ultraSmoothstep(float edge0, float edge1, float x) {
    float t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * t * t * (t * (t * (t * -20.0 + 70.0) - 84.0) + 35.0);
}

// =============================================================================
// BLENDING PRINCIPAL
// =============================================================================

void main() {
    vec2 uv = fragTexCoord;
    
    // Obtener pesos de mezcla
    vec4 blend = texture(blendTex, uv);
    
    // Si no hay mezcla, usar color original
    if (dot(blend, vec4(1.0)) < 0.0001) {
        outColor = texture(colorTex, uv);
        return;
    }
    
    // Calcular offsets basados en pesos
    vec4 offset;
    offset.xy = vec2(blend.r, blend.g) * params.texelSize;
    offset.zw = vec2(blend.b, blend.a) * params.texelSize;
    
    // Muestrear colores vecinos
    vec4 colorC = texture(colorTex, uv);
    vec4 colorL = texture(colorTex, uv - vec2(params.texelSize.x, 0.0));
    vec4 colorR = texture(colorTex, uv + vec2(params.texelSize.x, 0.0));
    vec4 colorT = texture(colorTex, uv - vec2(0.0, params.texelSize.y));
    vec4 colorB = texture(colorTex, uv + vec2(0.0, params.texelSize.y));
    
    // Calcular pesos normalizados
    float weightSum = blend.r + blend.g + blend.b + blend.a;
    vec4 weights = blend / max(weightSum, 0.0001);
    
    // Mezclar colores con suavizado ultra
    vec4 blendedColor = colorC;
    
    // Aplicar mezcla direccional
    blendedColor = mix(blendedColor, colorL, ultraSmoothstep(0.0, 1.0, weights.r));
    blendedColor = mix(blendedColor, colorR, ultraSmoothstep(0.0, 1.0, weights.g));
    blendedColor = mix(blendedColor, colorT, ultraSmoothstep(0.0, 1.0, weights.b));
    blendedColor = mix(blendedColor, colorB, ultraSmoothstep(0.0, 1.0, weights.a));
    
    // Mezcla final con factor configurable
    outColor = mix(colorC, blendedColor, params.blendFactor);
}
