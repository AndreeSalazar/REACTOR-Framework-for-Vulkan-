#version 450

// Fragment shader simplificado para prueba de renderizado
// Stack-GPU-OP - Vulkan Puro

layout(location = 0) in vec2 fragUV;
layout(location = 0) out vec4 outColor;

void main() {
    // Renderizar patrón de prueba animado
    vec2 uv = fragUV * 2.0 - 1.0;
    
    // Crear patrón circular que simula un cubo en perspectiva
    float dist = length(uv);
    float angle = atan(uv.y, uv.x);
    
    // Gradiente radial con variación angular
    vec3 color = vec3(0.5 + 0.3 * cos(dist * 3.0));
    color += vec3(0.2 * sin(angle * 4.0));
    
    // Agregar brillo en el centro
    color += vec3(0.3) * (1.0 - smoothstep(0.0, 0.5, dist));
    
    outColor = vec4(color, 1.0);
}
