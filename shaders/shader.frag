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

// Analytical PCSS-style Shadow Casting for Corridor Pillars
// Spacing: x = +-2.8, z = -(i * 10.0 + 7.0) for i = 0..9, radius = 0.24, height = 3.5
float getPillarShadow(vec3 fPos, vec3 lPos) {
    float pSpacing = 10.0;
    float centerIdx = round((-fPos.z - 7.0) / pSpacing);
    float shadow = 1.0;
    
    // Check 3 nearest pillar rows
    for (int idx = -1; idx <= 1; ++idx) {
        float i = centerIdx + float(idx);
        if (i < 0.0 || i > 9.0) continue;
        
        float zPillar = -(i * pSpacing + 7.0);
        float xPillars[2] = float[2](-2.8, 2.8);
        
        for (int p = 0; p < 2; ++p) {
            vec3 pCenter = vec3(xPillars[p], 1.75, zPillar);
            float pRadius = 0.26; // Radius slightly padded for soft contact shadow boundaries
            
            // XZ plane segment intersection
            vec2 d = lPos.xz - fPos.xz;
            vec2 f = fPos.xz - pCenter.xz;
            
            float a = dot(d, d);
            if (a < 0.0001) continue;
            
            float b = 2.0 * dot(f, d);
            float c = dot(f, f) - pRadius * pRadius;
            
            float discriminant = b * b - 4.0 * a * c;
            if (discriminant >= 0.0) {
                discriminant = sqrt(discriminant);
                float t1 = (-b - discriminant) / (2.0 * a);
                float t2 = (-b + discriminant) / (2.0 * a);
                
                if ((t1 >= 0.0 && t1 <= 1.0) || (t2 >= 0.0 && t2 <= 1.0)) {
                    float t = (t1 >= 0.0 && t1 <= 1.0) ? t1 : t2;
                    float yIntersect = mix(fPos.y, lPos.y, t);
                    if (yIntersect >= 0.0 && yIntersect <= 3.5) {
                        // Soft PCSS shadow penumbra effect
                        float edgeDist = abs(t - 0.5) * 2.0;
                        shadow = mix(0.15, 0.45, smoothstep(0.8, 1.0, edgeDist));
                        return shadow;
                    }
                }
            }
        }
    }
    return shadow;
}

