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

    // 1. Cyberpunk 2077 Hemispherical Ambient (Sky / Ground gradient)
    // Deep midnight purple/blue (top) transitioning to a dark steel gray (bottom)
    vec3 skyAmbient = vec3(0.04, 0.02, 0.08);
    vec3 groundAmbient = vec3(0.01, 0.01, 0.02);
    float hemiW = N.y * 0.5 + 0.5;
    vec3 ambient = mix(groundAmbient, skyAmbient, hemiW);

    // Determine base color based on surface alignment (procedural detailing)
    vec3 baseColor;
    if (N.y > 0.7) {
        // Ceiling / floor surfaces (metallic slate grey)
        baseColor = vec3(0.20, 0.22, 0.25);
    } else if (abs(N.x) > 0.7) {
        // Walls (brushed dark steel)
        baseColor = vec3(0.15, 0.17, 0.20);
    } else {
        // Characters / organic curves (decaying tactical gray-green)
        baseColor = vec3(0.12, 0.18, 0.14);
    }

    // 2. Main Key Light: Warm Cyberpunk Neon Pink/Orange (Streetlights, holographic billboards)
    vec3 keyLightPos = vec3(10.0, 15.0, 10.0);
    vec3 L_key = normalize(keyLightPos - fragPos);
    float diff_key = max(dot(N, L_key), 0.0);
    vec3 keyColor = vec3(1.0, 0.25, 0.55) * 1.5; // High intensity hot magenta/pink
    vec3 diffuse_key = diff_key * keyColor;

    // 3. Fill Light: Cool Cyberpunk Teal/Cyan (City sky glow, neon signs)
    vec3 fillLightDir = normalize(vec3(-1.0, 0.2, -0.5));
    float diff_fill = max(dot(N, fillLightDir), 0.0);
    vec3 fillColor = vec3(0.0, 0.9, 0.95) * 0.7; // Cooler cyber-teal
    vec3 diffuse_fill = diff_fill * fillColor;

    // 4. Stylized Specular Highlights (Blinn-Phong) for a wet, high-quality, metallic finish
    // Key Light specular (pinkish glow)
    vec3 H_key = normalize(L_key + V);
    float spec_key = pow(max(dot(N, H_key), 0.0), 32.0);
    vec3 specular_key = spec_key * keyColor * 0.8;

    // Fill Light specular (teal glow)
    vec3 H_fill = normalize(fillLightDir + V);
    float spec_fill = pow(max(dot(N, H_fill), 0.0), 16.0);
    vec3 specular_fill = spec_fill * fillColor * 0.4;

    // 5. Creepy Sci-Fi Neon Rim Light / Backlight (wraps beautifully around character silhouettes)
    // High intensity emerald green/electric cyan rim light
    float rimFactor = 1.0 - max(dot(N, V), 0.0);
    rimFactor = pow(rimFactor, 4.0); // Sharp, fine rim glow
    vec3 rimColor = vec3(0.05, 1.0, 0.6) * 1.8; // Electric toxic emerald
    vec3 rimLight = rimFactor * rimColor;

    // 6. Combine all lighting contributions
    vec3 finalLight = ambient + diffuse_key + diffuse_fill;
    vec3 finalColor = baseColor * finalLight + specular_key + specular_fill + rimLight;

    // 7. Cinematic Contrast & Color Grading (Cyberpunk grade)
    finalColor = mix(finalColor, vec3(dot(finalColor, vec3(0.2126, 0.7152, 0.0722))), 0.15); // Slight desaturation for gritty look
    finalColor.r *= 1.05; // Slightly boost reds for organic tissue/zombie feel
    finalColor.b *= 1.02; // Warm cyberpunk shadows

    // 8. ACES Filmic Tone Mapping (AAA standard)
    finalColor = aces_tonemap(finalColor);

    // 9. Gamma Correction (sRGB space)
    finalColor = pow(finalColor, vec3(1.0 / 2.2));

    outColor = vec4(finalColor, 1.0);
}
