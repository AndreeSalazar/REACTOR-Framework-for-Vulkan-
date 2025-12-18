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
#include <array>
#include <iomanip>
#include <thread>
#include <fstream>
#include <sstream>
#include <cstdio>
#include <windows.h>
#include <GLFW/glfw3.h>

// #region agent log - Debug logging helper
inline void debugLog(const char* location, const char* message, const char* hypothesisId, const char* data = nullptr) {
    try {
        // Crear directorio si no existe (ignorar errores si ya existe)
        CreateDirectoryA("c:\\Users\\andre\\OneDrive\\Documentos\\REACTOR (Framework for Vulkan)\\.cursor", nullptr);
        
        const char* logPath = "c:\\Users\\andre\\OneDrive\\Documentos\\REACTOR (Framework for Vulkan)\\.cursor\\debug.log";
        std::ofstream logFile(logPath, std::ios::app | std::ios::binary);
        if (logFile.is_open()) {
            auto now = std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::system_clock::now().time_since_epoch()).count();
            logFile << "{\"timestamp\":" << now << ",\"location\":\"" << (location ? location : "") << "\",\"message\":\"" << (message ? message : "") << "\",\"hypothesisId\":\"" << (hypothesisId ? hypothesisId : "") << "\",\"sessionId\":\"debug-session\",\"runId\":\"run1\"";
            if (data && strlen(data) > 0) {
                logFile << ",\"data\":\"" << data << "\"";
            }
            logFile << "}\n";
            logFile.flush();
            logFile.close();
        } else {
            // Si falla, intentar escribir a un archivo alternativo
            std::ofstream altFile("debug_alt.log", std::ios::app);
            if (altFile.is_open()) {
                altFile << "LOG_ERROR: Could not open main log file\n";
                altFile.close();
            }
        }
    } catch (...) {
        // Intentar escribir a archivo alternativo si hay excepción
        try {
            std::ofstream altFile("debug_exception.log", std::ios::app);
            if (altFile.is_open()) {
                altFile << "LOG_EXCEPTION: Exception in debugLog\n";
                altFile.close();
            }
        } catch (...) {}
    }
}
// #endregion

/**
 * Stack-GPU-OP Cube Demo
 * 
 * Renderiza un cubo 3D usando:
 * - SDF Ray Marching (ADead-Vector3D adaptado a Vulkan)
 * - React-Style API de REACTOR
 * - Vulkan puro (NO DirectX 12)
 */

