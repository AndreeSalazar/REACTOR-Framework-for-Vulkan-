#include "reactor/rendering/particle_system.hpp"
#include "reactor/memory_allocator.hpp"
#include <glm/gtc/constants.hpp>
#include <iostream>
#include <random>

namespace reactor {

static std::random_device rd;
static std::mt19937 gen(rd());

ParticleEmitter::ParticleEmitter(std::shared_ptr<MemoryAllocator> allocator, size_t maxParticles)
    : allocator(allocator) {
    particleList.resize(maxParticles);
    std::cout << "[ParticleEmitter] Created with " << maxParticles << " max particles" << std::endl;
}

ParticleEmitter::~ParticleEmitter() = default;

void ParticleEmitter::update(float deltaTime) {
    // Update existing particles
    for (auto& particle : particleList) {
        if (!particle.active) continue;
        
        particle.age += deltaTime;
        if (particle.age >= particle.lifetime) {
            particle.active = false;
            continue;
        }
        
        // Update position
        particle.position += particle.velocity * deltaTime;
        
        // Update color (fade out)
        float t = particle.age / particle.lifetime;
        particle.color = glm::mix(startColor, endColor, t);
    }
    
    // Emit new particles
    emissionAccumulator += deltaTime * emissionRate;
    while (emissionAccumulator >= 1.0f) {
        spawnParticle();
        emissionAccumulator -= 1.0f;
    }
}

void ParticleEmitter::emit(size_t count) {
    for (size_t i = 0; i < count; i++) {
        spawnParticle();
    }
}

void ParticleEmitter::spawnParticle() {
    // Find inactive particle
    for (auto& particle : particleList) {
        if (particle.active) continue;
        
        // Initialize particle
        particle.active = true;
        particle.position = position;
        particle.age = 0.0f;
        particle.lifetime = particleLifetime;
        particle.size = particleSize;
        particle.color = startColor;
        
        // Random velocity within spread cone
        std::uniform_real_distribution<float> spreadDist(-spread, spread);
        float angleX = glm::radians(spreadDist(gen));
        float angleY = glm::radians(spreadDist(gen));
        
        // Use matrix rotation instead of glm::rotate on Vec3
        Mat4 rotMat = glm::rotate(Mat4(1.0f), angleX, Vec3(1, 0, 0));
        rotMat = glm::rotate(rotMat, angleY, Vec3(0, 1, 0));
        Vec3 vel = Vec3(rotMat * Vec4(direction, 0.0f));
        particle.velocity = glm::normalize(vel) * speed;
        
        return;
    }
}

size_t ParticleEmitter::activeCount() const {
    size_t count = 0;
    for (const auto& particle : particleList) {
        if (particle.active) count++;
    }
    return count;
}

ParticleEmitter ParticleEmitter::fire(std::shared_ptr<MemoryAllocator> allocator) {
    ParticleEmitter emitter(allocator, 500);
    emitter.emissionRate = 50.0f;
    emitter.particleLifetime = 1.5f;
    emitter.particleSize = 0.5f;
    emitter.startColor = Vec4(1.0f, 0.5f, 0.0f, 1.0f);
    emitter.endColor = Vec4(0.5f, 0.0f, 0.0f, 0.0f);
    emitter.speed = 3.0f;
    emitter.spread = 20.0f;
    emitter.direction = Vec3(0, 1, 0);
    std::cout << "[ParticleEmitter] Created fire preset" << std::endl;
    return emitter;
}

ParticleEmitter ParticleEmitter::smoke(std::shared_ptr<MemoryAllocator> allocator) {
    ParticleEmitter emitter(allocator, 300);
    emitter.emissionRate = 20.0f;
    emitter.particleLifetime = 3.0f;
    emitter.particleSize = 1.0f;
    emitter.startColor = Vec4(0.5f, 0.5f, 0.5f, 0.8f);
    emitter.endColor = Vec4(0.3f, 0.3f, 0.3f, 0.0f);
    emitter.speed = 1.5f;
    emitter.spread = 30.0f;
    emitter.direction = Vec3(0, 1, 0);
    std::cout << "[ParticleEmitter] Created smoke preset" << std::endl;
    return emitter;
}

ParticleEmitter ParticleEmitter::explosion(std::shared_ptr<MemoryAllocator> allocator) {
    ParticleEmitter emitter(allocator, 1000);
    emitter.emissionRate = 0.0f;  // Manual emission
    emitter.particleLifetime = 2.0f;
    emitter.particleSize = 0.3f;
    emitter.startColor = Vec4(1.0f, 0.8f, 0.0f, 1.0f);
    emitter.endColor = Vec4(0.5f, 0.0f, 0.0f, 0.0f);
    emitter.speed = 10.0f;
    emitter.spread = 180.0f;
    emitter.direction = Vec3(0, 1, 0);
    std::cout << "[ParticleEmitter] Created explosion preset" << std::endl;
    return emitter;
}

} // namespace reactor
