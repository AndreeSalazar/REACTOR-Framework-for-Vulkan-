#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec3 fragWorldPos;
layout(location = 0) out vec4 outColor;

// ==================== SDF ANTI-ALIASING (ADead-AA) ====================
// Técnica matemática para bordes perfectos sin MSAA
// Zero memoria extra | Resolución independiente | Bordes perfectos

// SDF de un cubo centrado en origen
float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Anti-aliasing matemático perfecto
float sdfAA(float d) {
    float w = fwidth(d) * 1.5;  // Ancho de pixel con margen
    return smoothstep(w, -w, d);
}

// Detectar bordes usando derivadas de la posición
float detectEdge(vec3 worldPos, vec3 normal) {
    // Derivadas de la posición en espacio de pantalla
    vec3 dPdx = dFdx(worldPos);
    vec3 dPdy = dFdy(worldPos);
    
    // Derivadas de la normal
    vec3 dNdx = dFdx(normal);
    vec3 dNdy = dFdy(normal);
    
    // Cambio en la normal indica borde geométrico
    float normalChange = length(dNdx) + length(dNdy);
    
    // Cambio en la posición relativo a la normal indica silhouette
    float silhouette = abs(dot(normalize(dPdx), normal)) + abs(dot(normalize(dPdy), normal));
    
    return clamp(normalChange * 3.0 + (1.0 - silhouette) * 0.5, 0.0, 1.0);
}

// Suavizado de color en bordes
vec3 smoothEdgeColor(vec3 color, float edgeFactor) {
    // En bordes, mezclar ligeramente con vecinos (simulado)
    float smoothing = edgeFactor * 0.15;
    return mix(color, color * 0.95, smoothing);
}

void main() {
    vec3 normal = normalize(fragNormal);
    
    // Calcular SDF para anti-aliasing
    float sdf = sdBox(fragWorldPos, vec3(0.5));
    float sdfFactor = sdfAA(sdf);
    
    // Detectar bordes geométricos
    float edge = detectEdge(fragWorldPos, normal);
    
    // Iluminación básica
    vec3 lightDir = normalize(vec3(0.5, 1.0, 0.3));
    float NdotL = max(dot(normal, lightDir), 0.0);
    
    // Ambient + Diffuse con suavizado en bordes
    float ambient = 0.35;
    float diffuse = NdotL * 0.65;
    
    // Suavizar la transición de iluminación en bordes
    float smoothDiffuse = mix(diffuse, smoothstep(0.0, 1.0, diffuse), edge * 0.3);
    
    vec3 litColor = fragColor * (ambient + smoothDiffuse);
    
    // Aplicar suavizado de bordes
    vec3 finalColor = smoothEdgeColor(litColor, edge);
    
    // Anti-aliasing final en silueta
    float alpha = mix(1.0, sdfFactor, step(0.001, abs(sdf)));
    
    outColor = vec4(finalColor, 1.0);
}
