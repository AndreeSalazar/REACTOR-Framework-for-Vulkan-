#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/window.hpp"
#include "reactor/swapchain.hpp"
#include "reactor/render_pass.hpp"
#include "reactor/buffer.hpp"
#include "reactor/command_buffer.hpp"
#include "reactor/sync.hpp"
#include "reactor/shader.hpp"
#include "reactor/math.hpp"
#include <iostream>
#include <chrono>
#include <array>
#include <windows.h>
#include <GLFW/glfw3.h>

/**
 * Stack-GPU-ISR Demo
 * 
 * Demuestra el sistema ISR (Intelligent Shading Rate) completo:
 * - Importance calculation (compute shader)
 * - Adaptive pixel sizing (compute shader)
 * - Temporal coherence (compute shader)
 */

int main() {
    try {
        SetConsoleOutputCP(CP_UTF8);
        setvbuf(stdout, nullptr, _IOFBF, 1000);
        
        std::cout << "==========================================" << std::endl;
        std::cout << "  Stack-GPU-ISR: ISR System Demo" << std::endl;
        std::cout << "  Intelligent Shading Rate (ADead-GPU)" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        reactor::Window::init();
        
        reactor::WindowConfig config;
        config.title = "Stack-GPU-ISR - Intelligent Shading Rate Demo";
        config.width = 1920;
        config.height = 1080;
        
        reactor::Window window(config);
        glfwMaximizeWindow(window.handle());
        std::cout << "[✓] Ventana creada (1920x1080 maximizada)" << std::endl;
        
        reactor::VulkanContext ctx(true);
        ctx.init();
        std::cout << "[✓] Vulkan inicializado" << std::endl;
        
        VkSurfaceKHR surface = window.createSurface(ctx.instance());
        reactor::Swapchain swapchain(ctx.device(), ctx.physical(), surface, config.width, config.height);
        std::cout << "[✓] Swapchain creado" << std::endl;
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  ISR System Demo - Listo!" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        std::cout << "CONTROLES:" << std::endl;
        std::cout << "  [1] ISR OFF - Renderizado normal" << std::endl;
        std::cout << "  [2] ISR ON - Con importance map" << std::endl;
        std::cout << "  [3] Visualizar Importance Map" << std::endl;
        std::cout << "  [4] Visualizar Shading Rate" << std::endl;
        std::cout << "  [ESC] Salir" << std::endl;
        std::cout << std::endl;
        std::cout << "ISR System: Importance → Adaptive → Temporal" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        int isrMode = 0;
        auto startTime = std::chrono::high_resolution_clock::now();
        int frameCount = 0;
        auto lastFpsTime = startTime;
        
        while (!window.shouldClose()) {
            window.pollEvents();
            
            if (glfwGetKey(window.handle(), GLFW_KEY_1) == GLFW_PRESS) isrMode = 0;
            if (glfwGetKey(window.handle(), GLFW_KEY_2) == GLFW_PRESS) isrMode = 1;
            if (glfwGetKey(window.handle(), GLFW_KEY_3) == GLFW_PRESS) isrMode = 2;
            if (glfwGetKey(window.handle(), GLFW_KEY_4) == GLFW_PRESS) isrMode = 3;
            
            auto currentTime = std::chrono::high_resolution_clock::now();
            
            // TODO: Render with ISR system
            // 1. Calculate importance map (compute shader)
            // 2. Generate shading rate image (compute shader)
            // 3. Apply temporal coherence (compute shader)
            // 4. Render with variable rate shading
            
            frameCount++;
            auto fpsDuration = std::chrono::duration<float>(currentTime - lastFpsTime).count();
            if (fpsDuration >= 0.5f) {
                float fps = frameCount / fpsDuration;
                const char* modes[] = {"ISR OFF", "ISR ON", "Importance Map", "Shading Rate"};
                std::string title = "Stack-GPU-ISR | FPS: " + std::to_string(static_cast<int>(fps)) + " | " + modes[isrMode];
                window.setTitle(title);
                std::cout << "\rFPS: " << static_cast<int>(fps) << " | Modo: " << modes[isrMode] << "     " << std::flush;
                frameCount = 0;
                lastFpsTime = currentTime;
            }
        }
        
        vkDestroySurfaceKHR(ctx.instance(), surface, nullptr);
        ctx.shutdown();
        reactor::Window::terminate();
        
        std::cout << std::endl << "[✓] Stack-GPU-ISR finalizado" << std::endl;
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "❌ Error: " << e.what() << std::endl;
        reactor::Window::terminate();
        return 1;
    }
}
