#pragma once
#include <vulkan/vulkan.h>
#include <vector>
#include <memory>
#include <mutex>

namespace reactor {

enum class MemoryType {
    DeviceLocal,
    HostVisible,
    HostCoherent,
    HostCached
};

struct MemoryBlock {
    VkDeviceMemory memory{VK_NULL_HANDLE};
    VkDeviceSize size{0};
    VkDeviceSize offset{0};
    uint32_t memoryTypeIndex{0};
    void* mapped{nullptr};
};

class MemoryAllocator {
public:
    MemoryAllocator(VkDevice device, VkPhysicalDevice physicalDevice);
    ~MemoryAllocator();

    MemoryBlock allocate(VkMemoryRequirements requirements, MemoryType type);
    void free(const MemoryBlock& block);
    
    void* map(const MemoryBlock& block);
    void unmap(const MemoryBlock& block);
    
    VkDevice device() const { return dev; }

private:
    VkDevice dev;
    VkPhysicalDevice physDev;
    VkPhysicalDeviceMemoryProperties memProps;
    std::mutex mutex;
    
    uint32_t findMemoryType(uint32_t typeFilter, VkMemoryPropertyFlags properties);
    VkMemoryPropertyFlags getMemoryProperties(MemoryType type);
};

}
