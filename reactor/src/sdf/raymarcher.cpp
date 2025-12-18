#include "reactor/sdf/raymarcher.hpp"
#include <stdexcept>

namespace reactor {
namespace sdf {

RayMarcher::RayMarcher(VkDevice device, VkRenderPass renderPass, const Config& config)
    : device(device), renderPass(renderPass), config(config) {
    
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
    CommandBuffer& commandBuffer,
    const SDFScene& scene,
    const glm::mat4& view,
    const glm::mat4& proj
) {
    // Update uniforms
    updateUniforms(scene, view, proj);
    
    // Bind pipeline
    vkCmdBindPipeline(commandBuffer.handle(), VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline->handle());
    
    // Bind descriptor sets
    vkCmdBindDescriptorSets(
        commandBuffer.handle(),
        VK_PIPELINE_BIND_POINT_GRAPHICS,
        pipeline->layout(),
        0, 1, &descriptorSet,
        0, nullptr
    );
    
    // Draw fullscreen triangle
    vkCmdDraw(commandBuffer.handle(), 3, 1, 0, 0);
}

void RayMarcher::updateConfig(const Config& newConfig) {
    config = newConfig;
}

void RayMarcher::createPipeline() {
    // TODO: Implementar creación de pipeline
    // Por ahora, placeholder
}

void RayMarcher::createDescriptorSets() {
    // TODO: Implementar descriptor sets
    // Por ahora, placeholder
}

void RayMarcher::updateUniforms(const SDFScene& scene, const glm::mat4& view, const glm::mat4& proj) {
    // TODO: Implementar actualización de uniforms
    // Por ahora, placeholder
}

// Builder implementation
RayMarcher::Builder::Builder(VkDevice device, VkRenderPass renderPass)
    : dev(device), pass(renderPass) {}

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
    return RayMarcher(dev, pass, config);
}

RayMarcher::Builder RayMarcher::create(VkDevice device, VkRenderPass renderPass) {
    return Builder(device, renderPass);
}

} // namespace sdf
} // namespace reactor
