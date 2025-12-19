#pragma once
#include "../math.hpp"
#include <vector>
#include <memory>

namespace reactor {

// Forward declarations
class MemoryAllocator;

/**
 * @brief Particle - Partícula individual
 */
struct Particle {
    Vec3 position{0, 0, 0};
    Vec3 velocity{0, 0, 0};
    Vec4 color{1, 1, 1, 1};
    float size{1.0f};
    float lifetime{1.0f};
    float age{0.0f};
    bool active{false};
};

/**
 * @brief ParticleEmitter - Emisor de partículas
 * 
 * Uso simple:
 * ParticleEmitter emitter(allocator, 1000);
 * emitter.position = Vec3(0, 0, 0);
 * emitter.emissionRate = 50.0f;
 * emitter.update(deltaTime);
 */
class ParticleEmitter {
public:
    ParticleEmitter(std::shared_ptr<MemoryAllocator> allocator, size_t maxParticles);
    ~ParticleEmitter();

    ParticleEmitter(const ParticleEmitter&) = delete;
    ParticleEmitter& operator=(const ParticleEmitter&) = delete;
    ParticleEmitter(ParticleEmitter&& other) noexcept = default;
    ParticleEmitter& operator=(ParticleEmitter&& other) noexcept = default;

    /**
     * @brief Propiedades del emisor
     */
    Vec3 position{0, 0, 0};
    Vec3 direction{0, 1, 0};
    float emissionRate{10.0f};  // particles per second
    float particleLifetime{2.0f};
    float particleSize{1.0f};
    Vec4 startColor{1, 1, 1, 1};
    Vec4 endColor{1, 1, 1, 0};
    float speed{5.0f};
    float spread{30.0f};  // degrees
    
    /**
     * @brief Update
     */
    void update(float deltaTime);
    
    /**
     * @brief Emit manual
     */
    void emit(size_t count = 1);
    
    /**
     * @brief Getters
     */
    const std::vector<Particle>& particles() const { return particleList; }
    size_t activeCount() const;
    size_t maxCount() const { return particleList.size(); }
    
    /**
     * @brief Presets
     */
    static ParticleEmitter fire(std::shared_ptr<MemoryAllocator> allocator);
    static ParticleEmitter smoke(std::shared_ptr<MemoryAllocator> allocator);
    static ParticleEmitter explosion(std::shared_ptr<MemoryAllocator> allocator);

private:
    std::shared_ptr<MemoryAllocator> allocator;
    std::vector<Particle> particleList;
    float emissionAccumulator{0.0f};
    
    void spawnParticle();
};

} // namespace reactor
