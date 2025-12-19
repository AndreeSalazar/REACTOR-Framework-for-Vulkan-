#include "simple_renderer.hpp"
#include "reactor/rendering/easy_renderer.hpp"
#include <iostream>
#include <array>
#include <fstream>
#include <vector>

namespace test_game {

// Usar EasyRenderer de FASE 8
static reactor::EasyRenderer* easyRenderer = nullptr;

SimpleRenderer::SimpleRenderer(reactor::VulkanContext& ctx, reactor::Window& window)
    : ctx(ctx), window(window) {
    
    std::cout << "[SimpleRenderer] Inicializando..." << std::endl;
    
    createSwapchain();
    createRenderPass();
    createFramebuffers();
    createPipeline();
    createCommandBuffers();
    createSyncObjects();
    
    std::cout << "[SimpleRenderer] ✓ Listo para renderizar" << std::endl;
}

SimpleRenderer::~SimpleRenderer() {
    cleanup();
}

void SimpleRenderer::createSwapchain() {
    // Por ahora, solo placeholder
    // TODO: Implementar swapchain real
    std::cout << "[SimpleRenderer] Swapchain creado (placeholder)" << std::endl;
}

void SimpleRenderer::createRenderPass() {
    // Por ahora, solo placeholder
    std::cout << "[SimpleRenderer] RenderPass creado (placeholder)" << std::endl;
}

void SimpleRenderer::createFramebuffers() {
    std::cout << "[SimpleRenderer] Framebuffers creados (placeholder)" << std::endl;
}

void SimpleRenderer::createPipeline() {
    std::cout << "[SimpleRenderer] Pipeline creado (placeholder)" << std::endl;
}

void SimpleRenderer::createCommandBuffers() {
    std::cout << "[SimpleRenderer] Command buffers creados (placeholder)" << std::endl;
}

void SimpleRenderer::createSyncObjects() {
    std::cout << "[SimpleRenderer] Sync objects creados (placeholder)" << std::endl;
}

void SimpleRenderer::beginFrame() {
    // Por ahora, rendering simplificado
    // En una implementación completa, aquí iría vkAcquireNextImageKHR
}

void SimpleRenderer::drawCube(const reactor::Mat4& mvp, const reactor::Vec3& color) {
    // Simulación de rendering
    // En una implementación completa, aquí irían los comandos de dibujo Vulkan
    static int frameCounter = 0;
    if (frameCounter++ % 60 == 0) {
        std::cout << "[SimpleRenderer] Renderizando cubo (color: " 
                  << color.r << ", " << color.g << ", " << color.b << ")" << std::endl;
    }
}

void SimpleRenderer::endFrame() {
    // Por ahora, rendering simplificado
    // En una implementación completa, aquí iría vkQueuePresentKHR
}

void SimpleRenderer::setClearColor(float r, float g, float b) {
    clearColor = reactor::Vec3(r, g, b);
}

void SimpleRenderer::setWireframe(bool enabled) {
    std::cout << "[SimpleRenderer] Wireframe: " << (enabled ? "ON" : "OFF") << std::endl;
}

void SimpleRenderer::cleanup() {
    std::cout << "[SimpleRenderer] Limpiando recursos..." << std::endl;
    
    // Cleanup Vulkan objects
    if (inFlightFence != VK_NULL_HANDLE) {
        vkDestroyFence(ctx.device(), inFlightFence, nullptr);
    }
    if (renderFinishedSemaphore != VK_NULL_HANDLE) {
        vkDestroySemaphore(ctx.device(), renderFinishedSemaphore, nullptr);
    }
    if (imageAvailableSemaphore != VK_NULL_HANDLE) {
        vkDestroySemaphore(ctx.device(), imageAvailableSemaphore, nullptr);
    }
    if (commandPool != VK_NULL_HANDLE) {
        vkDestroyCommandPool(ctx.device(), commandPool, nullptr);
    }
    for (auto framebuffer : framebuffers) {
        if (framebuffer != VK_NULL_HANDLE) {
            vkDestroyFramebuffer(ctx.device(), framebuffer, nullptr);
        }
    }
    if (pipeline != VK_NULL_HANDLE) {
        vkDestroyPipeline(ctx.device(), pipeline, nullptr);
    }
    if (pipelineLayout != VK_NULL_HANDLE) {
        vkDestroyPipelineLayout(ctx.device(), pipelineLayout, nullptr);
    }
    if (renderPass != VK_NULL_HANDLE) {
        vkDestroyRenderPass(ctx.device(), renderPass, nullptr);
    }
    for (auto imageView : swapchainImageViews) {
        if (imageView != VK_NULL_HANDLE) {
            vkDestroyImageView(ctx.device(), imageView, nullptr);
        }
    }
    if (swapchain != VK_NULL_HANDLE) {
        vkDestroySwapchainKHR(ctx.device(), swapchain, nullptr);
    }
    
    std::cout << "[SimpleRenderer] ✓ Limpieza completada" << std::endl;
}

} // namespace test_game
