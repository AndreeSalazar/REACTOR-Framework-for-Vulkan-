#include "reactor/memory_allocator.hpp"
#include <stdexcept>
#include <cstring>

namespace reactor {

MemoryAllocator::MemoryAllocator(VkDevice device, VkPhysicalDevice physicalDevice)
    : dev(device), physDev(physicalDevice) {
    vkGetPhysicalDeviceMemoryProperties(physDev, &memProps);
}

MemoryAllocator::~MemoryAllocator() {
}

VkMemoryPropertyFlags MemoryAllocator::getMemoryProperties(MemoryType type) {
    switch (type) {
        case MemoryType::DeviceLocal:
            return VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT;
        case MemoryType::HostVisible:
            return VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        case MemoryType::HostCoherent:
            return VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        case MemoryType::HostCached:
            return VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_CACHED_BIT;
        default:
            return 0;
    }
}

uint32_t MemoryAllocator::findMemoryType(uint32_t typeFilter, VkMemoryPropertyFlags properties) {
    for (uint32_t i = 0; i < memProps.memoryTypeCount; i++) {
        if ((typeFilter & (1 << i)) && 
            (memProps.memoryTypes[i].propertyFlags & properties) == properties) {
            return i;
        }
    }
    throw std::runtime_error("failed to find suitable memory type");
}

MemoryBlock MemoryAllocator::allocate(VkMemoryRequirements requirements, MemoryType type) {
    std::lock_guard<std::mutex> lock(mutex);
    
    VkMemoryPropertyFlags props = getMemoryProperties(type);
    uint32_t memTypeIndex = findMemoryType(requirements.memoryTypeBits, props);
    
    VkMemoryAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    allocInfo.allocationSize = requirements.size;
    allocInfo.memoryTypeIndex = memTypeIndex;
    
    MemoryBlock block;
    block.size = requirements.size;
    block.memoryTypeIndex = memTypeIndex;
    
    if (vkAllocateMemory(dev, &allocInfo, nullptr, &block.memory) != VK_SUCCESS) {
        throw std::runtime_error("failed to allocate memory");
    }
    
    return block;
}

void MemoryAllocator::free(const MemoryBlock& block) {
    std::lock_guard<std::mutex> lock(mutex);
    if (block.memory != VK_NULL_HANDLE) {
        vkFreeMemory(dev, block.memory, nullptr);
    }
}

void* MemoryAllocator::map(const MemoryBlock& block) {
    void* data = nullptr;
    if (vkMapMemory(dev, block.memory, block.offset, block.size, 0, &data) != VK_SUCCESS) {
        throw std::runtime_error("failed to map memory");
    }
    return data;
}

void MemoryAllocator::unmap(const MemoryBlock& block) {
    vkUnmapMemory(dev, block.memory);
}

}
