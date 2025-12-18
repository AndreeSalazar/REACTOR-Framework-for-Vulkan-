#pragma once
#include <vulkan/vulkan.h>
#include <glm/glm.hpp>

namespace reactor {
namespace isr {

/**
 * @brief Adaptive Pixel Sizing - Ajusta tamaño de pixel basado en importancia
 * 
 * Basado en ADead-ISR:
 * - Importancia alta (0.8-1.0) → 1x1 pixels (máxima calidad)
 * - Importancia media (0.5-0.8) → 2x2 pixels
 * - Importancia baja (0.3-0.5) → 4x4 pixels
 * - Importancia muy baja (0.0-0.3) → 8x8 pixels
 */
class AdaptivePixelSizer {
public:
    struct Config {
        uint32_t minPixelSize = 1;      // Mínimo: 1x1 (máxima calidad)
        uint32_t maxPixelSize = 8;      // Máximo: 8x8 (mínima calidad)
        
        // Umbrales de importancia
        float threshold1x1 = 0.8f;      // >= 0.8 → 1x1
        float threshold2x2 = 0.5f;      // >= 0.5 → 2x2
        float threshold4x4 = 0.3f;      // >= 0.3 → 4x4
        // < 0.3 → 8x8
        
        bool enableHierarchical = true; // Análisis jerárquico (tiles 8x8)
    };

    AdaptivePixelSizer(VkDevice device, const Config& config = {});
    ~AdaptivePixelSizer();

    AdaptivePixelSizer(const AdaptivePixelSizer&) = delete;
    AdaptivePixelSizer& operator=(const AdaptivePixelSizer&) = delete;

    /**
     * @brief Genera imagen de shading rate basada en importancia
     * @param importanceImage Imagen de importancia [0.0, 1.0]
     * @return VkImage compatible con VK_EXT_fragment_shading_rate
     */
    VkImage generateShadingRateImage(VkImage importanceImage);

    /**
     * @brief Obtiene estadísticas de uso
     */
    struct Stats {
        uint32_t pixels1x1 = 0;
        uint32_t pixels2x2 = 0;
        uint32_t pixels4x4 = 0;
        uint32_t pixels8x8 = 0;
        float averagePixelSize = 1.0f;
        float performanceGain = 0.0f;  // vs todo 1x1
    };

    Stats getStats() const { return stats; }

    void updateConfig(const Config& config);

private:
    VkDevice device;
    Config config;
    Stats stats;

    // Compute pipeline
    VkPipeline computePipeline{VK_NULL_HANDLE};
    VkPipelineLayout pipelineLayout{VK_NULL_HANDLE};
    VkDescriptorSetLayout descriptorLayout{VK_NULL_HANDLE};
    VkDescriptorPool descriptorPool{VK_NULL_HANDLE};
    VkDescriptorSet descriptorSet{VK_NULL_HANDLE};

    // Shading rate image
    VkImage shadingRateImage{VK_NULL_HANDLE};
    VkDeviceMemory shadingRateMemory{VK_NULL_HANDLE};
    VkImageView shadingRateView{VK_NULL_HANDLE};

    void createComputePipeline();
    void createShadingRateImage(uint32_t width, uint32_t height);
    void updateStats(VkImage shadingRateImage);
};

} // namespace isr
} // namespace reactor
