#include "reactor/rendering/light.hpp"
#include <iostream>

namespace reactor {

Light Light::directional(const Vec3& direction) {
    Light light;
    light.type = Type::Directional;
    light.direction = glm::normalize(direction);
    std::cout << "[Light] Created directional light" << std::endl;
    return light;
}

Light Light::point(const Vec3& position, float range) {
    Light light;
    light.type = Type::Point;
    light.position = position;
    light.range = range;
    std::cout << "[Light] Created point light at (" << position.x << ", " << position.y << ", " << position.z << ")" << std::endl;
    return light;
}

Light Light::spot(const Vec3& position, const Vec3& direction, float angle) {
    Light light;
    light.type = Type::Spot;
    light.position = position;
    light.direction = glm::normalize(direction);
    light.outerConeAngle = angle;
    light.innerConeAngle = angle * 0.8f;
    std::cout << "[Light] Created spot light" << std::endl;
    return light;
}

Light& Light::setColor(float r, float g, float b) {
    color = Vec3(r, g, b);
    return *this;
}

Light& Light::setIntensity(float i) {
    intensity = i;
    return *this;
}

Light& Light::enableShadows(bool enable) {
    castShadows = enable;
    return *this;
}

Light* LightManager::addLight(const Light& light) {
    lightList.push_back(light);
    return &lightList.back();
}

size_t LightManager::directionalCount() const {
    size_t count = 0;
    for (const auto& light : lightList) {
        if (light.type == Light::Type::Directional) count++;
    }
    return count;
}

size_t LightManager::pointCount() const {
    size_t count = 0;
    for (const auto& light : lightList) {
        if (light.type == Light::Type::Point) count++;
    }
    return count;
}

size_t LightManager::spotCount() const {
    size_t count = 0;
    for (const auto& light : lightList) {
        if (light.type == Light::Type::Spot) count++;
    }
    return count;
}

} // namespace reactor
