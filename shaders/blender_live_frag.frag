#version 450

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in vec3 fragPos;
layout(location = 3) in vec4 fragColor;

layout(location = 0) out vec4 outColor;

layout(push_constant) uniform Constants {
    mat4 mvp;
    mat4 model;
    vec4 camera_pos;
    vec4 light_pos;
    vec4 color;
} push;

// ACES Tone Mapping
vec3 aces_tonemap(vec3 color) {
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), 0.0, 1.0);
}

void main() {
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(push.camera_pos.xyz - fragPos);
    vec3 baseColor = fragColor.rgb;

    // 1. Hemispherical Ambient Light (Neutral Grey Studio Sky & Floor)
    vec3 skyColor = vec3(0.35, 0.35, 0.38);
    vec3 groundColor = vec3(0.12, 0.12, 0.14);
    float up = dot(N, vec3(0.0, 1.0, 0.0)) * 0.5 + 0.5;
    vec3 ambient = mix(groundColor, skyColor, up);

    // 2. Directional Sun (Neutral Soft White Sunlight)
    vec3 sunDir = normalize(vec3(-0.4, 0.8, 0.5)); // Direction pointing from top-front-right
    float sunDiffuse = max(dot(N, sunDir), 0.0);
    vec3 sunColor = vec3(0.65, 0.65, 0.62) * sunDiffuse;

    // Sun Specular (Blinn-Phong)
    vec3 H_sun = normalize(sunDir + V);
    float sunSpec = pow(max(dot(N, H_sun), 0.0), 32.0);
    vec3 sunSpecular = vec3(0.2) * sunSpec;

    // 3. Dynamic Point Light (Neutral White Point Light from Blender)
    vec3 pointLightColor = vec3(0.0);
    vec3 pointLightSpecular = vec3(0.0);
    
    vec3 L = push.light_pos.xyz - fragPos;
    float dist = length(L);
    if (dist > 0.001 && dist < 40.0) {
        vec3 L_dir = normalize(L);
        float att = 1.0 / (1.0 + 0.1 * dist + 0.05 * dist * dist);
        
        float diff = max(dot(N, L_dir), 0.0);
        pointLightColor = vec3(1.0, 1.0, 1.0) * diff * att * 1.5;
        
        // Point Specular
        vec3 H_point = normalize(L_dir + V);
        float pointSpec = pow(max(dot(N, H_point), 0.0), 32.0);
        pointLightSpecular = vec3(0.4) * pointSpec * att;
    }

    // 4. Assemble Lighting
    vec3 diffuseAccum = ambient + sunColor + pointLightColor;
    vec3 specularAccum = sunSpecular + pointLightSpecular;
    
    vec3 finalColor = baseColor * diffuseAccum + specularAccum;

    // 5. ACES Tone Mapping for professional color balance
    finalColor = aces_tonemap(finalColor);

    // 6. Gamma Correction
    finalColor = pow(finalColor, vec3(1.0 / 2.2));

    outColor = vec4(finalColor, 1.0);
}
