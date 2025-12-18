#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/window.hpp"
#include "reactor/swapchain.hpp"
#include "reactor/render_pass.hpp"
#include "reactor/pipeline.hpp"
#include "reactor/buffer.hpp"
#include "reactor/descriptor.hpp"
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
    
    static std::vector<VkVertexInputBindingDescription> getBindingDescription() {
        VkVertexInputBindingDescription binding{};
        binding.binding = 0;
        binding.stride = sizeof(Vertex);
        binding.inputRate = VK_VERTEX_INPUT_RATE_VERTEX;
        return {binding};
    }
    
    static std::vector<VkVertexInputAttributeDescription> getAttributeDescriptions() {
        std::vector<VkVertexInputAttributeDescription> attributes(2);
        
        attributes[0].binding = 0;
        attributes[0].location = 0;
        attributes[0].format = VK_FORMAT_R32G32B32_SFLOAT;
        attributes[0].offset = offsetof(Vertex, pos);
        
        attributes[1].binding = 0;
        attributes[1].location = 1;
        attributes[1].format = VK_FORMAT_R32G32B32_SFLOAT;
        attributes[1].offset = offsetof(Vertex, color);
        
        return attributes;
    }
};

// Cubo 3D con colores (React-style component data)
const std::vector<Vertex> cubeVertices = {
    // Front face (rojo)
    {{-0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{ 0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{ 0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{-0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    
    // Back face (verde)
    {{-0.5f, -0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{ 0.5f, -0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{ 0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{-0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    
    // Top face (azul)
    {{-0.5f,  0.5f, -0.5f}, {0.0f, 0.0f, 1.0f}},
    {{ 0.5f,  0.5f, -0.5f}, {0.0f, 0.0f, 1.0f}},
    {{ 0.5f,  0.5f,  0.5f}, {0.0f, 0.0f, 1.0f}},
    {{-0.5f,  0.5f,  0.5f}, {0.0f, 0.0f, 1.0f}},
    
    // Bottom face (amarillo)
    {{-0.5f, -0.5f, -0.5f}, {1.0f, 1.0f, 0.0f}},
    {{ 0.5f, -0.5f, -0.5f}, {1.0f, 1.0f, 0.0f}},
    {{ 0.5f, -0.5f,  0.5f}, {1.0f, 1.0f, 0.0f}},
    {{-0.5f, -0.5f,  0.5f}, {1.0f, 1.0f, 0.0f}},
    
    // Right face (magenta)
    {{ 0.5f, -0.5f, -0.5f}, {1.0f, 0.0f, 1.0f}},
    {{ 0.5f,  0.5f, -0.5f}, {1.0f, 0.0f, 1.0f}},
    {{ 0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 1.0f}},
    {{ 0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 1.0f}},
    
    // Left face (cyan)
    {{-0.5f, -0.5f, -0.5f}, {0.0f, 1.0f, 1.0f}},
    {{-0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 1.0f}},
    {{-0.5f,  0.5f,  0.5f}, {0.0f, 1.0f, 1.0f}},
    {{-0.5f, -0.5f,  0.5f}, {0.0f, 1.0f, 1.0f}}
};

const std::vector<uint16_t> cubeIndices = {
    0,  1,  2,  2,  3,  0,   // Front
    4,  5,  6,  6,  7,  4,   // Back
    8,  9,  10, 10, 11, 8,   // Top
    12, 13, 14, 14, 15, 12,  // Bottom
    16, 17, 18, 18, 19, 16,  // Right
    20, 21, 22, 22, 23, 20   // Left
};

int main() {
    try {
        std::cout << "==========================================" << std::endl;
        std::cout << "  REACTOR - 3D Animated Cube (React-Style)" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        // Initialize window system
        reactor::Window::init();
        
        // Create window (React-style config)
        reactor::WindowConfig windowConfig;
        windowConfig.title = "REACTOR - Animated 3D Cube";
        windowConfig.width = 1280;
        windowConfig.height = 720;
        windowConfig.vsync = true;
        
        reactor::Window window(windowConfig);
        std::cout << "[✓] Window created: " << windowConfig.width << "x" << windowConfig.height << std::endl;
        
        // Initialize Vulkan
        reactor::VulkanContext ctx(true);
        ctx.init();
        std::cout << "[✓] Vulkan initialized" << std::endl;
        
        // Create surface
        VkSurfaceKHR surface = window.createSurface(ctx.instance());
        std::cout << "[✓] Surface created" << std::endl;
        
        // Create swapchain
        reactor::Swapchain swapchain(ctx.device(), ctx.physical(), surface,
                                     windowConfig.width, windowConfig.height);
        std::cout << "[✓] Swapchain created" << std::endl;
        
        // Create render pass
        auto renderPass = reactor::RenderPass::create(ctx.device())
            .colorAttachment({
                .format = swapchain.format(),
                .loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR,
                .storeOp = VK_ATTACHMENT_STORE_OP_STORE,
                .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
                .finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR
            })
            .build();
        std::cout << "[✓] Render pass created" << std::endl;
        
        // Load shaders
        auto vertShader = reactor::Shader::fromFile(ctx.device(), "shaders/cube.vert.spv", 
                                                    reactor::ShaderStage::Vertex);
        auto fragShader = reactor::Shader::fromFile(ctx.device(), "shaders/cube.frag.spv",
                                                    reactor::ShaderStage::Fragment);
        std::cout << "[✓] Shaders loaded" << std::endl;
        
        // Create descriptor set layout
        auto descriptorLayout = reactor::DescriptorSetLayout::create(ctx.device())
            .binding(0, reactor::DescriptorType::UniformBuffer, reactor::ShaderStage::Vertex)
            .build();
        
        // Create graphics pipeline
        auto bindings = Vertex::getBindingDescription();
        auto attributes = Vertex::getAttributeDescriptions();
        
        auto pipeline = reactor::GraphicsPipeline::create(ctx.device(), renderPass.handle())
            .shader(vertShader)
            .shader(fragShader)
            .vertexInput(bindings, attributes)
            .topology(reactor::Topology::TriangleList)
            .viewport(static_cast<float>(windowConfig.width), 
                     static_cast<float>(windowConfig.height))
            .cullMode(reactor::CullMode::Back)
            .depthTest(true)
            .descriptorSetLayout(descriptorLayout.handle())
            .build();
        std::cout << "[✓] Graphics pipeline created" << std::endl;
        
        // Create vertex buffer
        auto vertexBuffer = reactor::Buffer::create(ctx.allocator())
            .size(sizeof(Vertex) * cubeVertices.size())
            .usage(reactor::BufferUsage::Vertex)
            .memoryType(reactor::MemoryType::HostVisible)
            .build();
        vertexBuffer.upload(cubeVertices.data(), sizeof(Vertex) * cubeVertices.size());
        
        // Create index buffer
        auto indexBuffer = reactor::Buffer::create(ctx.allocator())
            .size(sizeof(uint16_t) * cubeIndices.size())
            .usage(reactor::BufferUsage::Index)
            .memoryType(reactor::MemoryType::HostVisible)
            .build();
        indexBuffer.upload(cubeIndices.data(), sizeof(uint16_t) * cubeIndices.size());
        std::cout << "[✓] Buffers created" << std::endl;
        
        // Create uniform buffers
        std::vector<reactor::Buffer> uniformBuffers;
        for (size_t i = 0; i < swapchain.imageCount(); i++) {
            uniformBuffers.push_back(
                reactor::Buffer::create(ctx.allocator())
                    .size(sizeof(reactor::UniformBufferObject))
                    .usage(reactor::BufferUsage::Uniform)
                    .memoryType(reactor::MemoryType::HostVisible)
                    .build()
            );
        }
        
        // Create descriptor pool and sets
        auto descriptorPool = reactor::DescriptorPool::create(ctx.device())
            .maxSets(swapchain.imageCount())
            .poolSize(reactor::DescriptorType::UniformBuffer, swapchain.imageCount())
            .build();
        
        std::vector<reactor::DescriptorSet> descriptorSets;
        for (size_t i = 0; i < swapchain.imageCount(); i++) {
            auto descSet = descriptorPool.allocate(descriptorLayout.handle());
            descSet.updateBuffer(0, uniformBuffers[i].handle(), 
                               sizeof(reactor::UniformBufferObject));
            descriptorSets.push_back(std::move(descSet));
        }
        std::cout << "[✓] Descriptors created" << std::endl;
        
        // Create framebuffers
        std::vector<reactor::Framebuffer> framebuffers;
        for (size_t i = 0; i < swapchain.imageCount(); i++) {
            framebuffers.push_back(
                reactor::Framebuffer::create(ctx.device(), renderPass.handle())
                    .attachment(swapchain.imageView(i))
                    .extent(windowConfig.width, windowConfig.height)
                    .build()
            );
        }
        
        // Create command pool and buffers
        reactor::CommandPool commandPool(ctx.device(), ctx.queueFamilyIndices().graphics.value());
        auto commandBuffers = commandPool.allocate(swapchain.imageCount());
        
        // Create sync objects
        const int MAX_FRAMES_IN_FLIGHT = 2;
        std::vector<reactor::Semaphore> imageAvailableSemaphores;
        std::vector<reactor::Semaphore> renderFinishedSemaphores;
        std::vector<reactor::Fence> inFlightFences;
        
        for (int i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
            imageAvailableSemaphores.emplace_back(ctx.device());
            renderFinishedSemaphores.emplace_back(ctx.device());
            inFlightFences.emplace_back(ctx.device(), true);
        }
        std::cout << "[✓] Synchronization objects created" << std::endl;
        
        // React-style state (camera and transform)
        reactor::Camera camera;
        camera.position = reactor::Vec3(2.0f, 2.0f, 2.0f);
        camera.target = reactor::Vec3(0.0f, 0.0f, 0.0f);
        camera.aspectRatio = static_cast<float>(windowConfig.width) / windowConfig.height;
        
        reactor::Transform cubeTransform;
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  [✓] Initialization complete!" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        std::cout << "Controls:" << std::endl;
        std::cout << "  ESC - Exit" << std::endl;
        std::cout << "  Cube rotates automatically" << std::endl;
        std::cout << std::endl;
        
        // Render loop
        size_t currentFrame = 0;
        size_t frameCount = 0;
        auto startTime = std::chrono::high_resolution_clock::now();
        auto lastFpsTime = startTime;
        
        while (!window.shouldClose()) {
            window.pollEvents();
            
            auto currentTime = std::chrono::high_resolution_clock::now();
            float time = std::chrono::duration<float>(currentTime - startTime).count();
            
            // Update cube rotation (React-style state update)
            cubeTransform.rotation.y = time * glm::radians(90.0f);
            cubeTransform.rotation.x = time * glm::radians(45.0f);
            
            // Wait for previous frame
            inFlightFences[currentFrame].wait();
            
            // Acquire image
            uint32_t imageIndex;
            VkResult result = swapchain.acquireNextImage(
                imageAvailableSemaphores[currentFrame].handle(),
                &imageIndex
            );
            
            if (result == VK_ERROR_OUT_OF_DATE_KHR) {
                continue;
            }
            
            inFlightFences[currentFrame].reset();
            
            // Update uniform buffer (React-style props)
            reactor::UniformBufferObject ubo{};
            ubo.model = cubeTransform.getMatrix();
            ubo.view = camera.getViewMatrix();
            ubo.proj = camera.getProjectionMatrix();
            
            uniformBuffers[imageIndex].upload(&ubo, sizeof(ubo));
            
            // Record command buffer
            auto& cmd = commandBuffers[imageIndex];
            cmd.reset();
            cmd.begin();
            
            VkClearValue clearColor = {{{0.1f, 0.1f, 0.1f, 1.0f}}};
            cmd.beginRenderPass(renderPass.handle(), framebuffers[imageIndex].handle(),
                              {windowConfig.width, windowConfig.height}, {clearColor});
            
            cmd.bindPipeline(VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline.handle());
            cmd.bindVertexBuffers(0, {vertexBuffer.handle()}, {0});
            cmd.bindIndexBuffer(indexBuffer.handle(), 0, VK_INDEX_TYPE_UINT16);
            cmd.bindDescriptorSets(VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline.layout(),
                                  0, {descriptorSets[imageIndex].handle()});
            cmd.drawIndexed(static_cast<uint32_t>(cubeIndices.size()));
            
            cmd.endRenderPass();
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
            swapchain.present(ctx.graphicsQueue(), imageIndex,
                            renderFinishedSemaphores[currentFrame].handle());
            
            currentFrame = (currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
            frameCount++;
            
            // FPS counter
            auto elapsed = std::chrono::duration<double>(currentTime - lastFpsTime).count();
            if (elapsed >= 1.0) {
                double fps = frameCount / elapsed;
                std::cout << "FPS: " << static_cast<int>(fps) << " | Rotation: " 
                         << glm::degrees(cubeTransform.rotation.y) << "°" << std::endl;
                frameCount = 0;
                lastFpsTime = currentTime;
            }
        }
        
        // Cleanup
        vkDeviceWaitIdle(ctx.device());
        vkDestroySurfaceKHR(ctx.instance(), surface, nullptr);
        ctx.shutdown();
        reactor::Window::terminate();
        
        std::cout << std::endl;
        std::cout << "[✓] Application finished successfully" << std::endl;
        
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "❌ Error: " << e.what() << std::endl;
        reactor::Window::terminate();
        return 1;
    }
}
