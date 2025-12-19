#include "reactor/window.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/image.hpp"
#include "reactor/command_buffer.hpp"
#include "reactor/sync.hpp"
#include "reactor/shader.hpp"
#include "reactor/pipeline.hpp"
#include "reactor/math.hpp"
#include <iostream>
#include <chrono>
#include <thread>
#include <windows.h>
#include <GLFW/glfw3.h>

int main() {
    try {
        SetConsoleOutputCP(CP_UTF8);
        setvbuf(stdout, nullptr, _IOFBF, 1000);
        
        std::cout << "============================================" << std::endl;
        std::cout << "  🔺 KILLER TRIANGLE - Rendering Sin Triángulos" << std::endl;
        std::cout << "  SDF Matemáticas Puras + Ray Marching GPU" << std::endl;
        std::cout << "============================================" << std::endl;
        std::cout << std::endl;
        
        reactor::Window::init();
        
        reactor::WindowConfig config;
        config.title = "Killer Triangle - SDF Ray Marching (Sin Triángulos)";
        config.width = 1920;
        config.height = 1080;
        
        reactor::Window window(config);
        glfwMaximizeWindow(window.handle());
        std::cout << "[✓] Ventana creada (1920x1080)" << std::endl;
        
        reactor::VulkanContext ctx(true);
        ctx.init();
        std::cout << "[✓] Vulkan inicializado" << std::endl;
        
        // Crear imagen de salida para compute shader
        VkImageCreateInfo imageInfo{};
        imageInfo.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
        imageInfo.imageType = VK_IMAGE_TYPE_2D;
        imageInfo.format = VK_FORMAT_R8G8B8A8_UNORM;
        imageInfo.extent = {static_cast<uint32_t>(config.width), static_cast<uint32_t>(config.height), 1};
        imageInfo.mipLevels = 1;
        imageInfo.arrayLayers = 1;
        imageInfo.samples = VK_SAMPLE_COUNT_1_BIT;
        imageInfo.tiling = VK_IMAGE_TILING_OPTIMAL;
        imageInfo.usage = VK_IMAGE_USAGE_STORAGE_BIT | VK_IMAGE_USAGE_SAMPLED_BIT | VK_IMAGE_USAGE_TRANSFER_SRC_BIT;
        imageInfo.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
        
        VkImage outputImage;
        vkCreateImage(ctx.device(), &imageInfo, nullptr, &outputImage);
        
        VkMemoryRequirements memReqs;
        vkGetImageMemoryRequirements(ctx.device(), outputImage, &memReqs);
        auto imageBlock = ctx.allocator()->allocate(memReqs, reactor::MemoryType::DeviceLocal);
        vkBindImageMemory(ctx.device(), outputImage, imageBlock.memory, imageBlock.offset);
        
        VkImageViewCreateInfo viewInfo{};
        viewInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
        viewInfo.image = outputImage;
        viewInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
        viewInfo.format = VK_FORMAT_R8G8B8A8_UNORM;
        viewInfo.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
        viewInfo.subresourceRange.levelCount = 1;
        viewInfo.subresourceRange.layerCount = 1;
        
        VkImageView outputView;
        vkCreateImageView(ctx.device(), &viewInfo, nullptr, &outputView);
        std::cout << "[✓] Output image creada" << std::endl;
        
        // Cargar compute shader de ray marching
        reactor::Shader computeShader(ctx.device(), "shaders/sdf/raymarch.comp.spv", VK_SHADER_STAGE_COMPUTE_BIT);
        std::cout << "[✓] Ray marching compute shader cargado" << std::endl;
        
        // Crear descriptor set layout
        VkDescriptorSetLayoutBinding binding{};
        binding.binding = 0;
        binding.descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
        binding.descriptorCount = 1;
        binding.stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
        
        VkDescriptorSetLayoutCreateInfo layoutInfo{};
        layoutInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
        layoutInfo.bindingCount = 1;
        layoutInfo.pBindings = &binding;
        
        VkDescriptorSetLayout descriptorLayout;
        vkCreateDescriptorSetLayout(ctx.device(), &layoutInfo, nullptr, &descriptorLayout);
        
        // Crear descriptor pool
        VkDescriptorPoolSize poolSize{};
        poolSize.type = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
        poolSize.descriptorCount = 1;
        
        VkDescriptorPoolCreateInfo poolInfo{};
        poolInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
        poolInfo.maxSets = 1;
        poolInfo.poolSizeCount = 1;
        poolInfo.pPoolSizes = &poolSize;
        
        VkDescriptorPool descriptorPool;
        vkCreateDescriptorPool(ctx.device(), &poolInfo, nullptr, &descriptorPool);
        
        // Allocate descriptor set
        VkDescriptorSetAllocateInfo allocInfo{};
        allocInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
        allocInfo.descriptorPool = descriptorPool;
        allocInfo.descriptorSetCount = 1;
        allocInfo.pSetLayouts = &descriptorLayout;
        
        VkDescriptorSet descriptorSet;
        vkAllocateDescriptorSets(ctx.device(), &allocInfo, &descriptorSet);
        
        // Update descriptor set
        VkDescriptorImageInfo imageDescriptor{};
        imageDescriptor.imageView = outputView;
        imageDescriptor.imageLayout = VK_IMAGE_LAYOUT_GENERAL;
        
        VkWriteDescriptorSet write{};
        write.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        write.dstSet = descriptorSet;
        write.dstBinding = 0;
        write.descriptorCount = 1;
        write.descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
        write.pImageInfo = &imageDescriptor;
        
        vkUpdateDescriptorSets(ctx.device(), 1, &write, 0, nullptr);
        std::cout << "[✓] Descriptors configurados" << std::endl;
        
        // Push constants
        struct PushConstants {
            glm::mat4 invViewProj;
            glm::vec3 cameraPos;
            float time;
            glm::ivec2 resolution;
            int debugMode;
            int padding;
        };
        
        VkPushConstantRange pushConstant{};
        pushConstant.stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
        pushConstant.offset = 0;
        pushConstant.size = sizeof(PushConstants);
        
        // Crear compute pipeline
        VkPipelineLayoutCreateInfo pipelineLayoutInfo{};
        pipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
        pipelineLayoutInfo.setLayoutCount = 1;
        pipelineLayoutInfo.pSetLayouts = &descriptorLayout;
        pipelineLayoutInfo.pushConstantRangeCount = 1;
        pipelineLayoutInfo.pPushConstantRanges = &pushConstant;
        
        VkPipelineLayout pipelineLayout;
        vkCreatePipelineLayout(ctx.device(), &pipelineLayoutInfo, nullptr, &pipelineLayout);
        
        VkComputePipelineCreateInfo pipelineInfo{};
        pipelineInfo.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
        pipelineInfo.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
        pipelineInfo.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT;
        pipelineInfo.stage.module = computeShader.module();
        pipelineInfo.stage.pName = "main";
        pipelineInfo.layout = pipelineLayout;
        
        VkPipeline computePipeline;
        vkCreateComputePipelines(ctx.device(), VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &computePipeline);
        std::cout << "[✓] Compute pipeline creado" << std::endl;
        
        // Command pool y buffers
        reactor::CommandPool cmdPool(ctx.device(), ctx.queueFamilyIndices().compute.value());
        auto cmdPoolPtr = std::make_shared<reactor::CommandPool>(std::move(cmdPool));
        reactor::CommandBuffer cmd(cmdPoolPtr);
        
        // Sincronización
        reactor::Fence fence(ctx.device(), false);
        
        // Cámara
        reactor::Camera camera;
        camera.position = glm::vec3(0.0f, 2.0f, 8.0f);
        camera.target = glm::vec3(0.0f, 0.0f, 0.0f);
        camera.aspectRatio = static_cast<float>(config.width) / config.height;
        
        int debugMode = 0;
        
        std::cout << std::endl;
        std::cout << "============================================" << std::endl;
        std::cout << "  CONTROLES:" << std::endl;
        std::cout << "  [1] Normal - Phong Shading" << std::endl;
        std::cout << "  [2] Wireframe Mode 🔥" << std::endl;
        std::cout << "  [3] Distance Visualization" << std::endl;
        std::cout << "  [4] Performance (Steps)" << std::endl;
        std::cout << "  [5] Normals RGB" << std::endl;
        std::cout << "  [ESC] Salir" << std::endl;
        std::cout << "============================================" << std::endl;
        std::cout << std::endl;
        std::cout << "🔺 Rendering SIN triángulos - Solo matemáticas SDF" << std::endl;
        std::cout << std::endl;
        
        auto startTime = std::chrono::high_resolution_clock::now();
        int frameCount = 0;
        auto lastFpsTime = startTime;
        
        const char* modes[] = {"[1] Normal", "[2] Wireframe", "[3] Distance", "[4] Steps", "[5] Normals"};
        
        while (!window.shouldClose()) {
            window.pollEvents();
            
            int width, height;
            glfwGetFramebufferSize(window.handle(), &width, &height);
            if (width == 0 || height == 0) {
                std::this_thread::sleep_for(std::chrono::milliseconds(100));
                continue;
            }
            
            // Input
            if (glfwGetKey(window.handle(), GLFW_KEY_1) == GLFW_PRESS) debugMode = 0;
            if (glfwGetKey(window.handle(), GLFW_KEY_2) == GLFW_PRESS) debugMode = 1;
            if (glfwGetKey(window.handle(), GLFW_KEY_3) == GLFW_PRESS) debugMode = 2;
            if (glfwGetKey(window.handle(), GLFW_KEY_4) == GLFW_PRESS) debugMode = 3;
            if (glfwGetKey(window.handle(), GLFW_KEY_5) == GLFW_PRESS) debugMode = 4;
            
            auto currentTime = std::chrono::high_resolution_clock::now();
            float time = std::chrono::duration<float>(currentTime - startTime).count();
            
            // Preparar push constants
            PushConstants pc{};
            glm::mat4 view = camera.getViewMatrix();
            glm::mat4 proj = camera.getProjectionMatrix();
            pc.invViewProj = glm::inverse(proj * view);
            pc.cameraPos = camera.position;
            pc.time = time;
            pc.resolution = glm::ivec2(config.width, config.height);
            pc.debugMode = debugMode;
            
            // Record command buffer
            cmd.reset();
            cmd.begin();
            
            // Transition image to general layout
            VkImageMemoryBarrier barrier{};
            barrier.sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER;
            barrier.oldLayout = VK_IMAGE_LAYOUT_UNDEFINED;
            barrier.newLayout = VK_IMAGE_LAYOUT_GENERAL;
            barrier.srcQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
            barrier.dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
            barrier.image = outputImage;
            barrier.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
            barrier.subresourceRange.levelCount = 1;
            barrier.subresourceRange.layerCount = 1;
            barrier.srcAccessMask = 0;
            barrier.dstAccessMask = VK_ACCESS_SHADER_WRITE_BIT;
            
            vkCmdPipelineBarrier(cmd.handle(), VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, 
                               VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT, 0, 0, nullptr, 0, nullptr, 1, &barrier);
            
            // Bind compute pipeline
            vkCmdBindPipeline(cmd.handle(), VK_PIPELINE_BIND_POINT_COMPUTE, computePipeline);
            vkCmdBindDescriptorSets(cmd.handle(), VK_PIPELINE_BIND_POINT_COMPUTE, pipelineLayout, 
                                   0, 1, &descriptorSet, 0, nullptr);
            vkCmdPushConstants(cmd.handle(), pipelineLayout, VK_SHADER_STAGE_COMPUTE_BIT, 
                             0, sizeof(PushConstants), &pc);
            
            // Dispatch compute shader (8x8 local size)
            uint32_t groupsX = (config.width + 7) / 8;
            uint32_t groupsY = (config.height + 7) / 8;
            vkCmdDispatch(cmd.handle(), groupsX, groupsY, 1);
            
            cmd.end();
            
            // Submit
            VkSubmitInfo submitInfo{};
            submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
            submitInfo.commandBufferCount = 1;
            VkCommandBuffer cmdHandle = cmd.handle();
            submitInfo.pCommandBuffers = &cmdHandle;
            
            vkQueueSubmit(ctx.computeQueue(), 1, &submitInfo, fence.handle());
            fence.wait();
            fence.reset();
            
            // FPS counter
            frameCount++;
            auto fpsDuration = std::chrono::duration<float>(currentTime - lastFpsTime).count();
            if (fpsDuration >= 0.5f) {
                float fps = frameCount / fpsDuration;
                std::cout << "\rFPS: " << static_cast<int>(fps) 
                         << " | Modo: " << modes[debugMode]
                         << " | 🔺 SIN triángulos - Solo SDF matemáticas"
                         << std::flush;
                frameCount = 0;
                lastFpsTime = currentTime;
            }
        }
        
        vkDeviceWaitIdle(ctx.device());
        
        vkDestroyPipeline(ctx.device(), computePipeline, nullptr);
        vkDestroyPipelineLayout(ctx.device(), pipelineLayout, nullptr);
        vkDestroyDescriptorPool(ctx.device(), descriptorPool, nullptr);
        vkDestroyDescriptorSetLayout(ctx.device(), descriptorLayout, nullptr);
        vkDestroyImageView(ctx.device(), outputView, nullptr);
        vkDestroyImage(ctx.device(), outputImage, nullptr);
        
        reactor::Window::terminate();
        
        std::cout << std::endl << "[✓] Killer Triangle finalizado" << std::endl;
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "ERROR: " << e.what() << std::endl;
        return 1;
    }
}
