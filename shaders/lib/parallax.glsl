// =============================================================================
// REACTOR · shaders/lib/parallax.glsl — Parallax Occlusion Mapping (POM)
// =============================================================================
// Ray-march del heightmap en espacio tangente para simular profundidad
// geométrica en superficies planas:
//   • Muros de ladrillo, adoquines, terreno rocoso, paneles metálicos
//   • Auto-sombra interna (self-shadowing) basada en dirección de luz
//   • Linear search (8 steps) + binary refinement (5 steps)
//
// Referencia: Cyberpunk 2077, The Last of Us Part II, UE5 Material Nodes
// =============================================================================
#ifndef REACTOR_LIB_PARALLAX
#define REACTOR_LIB_PARALLAX

// ── Parallax Occlusion Mapping ───────────────────────────────────────────────
// Desplaza las UVs basándose en el heightmap para simular profundidad.
//
// Parámetros:
//   uv            — coordenadas UV originales
//   view_dir_ts   — dirección de vista en espacio tangente (TBN^T * V)
//   height_scale  — escala de profundidad (típ. 0.02-0.08)
//   height_map    — sampler del heightmap (canal R, 0=fondo, 1=superficie)
//   min_layers    — mínimo de pasos de búsqueda (ej. 8)
//   max_layers    — máximo de pasos de búsqueda (ej. 32)
//
// Retorna:
//   vec2 — UVs desplazadas para usar en todos los muestreos de textura

vec2 parallax_occlusion_map(vec2 uv, vec3 view_dir_ts, float height_scale,
                             sampler2D height_map, float min_layers, float max_layers) {
    // Más capas cuando la vista es rasante (mayor parallax visible)
    float num_layers = mix(max_layers, min_layers, max(dot(vec3(0.0, 0.0, 1.0), view_dir_ts), 0.0));
    
    float layer_depth = 1.0 / num_layers;
    float current_layer_depth = 0.0;
    
    // Dirección de desplazamiento UV por capa (escalada por profundidad)
    vec2 p = view_dir_ts.xy / max(view_dir_ts.z, 0.001) * height_scale;
    vec2 delta_uv = p / num_layers;
    
    // ── Linear search: buscar la primera intersección ──
    vec2  current_uv = uv;
    float current_height = texture(height_map, current_uv).r;
    
    for (float i = 0.0; i < 64.0; i += 1.0) {
        if (i >= num_layers) break;
        if (current_layer_depth >= current_height) break;
        
        current_uv -= delta_uv;
        current_height = texture(height_map, current_uv).r;
        current_layer_depth += layer_depth;
    }
    
    // ── Binary refinement: refinar la intersección ──
    vec2 prev_uv = current_uv + delta_uv;
    float prev_layer_depth = current_layer_depth - layer_depth;
    
    // Profundidades relativas para interpolación
    float after_depth  = current_height - current_layer_depth;
    float before_depth = texture(height_map, prev_uv).r - prev_layer_depth;
    
    // 5 pasos de refinamiento binario
    for (int j = 0; j < 5; j++) {
        vec2 mid_uv = (current_uv + prev_uv) * 0.5;
        float mid_height = texture(height_map, mid_uv).r;
        float mid_layer = (current_layer_depth + prev_layer_depth) * 0.5;
        
        if (mid_height > mid_layer) {
            // La intersección está más profunda
            prev_uv = mid_uv;
            prev_layer_depth = mid_layer;
        } else {
            // La intersección está más arriba
            current_uv = mid_uv;
            current_layer_depth = mid_layer;
        }
    }
    
    // Interpolación lineal final entre los dos puntos más cercanos
    float weight = after_depth / (after_depth - before_depth + 0.0001);
    return mix(current_uv, prev_uv, weight);
}

// ── Self-Shadowing por Parallax ──────────────────────────────────────────────
// Calcula si un punto está en sombra propia del relieve del heightmap.
// Se traza un rayo desde el punto POM hacia la luz en espacio tangente.
//
// Parámetros:
//   uv             — UVs ya desplazadas por POM
//   light_dir_ts   — dirección de la luz en espacio tangente
//   height_scale   — misma escala que se usó en parallax_occlusion_map
//   height_map     — sampler del heightmap
//   initial_height — altura del punto donde se calculó el POM
//
// Retorna:
//   float — factor de sombra [0..1] (0 = totalmente en sombra, 1 = iluminado)

float parallax_self_shadow(vec2 uv, vec3 light_dir_ts, float height_scale,
                            sampler2D height_map, float initial_height) {
    // Si la luz viene desde atrás de la superficie, totalmente en sombra
    if (light_dir_ts.z <= 0.0) return 0.0;
    
    const float num_steps = 8.0;
    float layer_depth = initial_height / num_steps;
    
    vec2 delta_uv = light_dir_ts.xy / max(light_dir_ts.z, 0.001) * height_scale / num_steps;
    
    float current_layer_depth = initial_height - layer_depth;
    vec2 current_uv = uv + delta_uv;
    float current_height = texture(height_map, current_uv).r;
    
    float shadow_factor = 1.0;
    
    for (float i = 0.0; i < 8.0; i += 1.0) {
        if (current_layer_depth <= 0.0) break;
        
        // Si la altura del heightmap es mayor que la capa actual,
        // el punto está ocluido por el relieve
        if (current_height > current_layer_depth) {
            // Soft shadow: cuanto más ocluido, más oscuro
            float occlusion = (current_height - current_layer_depth);
            shadow_factor = min(shadow_factor, 1.0 - occlusion * 8.0);
        }
        
        current_uv += delta_uv;
        current_layer_depth -= layer_depth;
        current_height = texture(height_map, current_uv).r;
    }
    
    return clamp(shadow_factor, 0.0, 1.0);
}

#endif
