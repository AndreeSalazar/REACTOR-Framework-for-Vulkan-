// =============================================================================
// REACTOR ⇄ Blender Live Link — Vertex shader (PBR-ready)
// =============================================================================
// Pasa al fragment: normal mundial, posición mundial, UV, color de material y
// dirección de vista. El TBN aproximado para detalle se deriva en el fragment
// con dFdx/dFdy (suficiente sin normal map).
// =============================================================================
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 vWorldNormal;
layout(location = 1) out vec2 vUV;
layout(location = 2) out vec3 vWorldPos;
layout(location = 3) out vec4 vColor;
layout(location = 4) out vec3 vViewDir;

layout(push_constant) uniform Constants {
    mat4 mvp;
    mat4 model;
    vec4 camera_pos;
    vec4 light_pos;
    vec4 color;
} push;

void main() {
    vec4 wp = push.model * vec4(position, 1.0);
    gl_Position = push.mvp * vec4(position, 1.0);

    // Para escala uniforme `mat3(model)` basta. Si Blender empieza a enviar
    // escalas no uniformes habrá que pasar la inversa traspuesta del modelo.
    vWorldNormal = normalize(mat3(push.model) * normal);
    vWorldPos    = wp.xyz;
    vUV          = uv;
    vColor       = push.color;
    vViewDir     = push.camera_pos.xyz - wp.xyz;
}
