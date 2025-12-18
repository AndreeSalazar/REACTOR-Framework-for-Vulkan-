#pragma once
#include <vulkan/vulkan.h>
#include <glm/glm.hpp>

namespace reactor {
namespace isr {

/**
 * @brief Temporal Coherence - Reutiliza importancia del frame anterior
 * 
 * Basado en ADead-ISR:
 * - Blend 90% frame anterior + 10% frame actual
 * - Reduce flickering
 * - Mejora estabilidad temporal
 * - Usa motion vectors para reprojection
 */
class TemporalCoherence {
public:
    struct Config {
        float blendFactor = 0.9f;       // 90% anterior, 10% actual
        bool useMotionVectors = true;   // Usar motion vectors para reprojection
        float motionThreshold = 0.1f;   // Umbral de movimiento para invalidar
        bool enableJitterCompensation = true; // Compensar jitter de TAA
    };

    TemporalCoherence(VkDevice device, const Config& config = {});
    ~TemporalCoherence();

    TemporalCoherence(const TemporalCoherence&) = delete;
    TemporalCoherence& operator=(const TemporalCoherence&) = delete;

    /**
     * @brief Aplica coherencia temporal a importancia
     * @param currentImportance Importancia del frame actual
     * @param motionVectors Vectores de movimiento (opcional)
     * @return Importancia con coherencia temporal aplicada
     */
    VkImage applyTemporalCoherence(
        VkImage currentImportance,
        VkImage motionVectors = VK_NULL_HANDLE
    );

    /**
     * @brief Reset temporal history (cambio de escena, etc.)
     */
    void resetHistory();

    /**
     * @brief Obtiene estadísticas
     */
    struct Stats {
        float temporalStability = 0.0f; // [0.0, 1.0] - estabilidad temporal
        uint32_t pixelsReprojected = 0; // Pixels con reprojection exitosa
        uint32_t pixelsInvalidated = 0; // Pixels invalidados por movimiento
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

    // History buffers
    VkImage previousImportance{VK_NULL_HANDLE};
    VkDeviceMemory previousMemory{VK_NULL_HANDLE};
    VkImageView previousView{VK_NULL_HANDLE};

    VkImage outputImportance{VK_NULL_HANDLE};
    VkDeviceMemory outputMemory{VK_NULL_HANDLE};
    VkImageView outputView{VK_NULL_HANDLE};

    bool historyValid = false;

    void createComputePipeline();
    void createHistoryBuffers(uint32_t width, uint32_t height);
    void updateHistory(VkImage current);
};

} // namespace isr
} // namespace reactor
