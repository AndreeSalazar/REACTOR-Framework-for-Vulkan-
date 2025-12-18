#include "reactor/isr/temporal.hpp"
#include <stdexcept>

namespace reactor {
namespace isr {

TemporalCoherence::TemporalCoherence(VkDevice dev, const Config& cfg)
    : device(dev), config(cfg) {
    createDescriptorSets();
    createComputePipeline();
}

TemporalCoherence::~TemporalCoherence() {
    if (currentFrameView != VK_NULL_HANDLE) {
        vkDestroyImageView(device, currentFrameView, nullptr);
    }
    if (currentFrameImage != VK_NULL_HANDLE) {
        vkDestroyImage(device, currentFrameImage, nullptr);
    }
    if (currentFrameMemory != VK_NULL_HANDLE) {
        vkFreeMemory(device, currentFrameMemory, nullptr);
    }
    if (previousFrameView != VK_NULL_HANDLE) {
        vkDestroyImageView(device, previousFrameView, nullptr);
    }
    if (previousFrameImage != VK_NULL_HANDLE) {
        vkDestroyImage(device, previousFrameImage, nullptr);
    }
    if (previousFrameMemory != VK_NULL_HANDLE) {
        vkFreeMemory(device, previousFrameMemory, nullptr);
    }
    if (descriptorSet != VK_NULL_HANDLE) {
        vkFreeDescriptorSets(device, descriptorPool, 1, &descriptorSet);
    }
    if (descriptorPool != VK_NULL_HANDLE) {
        vkDestroyDescriptorPool(device, descriptorPool, nullptr);
    }
    if (descriptorLayout != VK_NULL_HANDLE) {
        vkDestroyDescriptorSetLayout(device, descriptorLayout, nullptr);
    }
    if (computePipeline != VK_NULL_HANDLE) {
        vkDestroyPipeline(device, computePipeline, nullptr);
    }
    if (pipelineLayout != VK_NULL_HANDLE) {
        vkDestroyPipelineLayout(device, pipelineLayout, nullptr);
    }
}

void TemporalCoherence::createDescriptorSets() {
    // Descriptor set layout
    VkDescriptorSetLayoutBinding bindings[3] = {};
    
    // Binding 0: Current frame shading rate (input)
    bindings[0].binding = 0;
    bindings[0].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
    bindings[0].descriptorCount = 1;
    bindings[0].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
    
    // Binding 1: Previous frame shading rate (input)
    bindings[1].binding = 1;
    bindings[1].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
    bindings[1].descriptorCount = 1;
    bindings[1].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
    
    // Binding 2: Blended output
    bindings[2].binding = 2;
    bindings[2].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
    bindings[2].descriptorCount = 1;
    bindings[2].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
    
    VkDescriptorSetLayoutCreateInfo layoutInfo{};
    layoutInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
    layoutInfo.bindingCount = 3;
    layoutInfo.pBindings = bindings;
    
    if (vkCreateDescriptorSetLayout(device, &layoutInfo, nullptr, &descriptorLayout) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create descriptor set layout for temporal coherence");
    }
    
    // Descriptor pool
    VkDescriptorPoolSize poolSize{};
    poolSize.type = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
    poolSize.descriptorCount = 3;
    
    VkDescriptorPoolCreateInfo poolInfo{};
    poolInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
    poolInfo.poolSizeCount = 1;
    poolInfo.pPoolSizes = &poolSize;
    poolInfo.maxSets = 1;
    
    if (vkCreateDescriptorPool(device, &poolInfo, nullptr, &descriptorPool) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create descriptor pool for temporal coherence");
    }
    
    // Allocate descriptor set
    VkDescriptorSetAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
    allocInfo.descriptorPool = descriptorPool;
    allocInfo.descriptorSetCount = 1;
    allocInfo.pSetLayouts = &descriptorLayout;
    
    if (vkAllocateDescriptorSets(device, &allocInfo, &descriptorSet) != VK_SUCCESS) {
        throw std::runtime_error("Failed to allocate descriptor set for temporal coherence");
    }
}

void TemporalCoherence::createComputePipeline() {
    // Push constants for config
    VkPushConstantRange pushConstant{};
    pushConstant.stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
    pushConstant.offset = 0;
    pushConstant.size = sizeof(Config);
    
    // Pipeline layout
    VkPipelineLayoutCreateInfo layoutInfo{};
    layoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
    layoutInfo.setLayoutCount = 1;
    layoutInfo.pSetLayouts = &descriptorLayout;
    layoutInfo.pushConstantRangeCount = 1;
    layoutInfo.pPushConstantRanges = &pushConstant;
    
    if (vkCreatePipelineLayout(device, &layoutInfo, nullptr, &pipelineLayout) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create pipeline layout for temporal coherence");
    }
    
    // TODO: Load compute shader and create pipeline
}

void TemporalCoherence::createHistoryBuffers(uint32_t width, uint32_t height) {
    // Create history images (R32_SFLOAT for importance values)
    VkImageCreateInfo imageInfo{};
    imageInfo.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
    imageInfo.imageType = VK_IMAGE_TYPE_2D;
    imageInfo.format = VK_FORMAT_R32_SFLOAT;
    imageInfo.extent = {width, height, 1};
    imageInfo.mipLevels = 1;
    imageInfo.arrayLayers = 1;
    imageInfo.samples = VK_SAMPLE_COUNT_1_BIT;
    imageInfo.tiling = VK_IMAGE_TILING_OPTIMAL;
    imageInfo.usage = VK_IMAGE_USAGE_STORAGE_BIT | VK_IMAGE_USAGE_SAMPLED_BIT;
    imageInfo.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    imageInfo.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    
    // Previous importance
    if (vkCreateImage(device, &imageInfo, nullptr, &previousImportance) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create previous importance image");
    }
    
    VkMemoryRequirements memReqs;
    vkGetImageMemoryRequirements(device, previousImportance, &memReqs);
    
    VkMemoryAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    allocInfo.allocationSize = memReqs.size;
    allocInfo.memoryTypeIndex = 0;
    
    if (vkAllocateMemory(device, &allocInfo, nullptr, &previousMemory) != VK_SUCCESS) {
        throw std::runtime_error("Failed to allocate previous importance memory");
    }
    
    vkBindImageMemory(device, previousImportance, previousMemory, 0);
    
    // Output importance
    if (vkCreateImage(device, &imageInfo, nullptr, &outputImportance) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create output importance image");
    }
    
    vkGetImageMemoryRequirements(device, outputImportance, &memReqs);
    
    if (vkAllocateMemory(device, &allocInfo, nullptr, &outputMemory) != VK_SUCCESS) {
        throw std::runtime_error("Failed to allocate output importance memory");
    }
    
    vkBindImageMemory(device, outputImportance, outputMemory, 0);
    
    // Create image views
    VkImageViewCreateInfo viewInfo{};
    viewInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    viewInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
    viewInfo.format = VK_FORMAT_R32_SFLOAT;
    viewInfo.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    viewInfo.subresourceRange.baseMipLevel = 0;
    viewInfo.subresourceRange.levelCount = 1;
    viewInfo.subresourceRange.baseArrayLayer = 0;
    viewInfo.subresourceRange.layerCount = 1;
    
    viewInfo.image = previousImportance;
    if (vkCreateImageView(device, &viewInfo, nullptr, &previousView) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create previous importance view");
    }
    
    viewInfo.image = outputImportance;
    if (vkCreateImageView(device, &viewInfo, nullptr, &outputView) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create output importance view");
    }
}

VkImage TemporalCoherence::applyTemporalCoherence(VkImage currentImportance, VkImage motionVectors) {
    // TODO: Implement compute dispatch
    // 1. Update descriptor set with current and previous importance
    // 2. Push constants with blend factor
    // 3. Dispatch compute shader
    // 4. Swap current and previous
    // 5. Return blended result
    
    return outputImportance;
}

void TemporalCoherence::updateConfig(const Config& newConfig) {
    config = newConfig;
}

void TemporalCoherence::resetHistory() {
    // Clear previous frame data
    historyValid = false;
}

void TemporalCoherence::updateHistory(VkImage current) {
    // Copy current to previous
    // TODO: Implement copy operation
}

} // namespace isr
} // namespace reactor
