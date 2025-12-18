#include "reactor/isr/importance.hpp"
#include <iostream>

namespace reactor {
namespace isr {

ImportanceCalculator::ImportanceCalculator(VkDevice dev, const Config& cfg)
    : device(dev), config(cfg) {
    std::cout << "[ISR] ImportanceCalculator creado" << std::endl;
}

ImportanceCalculator::~ImportanceCalculator() {
    if (computePipeline != VK_NULL_HANDLE) {
        vkDestroyPipeline(device, computePipeline, nullptr);
    }
    if (pipelineLayout != VK_NULL_HANDLE) {
        vkDestroyPipelineLayout(device, pipelineLayout, nullptr);
    }
    if (descriptorLayout != VK_NULL_HANDLE) {
        vkDestroyDescriptorSetLayout(device, descriptorLayout, nullptr);
    }
    if (descriptorPool != VK_NULL_HANDLE) {
        vkDestroyDescriptorPool(device, descriptorPool, nullptr);
    }
    if (importanceView != VK_NULL_HANDLE) {
        vkDestroyImageView(device, importanceView, nullptr);
    }
    if (importanceImage != VK_NULL_HANDLE) {
        vkDestroyImage(device, importanceImage, nullptr);
    }
    if (importanceMemory != VK_NULL_HANDLE) {
        vkFreeMemory(device, importanceMemory, nullptr);
    }
}

VkImage ImportanceCalculator::calculateImportance(
    VkImage colorBuffer,
    VkImage normalBuffer,
    VkImage depthBuffer,
    VkImage motionBuffer
) {
    return importanceImage;
}

void ImportanceCalculator::updateConfig(const Config& newConfig) {
    config = newConfig;
}

void ImportanceCalculator::createComputePipeline() {
}

void ImportanceCalculator::createDescriptorSets() {
}

void ImportanceCalculator::createImportanceImage(uint32_t width, uint32_t height) {
}

} // namespace isr
} // namespace reactor
