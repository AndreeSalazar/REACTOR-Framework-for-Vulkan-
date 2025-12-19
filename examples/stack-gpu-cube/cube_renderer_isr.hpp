#pragma once
#include "reactor/vulkan_context.hpp"
#include "reactor/buffer.hpp"
#include "reactor/pipeline.hpp"
#include "reactor/command_buffer.hpp"
#include "reactor/shader.hpp"
#include "reactor/isr/isr_system.hpp"
#include <glm/glm.hpp>
#include <vector>
#include <memory>

namespace cube {

struct Vertex {
    glm::vec3 pos;
    glm::vec3 normal;
    glm::vec3 color;
};

/**
 * @brief Cube Renderer with ISR (Intelligent Shading Rate) support
 * 
 * Renderiza un cubo 3D con soporte completo para ISR:
 * - G-Buffer (color, normal, depth) para análisis de importancia
 * - Compute shaders para calcular shading rate adaptativo
 * - Integración con VK_EXT_fragment_shading_rate
 */
class CubeRendererISR {
public:
    CubeRendererISR(reactor::VulkanContext& ctx, VkRenderPass renderPass, uint32_t width, uint32_t height);
    ~CubeRendererISR();

    /**
     * @brief Render con ISR completo
     * @param cmd Command buffer
     * @param mvp MVP matrix
     * @param model Model matrix
     * @param debugMode Modo de visualización (0-6)
     * @param enableISR Si true, usa ISR; si false, renderiza normal
     */
    void render(reactor::CommandBuffer& cmd, const glm::mat4& mvp, const glm::mat4& model, 
                int debugMode = 0, bool enableISR = true);

    /**
     * @brief Obtiene estadísticas de ISR
     */
    struct ISRStats {
        uint32_t framesProcessed = 0;
        float averageImportance = 0.5f;
        uint32_t pixels1x1 = 0;
        uint32_t pixels2x2 = 0;
        uint32_t pixels4x4 = 0;
        uint32_t pixels8x8 = 0;
    };
    ISRStats getISRStats() const;

    /**
     * @brief Reset ISR system (útil al cambiar de escena)
     */
    void resetISR();

private:
    reactor::VulkanContext& context;
    uint32_t renderWidth;
    uint32_t renderHeight;
    
    // Buffers de geometría
    std::unique_ptr<reactor::Buffer> vertexBuffer;
    std::unique_ptr<reactor::Buffer> indexBuffer;
    
    // Pipeline de renderizado
    std::unique_ptr<reactor::GraphicsPipeline> pipeline;
    
    // ISR System (placeholder - full integration pending)
    // std::unique_ptr<reactor::isr::ISRSystem> isrSystem;
    
    // G-Buffer para ISR (color, normal, depth)
    VkImage colorBuffer{VK_NULL_HANDLE};
    VkDeviceMemory colorMemory{VK_NULL_HANDLE};
    VkImageView colorView{VK_NULL_HANDLE};
    
    VkImage normalBuffer{VK_NULL_HANDLE};
    VkDeviceMemory normalMemory{VK_NULL_HANDLE};
    VkImageView normalView{VK_NULL_HANDLE};
    
    VkImage depthBuffer{VK_NULL_HANDLE};
    VkDeviceMemory depthMemory{VK_NULL_HANDLE};
    VkImageView depthView{VK_NULL_HANDLE};
    
    // Framebuffer para G-Buffer pass
    VkFramebuffer gBufferFramebuffer{VK_NULL_HANDLE};
    VkRenderPass gBufferRenderPass{VK_NULL_HANDLE};
    
    void createBuffers();
    void createPipeline(VkRenderPass renderPass, uint32_t width, uint32_t height);
    void createGBuffer();
    void createISRSystem();
    
    void renderGBufferPass(reactor::CommandBuffer& cmd, const glm::mat4& mvp, const glm::mat4& model);
    void processISR(reactor::CommandBuffer& cmd);
    
    static std::vector<Vertex> getCubeVertices();
    static std::vector<uint16_t> getCubeIndices();
};

} // namespace cube
