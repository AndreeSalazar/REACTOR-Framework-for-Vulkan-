#include "reactor/descriptor.hpp"
#include <stdexcept>

namespace reactor {

DescriptorSetLayout::DescriptorSetLayout(VkDevice device, const std::vector<DescriptorBinding>& bindings)
    : device(device) {
    
    std::vector<VkDescriptorSetLayoutBinding> layoutBindings;
    for (const auto& binding : bindings) {
        VkDescriptorSetLayoutBinding layoutBinding{};
        layoutBinding.binding = binding.binding;
        layoutBinding.descriptorType = static_cast<VkDescriptorType>(binding.type);
        layoutBinding.descriptorCount = binding.count;
        layoutBinding.stageFlags = binding.stageFlags;
        layoutBinding.pImmutableSamplers = nullptr;
        layoutBindings.push_back(layoutBinding);
    }
    
    VkDescriptorSetLayoutCreateInfo layoutInfo{};
    layoutInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
    layoutInfo.bindingCount = static_cast<uint32_t>(layoutBindings.size());
    layoutInfo.pBindings = layoutBindings.data();
    
    if (vkCreateDescriptorSetLayout(device, &layoutInfo, nullptr, &layout) != VK_SUCCESS) {
        throw std::runtime_error("failed to create descriptor set layout");
    }
}

DescriptorSetLayout::~DescriptorSetLayout() {
    if (layout != VK_NULL_HANDLE) {
        vkDestroyDescriptorSetLayout(device, layout, nullptr);
    }
}

DescriptorSetLayout::DescriptorSetLayout(DescriptorSetLayout&& other) noexcept
    : device(other.device), layout(other.layout) {
    other.layout = VK_NULL_HANDLE;
}

DescriptorSetLayout& DescriptorSetLayout::operator=(DescriptorSetLayout&& other) noexcept {
    if (this != &other) {
        if (layout != VK_NULL_HANDLE) {
            vkDestroyDescriptorSetLayout(device, layout, nullptr);
        }
        device = other.device;
        layout = other.layout;
        other.layout = VK_NULL_HANDLE;
    }
    return *this;
}

DescriptorSetLayout::Builder::Builder(VkDevice device) : dev(device) {}

DescriptorSetLayout::Builder& DescriptorSetLayout::Builder::binding(
    uint32_t binding, DescriptorType type, VkShaderStageFlags stages, uint32_t count) {
    bindings.push_back({binding, type, count, stages});
    return *this;
}

DescriptorSetLayout DescriptorSetLayout::Builder::build() {
    return DescriptorSetLayout(dev, bindings);
}

DescriptorSetLayout::Builder DescriptorSetLayout::create(VkDevice device) {
    return Builder(device);
}

DescriptorPool::DescriptorPool(VkDevice device, uint32_t maxSets, 
                               const std::vector<VkDescriptorPoolSize>& poolSizes)
    : dev(device) {
    
    VkDescriptorPoolCreateInfo poolInfo{};
    poolInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
    poolInfo.poolSizeCount = static_cast<uint32_t>(poolSizes.size());
    poolInfo.pPoolSizes = poolSizes.data();
    poolInfo.maxSets = maxSets;
    poolInfo.flags = VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT;
    
    if (vkCreateDescriptorPool(dev, &poolInfo, nullptr, &pool) != VK_SUCCESS) {
        throw std::runtime_error("failed to create descriptor pool");
    }
}

DescriptorPool::~DescriptorPool() {
    if (pool != VK_NULL_HANDLE) {
        vkDestroyDescriptorPool(dev, pool, nullptr);
    }
}

DescriptorPool::DescriptorPool(DescriptorPool&& other) noexcept
    : dev(other.dev), pool(other.pool) {
    other.pool = VK_NULL_HANDLE;
}

DescriptorPool& DescriptorPool::operator=(DescriptorPool&& other) noexcept {
    if (this != &other) {
        if (pool != VK_NULL_HANDLE) {
            vkDestroyDescriptorPool(dev, pool, nullptr);
        }
        dev = other.dev;
        pool = other.pool;
        other.pool = VK_NULL_HANDLE;
    }
    return *this;
}

void DescriptorPool::reset() {
    vkResetDescriptorPool(dev, pool, 0);
}

DescriptorPool::Builder::Builder(VkDevice device) : dev(device) {}

DescriptorPool::Builder& DescriptorPool::Builder::maxSets(uint32_t count) {
    maxSetCount = count;
    return *this;
}

DescriptorPool::Builder& DescriptorPool::Builder::poolSize(DescriptorType type, uint32_t count) {
    VkDescriptorPoolSize size{};
    size.type = static_cast<VkDescriptorType>(type);
    size.descriptorCount = count;
    sizes.push_back(size);
    return *this;
}

DescriptorPool DescriptorPool::Builder::build() {
    if (sizes.empty()) {
        throw std::runtime_error("at least one pool size is required");
    }
    return DescriptorPool(dev, maxSetCount, sizes);
}

DescriptorPool::Builder DescriptorPool::create(VkDevice device) {
    return Builder(device);
}

DescriptorSet::DescriptorSet(std::shared_ptr<DescriptorPool> pool, VkDescriptorSetLayout layout)
    : pool(pool) {
    
    VkDescriptorSetAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
    allocInfo.descriptorPool = pool->handle();
    allocInfo.descriptorSetCount = 1;
    allocInfo.pSetLayouts = &layout;
    
    if (vkAllocateDescriptorSets(pool->device(), &allocInfo, &set) != VK_SUCCESS) {
        throw std::runtime_error("failed to allocate descriptor set");
    }
}

DescriptorSet::~DescriptorSet() {
    if (set != VK_NULL_HANDLE) {
        vkFreeDescriptorSets(pool->device(), pool->handle(), 1, &set);
    }
}

DescriptorSet::DescriptorSet(DescriptorSet&& other) noexcept
    : pool(std::move(other.pool)), set(other.set) {
    other.set = VK_NULL_HANDLE;
}

DescriptorSet& DescriptorSet::operator=(DescriptorSet&& other) noexcept {
    if (this != &other) {
        if (set != VK_NULL_HANDLE) {
            vkFreeDescriptorSets(pool->device(), pool->handle(), 1, &set);
        }
        pool = std::move(other.pool);
        set = other.set;
        other.set = VK_NULL_HANDLE;
    }
    return *this;
}

void DescriptorSet::updateBuffer(uint32_t binding, VkBuffer buffer, 
                                VkDeviceSize offset, VkDeviceSize range) {
    VkDescriptorBufferInfo bufferInfo{};
    bufferInfo.buffer = buffer;
    bufferInfo.offset = offset;
    bufferInfo.range = range;
    
    VkWriteDescriptorSet descriptorWrite{};
    descriptorWrite.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
    descriptorWrite.dstSet = set;
    descriptorWrite.dstBinding = binding;
    descriptorWrite.dstArrayElement = 0;
    descriptorWrite.descriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER;
    descriptorWrite.descriptorCount = 1;
    descriptorWrite.pBufferInfo = &bufferInfo;
    
    vkUpdateDescriptorSets(pool->device(), 1, &descriptorWrite, 0, nullptr);
}

void DescriptorSet::updateImage(uint32_t binding, VkImageView imageView, 
                               VkSampler sampler, VkImageLayout layout) {
    VkDescriptorImageInfo imageInfo{};
    imageInfo.imageLayout = layout;
    imageInfo.imageView = imageView;
    imageInfo.sampler = sampler;
    
    VkWriteDescriptorSet descriptorWrite{};
    descriptorWrite.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
    descriptorWrite.dstSet = set;
    descriptorWrite.dstBinding = binding;
    descriptorWrite.dstArrayElement = 0;
    descriptorWrite.descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
    descriptorWrite.descriptorCount = 1;
    descriptorWrite.pImageInfo = &imageInfo;
    
    vkUpdateDescriptorSets(pool->device(), 1, &descriptorWrite, 0, nullptr);
}

}
