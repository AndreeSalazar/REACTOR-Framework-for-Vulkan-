#pragma once
#include "../math.hpp"
#include "../vulkan_context.hpp"
#include <memory>
#include <vector>
#include <string>

namespace reactor {

// Forward declarations
class Window;

/**
 * @brief EasyRenderer - FASE 8: Rendering simplificado
 * 
 * Reduce ~500 líneas de Vulkan a ~10 líneas de código.
 * 
 * Uso ultra simple:
 * EasyRenderer renderer(ctx, window);
 * renderer.beginFrame();
 * renderer.drawMesh(mesh, mvp, color);
 * renderer.endFrame();
 */
class EasyRenderer {
public:
    EasyRenderer(VulkanContext& ctx, Window& window);
    ~EasyRenderer();
    
    // API ultra simple - Solo 3 métodos principales
    void beginFrame();
    void endFrame();
    void drawMesh(const void* vertices, size_t vertexCount, 
                  const uint16_t* indices, size_t indexCount,
                  const Mat4& mvp, const Vec3& color);
    
    // Helpers opcionales
    void setClearColor(float r, float g, float b, float a = 1.0f);
    void setWireframe(bool enabled);
    
    // Getters
    bool isReady() const { return ready; }
    uint32_t getCurrentFrame() const { return currentFrame; }

private:
    VulkanContext& ctx;
    Window& window;
    bool ready{false};
    uint32_t currentFrame{0};
    
    // Vulkan objects (encapsulados)
    VkSurfaceKHR surface{VK_NULL_HANDLE};
    VkSwapchainKHR swapchain{VK_NULL_HANDLE};
    std::vector<VkImage> swapchainImages;
    std::vector<VkImageView> swapchainImageViews;
    VkFormat swapchainFormat;
    VkExtent2D swapchainExtent;
    
    VkRenderPass renderPass{VK_NULL_HANDLE};
    std::vector<VkFramebuffer> framebuffers;
    
    // Depth buffer
    VkImage depthImage{VK_NULL_HANDLE};
    VkDeviceMemory depthImageMemory{VK_NULL_HANDLE};
    VkImageView depthImageView{VK_NULL_HANDLE};
    VkFormat depthFormat{VK_FORMAT_D32_SFLOAT};
    
    // MSAA (Anti-Aliasing) - 4x samples para bordes suaves
    VkSampleCountFlagBits msaaSamples{VK_SAMPLE_COUNT_4_BIT};
    VkImage msaaColorImage{VK_NULL_HANDLE};
    VkDeviceMemory msaaColorMemory{VK_NULL_HANDLE};
    VkImageView msaaColorImageView{VK_NULL_HANDLE};
    VkImage msaaDepthImage{VK_NULL_HANDLE};
    VkDeviceMemory msaaDepthMemory{VK_NULL_HANDLE};
    VkImageView msaaDepthImageView{VK_NULL_HANDLE};
    
    VkPipelineLayout pipelineLayout{VK_NULL_HANDLE};
    VkPipeline pipeline{VK_NULL_HANDLE};
    
    VkCommandPool commandPool{VK_NULL_HANDLE};
    std::vector<VkCommandBuffer> commandBuffers;
    
    VkSemaphore imageAvailableSemaphore{VK_NULL_HANDLE};
    VkSemaphore renderFinishedSemaphore{VK_NULL_HANDLE};
    VkFence inFlightFence{VK_NULL_HANDLE};
    
    VkBuffer vertexBuffer{VK_NULL_HANDLE};
    VkDeviceMemory vertexBufferMemory{VK_NULL_HANDLE};
    VkBuffer indexBuffer{VK_NULL_HANDLE};
    VkDeviceMemory indexBufferMemory{VK_NULL_HANDLE};
    
    uint32_t currentImageIndex{0};
    Vec4 clearColor{0.1f, 0.1f, 0.1f, 1.0f};
    bool wireframeMode{false};
    
    // Métodos internos (simplificados)
    void createSwapchain();
    void createRenderPass();
    void createFramebuffers();
    void createPipeline();
    void createCommandPool();
    void createCommandBuffers();
    void createSyncObjects();
    void createBuffers(const void* vertices, size_t vertexSize,
                      const uint16_t* indices, size_t indexSize);
    
    void cleanup();
    
    // Helpers
    VkShaderModule createShaderModule(const std::vector<char>& code);
    std::vector<char> readFile(const std::string& filename);
    uint32_t findMemoryType(uint32_t typeFilter, VkMemoryPropertyFlags properties);
    void createBuffer(VkDeviceSize size, VkBufferUsageFlags usage, 
                     VkMemoryPropertyFlags properties, VkBuffer& buffer, 
                     VkDeviceMemory& bufferMemory);
};

/**
 * @brief QuickDraw - Helpers para dibujar primitivas comunes
 */
class QuickDraw {
public:
    // Crear geometría simple
    static void cube(std::vector<float>& vertices, std::vector<uint16_t>& indices);
    static void sphere(std::vector<float>& vertices, std::vector<uint16_t>& indices, int segments = 16);
    static void plane(std::vector<float>& vertices, std::vector<uint16_t>& indices);
    
    // Helpers de color
    static Vec3 colorFromHSV(float h, float s, float v);
    static Vec3 colorLerp(const Vec3& a, const Vec3& b, float t);
};

} // namespace reactor
