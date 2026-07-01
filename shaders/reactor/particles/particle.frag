#version 450
// =============================================================================
// REACTOR · shaders/particles/particle.frag — Soft Particle Fragment Shader
// =============================================================================
// Renderiza partículas con:
//   • Soft depth-fade (evita cortes duros contra geometría)
//   • Gradiente circular (punto → borde transparente)
//   • Iluminación simple hemisférica
//   • Soporte para modo aditivo (fuego/chispas) y alpha-over (humo/nieve)
// =============================================================================

layout(location = 0) in vec2 vUV;
layout(location = 1) in vec4 vColor;
layout(location = 2) in float vLife;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler2D depthTexture;

layout(push_constant) uniform ParticlePushConstants {
    mat4 view_proj;
    vec4 camera_right;
    vec4 camera_up;
    uint particle_count;
    float depth_near;
    float depth_far;
    float soft_depth_range;  // rango de fade suave (típ. 0.5-2.0 world units)
} push;

// Linearizar profundidad de Vulkan [0,1] reverse-Z
float linearize_depth(float d) {
    float z_near = push.depth_near;
    float z_far  = push.depth_far;
    return z_near * z_far / (z_far - d * (z_far - z_near));
}

void main() {
    // ── Gradiente circular — punto suave en el centro ──
    vec2 center = vUV - 0.5;
    float dist = length(center) * 2.0;
    
    // Soft circle con falloff cuadrático
    float circle_alpha = 1.0 - smoothstep(0.0, 1.0, dist);
    circle_alpha *= circle_alpha; // Cuadrático para bordes más suaves

    if (circle_alpha < 0.001) discard;

    // ── Soft particle depth fade ──
    // Evita cortes duros cuando la partícula intersecta geometría
    vec2 screen_uv = gl_FragCoord.xy / vec2(textureSize(depthTexture, 0));
    float scene_depth = linearize_depth(texture(depthTexture, screen_uv).r);
    float particle_depth = linearize_depth(gl_FragCoord.z);
    
    float depth_diff = scene_depth - particle_depth;
    float soft_factor = smoothstep(0.0, push.soft_depth_range, depth_diff);

    // ── Life-based alpha fade ──
    // Fade in al nacer (primer 10%) y fade out al morir (último 30%)
    float life_fade = smoothstep(0.0, 0.1, vLife) * smoothstep(0.0, 0.3, vLife);

    // ── Iluminación hemisférica simple ──
    // Las partículas reciben luz ambiental hemisférica (arriba = cielo, abajo = suelo)
    // No calculamos normales reales — usamos la UV como proxy de orientación
    float hemisphere_light = mix(0.6, 1.0, vUV.y); // Más brillante arriba

    // ── Color final ──
    vec3 final_color = vColor.rgb * hemisphere_light;
    float final_alpha = vColor.a * circle_alpha * soft_factor * life_fade;

    // Clamp para evitar valores negativos
    outColor = vec4(max(final_color, vec3(0.0)), clamp(final_alpha, 0.0, 1.0));
}
