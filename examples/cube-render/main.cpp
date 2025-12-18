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
#include <iostream>
#include <chrono>
#include <array>

struct Vertex {
    reactor::Vec3 pos;
    reactor::Vec3 color;
};

// Cubo 3D completo con índices
const std::vector<Vertex> cubeVertices = {
    // Front (rojo)
    {{-0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{ 0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{ 0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{-0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    // Back (verde)
    {{-0.5f, -0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{ 0.5f, -0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{ 0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{-0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
};

const std::vector<uint16_t> cubeIndices = {
    0,1,2, 2,3,0,  // Front
    5,4,7, 7,6,5,  // Back
    4,0,3, 3,7,4,  // Left
    1,5,6, 6,2,1,  // Right
    3,2,6, 6,7,3,  // Top
    4,5,1, 1,0,4   // Bottom
};

int main() {
    try {
        std::cout << "==========================================" << std::endl;
        std::cout << "  REACTOR - Cubo 3D Renderizado" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        // Inicializar GLFW
        reactor::Window::init();
        
        // Crear ventana
        reactor::WindowConfig config;
        config.title = "REACTOR - Cubo 3D Animado";
        config.width = 1280;
        config.height = 720;
        
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
        
        // Cargar shaders
        reactor::Shader vertShader = reactor::Shader::fromFile(ctx.device(), "shaders/cube.vert.spv", reactor::ShaderStage::Vertex);
        reactor::Shader fragShader = reactor::Shader::fromFile(ctx.device(), "shaders/cube.frag.spv", reactor::ShaderStage::Fragment);
        std::cout << "[✓] Shaders cargados" << std::endl;
        
        // Crear pipeline
        std::vector<VkVertexInputBindingDescription> bindings = {{
            .binding = 0,
            .stride = sizeof(Vertex),
            .inputRate = VK_VERTEX_INPUT_RATE_VERTEX
        }};
        
        std::vector<VkVertexInputAttributeDescription> attributes = {
            {0, 0, VK_FORMAT_R32G32B32_SFLOAT, offsetof(Vertex, pos)},
            {1, 0, VK_FORMAT_R32G32B32_SFLOAT, offsetof(Vertex, color)}
        };
        
        // Push constant para MVP
        VkPushConstantRange pushConstant{};
        pushConstant.stageFlags = VK_SHADER_STAGE_VERTEX_BIT;
        pushConstant.offset = 0;
        pushConstant.size = sizeof(reactor::Mat4);
        
        auto pipeline = reactor::GraphicsPipeline::create(ctx.device(), renderPass.handle())
            .shader(vertShader)
            .shader(fragShader)
            .vertexInput(bindings, attributes)
            .topology(reactor::Topology::TriangleList)
            .viewport(static_cast<float>(config.width), static_cast<float>(config.height))
            .cullMode(reactor::CullMode::Back)
            .pushConstant(pushConstant)
            .build();
        std::cout << "[✓] Pipeline creado" << std::endl;
        
        // Crear buffers
        auto vertexBuffer = reactor::Buffer::create(ctx.allocator())
            .size(sizeof(Vertex) * cubeVertices.size())
            .usage(reactor::BufferUsage::Vertex)
            .memoryType(reactor::MemoryType::HostVisible)
            .build();
        vertexBuffer.upload(cubeVertices.data(), sizeof(Vertex) * cubeVertices.size());
        
        auto indexBuffer = reactor::Buffer::create(ctx.allocator())
            .size(sizeof(uint16_t) * cubeIndices.size())
            .usage(reactor::BufferUsage::Index)
            .memoryType(reactor::MemoryType::HostVisible)
            .build();
        indexBuffer.upload(cubeIndices.data(), sizeof(uint16_t) * cubeIndices.size());
        std::cout << "[✓] Buffers creados" << std::endl;
        
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
        auto cmdBuffers = cmdPool.allocate(swapchain.imageCount());
        
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
        
        // Camera y transform
        reactor::Camera camera;
        camera.position = reactor::Vec3(3.0f, 3.0f, 3.0f);
        camera.target = reactor::Vec3(0.0f, 0.0f, 0.0f);
        camera.aspectRatio = static_cast<float>(config.width) / config.height;
        
        reactor::Transform cubeTransform;
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  [✓] ¡Listo! Renderizando cubo 3D..." << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "Controles: ESC para salir" << std::endl;
        std::cout << std::endl;
        
        // Render loop
        size_t currentFrame = 0;
        auto startTime = std::chrono::high_resolution_clock::now();
        int frameCount = 0;
        auto lastFpsTime = startTime;
        
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
            
            inFlight[currentFrame].reset();
            
            // Calcular MVP
            reactor::Mat4 model = cubeTransform.getMatrix();
            reactor::Mat4 view = camera.getViewMatrix();
            reactor::Mat4 proj = camera.getProjectionMatrix();
            reactor::Mat4 mvp = proj * view * model;
            
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
            vkCmdBindPipeline(cmd.handle(), VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline.handle());
            
            VkBuffer vb = vertexBuffer.handle();
            VkDeviceSize offset = 0;
            vkCmdBindVertexBuffers(cmd.handle(), 0, 1, &vb, &offset);
            vkCmdBindIndexBuffer(cmd.handle(), indexBuffer.handle(), 0, VK_INDEX_TYPE_UINT16);
            
            vkCmdPushConstants(cmd.handle(), pipeline.layout(), VK_SHADER_STAGE_VERTEX_BIT, 0, sizeof(reactor::Mat4), &mvp);
            
            vkCmdDrawIndexed(cmd.handle(), static_cast<uint32_t>(cubeIndices.size()), 1, 0, 0, 0);
            
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
                std::cout << "FPS: " << static_cast<int>(fps) << " | Rotación: " << glm::degrees(cubeTransform.rotation.y) << "°" << std::endl;
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
        
        std::cout << std::endl << "[✓] Aplicación finalizada" << std::endl;
        
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "❌ Error: " << e.what() << std::endl;
        reactor::Window::terminate();
        return 1;
    }
}
