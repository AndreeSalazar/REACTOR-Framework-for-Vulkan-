#version 450

// Vertex shader para cubo 3D con Phong shading
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec3 inColor;

layout(location = 0) out vec3 fragWorldPos;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec3 fragColor;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
} push;

void main() {
    // Transform position
    vec4 worldPos = push.model * vec4(inPosition, 1.0);
    fragWorldPos = worldPos.xyz;
    
    // Transform normal (use model matrix, assuming uniform scale)
    fragNormal = mat3(push.model) * inNormal;
    
    // Pass color
    fragColor = inColor;
    
    // Final position
    gl_Position = push.mvp * vec4(inPosition, 1.0);
}
