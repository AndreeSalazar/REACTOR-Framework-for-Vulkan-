#version 450
// =============================================================================
// REACTOR · shaders/post/decal.frag — MRT Screen-Space Decal Shader
// =============================================================================
// Proyecta texturas de albedo, normales y materiales sobre la geometría del
// G-Buffer en espacio de pantalla escribiendo en múltiples attachments.
// =============================================================================

layout(location = 0) out vec4 outAlbedoAo;
layout(location = 1) out vec4 outNormalMaterial;

layout(set = 0, binding = 0) uniform sampler2D depthTexture;
layout(set = 0, binding = 1) uniform sampler2D decalAlbedoTexture;
layout(set = 0, binding = 2) uniform sampler2D decalNormalTexture;
layout(set = 0, binding = 3) uniform sampler2D decalMaterialTexture; // R = metallic, G = roughness

layout(push_constant) uniform DecalPushConstants {
    mat4 mvp;             // Matriz MVP para el Vertex Shader (ShadowVert)
    mat4 view_proj_inv;   // Matriz inversa de vista-proyección de la cámara
    mat4 decal_world_inv; // Matriz inversa del transform del decal (mundo → decal local)
    vec4 decal_color;     // Color de tinte + opacidad (.w)
    vec4 decal_params;    // .x = normal_strength, .y = metallic, .z = roughness, .w = blend_mode
} push;

vec2 encode_octahedral(vec3 n) {
    n /= abs(n.x) + abs(n.y) + abs(n.z) + 1e-6;
    vec2 enc = n.xy;
    if (n.z < 0.0) {
        enc = (1.0 - abs(enc.yx)) * sign(enc.xy);
    }
    return enc * 0.5 + 0.5;
}

void main() {
    // 1. Obtener coordenadas de pantalla normalized [0, 1]
    vec2 screen_size = vec2(textureSize(depthTexture, 0));
    vec2 uv = gl_FragCoord.xy / screen_size;

    // 2. Leer profundidad
    float depth = texture(depthTexture, uv).r;

    // 3. Reconstruir posición en Clip Space y transformar a World Space
    vec4 clip_pos = vec4(uv * 2.0 - 1.0, depth, 1.0);
    vec4 world_pos = push.view_proj_inv * clip_pos;
    world_pos /= world_pos.w;

    // 4. Transformar posición del mundo a espacio local del Decal OBB [-0.5, 0.5]
    vec4 local_pos = push.decal_world_inv * world_pos;

    // 5. Descartar si cae fuera del volumen del decal
    if (abs(local_pos.x) > 0.5 || abs(local_pos.y) > 0.5 || abs(local_pos.z) > 0.5) {
        discard;
    }

    // 6. Mapear coordenadas locales [-0.5, 0.5] a UV [0, 1]
    vec2 decal_uv = local_pos.xy + 0.5;

    // 7. Muestrear texturas
    vec4 albedo_sample = texture(decalAlbedoTexture, decal_uv) * push.decal_color;
    if (albedo_sample.a < 0.01) {
        discard;
    }

    // 8. Reconstruir normales perturbadas del decal
    // Extraemos los ejes del mundo a partir de las columnas (filas de la inversa de rotación)
    // de la matriz decal_world_inv para evitar exceder los límites de push constants
    vec3 tangent   = normalize(vec3(push.decal_world_inv[0].x, push.decal_world_inv[1].x, push.decal_world_inv[2].x));
    vec3 bitangent = normalize(vec3(push.decal_world_inv[0].y, push.decal_world_inv[1].y, push.decal_world_inv[2].y));
    vec3 normal    = normalize(vec3(push.decal_world_inv[0].z, push.decal_world_inv[1].z, push.decal_world_inv[2].z));
    mat3 tbn = mat3(tangent, bitangent, normal);

    // Muestrear normal en espacio tangente y mezclar con la normal por defecto
    vec3 normal_tangent = texture(decalNormalTexture, decal_uv).xyz * 2.0 - 1.0;
    vec3 perturbed_normal = normalize(tbn * normal_tangent);
    vec3 final_normal = mix(normal, perturbed_normal, clamp(push.decal_params.x, 0.0, 1.0));

    // Muestrear propiedades metálicas y rugosidad
    vec4 mat_sample = texture(decalMaterialTexture, decal_uv);
    float metallic  = mix(push.decal_params.y, mat_sample.r, mat_sample.a);
    float roughness = mix(push.decal_params.z, mat_sample.g, mat_sample.a);

    // Escribir outputs con el alpha del decal como factor de mezcla para blending
    outAlbedoAo = vec4(albedo_sample.rgb, albedo_sample.a);
    outNormalMaterial = vec4(encode_octahedral(final_normal), metallic, roughness);
}
