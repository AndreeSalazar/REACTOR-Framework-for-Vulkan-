#include "reactor/rendering/shadow_map.hpp"
#include "reactor/memory_allocator.hpp"
#include <glm/gtc/matrix_transform.hpp>
#include <iostream>

namespace reactor {

ShadowMap::ShadowMap(std::shared_ptr<MemoryAllocator> allocator, uint32_t width, uint32_t height)
    : allocator(allocator), shadowWidth(width), shadowHeight(height) {
    std::cout << "[ShadowMap] Created " << width << "x" << height << " shadow map" << std::endl;
    // TODO: Create depth image for shadow mapping
}

ShadowMap::~ShadowMap() = default;

ShadowMap::ShadowMap(ShadowMap&& other) noexcept
    : allocator(std::move(other.allocator))
    , shadowWidth(other.shadowWidth)
    , shadowHeight(other.shadowHeight) {
    other.shadowWidth = 0;
    other.shadowHeight = 0;
}

ShadowMap& ShadowMap::operator=(ShadowMap&& other) noexcept {
    if (this != &other) {
        allocator = std::move(other.allocator);
        shadowWidth = other.shadowWidth;
        shadowHeight = other.shadowHeight;
        
        other.shadowWidth = 0;
        other.shadowHeight = 0;
    }
    return *this;
}

Mat4 ShadowMap::getLightViewMatrix(const Vec3& lightPos, const Vec3& lightDir) const {
    return glm::lookAt(lightPos, lightPos + lightDir, Vec3(0, 1, 0));
}

Mat4 ShadowMap::getLightProjectionMatrix() const {
    // Orthographic projection for directional lights
    float size = 20.0f;
    return glm::ortho(-size, size, -size, size, 0.1f, 100.0f);
}

} // namespace reactor
