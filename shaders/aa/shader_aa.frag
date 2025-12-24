// =============================================================================
// ADead-AA: Fragment Shader con Anti-Aliasing Integrado
// =============================================================================
// Matemáticas vectoriales puras para bordes perfectos
// =============================================================================

#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec3 fragWorldPos;

layout(location = 0) out vec4 outColor;

// Push constants para AA
layout(push_constant) uniform AAParams {
    mat4 mvp;
    vec4 cameraPos;
    float edgeWidth;
    float smoothness;
    float time;
    float padding;
} params;

// =============================================================================
// FUNCIONES DE SUAVIZADO MATEMÁTICO PURO
// =============================================================================

// Smoothstep estándar (Hermite)
float smoothstepAA(float edge0, float edge1, float x) {
    float t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// Smootherstep (Quintic - Ken Perlin)
float smootherstepAA(float edge0, float edge1, float x) {
    float t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

// Ultra Smoothstep (Septic - Máxima suavidad)
float ultraSmoothstep(float edge0, float edge1, float x) {
    float t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * t * t * (t * (t * (t * -20.0 + 70.0) - 84.0) + 35.0);
}

// =============================================================================
// ANTI-ALIASING DE BORDES GEOMÉTRICOS
// =============================================================================

// Calcular factor de borde basado en la normal y dirección de vista
float calculateEdgeFactor(vec3 normal, vec3 viewDir) {
    // Fresnel-like edge detection
    float NdotV = abs(dot(normalize(normal), normalize(viewDir)));
    
    // Bordes son donde NdotV es bajo
    float edgeFactor = 1.0 - NdotV;
    
    // Aplicar suavizado ultra
    return ultraSmoothstep(0.0, params.edgeWidth, edgeFactor);
}

// Calcular derivadas de pantalla para AA
float getScreenSpaceDerivative(vec3 pos) {
    vec3 dFdxPos = dFdx(pos);
    vec3 dFdyPos = dFdy(pos);
    return length(dFdxPos) + length(dFdyPos);
}

// Anti-aliasing basado en derivadas de pantalla
vec3 applyScreenSpaceAA(vec3 color, vec3 normal) {
    // Calcular variación de la normal en pantalla
    vec3 dNdx = dFdx(normal);
    vec3 dNdy = dFdy(normal);
    float normalVariation = length(dNdx) + length(dNdy);
    
    // Suavizar color en áreas de alta variación
    float aaFactor = smootherstepAA(0.0, params.smoothness, normalVariation);
    
    // Aplicar suavizado sutil
    vec3 smoothedColor = color * (1.0 - aaFactor * 0.1);
    
    return smoothedColor;
}

// =============================================================================
// ILUMINACIÓN CON AA INTEGRADO
// =============================================================================

vec3 calculateLighting(vec3 baseColor, vec3 normal, vec3 worldPos) {
    // Dirección de luz (sol)
    vec3 lightDir = normalize(vec3(0.5, 1.0, 0.3));
    vec3 viewDir = normalize(params.cameraPos.xyz - worldPos);
    
    // Ambient
    vec3 ambient = baseColor * 0.3;
    
    // Diffuse con suavizado
    float NdotL = dot(normal, lightDir);
    // Usar smootherstep para suavizar la transición luz/sombra
    float diffuseFactor = smootherstepAA(-0.1, 0.3, NdotL);
    vec3 diffuse = baseColor * diffuseFactor * 0.7;
    
    // Specular con suavizado
    vec3 halfDir = normalize(lightDir + viewDir);
    float NdotH = max(dot(normal, halfDir), 0.0);
    float specularFactor = pow(NdotH, 32.0);
    // Suavizar el specular
    specularFactor = smootherstepAA(0.0, 1.0, specularFactor);
    vec3 specular = vec3(1.0) * specularFactor * 0.3;
    
    return ambient + diffuse + specular;
}

// =============================================================================
// MAIN
// =============================================================================

void main() {
    vec3 normal = normalize(fragNormal);
    vec3 viewDir = normalize(params.cameraPos.xyz - fragWorldPos);
    
    // Color base con iluminación
    vec3 litColor = calculateLighting(fragColor, normal, fragWorldPos);
    
    // Aplicar AA de pantalla
    litColor = applyScreenSpaceAA(litColor, normal);
    
    // Calcular factor de borde para suavizado adicional
    float edgeFactor = calculateEdgeFactor(normal, viewDir);
    
    // Mezclar con color de borde suavizado
    vec3 edgeColor = litColor * 0.9; // Bordes ligeramente más oscuros
    vec3 finalColor = mix(litColor, edgeColor, edgeFactor * 0.2);
    
    // Corrección de gamma
    finalColor = pow(finalColor, vec3(1.0 / 2.2));
    
    outColor = vec4(finalColor, 1.0);
}
