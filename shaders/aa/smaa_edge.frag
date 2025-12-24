// =============================================================================
// ADead-AA: SMAA Edge Detection Fragment Shader
// =============================================================================
// Paso 1 de SMAA: Detectar bordes usando luminancia y color
// =============================================================================

#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out vec4 outEdges;

layout(binding = 0) uniform sampler2D colorTex;

layout(push_constant) uniform SMAAParams {
    vec2 texelSize;
    float threshold;     // 0.1 default
    float depthThreshold; // 0.01 default
} params;

// Umbral para detección de bordes
#define SMAA_THRESHOLD 0.1

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

void main() {
    vec2 uv = fragTexCoord;
    
    // Muestrear luminancias
    float L = luminance(texture(colorTex, uv).rgb);
    float Lleft = luminance(texture(colorTex, uv + vec2(-1.0, 0.0) * params.texelSize).rgb);
    float Ltop = luminance(texture(colorTex, uv + vec2(0.0, -1.0) * params.texelSize).rgb);
    
    // Calcular deltas
    vec2 delta;
    delta.x = abs(L - Lleft);
    delta.y = abs(L - Ltop);
    
    // Detectar bordes
    vec2 edges = step(params.threshold, delta);
    
    // Si no hay bordes, descartar
    if (dot(edges, vec2(1.0)) == 0.0) {
        discard;
    }
    
    // Muestrear más vecinos para refinar
    float Lright = luminance(texture(colorTex, uv + vec2(1.0, 0.0) * params.texelSize).rgb);
    float Lbottom = luminance(texture(colorTex, uv + vec2(0.0, 1.0) * params.texelSize).rgb);
    
    // Calcular deltas adicionales
    float deltaRight = abs(L - Lright);
    float deltaBottom = abs(L - Lbottom);
    
    // Máximo local para mejor detección
    float maxDeltaX = max(delta.x, deltaRight);
    float maxDeltaY = max(delta.y, deltaBottom);
    
    // Refinar bordes
    edges.x = step(params.threshold, maxDeltaX);
    edges.y = step(params.threshold, maxDeltaY);
    
    outEdges = vec4(edges, 0.0, 1.0);
}
