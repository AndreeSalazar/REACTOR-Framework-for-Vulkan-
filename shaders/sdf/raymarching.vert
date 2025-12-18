#version 450

// Vertex shader para fullscreen quad (ray marching)

layout(location = 0) out vec2 fragUV;

// Fullscreen triangle trick
void main() {
    // Genera triángulo que cubre toda la pantalla
    fragUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    gl_Position = vec4(fragUV * 2.0 - 1.0, 0.0, 1.0);
}
