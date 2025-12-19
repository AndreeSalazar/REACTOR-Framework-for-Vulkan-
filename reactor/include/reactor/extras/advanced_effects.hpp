#pragma once
#include "../math.hpp"
#include <memory>

namespace reactor {

// Forward declarations
class MemoryAllocator;

/**
 * @brief VolumetricLighting - Iluminación volumétrica
 */
class VolumetricLighting {
public:
    VolumetricLighting(std::shared_ptr<MemoryAllocator> allocator);
    
    float density{0.5f};
    float scattering{0.8f};
    Vec3 color{1, 1, 1};
    
    void render();
};

/**
 * @brief ScreenSpaceReflections - Reflexiones en espacio de pantalla
 */
class ScreenSpaceReflections {
public:
    ScreenSpaceReflections(std::shared_ptr<MemoryAllocator> allocator);
    
    int maxSteps{32};
    float stepSize{0.1f};
    float thickness{0.5f};
    
    void render();
};

/**
 * @brief MotionBlur - Desenfoque de movimiento
 */
class MotionBlur {
public:
    MotionBlur(std::shared_ptr<MemoryAllocator> allocator);
    
    int samples{8};
    float strength{1.0f};
    
    void render();
};

/**
 * @brief DepthOfField - Profundidad de campo
 */
class DepthOfField {
public:
    DepthOfField(std::shared_ptr<MemoryAllocator> allocator);
    
    float focalDistance{10.0f};
    float focalRange{5.0f};
    float bokehSize{1.0f};
    
    void render();
};

} // namespace reactor
