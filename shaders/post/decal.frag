#version 450
// =============================================================================
// REACTOR · shaders/post/decal.frag — Screen-Space Decal Projection Shader
// =============================================================================
// Reconstruye la posición del fragmento en el mundo usando el depth buffer,
// la proyecta en el volumen local del Decal (OBB), y dibuja sobre el G-Buffer.
// =============================================================================

layout(location = 0) out vec4 outAlbedo;

layout(set = 0, binding = 0) uniform sampler2D depthTexture;
layout(set = 0, binding = 1) uniform sampler2D decalAlbedoTexture;

layout(push_constant) uniform DecalPushConstants {
    mat4 view_proj_inv;   // Matriz inversa de vista-proyección de la cámara
    mat4 decal_world_inv; // Matriz inversa del transform del decal (mundo → decal local)
    vec4 decal_color;     // Color de tinte + opacidad global (.w)
} push;

void main() {
    // 1. Obtener coordenadas de pantalla normalized [0, 1]
    // Reconstruidas dinámicamente según el viewport de renderizado
    vec2 screen_size = vec2(textureSize(depthTexture, 0));
    vec2 uv = gl_FragCoord.xy / screen_size;

    // 2. Leer profundidad no lineal del buffer
    float depth = texture(depthTexture, uv).r;

    // 3. Reconstruir posición en Clip Space y transformar a World Space
    vec4 clip_pos = vec4(uv * 2.0 - 1.0, depth, 1.0);
    vec4 world_pos = push.view_proj_inv * clip_pos;
    world_pos /= world_pos.w;

    // 4. Transformar posición del mundo a espacio local del Decal OBB [-0.5, 0.5]
    vec4 local_pos = push.decal_world_inv * world_pos;

    // 5. Descartar fragmento si cae fuera del volumen del cubo del Decal
    if (abs(local_pos.x) > 0.5 || abs(local_pos.y) > 0.5 || abs(local_pos.z) > 0.5) {
        discard;
    }

    // 6. Mapear coordenadas locales [-0.5, 0.5] a coordenadas UV de textura [0, 1]
    vec2 decal_uv = local_pos.xy + 0.5;

    // 7. Muestrear textura de decal con tinte de color
    vec4 col = texture(decalAlbedoTexture, decal_uv) * push.decal_color;

    // 8. Descartar si el alpha es transparente
    if (col.a < 0.01) {
        discard;
    }

    outAlbedo = col;
}