int main() {
    // Test logging function immediately - MUY TEMPRANO
    debugLog("main.cpp:51", "=== APPLICATION START ===", "ALL");
    
    try {
        debugLog("main.cpp:54", "Entering try block", "ALL");
        
        // Configurar consola para UTF-8 (soporte español con acentos)
        debugLog("main.cpp:57", "Before SetConsoleOutputCP", "ALL");
        SetConsoleOutputCP(CP_UTF8);
        debugLog("main.cpp:58", "After SetConsoleOutputCP", "ALL");
        setvbuf(stdout, nullptr, _IOFBF, 1000);
        debugLog("main.cpp:60", "Console configured", "ALL");
        
        std::cout << "==========================================" << std::endl;
        std::cout << "  Stack-GPU-OP: Debug Visualizer" << std::endl;
        std::cout << "  Vulkan + ADead-GPU ISR" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        // Inicializar GLFW
        debugLog("main.cpp:72", "Before Window::init()", "ALL");
        reactor::Window::init();
        debugLog("main.cpp:73", "After Window::init()", "ALL");
        
        // Crear ventana - Resolución adaptativa (se ajustará al tamaño real del framebuffer)
        debugLog("main.cpp:76", "Before creating window config", "ALL");
        reactor::WindowConfig config;
        config.title = "Stack-GPU-OP - Cubo 3D (Vulkan + ISR Debug Visualizer)";
        // Tamaño inicial, pero se adaptará automáticamente desde 800x600 hasta 8K (7680x4320)
        config.width = 1280;
        config.height = 720;
        
        reactor::Window window(config);
        std::cout << "[✓] Ventana creada" << std::endl;
        
        // Inicializar Vulkan
        debugLog("main.cpp:86", "Before creating VulkanContext", "ALL");
        reactor::VulkanContext ctx(true);
        debugLog("main.cpp:88", "Before ctx.init()", "ALL");
        ctx.init();
        debugLog("main.cpp:89", "After ctx.init()", "ALL");
        std::cout << "[✓] Vulkan inicializado" << std::endl;
        
        // Crear surface
        debugLog("main.cpp:92", "Before createSurface", "ALL");
        VkSurfaceKHR surface = window.createSurface(ctx.instance());
        debugLog("main.cpp:93", "After createSurface", "ALL");
        
        // Obtener tamaño real del framebuffer
        int actualWidth, actualHeight;
        window.getFramebufferSize(&actualWidth, &actualHeight);
        
        // Crear swapchain con el tamaño real del framebuffer
        debugLog("main.cpp:96", "Before creating swapchain", "ALL");
        reactor::Swapchain swapchain(ctx.device(), ctx.physical(), surface, actualWidth, actualHeight);
        debugLog("main.cpp:97", "After creating swapchain", "ALL");
        std::cout << "[✓] Swapchain creado (" << actualWidth << "x" << actualHeight << ")" << std::endl;
        
        // Variables para depth buffer (necesitan estar fuera del lambda)
        VkFormat depthFormat = VK_FORMAT_D32_SFLOAT;
        VkImageCreateInfo depthImageInfo{};
        depthImageInfo.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
        depthImageInfo.imageType = VK_IMAGE_TYPE_2D;
        depthImageInfo.format = depthFormat;
        // Usar el tamaño real del framebuffer, no el config
        depthImageInfo.extent = {static_cast<uint32_t>(actualWidth), static_cast<uint32_t>(actualHeight), 1};
        depthImageInfo.mipLevels = 1;
        depthImageInfo.arrayLayers = 1;
        depthImageInfo.samples = VK_SAMPLE_COUNT_1_BIT;
        depthImageInfo.tiling = VK_IMAGE_TILING_OPTIMAL;
        depthImageInfo.usage = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;
        
        VkImageViewCreateInfo depthViewInfo{};
        depthViewInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
        depthViewInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
        depthViewInfo.format = depthFormat;
        depthViewInfo.subresourceRange.aspectMask = VK_IMAGE_ASPECT_DEPTH_BIT;
        depthViewInfo.subresourceRange.baseMipLevel = 0;
        depthViewInfo.subresourceRange.levelCount = 1;
        depthViewInfo.subresourceRange.baseArrayLayer = 0;
        depthViewInfo.subresourceRange.layerCount = 1;
        
        // Crear depth image inicial
        VkImage depthImage;
        vkCreateImage(ctx.device(), &depthImageInfo, nullptr, &depthImage);
        
        VkMemoryRequirements memReqs;
        vkGetImageMemoryRequirements(ctx.device(), depthImage, &memReqs);
        
        // Usar el allocator de REACTOR
        reactor::MemoryBlock depthBlock = ctx.allocator()->allocate(memReqs, reactor::MemoryType::DeviceLocal);
        vkBindImageMemory(ctx.device(), depthImage, depthBlock.memory, depthBlock.offset);
        
        depthViewInfo.image = depthImage;
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
        
        // Crear cube renderer con el tamaño real del framebuffer
        std::cout << "[3/5] Creando cube renderer..." << std::endl;
        cube::CubeRenderer cubeRenderer(ctx, renderPass.handle(), actualWidth, actualHeight);
        std::cout << "[✓] Cube renderer creado" << std::endl;
        
        // Obtener dimensiones reales del swapchain (pueden diferir de las solicitadas)
        VkExtent2D swapchainExtent = swapchain.extent();
        
        // Asegurar que el depth image tenga las dimensiones correctas del swapchain
        if (depthImageInfo.extent.width != swapchainExtent.width || 
            depthImageInfo.extent.height != swapchainExtent.height) {
            // Destruir depth image viejo
            vkDestroyImageView(ctx.device(), depthView, nullptr);
            vkDestroyImage(ctx.device(), depthImage, nullptr);
            ctx.allocator()->free(depthBlock);
            
            // Recrear con dimensiones correctas
            depthImageInfo.extent = {swapchainExtent.width, swapchainExtent.height, 1};
            vkCreateImage(ctx.device(), &depthImageInfo, nullptr, &depthImage);
            vkGetImageMemoryRequirements(ctx.device(), depthImage, &memReqs);
            depthBlock = ctx.allocator()->allocate(memReqs, reactor::MemoryType::DeviceLocal);
            vkBindImageMemory(ctx.device(), depthImage, depthBlock.memory, depthBlock.offset);
            depthViewInfo.image = depthImage;
            vkCreateImageView(ctx.device(), &depthViewInfo, nullptr, &depthView);
        }
        
        // Crear framebuffers con dimensiones reales del swapchain
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
            fbInfo.width = swapchainExtent.width;
            fbInfo.height = swapchainExtent.height;
            fbInfo.layers = 1;
            
            VkFramebuffer fb;
            if (vkCreateFramebuffer(ctx.device(), &fbInfo, nullptr, &fb) != VK_SUCCESS) {
                throw std::runtime_error("Failed to create framebuffer");
            }
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
        
        // Vector para tracking de imágenes en vuelo
        std::vector<VkFence> imagesInFlight(swapchain.imageCount(), VK_NULL_HANDLE);
        
        // Protección contra recreaciones concurrentes (declarar antes del lambda)
        bool recreatingSwapchain = false;
        
        // Camera (React-style)
        reactor::Camera camera;
        camera.position = glm::vec3(3.0f, 3.0f, 3.0f);
        camera.target = glm::vec3(0.0f, 0.0f, 0.0f);
        camera.aspectRatio = static_cast<float>(swapchainExtent.width) / swapchainExtent.height;
        
        // Transform para rotación
        reactor::Transform cubeTransform;
        
        // Función helper para recrear swapchain y framebuffers
        auto recreateSwapchain = [&](uint32_t width, uint32_t height) {
            // Protección contra recreaciones concurrentes
            if (recreatingSwapchain) {
                debugLog("main.cpp:217", "Recreation already in progress, skipping", "C");
                return;
            }
            
            // Validar dimensiones antes de proceder
            if (width == 0 || height == 0) {
                debugLog("main.cpp:221", "Invalid dimensions, skipping recreate", "C");
                return;
            }
            
            // Validar dimensiones - Permitir desde 800x600 hasta 8K (7680x4320)
            const uint32_t MIN_WIDTH = 800;
            const uint32_t MIN_HEIGHT = 600;
            const uint32_t MAX_WIDTH = 7680;   // 8K width
            const uint32_t MAX_HEIGHT = 4320;  // 8K height
            
            if (width < MIN_WIDTH || height < MIN_HEIGHT) {
                char errorMsg[256];
                sprintf_s(errorMsg, sizeof(errorMsg), "Dimensions too small: %ux%u (min: %ux%u)", width, height, MIN_WIDTH, MIN_HEIGHT);
                debugLog("main.cpp:226", errorMsg, "C");
                return;
            }
            
            if (width > MAX_WIDTH || height > MAX_HEIGHT) {
                char errorMsg[256];
                sprintf_s(errorMsg, sizeof(errorMsg), "Dimensions too large: %ux%u (max: %ux%u)", width, height, MAX_WIDTH, MAX_HEIGHT);
                debugLog("main.cpp:226", errorMsg, "C");
                return;
            }
            
            recreatingSwapchain = true;
            
            // CRÍTICO: Usar try-catch para asegurar que recreatingSwapchain se resetee siempre
            try {
                char msg[256];
                sprintf_s(msg, sizeof(msg), "recreateSwapchain ENTRY width=%u height=%u", width, height);
                debugLog("main.cpp:232", msg, "C");
                
                // CRÍTICO: Limpiar el tracking de imágenes en vuelo ANTES de destruir recursos
                // No necesitamos esperar nada - los recursos viejos se destruirán cuando el swapchain viejo se destruya
                std::fill(imagesInFlight.begin(), imagesInFlight.end(), VK_NULL_HANDLE);
                debugLog("main.cpp:234", "Cleared imagesInFlight tracking", "D");
            
            // Destruir framebuffers viejos (que referencian image views del swapchain viejo)
            char msg2[256];
            sprintf_s(msg2, sizeof(msg2), "Destroying %zu old framebuffers", framebuffers.size());
            debugLog("main.cpp:244", msg2, "B");
            for (auto fb : framebuffers) {
                if (fb != VK_NULL_HANDLE) {
                    vkDestroyFramebuffer(ctx.device(), fb, nullptr);
                }
            }
            framebuffers.clear();
            debugLog("main.cpp:251", "Framebuffers destroyed and cleared", "B");
            
            // Destruir depth image viejo
            if (depthView != VK_NULL_HANDLE) {
                vkDestroyImageView(ctx.device(), depthView, nullptr);
                depthView = VK_NULL_HANDLE;
            }
            if (depthImage != VK_NULL_HANDLE) {
                vkDestroyImage(ctx.device(), depthImage, nullptr);
                depthImage = VK_NULL_HANDLE;
            }
            if (depthBlock.memory != VK_NULL_HANDLE) {
                ctx.allocator()->free(depthBlock);
                depthBlock = reactor::MemoryBlock{};
            }
            
            // CRÍTICO: Guardar handle del swapchain viejo para pasarlo como oldSwapchain
            // Esto permite que el driver optimice la recreación
            VkSwapchainKHR oldSwapchainHandle = swapchain.handle();
            debugLog("main.cpp:268", "Creating new swapchain with oldSwapchain", "B");
            reactor::Swapchain newSwapchain(ctx.device(), ctx.physical(), surface, width, height, true, oldSwapchainHandle);
            
            // CRÍTICO: Asegurar que el nuevo swapchain se creó correctamente antes de destruir el viejo
            // El move assignment destruirá el swapchain viejo y sus image views automáticamente
            debugLog("main.cpp:272", "Performing move assignment (old swapchain will be destroyed)", "B");
            
            // IMPORTANTE: El move assignment destruye el swapchain viejo, pero ya tenemos
            // el nuevo creado, así que esto es seguro
            swapchain = std::move(newSwapchain);
            debugLog("main.cpp:276", "Move assignment completed", "B");
            
            // NOTA: No necesitamos otro vkDeviceWaitIdle aquí, el swapchain ya está creado
            
            // Obtener dimensiones reales del swapchain (pueden diferir de las solicitadas)
            VkExtent2D swapchainExtent = swapchain.extent();
            
            // Recrear depth image con dimensiones correctas del swapchain
            depthImageInfo.extent = {swapchainExtent.width, swapchainExtent.height, 1};
            if (vkCreateImage(ctx.device(), &depthImageInfo, nullptr, &depthImage) != VK_SUCCESS) {
                throw std::runtime_error("Failed to recreate depth image");
            }
            vkGetImageMemoryRequirements(ctx.device(), depthImage, &memReqs);
            depthBlock = ctx.allocator()->allocate(memReqs, reactor::MemoryType::DeviceLocal);
            if (vkBindImageMemory(ctx.device(), depthImage, depthBlock.memory, depthBlock.offset) != VK_SUCCESS) {
                throw std::runtime_error("Failed to bind depth image memory");
            }
            depthViewInfo.image = depthImage;
            if (vkCreateImageView(ctx.device(), &depthViewInfo, nullptr, &depthView) != VK_SUCCESS) {
                throw std::runtime_error("Failed to recreate depth image view");
            }
            
            // Recrear framebuffers con dimensiones reales del swapchain
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
                fbInfo.width = swapchainExtent.width;
                fbInfo.height = swapchainExtent.height;
                fbInfo.layers = 1;
                
                VkFramebuffer fb;
                if (vkCreateFramebuffer(ctx.device(), &fbInfo, nullptr, &fb) != VK_SUCCESS) {
                    throw std::runtime_error("Failed to recreate framebuffer");
                }
                framebuffers.push_back(fb);
            }
            
            // Actualizar camera aspect ratio con dimensiones reales
            camera.aspectRatio = static_cast<float>(swapchainExtent.width) / swapchainExtent.height;
            
            // CRÍTICO: Solo recrear command buffers si el número de imágenes cambió
            // Si el número es el mismo, los command buffers viejos son válidos y solo necesitamos
            // asegurarnos de que los framebuffers se hayan recreado correctamente
            if (cmdBuffers.size() != swapchain.imageCount()) {
                // El número de imágenes cambió, necesitamos recrear los command buffers
                debugLog("main.cpp:395", "Command buffer count changed, recreating", "D");
                cmdBuffers.clear();
                imagesInFlight.resize(swapchain.imageCount(), VK_NULL_HANDLE);
                for (size_t i = 0; i < swapchain.imageCount(); i++) {
                    cmdBuffers.emplace_back(cmdPoolPtr);
                }
            } else {
                // El número es el mismo, solo necesitamos asegurarnos de que el tracking esté actualizado
                imagesInFlight.resize(swapchain.imageCount(), VK_NULL_HANDLE);
                // Resetear el command pool para poder reusar los command buffers con los nuevos framebuffers
                cmdPoolPtr->reset();
                debugLog("main.cpp:403", "Command buffer count unchanged, resetting pool", "D");
            }
            
            debugLog("main.cpp:405", "recreateSwapchain EXIT", "C");
            } catch (const std::exception& e) {
                // CRÍTICO: En caso de error, loguear y asegurar que el flag se resetee
                char errorMsg[512];
                sprintf_s(errorMsg, sizeof(errorMsg), "ERROR in recreateSwapchain: %s", e.what());
                debugLog("main.cpp:ERROR", errorMsg, "A");
                std::cerr << "ERROR recreating swapchain: " << e.what() << std::endl;
                // Resetear el flag para permitir intentos futuros
            } catch (...) {
                debugLog("main.cpp:ERROR", "Unknown error in recreateSwapchain", "A");
                std::cerr << "Unknown error recreating swapchain" << std::endl;
            }
            
            // CRÍTICO: Asegurar que el flag siempre se resetee, incluso si hubo error
            recreatingSwapchain = false;
        };
        
        // Modo de visualización
        int debugMode = 0;
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  Stack-GPU-OP Debug Visualizer Listo!" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        std::cout << "CONTROLES:" << std::endl;
        std::cout << "  [1] Normal - Phong Shading" << std::endl;
        std::cout << "  [2] Wireframe" << std::endl;
        std::cout << "  [3] Normales RGB" << std::endl;
        std::cout << "  [4] Depth Buffer" << std::endl;
        std::cout << "  [5] ISR: Importance Map" << std::endl;
        std::cout << "  [6] ISR: Pixel Sizing" << std::endl;
        std::cout << "  [7] ISR: Temporal" << std::endl;
        std::cout << "  [ESC] Salir" << std::endl;
        std::cout << std::endl;
        std::cout << "Modo: [1] Normal" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        // Render loop
        size_t currentFrame = 0;
        auto startTime = std::chrono::high_resolution_clock::now();
        int frameCount = 0;
        auto lastFpsTime = startTime;
        
        // Rastrear dimensiones actuales del swapchain para detectar cambios
        uint32_t currentSwapchainWidth = swapchain.extent().width;
        uint32_t currentSwapchainHeight = swapchain.extent().height;
        bool framebufferResized = false;
        debugLog("main.cpp:455", "Entering render loop", "ALL");
        
        while (!window.shouldClose()) {
            window.pollEvents();
            
            // Detectar si la ventana está minimizada o cambió de tamaño
            debugLog("main.cpp:472", "Before getFramebufferSize", "ALL");
            int width, height;
            window.getFramebufferSize(&width, &height);
            debugLog("main.cpp:474", "After getFramebufferSize", "ALL");
            bool windowMinimized = (width == 0 || height == 0);
            
            char sizeMsg[256];
            sprintf_s(sizeMsg, sizeof(sizeMsg), "Framebuffer size: %dx%d, minimized=%d", width, height, windowMinimized ? 1 : 0);
            debugLog("main.cpp:477", sizeMsg, "ALL");
            
            // Verificar si el framebuffer cambió de tamaño - DETECCIÓN INMEDIATA
            // Simplificado para evitar bloqueos: detectar y marcar inmediatamente
            if (!windowMinimized && !recreatingSwapchain && !framebufferResized &&
                (static_cast<uint32_t>(width) != currentSwapchainWidth || 
                 static_cast<uint32_t>(height) != currentSwapchainHeight)) {
                char msg5[256];
                sprintf_s(msg5, sizeof(msg5), "Size change detected: %ux%u -> %dx%d", currentSwapchainWidth, currentSwapchainHeight, width, height);
                debugLog("main.cpp:472", msg5, "C");
                framebufferResized = true;
            }
            
            // Detectar teclas 1-7 (hacerlo siempre, incluso cuando está minimizada)
            static int lastMode = 0;
            
            if (glfwGetKey(window.handle(), GLFW_KEY_1) == GLFW_PRESS) debugMode = 0;
            if (glfwGetKey(window.handle(), GLFW_KEY_2) == GLFW_PRESS) debugMode = 1;
            if (glfwGetKey(window.handle(), GLFW_KEY_3) == GLFW_PRESS) debugMode = 2;
            if (glfwGetKey(window.handle(), GLFW_KEY_4) == GLFW_PRESS) debugMode = 3;
            if (glfwGetKey(window.handle(), GLFW_KEY_5) == GLFW_PRESS) debugMode = 4;
            if (glfwGetKey(window.handle(), GLFW_KEY_6) == GLFW_PRESS) debugMode = 5;
            if (glfwGetKey(window.handle(), GLFW_KEY_7) == GLFW_PRESS) debugMode = 6;
            
            if (debugMode != lastMode) {
                std::cout << "\n========================================" << std::endl;
                std::cout << "MODO: ";
                const char* modes[] = {"[1] Normal", "[2] Wireframe", "[3] Normales", "[4] Depth", "[5] ISR:Importance", "[6] ISR:PixelSize", "[7] ISR:Temporal"};
                std::cout << modes[debugMode] << std::endl;
                std::cout << "========================================" << std::endl;
                lastMode = debugMode;
            }
            
            // Si la ventana está minimizada, saltar el render loop
            if (windowMinimized) {
                // Esperar un poco para no consumir CPU innecesariamente
                std::this_thread::sleep_for(std::chrono::milliseconds(100));
                continue;
            }
            
            // Si estamos recreando, saltar este frame
            if (recreatingSwapchain) {
                continue;
            }
            
            // Si detectamos cambio de tamaño, recrear ANTES de continuar con el render
            if (framebufferResized && !recreatingSwapchain) {
                char msg3[256];
                sprintf_s(msg3, sizeof(msg3), "=== framebufferResized detected, currentFrame=%zu ===", currentFrame);
                debugLog("main.cpp:515", msg3, "C");
                
                // CRÍTICO: Obtener el nuevo tamaño
                int newWidth, newHeight;
                window.getFramebufferSize(&newWidth, &newHeight);
                char msg4[256];
                sprintf_s(msg4, sizeof(msg4), "New framebuffer size: %dx%d", newWidth, newHeight);
                debugLog("main.cpp:518", msg4, "C");
                
                // Validar dimensiones antes de proceder - Adaptativo desde 800x600 hasta 8K
                const int MIN_WIDTH = 800;
                const int MIN_HEIGHT = 600;
                const int MAX_WIDTH = 7680;   // 8K width
                const int MAX_HEIGHT = 4320;  // 8K height
                
                if (newWidth >= MIN_WIDTH && newHeight >= MIN_HEIGHT &&
                    newWidth <= MAX_WIDTH && newHeight <= MAX_HEIGHT &&
                    (static_cast<uint32_t>(newWidth) != currentSwapchainWidth || 
                     static_cast<uint32_t>(newHeight) != currentSwapchainHeight)) {
                    
                    // CRÍTICO: Procesar eventos ANTES de esperar para mantener responsividad
                    window.pollEvents();
                    
                    // Usar vkQueueWaitIdle en lugar de vkDeviceWaitIdle (más rápido, menos bloqueante)
                    // Solo espera la cola de gráficos, no todo el dispositivo
                    debugLog("main.cpp:508", "Waiting for graphics queue before resize", "E");
                    vkQueueWaitIdle(ctx.graphicsQueue());
                    
                    // Procesar eventos DESPUÉS también
                    window.pollEvents();
                    debugLog("main.cpp:511", "Graphics queue idle, recreating swapchain", "E");
                    
                    // Limpiar imágenes en vuelo para evitar usar recursos del swapchain viejo
                    std::fill(imagesInFlight.begin(), imagesInFlight.end(), VK_NULL_HANDLE);
                    
                    recreateSwapchain(static_cast<uint32_t>(newWidth), static_cast<uint32_t>(newHeight));
                    // CRÍTICO: Actualizar dimensiones después de recrear
                    currentSwapchainWidth = swapchain.extent().width;
                    currentSwapchainHeight = swapchain.extent().height;
                    // CRÍTICO: Resetear currentFrame después de recrear para evitar problemas de índices
                    currentFrame = 0;
                    debugLog("main.cpp:522", "recreateSwapchain completed, currentFrame reset", "C");
                }
                framebufferResized = false;
                continue;
            }
            
            // Calcular tiempo para animación constante
            auto currentTime = std::chrono::high_resolution_clock::now();
            float time = std::chrono::duration<float>(currentTime - startTime).count();
            
            // Animar cubo (rotación continua y visible)
            // Rotación más notoria para que sea claramente visible
            cubeTransform.rotation.y = time * glm::radians(90.0f);  // 90 grados/segundo en Y (eje vertical)
            cubeTransform.rotation.x = time * glm::radians(60.0f);  // 60 grados/segundo en X (eje horizontal)
            
            // Wait for fence (esperar a que el frame anterior termine)
            // CRÍTICO: Verificar límites antes de acceder a arrays
            if (currentFrame >= inFlight.size()) {
                debugLog("main.cpp:ERROR", "currentFrame out of bounds, resetting to 0", "A");
                currentFrame = 0;
            }
            
            char fenceMsg[256];
            sprintf_s(fenceMsg, sizeof(fenceMsg), "Before waiting fence, currentFrame=%zu", currentFrame);
            debugLog("main.cpp:550", fenceMsg, "ALL");
            inFlight[currentFrame].wait();
            debugLog("main.cpp:552", "After waiting fence", "ALL");
            
            // Acquire image con manejo de errores
            debugLog("main.cpp:555", "Before vkAcquireNextImageKHR", "ALL");
            uint32_t imageIndex;
            VkResult acquireResult = vkAcquireNextImageKHR(
                ctx.device(),
                swapchain.handle(),
                UINT64_MAX,
                imageAvailable[currentFrame].handle(),
                VK_NULL_HANDLE,
                &imageIndex
            );
            
            char acquireMsg[256];
            sprintf_s(acquireMsg, sizeof(acquireMsg), "vkAcquireNextImageKHR result=%d, imageIndex=%u", acquireResult, imageIndex);
            debugLog("main.cpp:563", acquireMsg, "ALL");
            
            // Manejar errores del swapchain o cambios de tamaño
            if (acquireResult == VK_ERROR_OUT_OF_DATE_KHR || acquireResult == VK_SUBOPTIMAL_KHR) {
                char msg6[256];
                sprintf_s(msg6, sizeof(msg6), "acquireResult=%d needs recreate", acquireResult);
                debugLog("main.cpp:555", msg6, "A");
                // El swapchain necesita ser recreado
                // Procesar eventos antes y después de esperar
                window.pollEvents();
                debugLog("main.cpp:508", "Waiting for graphics queue (acquire error)", "E");
                vkQueueWaitIdle(ctx.graphicsQueue());
                window.pollEvents();
                std::fill(imagesInFlight.begin(), imagesInFlight.end(), VK_NULL_HANDLE);
                
                int newWidth, newHeight;
                window.getFramebufferSize(&newWidth, &newHeight);
                
                // Validar dimensiones - Adaptativo desde 800x600 hasta 8K
                const int MIN_WIDTH = 800;
                const int MIN_HEIGHT = 600;
                const int MAX_WIDTH = 7680;   // 8K width
                const int MAX_HEIGHT = 4320;  // 8K height
                
                if (newWidth >= MIN_WIDTH && newHeight >= MIN_HEIGHT &&
                    newWidth <= MAX_WIDTH && newHeight <= MAX_HEIGHT &&
                    !recreatingSwapchain) {
                    recreateSwapchain(static_cast<uint32_t>(newWidth), static_cast<uint32_t>(newHeight));
                    // CRÍTICO: Actualizar dimensiones después de recrear
                    currentSwapchainWidth = swapchain.extent().width;
                    currentSwapchainHeight = swapchain.extent().height;
                    // CRÍTICO: Resetear currentFrame después de recrear
                    currentFrame = 0;
                }
                framebufferResized = false;
                continue;
            } else if (acquireResult != VK_SUCCESS) {
                char msg7[256];
                sprintf_s(msg7, sizeof(msg7), "acquireResult error: %d", acquireResult);
                debugLog("main.cpp:571", msg7, "A");
                // Otro error, continuar al siguiente frame
                continue;
            }
            
            // Check if a previous frame is using this image
            debugLog("main.cpp:591", "Before checking imagesInFlight", "ALL");
            if (imageIndex < imagesInFlight.size() && imagesInFlight[imageIndex] != VK_NULL_HANDLE) {
                debugLog("main.cpp:593", "Previous frame using image, waiting", "ALL");
                vkWaitForFences(ctx.device(), 1, &imagesInFlight[imageIndex], VK_TRUE, UINT64_MAX);
            }
            // CRÍTICO: Verificar límites antes de acceder a arrays
            if (imageIndex < imagesInFlight.size() && currentFrame < inFlight.size()) {
                imagesInFlight[imageIndex] = inFlight[currentFrame].handle();
            }
            
            debugLog("main.cpp:597", "Before resetting fence", "ALL");
            if (currentFrame < inFlight.size()) {
                inFlight[currentFrame].reset();
            }
            debugLog("main.cpp:598", "After resetting fence", "ALL");
            
            // Actualizar aspect ratio de la cámara con las dimensiones actuales del swapchain
            // Esto asegura que el cubo se mantenga centrado y sin distorsión en cualquier resolución
            float currentAspect = static_cast<float>(swapchain.extent().width) / static_cast<float>(swapchain.extent().height);
            camera.aspectRatio = currentAspect;
            
            // Calcular matrices
            glm::mat4 view = camera.getViewMatrix();
            glm::mat4 proj = camera.getProjectionMatrix();
            
            // Record commands
            auto& cmd = cmdBuffers[imageIndex];
            cmd.reset();
            cmd.begin();
            
            // Configurar viewport y scissor dinámicamente para respetar la resolución actual
            // Esto asegura que el render se ajuste correctamente a cualquier tamaño de ventana
            float viewportWidth = static_cast<float>(swapchain.extent().width);
            float viewportHeight = static_cast<float>(swapchain.extent().height);
            cmd.setViewport(0.0f, 0.0f, viewportWidth, viewportHeight, 0.0f, 1.0f);
            cmd.setScissor(0, 0, swapchain.extent().width, swapchain.extent().height);
            
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
            
            // Calcular matriz del modelo (con rotación actualizada)
            glm::mat4 model = cubeTransform.getMatrix();
            // Calcular MVP: Projection * View * Model
            glm::mat4 mvp = proj * view * model;
            
            // Renderizar con modo debug
            cubeRenderer.render(cmd, mvp, model, debugMode);
            
            vkCmdEndRenderPass(cmd.handle());
            cmd.end();
            
            // Submit
            // CRÍTICO: Verificar límites antes de acceder a arrays
            if (currentFrame >= imageAvailable.size() || currentFrame >= renderFinished.size() || currentFrame >= inFlight.size()) {
                debugLog("main.cpp:ERROR", "currentFrame out of bounds, skipping submit", "A");
                continue;
            }
            
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
            
            debugLog("main.cpp:711", "Before vkQueueSubmit", "ALL");
            vkQueueSubmit(ctx.graphicsQueue(), 1, &submitInfo, inFlight[currentFrame].handle());
            debugLog("main.cpp:712", "After vkQueueSubmit", "ALL");
            
            // Present con manejo de errores
            // CRÍTICO: Verificar límites antes de acceder a arrays
            if (currentFrame >= renderFinished.size()) {
                debugLog("main.cpp:ERROR", "currentFrame out of bounds, skipping present", "A");
                continue;
            }
            
            VkPresentInfoKHR presentInfo{};
            presentInfo.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
            presentInfo.waitSemaphoreCount = 1;
            VkSemaphore signalSem = renderFinished[currentFrame].handle();
            presentInfo.pWaitSemaphores = &signalSem;
            presentInfo.swapchainCount = 1;
            VkSwapchainKHR swapchainHandle = swapchain.handle();
            presentInfo.pSwapchains = &swapchainHandle;
            presentInfo.pImageIndices = &imageIndex;
            
            debugLog("main.cpp:724", "Before vkQueuePresentKHR", "ALL");
            VkResult presentResult = vkQueuePresentKHR(ctx.graphicsQueue(), &presentInfo);
            
            char presentMsg[256];
            sprintf_s(presentMsg, sizeof(presentMsg), "vkQueuePresentKHR result=%d", presentResult);
            debugLog("main.cpp:727", presentMsg, "ALL");
            
            // Manejar errores del present
            if (presentResult == VK_ERROR_OUT_OF_DATE_KHR || presentResult == VK_SUBOPTIMAL_KHR) {
                char msg8[256];
                sprintf_s(msg8, sizeof(msg8), "presentResult=%d needs recreate", presentResult);
                debugLog("main.cpp:601", msg8, "B");
                // El swapchain necesita ser recreado después del present
                // Procesar eventos antes y después de esperar
                window.pollEvents();
                debugLog("main.cpp:508", "Waiting for graphics queue (present error)", "E");
                vkQueueWaitIdle(ctx.graphicsQueue());
                window.pollEvents();
                std::fill(imagesInFlight.begin(), imagesInFlight.end(), VK_NULL_HANDLE);
                
                int newWidth, newHeight;
                window.getFramebufferSize(&newWidth, &newHeight);
                
                // Validar dimensiones - Adaptativo desde 800x600 hasta 8K
                const int MIN_WIDTH = 800;
                const int MIN_HEIGHT = 600;
                const int MAX_WIDTH = 7680;   // 8K width
                const int MAX_HEIGHT = 4320;  // 8K height
                
                if (newWidth >= MIN_WIDTH && newHeight >= MIN_HEIGHT &&
                    newWidth <= MAX_WIDTH && newHeight <= MAX_HEIGHT &&
                    !recreatingSwapchain) {
                    recreateSwapchain(static_cast<uint32_t>(newWidth), static_cast<uint32_t>(newHeight));
                    // CRÍTICO: Actualizar dimensiones después de recrear
                    currentSwapchainWidth = swapchain.extent().width;
                    currentSwapchainHeight = swapchain.extent().height;
                    // CRÍTICO: Resetear currentFrame después de recrear
                    currentFrame = 0;
                }
                framebufferResized = false;
                continue;
            } else if (presentResult != VK_SUCCESS) {
                char msg9[256];
                sprintf_s(msg9, sizeof(msg9), "presentResult error: %d", presentResult);
                debugLog("main.cpp:614", msg9, "B");
                // Error en present, continuar
                continue;
            }
            
            // FPS y stats
            frameCount++;
            auto fpsDuration = std::chrono::duration<float>(currentTime - lastFpsTime).count();
            if (fpsDuration >= 0.5f) {
                float fps = frameCount / fpsDuration;
                const char* modes[] = {"Normal", "Wireframe", "Normales", "Depth", "ISR:Importance", "ISR:PixelSize", "ISR:Temporal"};
                std::string title = "Stack-GPU-OP | FPS: " + std::to_string(static_cast<int>(fps)) + " | " + modes[debugMode];
                window.setTitle(title);
                std::cout << "\rFPS: " << static_cast<int>(fps) << " | Modo: " << modes[debugMode] << "     " << std::flush;
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
