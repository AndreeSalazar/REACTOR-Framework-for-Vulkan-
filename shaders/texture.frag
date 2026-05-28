#version 450

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in vec3 fragPos;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform sampler2D texSampler;

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

float hash(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 4; ++i) {
        v += a * noise(p);
        p = p * 2.0 + vec2(37.0, 17.0);
        a *= 0.5;
    }
    return v;
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

vec3 sceneSpaceGlobalIllumination(vec3 fPos, vec3 normal, vec3 baseColor, float ao, float puddleFactor) {
    const float spacing = 8.0;
    float seg = round(fPos.z / spacing);
    vec3 bounce = vec3(0.0);

    for (int i = -2; i <= 2; ++i) {
        float zPos = (seg + float(i)) * spacing;
        bool isEven = (int(abs(seg + float(i))) % 2 == 0);
        vec3 lightPos = isEven ? vec3(-2.8, 1.8, zPos) : vec3(2.8, 1.8, zPos);
        vec3 lightColor = isEven ? vec3(1.0, 0.05, 0.5) : vec3(0.0, 0.75, 1.0);

        vec3 toLight = lightPos - fPos;
        float dist = length(toLight);
        vec3 L = toLight / max(dist, 0.0001);
        float visibility = getPillarShadow(fPos + normal * 0.04, lightPos);
        float attenuation = 1.0 / (1.0 + 0.12 * dist + 0.16 * dist * dist);
        float hemi = max(dot(normal, L) * 0.5 + 0.5, 0.0);
        float floorBounce = smoothstep(0.0, 1.5, fPos.y) * max(1.0 - normal.y, 0.0);
        float wallBounce = 1.0 - smoothstep(0.2, 2.8, min(abs(fPos.x - 3.5), abs(fPos.x + 3.5)));
        float weight = (hemi * 0.18 + floorBounce * 0.10 + wallBounce * 0.10) * attenuation * visibility;

        bounce += lightColor * weight;
    }

    vec3 ambientBounce = vec3(0.014, 0.011, 0.026) * mix(0.62, 1.0, ao);
    vec3 colorBleed = bounce;
    colorBleed *= mix(0.14, 0.28, puddleFactor);
    return baseColor * (ambientBounce + colorBleed);
}

vec3 cinematicLutGrade(vec3 color) {
    float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
    vec3 shadowGrade = vec3(0.78, 0.88, 1.16);
    vec3 midGrade = vec3(0.98, 1.02, 1.04);
    vec3 highlightGrade = vec3(1.14, 1.02, 0.88);
    vec3 grade = mix(shadowGrade, midGrade, smoothstep(0.04, 0.45, luma));
    grade = mix(grade, highlightGrade, smoothstep(0.48, 1.0, luma));

    vec3 graded = color * grade;
    float gradedLuma = dot(graded, vec3(0.2126, 0.7152, 0.0722));
    graded = mix(vec3(gradedLuma), graded, 1.08);
    graded = mix(vec3(0.5), graded, 1.08);
    return max(graded, vec3(0.0));
}

void main() {
    // Sample texture
    vec4 texColor = texture(texSampler, fragUV);
    
    // Discard completely transparent pixels if needed, or handle alpha blending
    if (texColor.a < 0.1) {
        discard;
    }

    // Detect if this is a shadow (transparent dark overlay under the zombies' feet)
    bool isShadow = (texColor.a < 0.9 && texColor.r < 0.1);

    vec3 N = normalize(fragNormal);
    vec3 V = normalize(push.camera_pos.xyz - fragPos);

    vec3 finalColor;

    if (isShadow) {
        // Flat dark shadow overlay: no neon reflections, no specular highlights, no rim light
        finalColor = texColor.rgb;
    } else {
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
        vec3 baseAlbedo = texColor.rgb;
        if (isFloor) {
            // Procedural wet floor puddles using sine patterns
            float wave = sin(fragPos.x * 2.2) * cos(fragPos.z * 1.8) * 0.5 + 0.5;
            puddleFactor = smoothstep(0.45, 0.65, wave);
            // Puddles are darker concrete
            baseAlbedo = mix(texColor.rgb, texColor.rgb * 0.35, puddleFactor);
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
        vec3 ssgiBounce = sceneSpaceGlobalIllumination(fragPos, N, baseAlbedo, ao, puddleFactor);
        vec3 envReflection = vec3(0.0);
        if (isFloor && puddleFactor > 0.1) {
            vec3 R = reflect(-V, N);
            float refSeg = round((fragPos.z + R.z * 3.0) / spacing);
            bool refEven = (int(abs(refSeg)) % 2 == 0);
            vec3 refColor = refEven ? vec3(1.0, 0.05, 0.5) : vec3(0.0, 0.75, 1.0);
            envReflection = refColor * max(R.y, 0.0) * puddleFactor * 0.35;
        }

        vec3 diffuseAccum = (ambient + totalDiffuse + flashDiffuse) * ao;
        vec3 specularAccum = totalSpecular + flashSpecular;
        finalColor = baseAlbedo * diffuseAccum + ssgiBounce + specularAccum + rimLight + envReflection;

        // 7. Atmospheric Volumetric Depth Fog (Fog gets denser/scattered based on camera flashlight)
        float depth = length(push.camera_pos.xyz - fragPos);
        float viewDotBeam = max(dot(normalize(fragPos - push.camera_pos.xyz), flashlightDir), 0.0);
        float phase = 0.35 + pow(viewDotBeam, 8.0) * 0.65;
        float scatter = intensity * att_flash * phase * 0.38;
        float fogNoise = fbm(fragPos.xz * 0.18 + vec2(fragPos.y * 0.11, depth * 0.015));
        float fogDensity = 0.035 * mix(0.78, 1.22, fogNoise);
        float fogFactor = exp(-fogDensity * depth);
        vec3 fogColor = vec3(0.015, 0.01, 0.025) + vec3(0.9, 0.95, 1.0) * scatter;
        finalColor = mix(fogColor, finalColor, fogFactor);

        // 8. Cinematic Color Grading & Contrast (Cyberpunk LUT simulation)
        finalColor = cinematicLutGrade(finalColor);

        // 9. ACES Filmic Tone Mapping
        finalColor = aces_tonemap(finalColor);

        // 10. Gamma Correction (sRGB space)
        finalColor = pow(finalColor, vec3(1.0 / 2.2));
    }

    outColor = vec4(finalColor, texColor.a);
}
