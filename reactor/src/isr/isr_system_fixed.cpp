#include "reactor/isr/isr_system.hpp"
#include <stdexcept>

namespace reactor {
namespace isr {

ISRSystem::ISRSystem(VkDevice dev, const Config& cfg)
    : device(dev), config(cfg) {
    // Initialize ISR components with configs
    ImportanceCalculator::Config importanceConfig;
    importanceConfig.edgeWeight = cfg.importanceEdgeWeight;
    importanceConfig.normalWeight = cfg.importanceNormalWeight;
    importanceConfig.distanceWeight = cfg.importanceDistanceWeight;
    importanceConfig.motionWeight = cfg.importanceMotionWeight;
    
    AdaptivePixelSizer::Config adaptiveConfig;
    adaptiveConfig.threshold1x1 = cfg.threshold1x1;
    adaptiveConfig.threshold2x2 = cfg.threshold2x2;
    adaptiveConfig.threshold4x4 = cfg.threshold4x4;
    
    TemporalCoherence::Config temporalConfig;
    temporalConfig.blendFactor = cfg.temporalBlendFactor;
    temporalConfig.useMotionVectors = cfg.useMotionVectors;
    
    // Create ISR components
    importance = std::make_unique<ImportanceCalculator>(device, importanceConfig);
    adaptive = std::make_unique<AdaptivePixelSizer>(device, adaptiveConfig);
    temporal = std::make_unique<TemporalCoherence>(device, temporalConfig);
}

ISRSystem::~ISRSystem() {
    // Cleanup handled by unique_ptrs
}

void ISRSystem::process(
    VkCommandBuffer cmd,
    VkImage colorBuffer,
    VkImage normalBuffer,
    VkImage depthBuffer,
    VkImage motionBuffer) {
    
    // ISR Pipeline - 3 stages:
    
    // 1. Calculate importance map from scene buffers
    VkImage importanceMap = importance->calculateImportance(
        cmd, colorBuffer, normalBuffer, depthBuffer, motionBuffer
    );
    
    // 2. Apply temporal coherence (blend with previous frame)
    VkImage blendedImportance = temporal->applyTemporalCoherence(
        cmd, importanceMap, motionBuffer
    );
    
    // 3. Generate shading rate image from importance
    currentShadingRate = adaptive->generateShadingRateImage(
        cmd, blendedImportance
    );
    
    // Update stats
    stats.framesProcessed++;
}

VkImage ISRSystem::getShadingRateImage() const {
    return currentShadingRate;
}

void ISRSystem::updateConfig(const Config& newConfig) {
    config = newConfig;
    
    // Update component configs
    ImportanceCalculator::Config importanceConfig;
    importanceConfig.edgeWeight = newConfig.importanceEdgeWeight;
    importanceConfig.normalWeight = newConfig.importanceNormalWeight;
    importanceConfig.distanceWeight = newConfig.importanceDistanceWeight;
    importanceConfig.motionWeight = newConfig.importanceMotionWeight;
    importance->updateConfig(importanceConfig);
    
    AdaptivePixelSizer::Config adaptiveConfig;
    adaptiveConfig.threshold1x1 = newConfig.threshold1x1;
    adaptiveConfig.threshold2x2 = newConfig.threshold2x2;
    adaptiveConfig.threshold4x4 = newConfig.threshold4x4;
    adaptive->updateConfig(adaptiveConfig);
    
    TemporalCoherence::Config temporalConfig;
    temporalConfig.blendFactor = newConfig.temporalBlendFactor;
    temporalConfig.useMotionVectors = newConfig.useMotionVectors;
    temporal->updateConfig(temporalConfig);
}

ISRSystem::Stats ISRSystem::getStats() const {
    return stats;
}

} // namespace isr
} // namespace reactor
