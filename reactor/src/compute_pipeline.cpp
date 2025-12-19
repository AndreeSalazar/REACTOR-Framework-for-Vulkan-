#include "reactor/compute_pipeline.hpp"
#include <stdexcept>

namespace reactor {

// ============================================================================
// ComputePipelineBuilder
// ============================================================================

ComputePipelineBuilder::ComputePipelineBuilder(VkDevice device)
    : device_(device) {
}

ComputePipelineBuilder::~ComputePipelineBuilder() {
    // Cleanup is handled by ComputePipeline RAII
}

ComputePipelineBuilder& ComputePipelineBuilder::shader(const Shader& computeShader) {
    shaderModule_ = computeShader.module();
    return *this;
}

ComputePipelineBuilder& ComputePipelineBuilder::descriptorSetLayout(VkDescriptorSetLayout layout) {
    descriptorLayouts_.push_back(layout);
    return *this;
}

ComputePipelineBuilder& ComputePipelineBuilder::pushConstantRange(VkPushConstantRange range) {
    pushConstants_.push_back(range);
    return *this;
}

VkPipeline ComputePipelineBuilder::build() {
    if (shaderModule_ == VK_NULL_HANDLE) {
        throw std::runtime_error("Compute shader not set");
    }

    // Create pipeline layout
    VkPipelineLayoutCreateInfo layoutInfo{};
    layoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
    layoutInfo.setLayoutCount = static_cast<uint32_t>(descriptorLayouts_.size());
    layoutInfo.pSetLayouts = descriptorLayouts_.empty() ? nullptr : descriptorLayouts_.data();
    layoutInfo.pushConstantRangeCount = static_cast<uint32_t>(pushConstants_.size());
    layoutInfo.pPushConstantRanges = pushConstants_.empty() ? nullptr : pushConstants_.data();

    if (vkCreatePipelineLayout(device_, &layoutInfo, nullptr, &pipelineLayout_) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create compute pipeline layout");
    }

    // Create compute pipeline
    VkComputePipelineCreateInfo pipelineInfo{};
    pipelineInfo.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    pipelineInfo.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    pipelineInfo.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT;
    pipelineInfo.stage.module = shaderModule_;
    pipelineInfo.stage.pName = "main";
    pipelineInfo.layout = pipelineLayout_;

    VkPipeline pipeline;
    if (vkCreateComputePipelines(device_, VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &pipeline) != VK_SUCCESS) {
        vkDestroyPipelineLayout(device_, pipelineLayout_, nullptr);
        throw std::runtime_error("Failed to create compute pipeline");
    }

    return pipeline;
}

// ============================================================================
// ComputePipelineWrapper
// ============================================================================

ComputePipelineWrapper::ComputePipelineWrapper(VkDevice device, VkPipeline pipeline, VkPipelineLayout layout)
    : device_(device), pipeline_(pipeline), layout_(layout) {
}

ComputePipelineWrapper::~ComputePipelineWrapper() {
    if (pipeline_ != VK_NULL_HANDLE) {
        vkDestroyPipeline(device_, pipeline_, nullptr);
    }
    if (layout_ != VK_NULL_HANDLE) {
        vkDestroyPipelineLayout(device_, layout_, nullptr);
    }
}

ComputePipelineWrapper::ComputePipelineWrapper(ComputePipelineWrapper&& other) noexcept
    : device_(other.device_)
    , pipeline_(other.pipeline_)
    , layout_(other.layout_) {
    other.pipeline_ = VK_NULL_HANDLE;
    other.layout_ = VK_NULL_HANDLE;
}

ComputePipelineWrapper& ComputePipelineWrapper::operator=(ComputePipelineWrapper&& other) noexcept {
    if (this != &other) {
        if (pipeline_ != VK_NULL_HANDLE) {
            vkDestroyPipeline(device_, pipeline_, nullptr);
        }
        if (layout_ != VK_NULL_HANDLE) {
            vkDestroyPipelineLayout(device_, layout_, nullptr);
        }

        device_ = other.device_;
        pipeline_ = other.pipeline_;
        layout_ = other.layout_;

        other.pipeline_ = VK_NULL_HANDLE;
        other.layout_ = VK_NULL_HANDLE;
    }
    return *this;
}

} // namespace reactor
