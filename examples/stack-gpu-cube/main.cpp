#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/window.hpp"
#include "reactor/swapchain.hpp"
#include "reactor/render_pass.hpp"
#include "reactor/pipeline.hpp"
#include "reactor/buffer.hpp"
#include "reactor/command_buffer.hpp"
#include "reactor/sync.hpp"
#include "reactor/shader.hpp"
#include "reactor/math.hpp"
#include "cube_renderer.hpp"
#include <iostream>
#include <chrono>

/**
 * Stack-GPU-OP Cube Demo
 * 
 * Renderiza un cubo 3D usando:
 * - SDF Ray Marching (ADead-Vector3D adaptado a Vulkan)
 * - React-Style API de REACTOR
 * - Vulkan puro (NO DirectX 12)
 */

int main() {
    try {
        std::cout << "==========================================" << std::endl;
        std::cout << "  Stack-GPU-OP: Cubo 3D con SDF" << std::endl;
        std::cout << "  Vulkan Puro + ADead-Vector3D" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        // Inicializar GLFW
        reactor::Window::init();
        
        // Crear ventana
        reactor::WindowConfig config;
        config.title = "Stack-GPU-OP - Cubo 3D (Vulkan + SDF)";
        config.width = 800;
        config.height = 600;
        
        reactor::Window window(config);
        std::cout << "[✓] Ventana creada" << std::endl;
        
        // Inicializar Vulkan
        reactor::VulkanContext ctx(true);
        ctx.init();
        std::cout << "[✓] Vulkan inicializado" << std::endl;
        
        // Crear surface
        VkSurfaceKHR surface = window.createSurface(ctx.instance());
        
        // Crear swapchain
        reactor::Swapchain swapchain(ctx.device(), ctx.physical(), surface, config.width, config.height);
        std::cout << "[✓] Swapchain creado" << std::endl;
        
        // Crear render pass
        std::vector<reactor::AttachmentDescription> attachments = {{
            .format = swapchain.imageFormat(),
            .loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR,
            .storeOp = VK_ATTACHMENT_STORE_OP_STORE,
            .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
            .finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR
        }};
        
        reactor::RenderPass renderPass(ctx.device(), attachments, false);
        std::cout << "[✓] Render pass creado" << std::endl;
        
        // Crear cube renderer
        std::cout << "[3/5] Creando cube renderer..." << std::endl;
        cube::CubeRenderer cubeRenderer(ctx, renderPass.handle(), config.width, config.height);
        std::cout << "[✓] Cube renderer creado" << std::endl;
        
        // Crear framebuffers
        std::vector<VkFramebuffer> framebuffers;
        for (size_t i = 0; i < swapchain.imageCount(); i++) {
            VkFramebufferCreateInfo fbInfo{};
            fbInfo.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
            fbInfo.renderPass = renderPass.handle();
            fbInfo.attachmentCount = 1;
            fbInfo.pAttachments = &swapchain.imageViews()[i];
            fbInfo.width = config.width;
            fbInfo.height = config.height;
            fbInfo.layers = 1;
            
            VkFramebuffer fb;
            vkCreateFramebuffer(ctx.device(), &fbInfo, nullptr, &fb);
            framebuffers.push_back(fb);
        }
        
        // Command pool y buffers
        reactor::CommandPool cmdPool(ctx.device(), ctx.queueFamilyIndices().graphics.value());
        auto cmdPoolPtr = std::make_shared<reactor::CommandPool>(std::move(cmdPool));
        
        std::vector<reactor::CommandBuffer> cmdBuffers;
        for (size_t i = 0; i < swapchain.imageCount(); i++) {
            cmdBuffers.emplace_back(cmdPoolPtr);
        }
        
        // Sync objects
        const int MAX_FRAMES = 2;
        std::vector<reactor::Semaphore> imageAvailable;
        std::vector<reactor::Semaphore> renderFinished;
        std::vector<reactor::Fence> inFlight;
        
        for (int i = 0; i < MAX_FRAMES; i++) {
            imageAvailable.emplace_back(ctx.device());
            renderFinished.emplace_back(ctx.device());
            inFlight.emplace_back(ctx.device(), true);
        }
        std::cout << "[✓] Sincronización configurada" << std::endl;
        
        // Camera (React-style)
        reactor::Camera camera;
        camera.position = glm::vec3(3.0f, 3.0f, 3.0f);
        camera.target = glm::vec3(0.0f, 0.0f, 0.0f);
        camera.aspectRatio = static_cast<float>(config.width) / config.height;
        
        // Transform para rotación
        reactor::Transform cubeTransform;
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  [✓] Stack-GPU-OP listo!" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "Renderizando cubo con SDF Ray Marching..." << std::endl;
        std::cout << "Controles: ESC para salir" << std::endl;
        std::cout << std::endl;
        
        // Render loop
        size_t currentFrame = 0;
        auto startTime = std::chrono::high_resolution_clock::now();
        int frameCount = 0;
        auto lastFpsTime = startTime;
        
        std::vector<VkFence> imagesInFlight(swapchain.imageCount(), VK_NULL_HANDLE);
        
        while (!window.shouldClose()) {
            window.pollEvents();
            
            auto currentTime = std::chrono::high_resolution_clock::now();
            float time = std::chrono::duration<float>(currentTime - startTime).count();
            
            // Animar cubo (rotación)
            cubeTransform.rotation.y = time * glm::radians(45.0f);
            cubeTransform.rotation.x = time * glm::radians(30.0f);
            
            // Wait for fence
            inFlight[currentFrame].wait();
            
            // Acquire image
            uint32_t imageIndex = swapchain.acquireNextImage(imageAvailable[currentFrame].handle());
            
            // Check if a previous frame is using this image
            if (imagesInFlight[imageIndex] != VK_NULL_HANDLE) {
                vkWaitForFences(ctx.device(), 1, &imagesInFlight[imageIndex], VK_TRUE, UINT64_MAX);
            }
            imagesInFlight[imageIndex] = inFlight[currentFrame].handle();
            
            inFlight[currentFrame].reset();
            
            // Calcular matrices
            glm::mat4 view = camera.getViewMatrix();
            glm::mat4 proj = camera.getProjectionMatrix();
            
            // Record commands
            auto& cmd = cmdBuffers[imageIndex];
            cmd.reset();
            cmd.begin();
            
            VkClearValue clearColor = {{{0.1f, 0.1f, 0.15f, 1.0f}}};
            
            VkRenderPassBeginInfo rpInfo{};
            rpInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
            rpInfo.renderPass = renderPass.handle();
            rpInfo.framebuffer = framebuffers[imageIndex];
            rpInfo.renderArea.offset = {0, 0};
            rpInfo.renderArea.extent = swapchain.extent();
            rpInfo.clearValueCount = 1;
            rpInfo.pClearValues = &clearColor;
            
            vkCmdBeginRenderPass(cmd.handle(), &rpInfo, VK_SUBPASS_CONTENTS_INLINE);
            
            // Calcular MVP
            glm::mat4 model = cubeTransform.getMatrix();
            glm::mat4 mvp = proj * view * model;
            
            // Renderizar cubo 3D
            cubeRenderer.render(cmd, mvp);
            
            vkCmdEndRenderPass(cmd.handle());
            cmd.end();
            
            // Submit
            VkSubmitInfo submitInfo{};
            submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
            VkSemaphore waitSems[] = {imageAvailable[currentFrame].handle()};
            VkPipelineStageFlags waitStages[] = {VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT};
            submitInfo.waitSemaphoreCount = 1;
            submitInfo.pWaitSemaphores = waitSems;
            submitInfo.pWaitDstStageMask = waitStages;
            VkCommandBuffer cmdBuf = cmd.handle();
            submitInfo.commandBufferCount = 1;
            submitInfo.pCommandBuffers = &cmdBuf;
            VkSemaphore signalSems[] = {renderFinished[currentFrame].handle()};
            submitInfo.signalSemaphoreCount = 1;
            submitInfo.pSignalSemaphores = signalSems;
            
            vkQueueSubmit(ctx.graphicsQueue(), 1, &submitInfo, inFlight[currentFrame].handle());
            
            // Present
            swapchain.present(ctx.graphicsQueue(), imageIndex, renderFinished[currentFrame].handle());
            
            currentFrame = (currentFrame + 1) % MAX_FRAMES;
            frameCount++;
            
            // FPS
            auto elapsed = std::chrono::duration<double>(currentTime - lastFpsTime).count();
            if (elapsed >= 1.0) {
                double fps = frameCount / elapsed;
                std::cout << "FPS: " << static_cast<int>(fps) 
                         << " | Rotación: " << glm::degrees(cubeTransform.rotation.y) << "°" << std::endl;
                frameCount = 0;
                lastFpsTime = currentTime;
            }
        }
        
        // Cleanup
        vkDeviceWaitIdle(ctx.device());
        
        for (auto fb : framebuffers) {
            vkDestroyFramebuffer(ctx.device(), fb, nullptr);
        }
        
        vkDestroySurfaceKHR(ctx.instance(), surface, nullptr);
        ctx.shutdown();
        reactor::Window::terminate();
        
        std::cout << std::endl << "[✓] Stack-GPU-OP finalizado" << std::endl;
        
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "❌ Error: " << e.what() << std::endl;
        reactor::Window::terminate();
        return 1;
    }
}
