#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec2 fragUV;
layout(location = 2) out vec3 fragPos;
layout(location = 3) out vec4 fragColor;

layout(push_constant) uniform Constants {
    mat4 mvp;
    mat4 model;
    vec4 camera_pos;
    vec4 light_pos;
    vec4 color;
} push;

void main() {
    gl_Position = push.mvp * vec4(position, 1.0);
    fragNormal = normalize(mat3(push.model) * normal);
    fragPos = vec3(push.model * vec4(position, 1.0));
    fragUV = uv;
    fragColor = push.color;
}
