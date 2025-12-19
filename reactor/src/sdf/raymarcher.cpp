#include "reactor/sdf/raymarcher.hpp"
#include <stdexcept>

namespace reactor {
namespace sdf {

RayMarcher::RayMarcher(VkDevice device, VkRenderPass renderPass, const Config& config)
    : device(device), renderPass(renderPass), config(config) {
    // Constructor sin physical device - no crea pipeline aún
}

RayMarcher::RayMarcher(VkDevice device, VkRenderPass renderPass, VkPhysicalDevice physicalDevice, const Config& config)
    : device(device), renderPass(renderPass), physicalDevice(physicalDevice), config(config) {
    
    createPipeline();
    createDescriptorSets();
}

RayMarcher::~RayMarcher() {
    if (descriptorPool != VK_NULL_HANDLE) {
        vkDestroyDescriptorPool(device, descriptorPool, nullptr);
    }
    if (descriptorLayout != VK_NULL_HANDLE) {
        vkDestroyDescriptorSetLayout(device, descriptorLayout, nullptr);
    }
}

void RayMarcher::render(
    VkCommandBuffer commandBuffer,
    const SDFScene& scene,
    const glm::mat4& view,
    const glm::mat4& proj
) {
    if (!pipeline) {
        return; // Pipeline no creado
    }
    
    // Bind pipeline
    vkCmdBindPipeline(commandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline->handle());
    
    // Draw fullscreen triangle (sin descriptors por ahora)
    vkCmdDraw(commandBuffer, 3, 1, 0, 0);
}

void RayMarcher::updateConfig(const Config& newConfig) {
    config = newConfig;
}

void RayMarcher::createPipeline() {
    try {
        // Intentar cargar shaders de prueba primero
        Shader vertShader(device, "shaders/test.vert.spv", ShaderStage::Vertex);
        Shader fragShader(device, "shaders/test.frag.spv", ShaderStage::Fragment);
        
        // Crear pipeline sin descriptors
        pipeline = std::make_unique<GraphicsPipeline>(
            GraphicsPipeline::create(device, renderPass)
                .shader(std::make_shared<Shader>(std::move(vertShader)))
                .shader(std::make_shared<Shader>(std::move(fragShader)))
                .topology(Topology::TriangleList)
                .viewport(static_cast<float>(config.width), static_cast<float>(config.height))
                .cullMode(CullMode::None)
                .build()
        );
    } catch (const std::exception& e) {
        // Log error pero no crash
        // El render simplemente no dibujará nada
    }
}

void RayMarcher::createDescriptorSets() {
    // Por ahora, no crear descriptor sets - renderizaremos color sólido
    // TODO: Implementar cuando tengamos allocator disponible
}

void RayMarcher::updateUniforms(const SDFScene* scene, const glm::mat4& view, const glm::mat4& proj) {
    // Por ahora, placeholder - se implementará cuando tengamos allocator
}

// Builder implementation
RayMarcher::Builder::Builder(VkDevice device, VkRenderPass renderPass)
    : dev(device), pass(renderPass) {}

RayMarcher::Builder::Builder(VkDevice device, VkRenderPass renderPass, VkPhysicalDevice physicalDevice)
    : dev(device), pass(renderPass), phys(physicalDevice) {}

RayMarcher::Builder& RayMarcher::Builder::resolution(uint32_t width, uint32_t height) {
    config.width = width;
    config.height = height;
    return *this;
}

RayMarcher::Builder& RayMarcher::Builder::maxSteps(uint32_t steps) {
    config.maxSteps = steps;
    return *this;
}

RayMarcher::Builder& RayMarcher::Builder::antialiasing(bool enable) {
    config.enableAntialiasing = enable;
    return *this;
}

RayMarcher::Builder& RayMarcher::Builder::softShadows(bool enable) {
    config.enableSoftShadows = enable;
    return *this;
}

RayMarcher::Builder& RayMarcher::Builder::ambientOcclusion(bool enable) {
    config.enableAO = enable;
    return *this;
}

RayMarcher RayMarcher::Builder::build() {
    if (phys != VK_NULL_HANDLE) {
        return RayMarcher(dev, pass, phys, config);
    }
    return RayMarcher(dev, pass, config);
}

RayMarcher::Builder RayMarcher::create(VkDevice device, VkRenderPass renderPass) {
    return Builder(device, renderPass);
}

RayMarcher::Builder RayMarcher::create(VkDevice device, VkRenderPass renderPass, VkPhysicalDevice physicalDevice) {
    return Builder(device, renderPass, physicalDevice);
}

} // namespace sdf
} // namespace reactor
