#version 450

layout(location = 0) in vec3 fragColor; // normal vector
layout(location = 0) out vec4 outColor;

void main() {
    // Treat the incoming color as a normal vector and normalize it
    vec3 normal = normalize(fragColor);
    
    // Simple but elegant multi-directional lighting
    // Key light from top-front-right, and a weaker fill light from bottom-left
    vec3 lightDir1 = normalize(vec3(0.5, 1.0, 0.4));
    vec3 lightDir2 = normalize(vec3(-0.5, -0.5, -0.4));
    
    float diff1 = max(dot(normal, lightDir1), 0.0);
    float diff2 = max(dot(normal, lightDir2), 0.0);
    
    float ambient = 0.25;
    float lighting = ambient + diff1 * 0.55 + diff2 * 0.2;
    
    // Modern tactical color palette (slate, steel, decaying organic)
    vec3 baseColor;
    if (normal.y > 0.7) {
        // Floor / horizontal top surfaces
        baseColor = vec3(0.35, 0.38, 0.40);
    } else if (abs(normal.x) > 0.7) {
        // Walls / vertical side surfaces
        baseColor = vec3(0.40, 0.42, 0.45);
    } else {
        // Zombies, pillars, rounded corners (decaying organic green-grey)
        baseColor = vec3(0.30, 0.38, 0.32);
    }
    
    vec3 finalColor = baseColor * lighting;
    
    // Apply gamma correction (sRGB space)
    finalColor = pow(finalColor, vec3(1.0 / 2.2));
    
    outColor = vec4(finalColor, 1.0);
}