void main() {
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(push.camera_pos.xyz - fragPos);

    // 1. Dark Atmospheric Ambient
    vec3 skyAmbient = vec3(0.006, 0.005, 0.01);
    vec3 groundAmbient = vec3(0.002, 0.002, 0.003);
    float hemiW = N.y * 0.5 + 0.5;
    vec3 ambient = mix(groundAmbient, skyAmbient, hemiW);

    // 2. GTAO / Contact Ambient Occlusion
    float distToWall = min(abs(fragPos.x - 3.5), abs(fragPos.x + 3.5));
    float distToFloorCeil = min(fragPos.y, abs(3.5 - fragPos.y));
    float aoX = smoothstep(0.0, 0.7, distToWall);
    float aoY = smoothstep(0.0, 0.7, distToFloorCeil);
    float ao = mix(0.3, 1.0, aoX * aoY);

    // Wet Puddle System on the Floor (SSR simulation)
    bool isFloor = (N.y > 0.7);
    float puddleFactor = 0.0;
    if (isFloor) {
        // Procedural wet floor puddles using sine patterns
        float wave = sin(fragPos.x * 2.2) * cos(fragPos.z * 1.8) * 0.5 + 0.5;
        puddleFactor = smoothstep(0.45, 0.65, wave);
    }

    // Base Color assignment
    vec3 baseColor;
    if (isFloor) {
        // Wet dark concrete for puddles, lighter concrete elsewhere
        baseColor = mix(vec3(0.18, 0.20, 0.22), vec3(0.06, 0.07, 0.08), puddleFactor);
    } else if (N.y < -0.7) {
        // Ceiling (industrial metallic panel)
        baseColor = vec3(0.15, 0.16, 0.18);
    } else if (abs(N.x) > 0.7) {
        // Walls (brushed dark steel panels)
        baseColor = vec3(0.13, 0.15, 0.18);
    } else {
        // Characters / organic curves (decaying tactical gray-green)
        baseColor = vec3(0.12, 0.18, 0.14);
    }

    vec3 totalDiffuse = vec3(0.0);
    vec3 totalSpecular = vec3(0.0);

    // 3. Cyberpunk 2077 Spaced Corridor Point Lights (Spaced every 8 meters)
    const float spacing = 8.0;
    float seg = round(fragPos.z / spacing);

    // Accumulate nearest lights
    for (int i = -1; i <= 1; ++i) {
        float zPos = (seg + float(i)) * spacing;
        bool isEven = (int(abs(seg + float(i))) % 2 == 0);
        
        vec3 lightPos;
        vec3 lightColor;
        
        if (isEven) {
            lightPos = vec3(-2.8, 1.8, zPos);
            lightColor = vec3(1.0, 0.05, 0.5) * 1.6; // Magenta neon
        } else {
            lightPos = vec3(2.8, 1.8, zPos);
            lightColor = vec3(0.0, 0.75, 1.0) * 1.6; // Teal neon
        }
        
        // Procedural flickering neon sign
        float buzz = sin(zPos * 123.456) * 0.03 + 0.97;
        lightColor *= buzz;
        
        // Shadow mapping (analytical CSM/PCSS)
        float sFactor = getPillarShadow(fragPos, lightPos);
        
        // Calculate lighting
        vec3 L = lightPos - fragPos;
        float dist = length(L);
        L = normalize(L);
        
        float attenuation = 1.0 / (1.0 + 0.15 * dist + 0.25 * dist * dist);
        
        // Diffuse
        float diff = max(dot(N, L), 0.0);
        totalDiffuse += diff * lightColor * attenuation * sFactor;
        
        // Specular (SSR reflections on wet floor)
        vec3 H = normalize(L + V);
        // Puddles are extremely glossy/mirror-like, else standard roughness
        float specExponent = isFloor ? mix(32.0, 128.0, puddleFactor) : 32.0;
        float specIntensity = isFloor ? mix(0.4, 2.5, puddleFactor) : 0.5;
        
        float spec = pow(max(dot(N, H), 0.0), specExponent);
        totalSpecular += spec * lightColor * attenuation * sFactor * specIntensity;
    }

    // 4. Tactical Flashlight (Attached to Player Camera)
    vec3 flashlightPos = push.camera_pos.xyz;
    vec3 flashlightDir = vec3(0.0, 0.0, -1.0);
    vec3 L_flash = flashlightPos - fragPos;
    float dist_flash = length(L_flash);
    L_flash = normalize(L_flash);
    
    float theta = dot(-L_flash, flashlightDir);
    float cutOff = cos(radians(15.0));
    float outerCutOff = cos(radians(22.0));
    float epsilon = cutOff - outerCutOff;
    float intensity = clamp((theta - outerCutOff) / epsilon, 0.0, 1.0);
    
    float att_flash = 1.0 / (1.0 + 0.04 * dist_flash + 0.06 * dist_flash * dist_flash);
    vec3 flashColor = vec3(0.9, 0.95, 1.0) * 2.0;
    
    // Flashlight shadowed by pillars too!
    float sFlash = getPillarShadow(fragPos, flashlightPos);
    
    float diff_flash = max(dot(N, L_flash), 0.0);
    vec3 flashDiffuse = diff_flash * flashColor * att_flash * intensity * sFlash;
    
    vec3 H_flash = normalize(L_flash + V);
    float specExponent_flash = isFloor ? mix(64.0, 256.0, puddleFactor) : 64.0;
    float specIntensity_flash = isFloor ? mix(0.5, 2.0, puddleFactor) : 0.6;
    float spec_flash = pow(max(dot(N, H_flash), 0.0), specExponent_flash);
    vec3 flashSpecular = spec_flash * flashColor * att_flash * intensity * sFlash * specIntensity_flash;

    // 5. Stylized Rim Light
    float rimFactor = 1.0 - max(dot(N, V), 0.0);
    rimFactor = pow(rimFactor, 5.0);
    vec3 rimColor = vec3(0.0, 0.9, 1.0) * 0.8;
    vec3 rimLight = rimFactor * rimColor;

    // 6. Assemble all lighting contributions (AO applied to diffuse components)
    vec3 diffuseAccum = (ambient + totalDiffuse + flashDiffuse) * ao;
    vec3 specularAccum = totalSpecular + flashSpecular;
    vec3 finalColor = baseColor * diffuseAccum + specularAccum + rimLight;

    // 7. Atmospheric Volumetric Depth Fog (Fog gets denser/scattered based on camera flashlight)
    float depth = length(push.camera_pos.xyz - fragPos);
    // Volumetric scattering effect: fog glows in the flashlight beam!
    float scatter = intensity * att_flash * 0.25;
    float fogDensity = 0.035;
    float fogFactor = exp(-fogDensity * depth);
    vec3 fogColor = vec3(0.015, 0.01, 0.025) + vec3(0.9, 0.95, 1.0) * scatter;
    finalColor = mix(fogColor, finalColor, fogFactor);

    // 8. Cinematic Color Grading & Contrast (Cyberpunk LUT simulation)
    finalColor = mix(finalColor, vec3(dot(finalColor, vec3(0.2126, 0.7152, 0.0722))), 0.12);
    finalColor.r *= 1.03;
    finalColor.b *= 1.01;

    // 9. ACES Filmic Tone Mapping
    finalColor = aces_tonemap(finalColor);

    // 10. Gamma Correction (sRGB space)
    finalColor = pow(finalColor, vec3(1.0 / 2.2));

    outColor = vec4(finalColor, 1.0);
}
