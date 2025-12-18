#include <reactor/vulkan_context.hpp>
#include <reactor/window.hpp>
#include <reactor/swapchain.hpp>
#include <reactor/render_pass.hpp>
#include <reactor/command_buffer.hpp>
#include <reactor/command_pool.hpp>
#include <reactor/fence.hpp>
#include <reactor/semaphore.hpp>
#include <reactor/camera.hpp>
#include <reactor/transform.hpp>
#include <reactor/isr/importance.hpp>
#include "../stack-gpu-cube/cube_renderer.hpp"

#include <iostream>
#include <chrono>
#include <array>

int main() {
    try {
        std::cout << "========================================" << std::endl;
        std::cout << "  Stack-GPU-OP - ISR Debug Visualizer" << std::endl;
        std::cout << "========================================" << std::endl;
        std::cout << std::endl;
        
        // Window config
        reactor::WindowConfig config;
        config.title = "ISR Debug - Importance Map Visualization";
        config.width = 1280;
        config.height = 720;
        
        std::cout << "[1/6] Inicializando window..." << std::endl;
        reactor::Window::initialize();
        reactor::Window window(config);
        
        std::cout << "[2/6] Creando Vulkan context..." << std::endl;
        reactor::VulkanContext ctx;
        ctx.initialize();
        
        VkSurfaceKHR surface = window.createSurface(ctx.instance());
        ctx.selectPhysicalDevice(surface);
        ctx.createLogicalDevice();
        
        std::cout << "[3/6] Creando swapchain..." << std::endl;
        reactor::Swapchain swapchain(ctx.device(), ctx.physicalDevice(), surface, config.width, config.height);
        
        // Crear depth buffer
        VkFormat depthFormat = VK_FORMAT_D32_SFLOAT;
        VkImageCreateInfo depthImageInfo{};
        depthImageInfo.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
        depthImageInfo.imageType = VK_IMAGE_TYPE_2D;
        depthImageInfo.format = depthFormat;
        depthImageInfo.extent = {static_cast<uint32_t>(config.width), static_cast<uint32_t>(config.height), 1};
        depthImageInfo.mipLevels = 1;
        depthImageInfo.arrayLayers = 1;
        depthImageInfo.samples = VK_SAMPLE_COUNT_1_BIT;
        depthImageInfo.tiling = VK_IMAGE_TILING_OPTIMAL;
        depthImageInfo.usage = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;
        
        VkImage depthImage;
        vkCreateImage(ctx.device(), &depthImageInfo, nullptr, &depthImage);
        
        VkMemoryRequirements memReqs;
        vkGetImageMemoryRequirements(ctx.device(), depthImage, &memReqs);
        
        auto depthBlock = ctx.allocator()->allocate(memReqs, reactor::MemoryType::DeviceLocal);
        vkBindImageMemory(ctx.device(), depthImage, depthBlock.memory, depthBlock.offset);
        
        VkImageViewCreateInfo depthViewInfo{};
        depthViewInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
        depthViewInfo.image = depthImage;
        depthViewInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
        depthViewInfo.format = depthFormat;
        depthViewInfo.subresourceRange.aspectMask = VK_IMAGE_ASPECT_DEPTH_BIT;
        depthViewInfo.subresourceRange.baseMipLevel = 0;
        depthViewInfo.subresourceRange.levelCount = 1;
        depthViewInfo.subresourceRange.baseArrayLayer = 0;
        depthViewInfo.subresourceRange.layerCount = 1;
        
        VkImageView depthView;
        vkCreateImageView(ctx.device(), &depthViewInfo, nullptr, &depthView);
        
        std::cout << "[✓] Depth buffer creado" << std::endl;
        
        // Crear render pass con depth
        std::vector<reactor::AttachmentDescription> attachments = {
            {
                .format = swapchain.imageFormat(),
                .loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR,
                .storeOp = VK_ATTACHMENT_STORE_OP_STORE,
                .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
                .finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR
            },
            {
                .format = depthFormat,
                .loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR,
                .storeOp = VK_ATTACHMENT_STORE_OP_DONT_CARE,
                .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
                .finalLayout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            }
        };
        
        reactor::RenderPass renderPass(ctx.device(), attachments, true);
        std::cout << "[✓] Render pass creado (con depth)" << std::endl;
        
        std::cout << "[4/6] Creando cube renderer..." << std::endl;
        cube::CubeRenderer cubeRenderer(ctx, renderPass.handle(), config.width, config.height);
        
        std::cout << "[5/6] Creando ISR ImportanceCalculator..." << std::endl;
        reactor::ImportanceCalculator importanceCalc(ctx, config.width, config.height);
        std::cout << "[✓] ISR ImportanceCalculator listo" << std::endl;
        
        // Crear framebuffers con depth
        std::vector<VkFramebuffer> framebuffers;
        for (size_t i = 0; i < swapchain.imageCount(); i++) {
            std::array<VkImageView, 2> attachmentViews = {
                swapchain.imageViews()[i],
                depthView
            };
            
            VkFramebufferCreateInfo fbInfo{};
            fbInfo.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
            fbInfo.renderPass = renderPass.handle();
            fbInfo.attachmentCount = static_cast<uint32_t>(attachmentViews.size());
            fbInfo.pAttachments = attachmentViews.data();
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
        
        // Camera
        reactor::Camera camera;
        camera.position = glm::vec3(3.0f, 3.0f, 3.0f);
        camera.target = glm::vec3(0.0f, 0.0f, 0.0f);
        camera.aspectRatio = static_cast<float>(config.width) / config.height;
        
        // Transform para rotación
        reactor::Transform cubeTransform;
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  [✓] ISR Debug Visualizer listo!" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "Renderizando cubo con ISR Importance Map..." << std::endl;
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
            
            // Animar cubo
            cubeTransform.rotation.y = time * glm::radians(45.0f);
            cubeTransform.rotation.x = time * glm::radians(30.0f);
            
            // Wait for fence
            inFlight[currentFrame].wait();
            
            // Acquire image
            uint32_t imageIndex = swapchain.acquireNextImage(imageAvailable[currentFrame].handle());
            
            if (imagesInFlight[imageIndex] != VK_NULL_HANDLE) {
                vkWaitForFences(ctx.device(), 1, &imagesInFlight[imageIndex], VK_TRUE, UINT64_MAX);
            }
            imagesInFlight[imageIndex] = inFlight[currentFrame].handle();
            
            inFlight[currentFrame].reset();
            
            // Calcular matrices
            glm::mat4 view = camera.getViewMatrix();
            glm::mat4 proj = camera.getProjectionMatrix();
            glm::mat4 model = cubeTransform.getMatrix();
            glm::mat4 mvp = proj * view * model;
            
            // Record commands
            auto& cmd = cmdBuffers[imageIndex];
            cmd.reset();
            cmd.begin();
            
            // TODO: Aquí calcularemos el importance map en el futuro
            // Por ahora solo renderizamos el cubo normal
            
            std::array<VkClearValue, 2> clearValues{};
            clearValues[0].color = {{0.1f, 0.1f, 0.15f, 1.0f}};
            clearValues[1].depthStencil = {1.0f, 0};
            
            VkRenderPassBeginInfo rpInfo{};
            rpInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
            rpInfo.renderPass = renderPass.handle();
            rpInfo.framebuffer = framebuffers[imageIndex];
            rpInfo.renderArea.offset = {0, 0};
            rpInfo.renderArea.extent = swapchain.extent();
            rpInfo.clearValueCount = static_cast<uint32_t>(clearValues.size());
            rpInfo.pClearValues = clearValues.data();
            
            vkCmdBeginRenderPass(cmd.handle(), &rpInfo, VK_SUBPASS_CONTENTS_INLINE);
            
            // Renderizar cubo
            cubeRenderer.render(cmd, mvp, model);
            
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
            
            // FPS counter
            frameCount++;
            auto fpsDuration = std::chrono::duration<float>(currentTime - lastFpsTime).count();
            if (fpsDuration >= 0.5f) {
                float fps = frameCount / fpsDuration;
                std::string title = "ISR Debug | FPS: " + std::to_string(static_cast<int>(fps)) + 
                                   " | Importance Map: READY";
                window.setTitle(title);
                frameCount = 0;
                lastFpsTime = currentTime;
            }
            
            currentFrame = (currentFrame + 1) % MAX_FRAMES;
        }
        
        // Cleanup
        vkDeviceWaitIdle(ctx.device());
        
        for (auto fb : framebuffers) {
            vkDestroyFramebuffer(ctx.device(), fb, nullptr);
        }
        
        vkDestroyImageView(ctx.device(), depthView, nullptr);
        vkDestroyImage(ctx.device(), depthImage, nullptr);
        ctx.allocator()->free(depthBlock);
        
        vkDestroySurfaceKHR(ctx.instance(), surface, nullptr);
        ctx.shutdown();
        reactor::Window::terminate();
        
        std::cout << std::endl << "[✓] ISR Debug finalizado" << std::endl;
        
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "❌ Error: " << e.what() << std::endl;
        reactor::Window::terminate();
        return 1;
    }
}
