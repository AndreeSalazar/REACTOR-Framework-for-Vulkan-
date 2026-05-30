// =============================================================================
// REACTOR ⇄ Blender Live Link — Vertex shader (PBR-ready)
// =============================================================================
// Pasa al fragment: normal mundial, posición mundial, UV y color de material.
// No usa tangentes (no hacemos normal mapping), pero el fragment puede derivar
// un frame TBN aproximado vía derivadas de pantalla para detalle procedural.
// =============================================================================
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 vWorldNormal;
layout(location = 1) out vec2 vUV;
layout(location = 2) out vec3 vWorldPos;
layout(location = 3) out vec4 vColor;
layout(location = 4) out vec3 vViewDir; // V = camera - pos (world space)

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

    // Normal mundial — usamos mat3(model). Asume escala uniforme; si en el
    // futuro Blender envía escalas no uniformes hay que pasar la inversa
    // traspuesta del modelo en otro push-constant.
    vWorldNormal = normalize(mat3(push.model) * normal);
    vWorldPos    = wp.xyz;
    vUV          = uv;
    vColor       = push.color;
    vViewDir     = push.camera_pos.xyz - wp.xyz;
}
