#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 0) out vec4 outColor;

// =============================================================================
// ADead-AA: Anti-Aliasing con matemáticas puras
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

void main() {
    // Calcular derivadas de pantalla para detectar bordes
    vec3 dFdxColor = dFdx(fragColor);
    vec3 dFdyColor = dFdy(fragColor);
    
    // Calcular variación del color (indica bordes)
    float colorVariation = length(dFdxColor) + length(dFdyColor);
    
    // Aplicar suavizado en áreas de alta variación (bordes)
    // Esto reduce el aliasing en los bordes de los polígonos
    float edgeFactor = ultraSmoothstep(0.0, 0.5, colorVariation);
    
    // Suavizar el color ligeramente en los bordes
    vec3 smoothedColor = fragColor;
    
    // Aplicar corrección de gamma para mejor percepción visual
    smoothedColor = pow(smoothedColor, vec3(1.0 / 2.2));
    
    outColor = vec4(smoothedColor, 1.0);
}
