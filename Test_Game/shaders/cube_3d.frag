#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragNormal;
layout(location = 0) out vec4 outColor;

void main() {
    // Usar color del vértice directamente (cada cara tiene su color)
    outColor = vec4(fragColor, 1.0);
}
