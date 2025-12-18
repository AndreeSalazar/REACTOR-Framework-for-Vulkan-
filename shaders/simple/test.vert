#version 450

// Vertex shader simple - genera triángulo fullscreen
layout(location = 0) out vec3 fragColor;

void main() {
    // Fullscreen triangle
    vec2 positions[3] = vec2[](
        vec2(-1.0, -1.0),
        vec2( 3.0, -1.0),
        vec2(-1.0,  3.0)
    );
    
    // Colores para cada vértice
    vec3 colors[3] = vec3[](
        vec3(1.0, 0.0, 0.0),  // Rojo
        vec3(0.0, 1.0, 0.0),  // Verde
        vec3(0.0, 0.0, 1.0)   // Azul
    );
    
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    fragColor = colors[gl_VertexIndex];
}
