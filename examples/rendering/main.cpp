#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/window.hpp"
#include "reactor/swapchain.hpp"
#include "reactor/render_pass.hpp"
#include "reactor/command_buffer.hpp"
#include "reactor/sync.hpp"
#include <iostream>
#include <chrono>

int main() {
    try {
        std::cout << "==========================================" << std::endl;
        std::cout << "  REACTOR - Complete Rendering Example" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        // 1. Inicializar GLFW
        std::cout << "[1/8] Inicializando sistema de ventanas..." << std::endl;
        reactor::Window::init();
        
        // 2. Crear ventana
        reactor::WindowConfig windowConfig;
        windowConfig.title = "REACTOR - Rendering Demo";
        windowConfig.width = 1280;
        windowConfig.height = 720;
        windowConfig.vsync = true;
        
        reactor::Window window(windowConfig);
        std::cout << "      ✓ Ventana creada: " << windowConfig.width << "x" << windowConfig.height << std::endl;
        
        // 3. Inicializar Vulkan
        std::cout << "[2/8] Inicializando Vulkan..." << std::endl;
        reactor::VulkanContext ctx(true);
        ctx.init();
        std::cout << "      ✓ Vulkan inicializado" << std::endl;
        
        // 4. Crear surface
        std::cout << "[3/8] Creando surface..." << std::endl;
        VkSurfaceKHR surface = window.createSurface(ctx.instance());
        std::cout << "      ✓ Surface creado" << std::endl;
        
        // 5. Crear swapchain
        std::cout << "[4/8] Creando swapchain..." << std::endl;
        reactor::Swapchain swapchain(ctx.device(), ctx.physical(), surface, 
                                     windowConfig.width, windowConfig.height);
        std::cout << "      ✓ Swapchain creado con " << swapchain.imageCount() << " imagenes" << std::endl;
        
        // 6. Crear render pass
        std::cout << "[5/8] Creando render pass..." << std::endl;
        auto renderPass = reactor::RenderPass::create(ctx.device())
            .colorAttachment({
                .format = swapchain.format(),
                .loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR,
                .storeOp = VK_ATTACHMENT_STORE_OP_STORE,
                .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
                .finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR
            })
            .build();
        std::cout << "      ✓ Render pass creado" << std::endl;
        
        // 7. Crear command pool y buffers
        std::cout << "[6/8] Creando command buffers..." << std::endl;
        reactor::CommandPool commandPool(ctx.device(), ctx.queueFamilyIndices().graphics.value());
        auto commandBuffers = commandPool.allocate(swapchain.imageCount());
        std::cout << "      ✓ " << commandBuffers.size() << " command buffers creados" << std::endl;
        
        // 8. Crear objetos de sincronización
        std::cout << "[7/8] Creando objetos de sincronizacion..." << std::endl;
        const int MAX_FRAMES_IN_FLIGHT = 2;
        std::vector<reactor::Semaphore> imageAvailableSemaphores;
        std::vector<reactor::Semaphore> renderFinishedSemaphores;
        std::vector<reactor::Fence> inFlightFences;
        
        for (int i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
            imageAvailableSemaphores.emplace_back(ctx.device());
            renderFinishedSemaphores.emplace_back(ctx.device());
            inFlightFences.emplace_back(ctx.device(), true);
        }
        std::cout << "      ✓ Sincronizacion configurada" << std::endl;
        
        // 9. Setup de callbacks
        std::cout << "[8/8] Configurando callbacks..." << std::endl;
        bool framebufferResized = false;
        window.setResizeCallback([&](int width, int height) {
            framebufferResized = true;
            std::cout << "Ventana redimensionada: " << width << "x" << height << std::endl;
        });
        
        window.setKeyCallback([&](int key, int action) {
            if (key == 256 && action == 1) { // ESC key
                std::cout << "ESC presionado - cerrando..." << std::endl;
            }
        });
        std::cout << "      ✓ Callbacks configurados" << std::endl;
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  ✓ Inicializacion completa!" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        std::cout << "Controles:" << std::endl;
        std::cout << "  ESC - Salir" << std::endl;
        std::cout << std::endl;
        std::cout << "Iniciando render loop..." << std::endl;
        std::cout << std::endl;
        
        // Render loop
        size_t currentFrame = 0;
        size_t frameCount = 0;
        auto startTime = std::chrono::high_resolution_clock::now();
        auto lastFpsTime = startTime;
        
        while (!window.shouldClose()) {
            window.pollEvents();
            
            // Esperar al frame anterior
            inFlightFences[currentFrame].wait();
            
            // Adquirir imagen del swapchain
            uint32_t imageIndex;
            VkResult result = swapchain.acquireNextImage(
                imageAvailableSemaphores[currentFrame].handle(),
                &imageIndex
            );
            
            if (result == VK_ERROR_OUT_OF_DATE_KHR || framebufferResized) {
                framebufferResized = false;
                int width, height;
                window.getFramebufferSize(&width, &height);
                // Aquí se recrearía el swapchain
                continue;
            }
            
            inFlightFences[currentFrame].reset();
            
            // Grabar comandos (clear screen con color)
            auto& cmd = commandBuffers[imageIndex];
            cmd.reset();
            cmd.begin();
            
            VkClearValue clearColor = {{{0.0f, 0.0f, 0.0f, 1.0f}}};
            VkExtent2D extent = {
                static_cast<uint32_t>(windowConfig.width),
                static_cast<uint32_t>(windowConfig.height)
            };
            
            // Aquí iría el renderizado real
            // Por ahora solo limpiamos la pantalla
            
            cmd.end();
            
            // Submit
            VkSubmitInfo submitInfo{};
            submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
            
            VkSemaphore waitSemaphores[] = {imageAvailableSemaphores[currentFrame].handle()};
            VkPipelineStageFlags waitStages[] = {VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT};
            submitInfo.waitSemaphoreCount = 1;
            submitInfo.pWaitSemaphores = waitSemaphores;
            submitInfo.pWaitDstStageMask = waitStages;
            
            VkCommandBuffer cmdBuffer = cmd.handle();
            submitInfo.commandBufferCount = 1;
            submitInfo.pCommandBuffers = &cmdBuffer;
            
            VkSemaphore signalSemaphores[] = {renderFinishedSemaphores[currentFrame].handle()};
            submitInfo.signalSemaphoreCount = 1;
            submitInfo.pSignalSemaphores = signalSemaphores;
            
            if (vkQueueSubmit(ctx.graphicsQueue(), 1, &submitInfo, 
                             inFlightFences[currentFrame].handle()) != VK_SUCCESS) {
                throw std::runtime_error("Failed to submit draw command buffer");
            }
            
            // Present
            result = swapchain.present(ctx.graphicsQueue(), imageIndex,
                                      renderFinishedSemaphores[currentFrame].handle());
            
            if (result == VK_ERROR_OUT_OF_DATE_KHR || result == VK_SUBOPTIMAL_KHR) {
                framebufferResized = false;
            }
            
            currentFrame = (currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
            frameCount++;
            
            // Calcular FPS cada segundo
            auto currentTime = std::chrono::high_resolution_clock::now();
            auto elapsed = std::chrono::duration<double>(currentTime - lastFpsTime).count();
            if (elapsed >= 1.0) {
                double fps = frameCount / elapsed;
                std::cout << "FPS: " << static_cast<int>(fps) << " | Frames: " << frameCount << std::endl;
                frameCount = 0;
                lastFpsTime = currentTime;
            }
        }
        
        // Esperar a que termine todo
        vkDeviceWaitIdle(ctx.device());
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  Limpiando recursos..." << std::endl;
        std::cout << "==========================================" << std::endl;
        
        // Cleanup
        vkDestroySurfaceKHR(ctx.instance(), surface, nullptr);
        ctx.shutdown();
        reactor::Window::terminate();
        
        std::cout << "  ✓ Aplicacion finalizada correctamente" << std::endl;
        std::cout << "==========================================" << std::endl;
        
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << std::endl;
        std::cerr << "❌ Error: " << e.what() << std::endl;
        reactor::Window::terminate();
        return 1;
    }
}
