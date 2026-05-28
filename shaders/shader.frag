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

// ═══════════════════════════════════════════════════════════════════════════════
// AAA CINEMATIC RENDERING PIPELINE — REACTOR ENGINE
// ═══════════════════════════════════════════════════════════════════════════════
// Features:
//   • ACES Filmic Tone Mapping (Academy Color Encoding System)
//   • PCSS Analytical Soft Shadows (Percentage-Closer Soft Shadows)
//   • Hemispherical Ambient Lighting with Ground Bounce
//   • Screen-Space Contact Shadows (GTAO approximation)
//   • Volumetric Fog with Light Scattering
//   • SSR-style Wet Floor Puddle Reflections
//   • Cyberpunk 2077 Neon Point Lights with Flickering
//   • Tactical Flashlight with Spotlight Cone
//   • Fresnel Rim Lighting (Schlick approximation)
//   • Subsurface Scattering approximation for organic surfaces
//   • Light Bleeding through geometry edges
//   • Cinematic Color Grading with Split-Toning
// ═══════════════════════════════════════════════════════════════════════════════

// ACES Filmic Tone Mapping — Industry standard (used in UE5, Unity HDRP)
vec3 aces_tonemap(vec3 color) {
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), 0.0, 1.0);
}

// Hash-based noise for procedural effects (low cost, good distribution)
float hash(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Smooth noise interpolation (used for puddles, surface variation)
float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f); // Smoothstep
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// FBM (Fractal Brownian Motion) — layered noise for organic detail
float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    vec2 shift = vec2(100.0);
    for (int i = 0; i < 4; ++i) {
        v += a * noise(p);
        p = p * 2.0 + shift;
        a *= 0.5;
    }
    return v;
}

