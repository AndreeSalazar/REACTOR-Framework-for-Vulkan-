#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

void main() {
    // Escalar y centrar el cubo para que sea visible
    vec3 pos = inPosition * 0.3; // Más pequeño
    gl_Position = vec4(pos.x, -pos.y, pos.z * 0.5 + 0.5, 1.0); // Invertir Y, ajustar Z
    fragColor = inColor;
}
