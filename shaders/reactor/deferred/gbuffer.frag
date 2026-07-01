#version 450

layout(location = 0) in vec3 vWorldNormal;
layout(location = 1) in vec2 vUV;
layout(location = 2) in vec3 vWorldPos;
layout(location = 3) in vec4 vColor;
layout(location = 4) in vec2 vMotion;

layout(location = 0) out vec4 outAlbedoAo;
layout(location = 1) out vec4 outNormalMaterial;
layout(location = 2) out vec4 outEmissiveMaterial;
layout(location = 3) out vec4 outMotionDepthFlags;

layout(push_constant) uniform Constants {
    mat4 mvp;
    mat4 model;
    mat4 prev_mvp;
    vec4 camera_pos;
    vec4 light_pos;
    vec4 color;
} push;

vec2 encode_octahedral(vec3 n) {
    n /= abs(n.x) + abs(n.y) + abs(n.z) + 1e-6;
    vec2 enc = n.xy;
    if (n.z < 0.0) {
        enc = (1.0 - abs(enc.yx)) * sign(enc.xy);
    }
    return enc * 0.5 + 0.5;
}

void main() {
    vec3 normal = normalize(vWorldNormal);
    vec3 albedo = max(vColor.rgb, vec3(0.0));
    float alpha = clamp(vColor.a, 0.0, 1.0);
    float metallic = clamp(push.camera_pos.w, 0.0, 1.0);
    float roughness = clamp(push.light_pos.w, 0.04, 1.0);
    float ao = 1.0;
    float material_id = 0.0;
    float flags = alpha < 0.999 ? 1.0 : 0.0;

    outAlbedoAo = vec4(albedo, ao);
    outNormalMaterial = vec4(encode_octahedral(normal), metallic, roughness);
    outEmissiveMaterial = vec4(0.0, 0.0, 0.0, material_id);
    outMotionDepthFlags = vec4(vMotion, gl_FragCoord.z, flags);
}
