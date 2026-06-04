#version 450
// =============================================================================
// REACTOR · shaders/particles/particle.vert — Billboard Particle Vertex Shader
// =============================================================================
// Expande partículas de puntos a quads orientados a cámara (billboards).
// Lee datos del SSBO de partículas (posición, vida, color, tamaño).
// Soporta rotación por velocidad y escalado por vida.
// =============================================================================

// Particle data from compute simulation SSBO
struct Particle {
    vec4 position_life;   // xyz = world pos, w = life [0..1] (0=dead, 1=just born)
    vec4 velocity_size;   // xyz = velocity,   w = size (world units)
    vec4 color;           // rgba color (premultiplied alpha)
};

layout(std430, set = 0, binding = 0) readonly buffer ParticleBuffer {
    Particle particles[];
};

layout(push_constant) uniform ParticlePushConstants {
    mat4 view_proj;
    vec4 camera_right;    // camera right vector in world space
    vec4 camera_up;       // camera up vector in world space
    uint particle_count;
    float _pad0;
    float _pad1;
    float _pad2;
} push;

layout(location = 0) out vec2 vUV;
layout(location = 1) out vec4 vColor;
layout(location = 2) out float vLife;

void main() {
    // Cada partícula genera 6 vértices (2 triángulos = 1 quad)
    uint particle_id = gl_VertexIndex / 6;
    uint vertex_id   = gl_VertexIndex % 6;

    if (particle_id >= push.particle_count) {
        gl_Position = vec4(0.0);
        return;
    }

    Particle p = particles[particle_id];

    // Skip dead particles
    if (p.position_life.w <= 0.0) {
        gl_Position = vec4(0.0);
        return;
    }

    // Quad corners: 2 triángulos formando un quad
    //   0--1    Triángulo 1: 0,1,2
    //   | /|    Triángulo 2: 2,1,3
    //   |/ |
    //   2--3
    vec2 corners[6] = vec2[](
        vec2(-0.5,  0.5),  // 0: top-left
        vec2( 0.5,  0.5),  // 1: top-right
        vec2(-0.5, -0.5),  // 2: bottom-left
        vec2(-0.5, -0.5),  // 2: bottom-left
        vec2( 0.5,  0.5),  // 1: top-right
        vec2( 0.5, -0.5)   // 3: bottom-right
    );

    vec2 corner = corners[vertex_id];
    vUV = corner + 0.5; // [0,1] range

    // Billboard expansion: expand point to camera-facing quad
    float size = p.velocity_size.w;
    
    // Fade size near death (last 20% of life)
    float life = p.position_life.w;
    float size_fade = smoothstep(0.0, 0.2, life);
    size *= size_fade;

    // Opcional: rotación basada en velocidad para partículas de chispas
    vec3 right = push.camera_right.xyz;
    vec3 up    = push.camera_up.xyz;

    // Si la partícula tiene velocidad significativa, orientar el billboard
    // a lo largo del vector de velocidad (streak effect para chispas/lluvia)
    vec3 vel = p.velocity_size.xyz;
    float speed = length(vel);
    if (speed > 0.5) {
        vec3 vel_dir = vel / speed;
        // Proyectar velocidad al plano de la cámara
        vec3 vel_screen = vel_dir - dot(vel_dir, cross(right, up)) * cross(right, up);
        float vel_len = length(vel_screen);
        if (vel_len > 0.01) {
            vec3 streak_dir = vel_screen / vel_len;
            vec3 streak_perp = cross(streak_dir, cross(right, up));
            right = streak_dir;
            up = normalize(streak_perp);
        }
    }

    vec3 world_pos = p.position_life.xyz
                   + right * corner.x * size
                   + up    * corner.y * size;

    gl_Position = push.view_proj * vec4(world_pos, 1.0);
    vColor = p.color;
    vLife = life;
}
