#pragma once
#include "primitives.hpp"
#include "../pipeline.hpp"
#include "../buffer.hpp"
#include "../command_buffer.hpp"
#include <memory>

namespace reactor {
namespace sdf {

/**
 * @brief Ray Marcher - Renderiza escenas SDF usando ray marching
 * 
 * Implementación pura Vulkan (NO DirectX 12)
 * Basado en ADead-Vector3D pero adaptado a Vulkan
 */
class RayMarcher {
public:
    struct Config {
        uint32_t width = 1920;
        uint32_t height = 1080;
        uint32_t maxSteps = 128;        // Máximo de pasos de ray marching
        float maxDistance = 100.0f;     // Distancia máxima
        float epsilon = 0.001f;         // Precisión de hit
        bool enableAntialiasing = true; // SDF Anti-Aliasing
        bool enableSoftShadows = false; // Soft shadows (cone tracing)
        bool enableAO = false;          // Ambient occlusion
    };

    RayMarcher(VkDevice device, VkRenderPass renderPass, const Config& config);
    ~RayMarcher();

    RayMarcher(const RayMarcher&) = delete;
    RayMarcher& operator=(const RayMarcher&) = delete;

    /**
     * @brief Renderiza escena SDF
     * @param commandBuffer Command buffer de Vulkan
     * @param scene Escena SDF a renderizar
     * @param camera Cámara (view + projection matrices)
     */
    void render(
        CommandBuffer& commandBuffer,
        const SDFScene& scene,
        const glm::mat4& view,
        const glm::mat4& proj
    );

    /**
     * @brief Actualiza configuración
     */
    void updateConfig(const Config& config);

    /**
     * @brief Builder pattern (React-style)
     */
    class Builder {
    public:
        Builder(VkDevice device, VkRenderPass renderPass);
        
        Builder& resolution(uint32_t width, uint32_t height);
        Builder& maxSteps(uint32_t steps);
        Builder& antialiasing(bool enable);
        Builder& softShadows(bool enable);
        Builder& ambientOcclusion(bool enable);
        
        RayMarcher build();
        
    private:
        VkDevice dev;
        VkRenderPass pass;
        Config config;
    };

    static Builder create(VkDevice device, VkRenderPass renderPass);

private:
    VkDevice device;
    VkRenderPass renderPass;
    Config config;

    // Pipeline para ray marching (fullscreen quad)
    std::unique_ptr<GraphicsPipeline> pipeline;
    
    // Uniform buffer para configuración
    std::unique_ptr<Buffer> configBuffer;
    
    // Descriptor sets
    VkDescriptorSetLayout descriptorLayout{VK_NULL_HANDLE};
    VkDescriptorPool descriptorPool{VK_NULL_HANDLE};
    VkDescriptorSet descriptorSet{VK_NULL_HANDLE};

    void createPipeline();
    void createDescriptorSets();
    void updateUniforms(const SDFScene& scene, const glm::mat4& view, const glm::mat4& proj);
};

} // namespace sdf
} // namespace reactor
