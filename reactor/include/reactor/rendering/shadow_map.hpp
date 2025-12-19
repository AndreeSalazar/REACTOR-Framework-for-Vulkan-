#pragma once
#include "../math.hpp"
#include <memory>

namespace reactor {

// Forward declarations
class MemoryAllocator;

/**
 * @brief ShadowMap - Sistema de sombras simplificado
 * 
 * Uso simple:
 * ShadowMap shadowMap(allocator, 2048, 2048);
 * shadowMap.beginRender();
 * // render scene from light POV
 * shadowMap.endRender();
 */
class ShadowMap {
public:
    ShadowMap(std::shared_ptr<MemoryAllocator> allocator, uint32_t width, uint32_t height);
    ~ShadowMap();

    ShadowMap(const ShadowMap&) = delete;
    ShadowMap& operator=(const ShadowMap&) = delete;
    ShadowMap(ShadowMap&& other) noexcept;
    ShadowMap& operator=(ShadowMap&& other) noexcept;

    /**
     * @brief Getters
     */
    uint32_t width() const { return shadowWidth; }
    uint32_t height() const { return shadowHeight; }
    
    /**
     * @brief Obtener matriz de proyección de luz
     */
    Mat4 getLightViewMatrix(const Vec3& lightPos, const Vec3& lightDir) const;
    Mat4 getLightProjectionMatrix() const;

private:
    std::shared_ptr<MemoryAllocator> allocator;
    uint32_t shadowWidth{0};
    uint32_t shadowHeight{0};
    // TODO: Add depth image when needed
};

} // namespace reactor
