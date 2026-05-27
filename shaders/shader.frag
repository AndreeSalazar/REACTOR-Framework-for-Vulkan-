#version 450

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in vec3 fragPos;

layout(location = 0) out vec4 outColor;

layout(push_constant) uniform Constants {
    mat4 mvp;
    mat4 model;
    vec4 camera_pos;
} push;

// AAA ACES Filmic Tone Mapping approximation curve
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

    // 1. Dark Atmospheric Ambient
    // Very subtle dark blue sky glow and near-black ground reflection to preserve shadow depth
    vec3 skyAmbient = vec3(0.008, 0.006, 0.012);
    vec3 groundAmbient = vec3(0.002, 0.002, 0.003);
    float hemiW = N.y * 0.5 + 0.5;
    vec3 ambient = mix(groundAmbient, skyAmbient, hemiW);

    // Determine base color based on surface alignment (procedural detailing)
    vec3 baseColor;
    if (N.y > 0.7) {
        // Ceiling / floor surfaces (slate grey)
        baseColor = vec3(0.20, 0.22, 0.25);
    } else if (abs(N.x) > 0.7) {
        // Walls (brushed dark steel)
        baseColor = vec3(0.15, 0.17, 0.20);
    } else {
        // Characters / organic curves (decaying tactical gray-green)
        baseColor = vec3(0.12, 0.18, 0.14);
    }

    vec3 totalDiffuse = vec3(0.0);
    vec3 totalSpecular = vec3(0.0);

    // 2. Cyberpunk 2077 Spaced Corridor Point Lights (Spaced every 8 meters)
    const float spacing = 8.0;
    float seg = round(fragPos.z / spacing);

    // Accumulate from nearest 3 light segments to ensure seamless transitions
    for (int i = -1; i <= 1; ++i) {
        float zPos = (seg + float(i)) * spacing;
        
        // Alternating sides and colors (Magenta / Teal)
        // Even segment -> Magenta light on the left
        // Odd segment -> Teal light on the right
        bool isEven = (int(abs(seg + float(i))) % 2 == 0);
        
        vec3 lightPos;
        vec3 lightColor;
        
        if (isEven) {
            // Magenta neon light on the left wall
            lightPos = vec3(-2.8, 1.8, zPos);
            lightColor = vec3(1.0, 0.05, 0.5) * 1.5; // Intense hot pink
        } else {
            // Teal neon light on the right wall
            lightPos = vec3(2.8, 1.8, zPos);
            lightColor = vec3(0.0, 0.75, 1.0) * 1.5; // Intense neon teal
        }
        
        // Buzz/flicker effect based on light Z position (stable procedural flickering neon)
        float buzz = sin(zPos * 123.456) * 0.03 + 0.97;
        lightColor *= buzz;
        
        // Calculate vector & distance
        vec3 L = lightPos - fragPos;
        float dist = length(L);
        L = normalize(L);
        
        // Physical quadratic attenuation
        float attenuation = 1.0 / (1.0 + 0.15 * dist + 0.25 * dist * dist);
        
        // Diffuse
        float diff = max(dot(N, L), 0.0);
        totalDiffuse += diff * lightColor * attenuation;
        
        // Specular (Blinn-Phong)
        vec3 H = normalize(L + V);
        float spec = pow(max(dot(N, H), 0.0), 32.0);
        totalSpecular += spec * lightColor * attenuation * 0.5;
    }

    // 3. Tactical Flashlight (Attached to Player Camera)
    vec3 flashlightPos = push.camera_pos.xyz;
    vec3 flashlightDir = vec3(0.0, 0.0, -1.0); // Facing straight down the corridor (-Z)
    vec3 L_flash = flashlightPos - fragPos;
    float dist_flash = length(L_flash);
    L_flash = normalize(L_flash);
    
    // Spotlight cone calculations
    float theta = dot(-L_flash, flashlightDir);
    float cutOff = cos(radians(15.0)); // 15 degrees inner cone
    float outerCutOff = cos(radians(22.0)); // 22 degrees outer cone
    float epsilon = cutOff - outerCutOff;
    float intensity = clamp((theta - outerCutOff) / epsilon, 0.0, 1.0);
    
    // Flashlight attenuation
    float att_flash = 1.0 / (1.0 + 0.04 * dist_flash + 0.06 * dist_flash * dist_flash);
    vec3 flashColor = vec3(0.9, 0.95, 1.0) * 2.0; // Sharp cool white
    
    float diff_flash = max(dot(N, L_flash), 0.0);
    vec3 flashDiffuse = diff_flash * flashColor * att_flash * intensity;
    
    vec3 H_flash = normalize(L_flash + V);
    float spec_flash = pow(max(dot(N, H_flash), 0.0), 64.0);
    vec3 flashSpecular = spec_flash * flashColor * att_flash * intensity * 0.6;

    // 4. Stylized Rim Light (Backlight silhouette halo)
    float rimFactor = 1.0 - max(dot(N, V), 0.0);
    rimFactor = pow(rimFactor, 5.0); // Fine sharp rim
    vec3 rimColor = vec3(0.0, 0.9, 1.0) * 0.8; // Cyan rim light
    vec3 rimLight = rimFactor * rimColor;

    // 5. Assemble all lighting contributions
    vec3 diffuseAccum = ambient + totalDiffuse + flashDiffuse;
    vec3 specularAccum = totalSpecular + flashSpecular;
    vec3 finalColor = baseColor * diffuseAccum + specularAccum + rimLight;

    // 6. Cinematic Color Grading & Contrast
    finalColor = mix(finalColor, vec3(dot(finalColor, vec3(0.2126, 0.7152, 0.0722))), 0.12); // subtle desaturation
    finalColor.r *= 1.03; // organic touch
    finalColor.b *= 1.01;

    // 7. ACES Filmic Tone Mapping (AAA standard)
    finalColor = aces_tonemap(finalColor);

    // 8. Gamma Correction (sRGB space)
    finalColor = pow(finalColor, vec3(1.0 / 2.2));

    outColor = vec4(finalColor, 1.0);
}
