#include "reactor/material.hpp"

namespace reactor {

Material Material::pbr() {
    Material mat;
    mat.albedo = Vec4(0.8f, 0.8f, 0.8f, 1.0f);
    mat.metallic = 0.0f;
    mat.roughness = 0.5f;
    mat.ao = 1.0f;
    return mat;
}

Material Material::unlit() {
    Material mat;
    mat.albedo = Vec4(1.0f, 1.0f, 1.0f, 1.0f);
    mat.metallic = 0.0f;
    mat.roughness = 1.0f;
    mat.ao = 1.0f;
    return mat;
}

Material Material::wireframe() {
    Material mat;
    mat.albedo = Vec4(0.0f, 1.0f, 0.0f, 1.0f);
    mat.metallic = 0.0f;
    mat.roughness = 1.0f;
    mat.ao = 1.0f;
    return mat;
}

Material& Material::setAlbedo(float r, float g, float b, float a) {
    albedo = Vec4(r, g, b, a);
    return *this;
}

Material& Material::setMetallic(float value) {
    metallic = value;
    return *this;
}

Material& Material::setRoughness(float value) {
    roughness = value;
    return *this;
}

} // namespace reactor
