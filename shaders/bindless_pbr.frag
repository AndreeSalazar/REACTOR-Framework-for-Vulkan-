#version 450
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_buffer_reference : enable

layout(set = 0, binding = 0) uniform texture2D textures[];
layout(set = 0, binding = 1) uniform sampler samplers[];
layout(set = 0, binding = 2) readonly buffer MaterialData {
    vec4 base_color;
    vec4 emissive;
    float metallic;
    float roughness;
    uint albedo_tex;
    uint normal_tex;
} materials[];

layout(push_constant) uniform PushConstants {
    uint material_idx;
    uint sampler_idx;
} pc;

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec3 v_normal;
layout(location = 0) out vec4 out_color;

void main() {
    uint mat_idx = nonuniformEXT(pc.material_idx);
    uint samp_idx = pc.sampler_idx;
    uint tex_idx = nonuniformEXT(materials[mat_idx].albedo_tex);
    vec4 albedo = texture(sampler2D(textures[tex_idx], samplers[samp_idx]), v_uv);
    out_color = albedo * materials[mat_idx].base_color;
}
