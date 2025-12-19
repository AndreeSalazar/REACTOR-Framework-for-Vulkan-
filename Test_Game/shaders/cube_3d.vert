#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
} push;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec3 fragNormal;

void main() {
    gl_Position = push.mvp * vec4(inPosition, 1.0);
    
    // Calcular normal basada en la posición del vértice (para cubo centrado en origen)
    vec3 absPos = abs(inPosition);
    vec3 normal = vec3(0.0);
    if (absPos.x > absPos.y && absPos.x > absPos.z) {
        normal = vec3(sign(inPosition.x), 0.0, 0.0);
    } else if (absPos.y > absPos.z) {
        normal = vec3(0.0, sign(inPosition.y), 0.0);
    } else {
        normal = vec3(0.0, 0.0, sign(inPosition.z));
    }
    
    fragNormal = normal;
    fragColor = inColor;
}
