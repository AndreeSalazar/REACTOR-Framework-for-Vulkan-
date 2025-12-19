#include "reactor/descriptor_manager.hpp"
#include <stdexcept>

namespace reactor {

DescriptorManager::DescriptorManager(VkDevice device)
    : device_(device) {
}

DescriptorManager::~DescriptorManager() {
    for (auto pool : pools_) {
        vkDestroyDescriptorPool(device_, pool, nullptr);
    }
    for (auto layout : layouts_) {
        vkDestroyDescriptorSetLayout(device_, layout, nullptr);
    }
}

VkDescriptorSetLayout DescriptorManager::createLayout(
    const std::vector<VkDescriptorSetLayoutBinding>& bindings
) {
    VkDescriptorSetLayoutCreateInfo layoutInfo{};
    layoutInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
    layoutInfo.bindingCount = static_cast<uint32_t>(bindings.size());
    layoutInfo.pBindings = bindings.data();

    VkDescriptorSetLayout layout;
    if (vkCreateDescriptorSetLayout(device_, &layoutInfo, nullptr, &layout) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create descriptor set layout");
    }

    layouts_.push_back(layout);
    return layout;
}

VkDescriptorPool DescriptorManager::createPool(
    const std::vector<VkDescriptorPoolSize>& poolSizes,
    uint32_t maxSets
) {
    VkDescriptorPoolCreateInfo poolInfo{};
    poolInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
    poolInfo.poolSizeCount = static_cast<uint32_t>(poolSizes.size());
    poolInfo.pPoolSizes = poolSizes.data();
    poolInfo.maxSets = maxSets;

    VkDescriptorPool pool;
    if (vkCreateDescriptorPool(device_, &poolInfo, nullptr, &pool) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create descriptor pool");
    }

    pools_.push_back(pool);
    return pool;
}

std::vector<VkDescriptorSet> DescriptorManager::allocateSets(
    VkDescriptorPool pool,
    const std::vector<VkDescriptorSetLayout>& layouts
) {
    VkDescriptorSetAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
    allocInfo.descriptorPool = pool;
    allocInfo.descriptorSetCount = static_cast<uint32_t>(layouts.size());
    allocInfo.pSetLayouts = layouts.data();

    std::vector<VkDescriptorSet> sets(layouts.size());
    if (vkAllocateDescriptorSets(device_, &allocInfo, sets.data()) != VK_SUCCESS) {
        throw std::runtime_error("Failed to allocate descriptor sets");
    }

    return sets;
}

void DescriptorManager::updateImageDescriptor(
    VkDescriptorSet set,
    uint32_t binding,
    VkDescriptorType type,
    VkImageView imageView,
    VkImageLayout layout,
    VkSampler sampler
) {
    VkDescriptorImageInfo imageInfo{};
    imageInfo.imageView = imageView;
    imageInfo.imageLayout = layout;
    imageInfo.sampler = sampler;

    VkWriteDescriptorSet write{};
    write.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
    write.dstSet = set;
    write.dstBinding = binding;
    write.dstArrayElement = 0;
    write.descriptorCount = 1;
    write.descriptorType = type;
    write.pImageInfo = &imageInfo;

    vkUpdateDescriptorSets(device_, 1, &write, 0, nullptr);
}

void DescriptorManager::updateBufferDescriptor(
    VkDescriptorSet set,
    uint32_t binding,
    VkDescriptorType type,
    VkBuffer buffer,
    VkDeviceSize offset,
    VkDeviceSize range
) {
    VkDescriptorBufferInfo bufferInfo{};
    bufferInfo.buffer = buffer;
    bufferInfo.offset = offset;
    bufferInfo.range = range;

    VkWriteDescriptorSet write{};
    write.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
    write.dstSet = set;
    write.dstBinding = binding;
    write.dstArrayElement = 0;
    write.descriptorCount = 1;
    write.descriptorType = type;
    write.pBufferInfo = &bufferInfo;

    vkUpdateDescriptorSets(device_, 1, &write, 0, nullptr);
}

} // namespace reactor
