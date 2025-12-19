#pragma once
#include "reactor/reactor.hpp"

namespace test_game {

/**
 * @brief SimpleRenderer - Módulo de rendering simple para Test_Game
 * 
 * Fácil de usar, modificar y eliminar.
 * Estilo modular como Blender.
 */
class SimpleRenderer {
public:
    SimpleRenderer(reactor::VulkanContext& ctx, reactor::Window& window);
    ~SimpleRenderer();
    
    // Fácil de usar
    void beginFrame();
    void drawCube(const reactor::Mat4& mvp, const reactor::Vec3& color);
    void endFrame();
    
    // Fácil de modificar
    void setClearColor(float r, float g, float b);
    void setWireframe(bool enabled);
    
private:
    reactor::VulkanContext& ctx;
    reactor::Window& window;
    
    // Vulkan objects (encapsulados - fácil de eliminar)
    VkSwapchainKHR swapchain{VK_NULL_HANDLE};
    std::vector<VkImage> swapchainImages;
    std::vector<VkImageView> swapchainImageViews;
    VkFormat swapchainFormat;
    VkExtent2D swapchainExtent;
    
    VkRenderPass renderPass{VK_NULL_HANDLE};
    std::vector<VkFramebuffer> framebuffers;
    
    VkPipeline pipeline{VK_NULL_HANDLE};
    VkPipelineLayout pipelineLayout{VK_NULL_HANDLE};
    
    VkCommandPool commandPool{VK_NULL_HANDLE};
    std::vector<VkCommandBuffer> commandBuffers;
    
    VkSemaphore imageAvailableSemaphore{VK_NULL_HANDLE};
    VkSemaphore renderFinishedSemaphore{VK_NULL_HANDLE};
    VkFence inFlightFence{VK_NULL_HANDLE};
    
    uint32_t currentImageIndex{0};
    reactor::Vec3 clearColor{0.1f, 0.1f, 0.1f};
    
    // Helper methods
    void createSwapchain();
    void createRenderPass();
    void createFramebuffers();
    void createPipeline();
    void createCommandBuffers();
    void createSyncObjects();
    
    void cleanup();
};

} // namespace test_game