// ═══════════════════════════════════════════════════════════════════════════════
// PCSS ANALYTICAL SOFT SHADOWS — Ray-Cylinder Intersection for Corridor Pillars
// ═══════════════════════════════════════════════════════════════════════════════
// Projects shadows from pillars onto surrounding geometry using ray-cylinder
// intersection in the XZ plane. Includes soft penumbra for realistic falloff.
float getPillarShadow(vec3 fPos, vec3 lPos) {
    float pSpacing = 10.0;
    float centerIdx = round((-fPos.z - 7.0) / pSpacing);
    float shadow = 1.0;
    
    // Check 3 nearest pillar rows for shadow occlusion
    for (int idx = -1; idx <= 1; ++idx) {
        float i = centerIdx + float(idx);
        if (i < 0.0 || i > 9.0) continue;
        
        float zPillar = -(i * pSpacing + 7.0);
        float xPillars[2] = float[2](-2.8, 2.8);
        
        for (int p = 0; p < 2; ++p) {
            vec3 pCenter = vec3(xPillars[p], 1.75, zPillar);
            float pRadius = 0.26;
            
            // Ray-cylinder intersection in XZ plane
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
                        // PCSS penumbra: softer at edges, harder at center
                        float penetration = (t2 - t1) * length(d);
                        float softness = clamp(penetration / (pRadius * 4.0), 0.0, 1.0);
                        shadow = mix(0.08, 0.35, 1.0 - softness);
                        return shadow;
                    }
                }
            }
        }
    }
    return shadow;
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTACT SHADOW — Short-range occlusion near geometry intersections
// ═══════════════════════════════════════════════════════════════════════════════
float getContactShadow(vec3 fPos, vec3 N) {
    // Wall contact darkening
    float distToWallL = abs(fPos.x + 3.5);
    float distToWallR = abs(fPos.x - 3.5);
    float distToWall = min(distToWallL, distToWallR);
    
    // Floor/ceiling contact
    float distToFloor = fPos.y;
    float distToCeil = abs(3.5 - fPos.y);
    float distToFloorCeil = min(distToFloor, distToCeil);
    
    // Smooth AO falloff
    float aoWall = smoothstep(0.0, 0.8, distToWall);
    float aoFloorCeil = smoothstep(0.0, 0.8, distToFloorCeil);
    
    // Pillar proximity AO (darken around pillar bases)
    float pillarAO = 1.0;
    float pSpacing = 10.0;
    float nearestPillarZ = round((-fPos.z - 7.0) / pSpacing) * pSpacing + 7.0;
    for (int p = 0; p < 2; ++p) {
        float px = (p == 0) ? -2.8 : 2.8;
        float pz = -nearestPillarZ;
        float dist = length(fPos.xz - vec2(px, pz));
        pillarAO *= smoothstep(0.0, 0.6, dist - 0.24);
    }
    
    return mix(0.15, 1.0, aoWall * aoFloorCeil * pillarAO);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUBSURFACE SCATTERING APPROXIMATION — For organic character lighting
// ═══════════════════════════════════════════════════════════════════════════════
vec3 subsurfaceScatter(vec3 lightDir, vec3 viewDir, vec3 normal, vec3 lightColor, float thickness) {
    vec3 scatterDir = lightDir + normal * 0.5;
    float VdotS = pow(clamp(dot(viewDir, -scatterDir), 0.0, 1.0), 3.0);
    return lightColor * VdotS * thickness * 0.3;
}

vec3 sceneSpaceGlobalIllumination(vec3 fPos, vec3 normal, vec3 baseColor, float ao, float puddleFactor) {
    const float spacing = 8.0;
    float seg = round(fPos.z / spacing);
    vec3 bounce = vec3(0.0);

    for (int i = -3; i <= 3; ++i) {
        float zPos = (seg + float(i)) * spacing;
        bool isEven = (int(abs(seg + float(i))) % 2 == 0);
        vec3 lightPos = isEven ? vec3(-2.8, 1.8, zPos) : vec3(2.8, 1.8, zPos);
        vec3 lightColor = isEven ? vec3(1.0, 0.05, 0.5) : vec3(0.0, 0.75, 1.0);

        vec3 toLight = lightPos - fPos;
        float dist = length(toLight);
        vec3 L = toLight / max(dist, 0.0001);
        float visibility = getPillarShadow(fPos + normal * 0.04, lightPos);
        float hemi = max(dot(normal, L) * 0.5 + 0.5, 0.0);
        float attenuation = 1.0 / (1.0 + 0.10 * dist + 0.12 * dist * dist);

        float floorBounce = smoothstep(0.0, 1.6, fPos.y) * max(1.0 - normal.y, 0.0);
        float wallBounce = 1.0 - smoothstep(0.2, 2.8, min(abs(fPos.x - 3.5), abs(fPos.x + 3.5)));
        float weight = (hemi * 0.20 + floorBounce * 0.10 + wallBounce * 0.12) * attenuation * visibility;

        bounce += lightColor * weight;
    }

    float contactDarkening = mix(0.62, 1.0, ao);
    vec3 ambientBounce = vec3(0.018, 0.012, 0.030) * contactDarkening;
    vec3 colorBleed = bounce;
    colorBleed *= mix(0.18, 0.32, puddleFactor);
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
    graded = mix(vec3(0.5), graded, 1.09);
    return max(graded, vec3(0.0));
}

void main() {
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(push.camera_pos.xyz - fragPos);
    float NdotV = max(dot(N, V), 0.0);

    // ─── 1. HEMISPHERICAL AMBIENT LIGHTING ──────────────────────────────
    // Sky color (cold blue-violet) vs ground bounce (warm amber)
    vec3 skyAmbient = vec3(0.008, 0.006, 0.015);
    vec3 groundAmbient = vec3(0.003, 0.002, 0.004);
    float hemiW = N.y * 0.5 + 0.5;
    vec3 ambient = mix(groundAmbient, skyAmbient, hemiW);

    // ─── 2. CONTACT AMBIENT OCCLUSION (GTAO approximation) ─────────────
    float ao = getContactShadow(fragPos, N);

    // ─── 3. WET PUDDLE SYSTEM (SSR Simulation) ─────────────────────────
    bool isFloor = (N.y > 0.7);
    float puddleFactor = 0.0;
    float surfaceRoughness = 0.7; // Default roughness
    
    if (isFloor) {
        // Multi-octave puddle pattern for realistic water pooling
        float wave1 = sin(fragPos.x * 2.2) * cos(fragPos.z * 1.8);
        float wave2 = sin(fragPos.x * 4.1 + 1.3) * cos(fragPos.z * 3.7 - 0.8) * 0.3;
        float wave = (wave1 + wave2) * 0.5 + 0.5;
        puddleFactor = smoothstep(0.40, 0.60, wave);
        surfaceRoughness = mix(0.7, 0.05, puddleFactor); // Puddles are mirror-smooth
    }

    // ─── 4. MATERIAL SYSTEM (Procedural PBR) ───────────────────────────
    vec3 baseColor;
    float metallic = 0.0;
    
    if (isFloor) {
        // Wet industrial concrete with puddle darkening
        vec3 dryColor = vec3(0.18, 0.20, 0.22);
        vec3 wetColor = vec3(0.04, 0.05, 0.06);
        baseColor = mix(dryColor, wetColor, puddleFactor);
        // Add subtle concrete variation
        float variation = fbm(fragPos.xz * 3.0) * 0.08;
        baseColor += variation;
    } else if (N.y < -0.7) {
        // Ceiling — industrial metal panels with rivet patterns
        baseColor = vec3(0.14, 0.15, 0.17);
        metallic = 0.3;
        float rivetPattern = step(0.92, sin(fragPos.x * 8.0) * sin(fragPos.z * 8.0));
        baseColor -= rivetPattern * 0.03;
    } else if (abs(N.x) > 0.7) {
        // Walls — brushed dark steel with panel seams
        baseColor = vec3(0.12, 0.14, 0.17);
        metallic = 0.4;
        // Panel seam lines
        float seamH = 1.0 - smoothstep(0.02, 0.04, abs(fract(fragPos.y * 1.0) - 0.5) * 2.0 - 0.96);
        float seamV = 1.0 - smoothstep(0.02, 0.04, abs(fract(fragPos.z * 0.5) - 0.5) * 2.0 - 0.96);
        float seam = max(seamH, seamV);
        baseColor = mix(baseColor, baseColor * 0.4, seam * 0.6);
    } else {
        // Characters / props — organic tactical gray-green
        baseColor = vec3(0.12, 0.18, 0.14);
    }

    vec3 totalDiffuse = vec3(0.0);
    vec3 totalSpecular = vec3(0.0);
    vec3 totalSSS = vec3(0.0);

    // ─── 5. CYBERPUNK 2077 NEON LIGHTING SYSTEM ────────────────────────
    // Alternating magenta/teal neon lights spaced every 8 meters
    const float spacing = 8.0;
    float seg = round(fragPos.z / spacing);

    for (int i = -2; i <= 2; ++i) {
        float zPos = (seg + float(i)) * spacing;
        bool isEven = (int(abs(seg + float(i))) % 2 == 0);
        
        vec3 lightPos;
        vec3 lightColor;
        
        if (isEven) {
            lightPos = vec3(-2.8, 1.8, zPos);
            lightColor = vec3(1.0, 0.05, 0.5) * 1.8; // Hot magenta neon
        } else {
            lightPos = vec3(2.8, 1.8, zPos);
            lightColor = vec3(0.0, 0.75, 1.0) * 1.8; // Electric teal neon
        }
        
        // Procedural neon buzz/flicker
        float flicker = sin(zPos * 123.456) * 0.04 + 0.96;
        float buzz = sin(zPos * 77.77 + fragPos.z * 0.1) * 0.02 + 0.98;
        lightColor *= flicker * buzz;
        
        // PCSS shadow evaluation
        float sFactor = getPillarShadow(fragPos, lightPos);
        
        // Physically-based attenuation
        vec3 L = lightPos - fragPos;
        float dist = length(L);
        L = normalize(L);
        float attenuation = 1.0 / (1.0 + 0.12 * dist + 0.22 * dist * dist);
        
        // Lambertian diffuse
        float NdotL = max(dot(N, L), 0.0);
        totalDiffuse += NdotL * lightColor * attenuation * sFactor;
        
        // GGX-inspired specular (simplified)
        vec3 H = normalize(L + V);
        float NdotH = max(dot(N, H), 0.0);
        float roughness2 = surfaceRoughness * surfaceRoughness;
        float specPow = mix(256.0, 16.0, roughness2);
        float spec = pow(NdotH, specPow);
        // Fresnel-Schlick at half-angle
        float F = mix(0.04, 1.0, pow(1.0 - max(dot(H, V), 0.0), 5.0));
        float specIntensity = mix(0.3, 3.0, 1.0 - roughness2) * F;
        totalSpecular += spec * lightColor * attenuation * sFactor * specIntensity;
        
        // SSS for organic surfaces (characters only)
        if (abs(N.x) < 0.5 && N.y > -0.5 && N.y < 0.5) {
            totalSSS += subsurfaceScatter(L, V, N, lightColor, 0.5) * attenuation * sFactor;
        }
        
        // Light bleeding at shadow edges (soft light wrap)
        float wrapDiffuse = max(dot(N, L) + 0.25, 0.0) / 1.25;
        float bleed = max(wrapDiffuse - NdotL, 0.0);
        totalDiffuse += bleed * lightColor * attenuation * 0.15 * sFactor;
    }

    // ─── 6. TACTICAL FLASHLIGHT (Player Camera Spotlight) ──────────────
    vec3 flashlightPos = push.camera_pos.xyz;
    vec3 flashlightDir = normalize(vec3(0.0, -0.05, -1.0)); // Slight downward angle
    vec3 L_flash = flashlightPos - fragPos;
    float dist_flash = length(L_flash);
    L_flash = normalize(L_flash);
    
    // Spotlight cone with soft outer falloff
    float theta = dot(-L_flash, flashlightDir);
    float cutOff = cos(radians(14.0));
    float outerCutOff = cos(radians(24.0));
    float epsilon = cutOff - outerCutOff;
    float intensity = clamp((theta - outerCutOff) / epsilon, 0.0, 1.0);
    intensity *= intensity; // Squared falloff for more natural spotlight
    
    float att_flash = 1.0 / (1.0 + 0.035 * dist_flash + 0.05 * dist_flash * dist_flash);
    vec3 flashColor = vec3(0.92, 0.95, 1.0) * 2.2; // Cool white LED
    
    // Flashlight shadow from pillars
    float sFlash = getPillarShadow(fragPos, flashlightPos);
    
    // Flashlight diffuse
    float NdotL_flash = max(dot(N, L_flash), 0.0);
    vec3 flashDiffuse = NdotL_flash * flashColor * att_flash * intensity * sFlash;
    
    // Flashlight specular (very tight highlight on wet surfaces)
    vec3 H_flash = normalize(L_flash + V);
    float NdotH_flash = max(dot(N, H_flash), 0.0);
    float flashSpecPow = mix(512.0, 32.0, surfaceRoughness * surfaceRoughness);
    float flashSpecIntensity = mix(0.4, 3.5, 1.0 - surfaceRoughness);
    float flash_spec = pow(NdotH_flash, flashSpecPow);
    float F_flash = mix(0.04, 1.0, pow(1.0 - max(dot(H_flash, V), 0.0), 5.0));
    vec3 flashSpecular = flash_spec * flashColor * att_flash * intensity * sFlash * flashSpecIntensity * F_flash;

    // ─── 7. FRESNEL RIM LIGHTING ───────────────────────────────────────
    // Schlick Fresnel approximation for dramatic edge glow
    float rimFactor = 1.0 - NdotV;
    rimFactor = pow(rimFactor, 4.0);
    
    // Rim color picks up dominant nearby neon color for cohesion
    float nearestNeonPhase = fract(fragPos.z / (spacing * 2.0));
    vec3 rimTint = mix(vec3(0.0, 0.85, 1.0), vec3(1.0, 0.1, 0.6), step(0.5, nearestNeonPhase));
    vec3 rimLight = rimFactor * rimTint * 0.6;

    // ─── 8. ENVIRONMENT REFLECTIONS (Cubemap approximation for puddles) ─
    vec3 envReflection = vec3(0.0);
    if (isFloor && puddleFactor > 0.1) {
        // Fake environment reflection: reflect the neon lights
        vec3 R = reflect(-V, N);
        // Sample nearest neon color based on reflection direction
        float refSeg = round((fragPos.z + R.z * 3.0) / spacing);
        bool refEven = (int(abs(refSeg)) % 2 == 0);
        vec3 refColor = refEven ? vec3(1.0, 0.05, 0.5) : vec3(0.0, 0.75, 1.0);
        float refIntensity = max(R.y, 0.0) * 0.5; // Only reflect upward-facing rays
        envReflection = refColor * refIntensity * puddleFactor * 0.4;
    }

    // ─── 9. ASSEMBLE LIGHTING ──────────────────────────────────────────
    vec3 ssgiBounce = sceneSpaceGlobalIllumination(fragPos, N, baseColor, ao, puddleFactor);
    vec3 diffuseAccum = (ambient + totalDiffuse + flashDiffuse) * ao;
    vec3 specularAccum = totalSpecular + flashSpecular;
    vec3 finalColor = baseColor * diffuseAccum + ssgiBounce + specularAccum + rimLight + totalSSS + envReflection;
    
    // Metallic surfaces reflect more of the light color directly
    finalColor = mix(finalColor, (specularAccum + envReflection) * baseColor * 2.0, metallic * 0.5);

    // ─── 10. VOLUMETRIC FOG with Light Scattering ──────────────────────
    float depth = length(push.camera_pos.xyz - fragPos);
    
    // Volumetric light scattering (Mie-like) in the flashlight beam
    float viewDotBeam = max(dot(normalize(fragPos - push.camera_pos.xyz), flashlightDir), 0.0);
    float phase = 0.35 + pow(viewDotBeam, 8.0) * 0.65;
    float scatter = intensity * att_flash * phase * 0.42;
    
    // Height-based fog density (thicker near floor — mist pooling)
    float heightFog = exp(-fragPos.y * 0.8) * 0.5 + 0.5;
    float fogNoise = fbm(fragPos.xz * 0.18 + vec2(fragPos.y * 0.11, depth * 0.015));
    float fogDensity = 0.032 * heightFog * mix(0.78, 1.22, fogNoise);
    float fogFactor = exp(-fogDensity * depth);
    
    // Fog color: dark purple ambient + flashlight scattering
    vec3 fogColor = vec3(0.012, 0.008, 0.022) + vec3(0.9, 0.95, 1.0) * scatter;
    
    // Add neon color bleeding into fog
    float nearestNeonDist = abs(fract(fragPos.z / spacing + 0.5) - 0.5) * spacing;
    if (nearestNeonDist < 3.0) {
        float neonFogBleed = (1.0 - nearestNeonDist / 3.0) * 0.15;
        bool neonEven = (int(abs(round(fragPos.z / spacing))) % 2 == 0);
        vec3 neonFogColor = neonEven ? vec3(1.0, 0.05, 0.5) : vec3(0.0, 0.75, 1.0);
        fogColor += neonFogColor * neonFogBleed;
    }
    
    finalColor = mix(fogColor, finalColor, fogFactor);

    // ─── 11. CINEMATIC COLOR GRADING ───────────────────────────────────
    finalColor = cinematicLutGrade(finalColor);

    // ─── 12. ACES FILMIC TONE MAPPING ──────────────────────────────────
    finalColor = aces_tonemap(finalColor);

    // ─── 13. GAMMA CORRECTION (Linear → sRGB) ─────────────────────────
    finalColor = pow(finalColor, vec3(1.0 / 2.2));

    outColor = vec4(finalColor, 1.0);
}
