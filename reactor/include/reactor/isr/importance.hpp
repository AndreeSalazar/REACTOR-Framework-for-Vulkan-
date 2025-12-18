#pragma once
#include <vulkan/vulkan.h>
#include <glm/glm.hpp>
#include <memory>

namespace reactor {
namespace isr {

/**
 * @brief Importance Calculator - Calcula la importancia visual de cada pixel
 * 
 * Basado en ADead-ISR, calcula importancia usando:
 * - Edge detection (cambios de color/normal)
 * - Distance to camera
 * - Motion vectors
 * - Silhouette detection
 */
class ImportanceCalculator {
public:
    struct Config {
        float edgeWeight = 0.4f;        // Peso de detección de bordes
        float normalWeight = 0.3f;      // Peso de variación de normales
        float distanceWeight = 0.2f;    // Peso de distancia a cámara
        float motionWeight = 0.1f;      // Peso de vectores de movimiento
        float silhouetteThreshold = 0.7f; // Umbral para siluetas
    };

    ImportanceCalculator(VkDevice device, const Config& config = {});
    ~ImportanceCalculator();

    ImportanceCalculator(const ImportanceCalculator&) = delete;
    ImportanceCalculator& operator=(const ImportanceCalculator&) = delete;

    /**
     * @brief Calcula mapa de importancia para un frame
     * @param colorBuffer Buffer de color del frame
     * @param normalBuffer Buffer de normales
     * @param depthBuffer Buffer de profundidad
     * @param motionBuffer Buffer de vectores de movimiento (opcional)
     * @return Imagen con valores de importancia [0.0, 1.0]
     */
    VkImage calculateImportance(
        VkImage colorBuffer,
        VkImage normalBuffer,
        VkImage depthBuffer,
        VkImage motionBuffer = VK_NULL_HANDLE
    );

    /**
     * @brief Actualiza configuración
     */
    void updateConfig(const Config& config);

    /**
     * @brief Obtiene imagen de importancia actual
     */
    VkImage getImportanceImage() const { return importanceImage; }

private:
    VkDevice device;
    Config config;

    // Compute pipeline para cálculo de importancia
    VkPipeline computePipeline{VK_NULL_HANDLE};
    VkPipelineLayout pipelineLayout{VK_NULL_HANDLE};
    VkDescriptorSetLayout descriptorLayout{VK_NULL_HANDLE};
    VkDescriptorPool descriptorPool{VK_NULL_HANDLE};
    VkDescriptorSet descriptorSet{VK_NULL_HANDLE};

    // Imagen de salida
    VkImage importanceImage{VK_NULL_HANDLE};
    VkDeviceMemory importanceMemory{VK_NULL_HANDLE};
    VkImageView importanceView{VK_NULL_HANDLE};

    void createComputePipeline();
    void createDescriptorSets();
    void createImportanceImage(uint32_t width, uint32_t height);
};

} // namespace isr
} // namespace reactor
