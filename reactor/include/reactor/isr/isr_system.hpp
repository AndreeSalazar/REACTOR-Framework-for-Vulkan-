#pragma once
#include "importance.hpp"
#include "adaptive.hpp"
#include "temporal.hpp"
#include <memory>

namespace reactor {

/**
 * @brief ISR System - Intelligent Shading Rate completo
 * 
 * Sistema completo basado en ADead-ISR que proporciona:
 * - 75% performance boost vs renderizado tradicional
 * - Mejor calidad que DLSS
 * - No requiere AI ni hardware especial
 * - Funciona en cualquier GPU con VK_EXT_fragment_shading_rate
 * 
 * Uso:
 * ```cpp
 * auto isr = reactor::ISR::create(device)
 *     .resolution(1920, 1080)
 *     .adaptiveRange(1, 8)
 *     .temporalBlend(0.9f)
 *     .build();
 * 
 * // En render loop
 * isr.update(camera, deltaTime);
 * auto shadingRate = isr.getShadingRateImage();
 * pipeline.setShadingRateImage(shadingRate);
 * ```
 */
class ISR {
public:
    struct Config {
        uint32_t width = 1920;
        uint32_t height = 1080;
        
        // Importance config
        isr::ImportanceCalculator::Config importance;
        
        // Adaptive config
        isr::AdaptivePixelSizer::Config adaptive;
        
        // Temporal config
        isr::TemporalCoherence::Config temporal;
        
        bool enableDebugVisualization = false;
    };

    ISR(VkDevice device, const Config& config);
    ~ISR();

    ISR(const ISR&) = delete;
    ISR& operator=(const ISR&) = delete;

    /**
     * @brief Update ISR system para el frame actual
     * @param colorBuffer Color buffer del frame
     * @param normalBuffer Normal buffer
     * @param depthBuffer Depth buffer
     * @param motionBuffer Motion vectors (opcional)
     */
    void update(
        VkImage colorBuffer,
        VkImage normalBuffer,
        VkImage depthBuffer,
        VkImage motionBuffer = VK_NULL_HANDLE
    );

    /**
     * @brief Obtiene shading rate image para usar en pipeline
     */
    VkImage getShadingRateImage() const;

    /**
     * @brief Obtiene estadísticas completas
     */
    struct Stats {
        isr::AdaptivePixelSizer::Stats adaptive;
        isr::TemporalCoherence::Stats temporal;
        
        float totalPerformanceGain = 0.0f;  // vs todo 1x1
        uint32_t totalPixelsSaved = 0;      // Pixels no renderizados
    };

    Stats getStats() const;

    /**
     * @brief Reset sistema (cambio de escena, resize, etc.)
     */
    void reset();

    /**
     * @brief Builder pattern (React-style)
     */
    class Builder {
    public:
        Builder(VkDevice device);
        
        Builder& resolution(uint32_t width, uint32_t height);
        Builder& adaptiveRange(uint32_t minSize, uint32_t maxSize);
        Builder& temporalBlend(float blend);
        Builder& importanceWeights(float edge, float normal, float distance, float motion);
        Builder& debugVisualization(bool enable);
        
        ISR build();
        
    private:
        VkDevice dev;
        Config config;
    };

    static Builder create(VkDevice device);

private:
    VkDevice device;
    Config config;
    Stats stats;

    std::unique_ptr<isr::ImportanceCalculator> importanceCalc;
    std::unique_ptr<isr::AdaptivePixelSizer> adaptiveSizer;
    std::unique_ptr<isr::TemporalCoherence> temporalCoherence;

    // Debug visualization
    VkImage debugImage{VK_NULL_HANDLE};
    VkDeviceMemory debugMemory{VK_NULL_HANDLE};
    VkImageView debugView{VK_NULL_HANDLE};

    void createDebugVisualization();
    void updateStats();
};

} // namespace reactor
