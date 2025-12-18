#pragma once
#include "reactor/memory_allocator.hpp"
#include <vulkan/vulkan.h>
#include <memory>
#include <functional>

namespace reactor {

enum class BufferUsage {
    None = 0,
    Vertex = VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
    Index = VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
    Uniform = VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
    Storage = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT,
    Transfer = VK_BUFFER_USAGE_TRANSFER_SRC_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT,
    TransferSrc = VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
    TransferDst = VK_BUFFER_USAGE_TRANSFER_DST_BIT
};

inline BufferUsage operator|(BufferUsage a, BufferUsage b) {
    return static_cast<BufferUsage>(static_cast<int>(a) | static_cast<int>(b));
}

inline VkBufferUsageFlags toVkFlags(BufferUsage usage) {
    return static_cast<VkBufferUsageFlags>(usage);
}

class Buffer {
public:
    Buffer(std::shared_ptr<MemoryAllocator> allocator, VkDeviceSize size, 
           BufferUsage usage, MemoryType memType);
    ~Buffer();

    Buffer(const Buffer&) = delete;
    Buffer& operator=(const Buffer&) = delete;
    Buffer(Buffer&& other) noexcept;
    Buffer& operator=(Buffer&& other) noexcept;

    VkBuffer handle() const { return buffer; }
    VkDeviceSize size() const { return bufferSize; }
    
    void upload(const void* data, VkDeviceSize size, VkDeviceSize offset = 0);
    void* map();
    void unmap();
    
    template<typename Func>
    void mapScoped(Func&& func) {
        void* data = map();
        func(data);
        unmap();
    }

    class Builder {
    public:
        Builder(std::shared_ptr<MemoryAllocator> allocator);
        Builder& size(VkDeviceSize size);
        Builder& usage(BufferUsage usage);
        Builder& memoryType(MemoryType type);
        Buffer build();
    private:
        std::shared_ptr<MemoryAllocator> alloc;
        VkDeviceSize bufSize{0};
        BufferUsage bufUsage{BufferUsage::None};
        MemoryType memType{MemoryType::DeviceLocal};
    };

    static Builder create(std::shared_ptr<MemoryAllocator> allocator);

private:
    std::shared_ptr<MemoryAllocator> allocator;
    VkBuffer buffer{VK_NULL_HANDLE};
    MemoryBlock memory;
    VkDeviceSize bufferSize{0};
    void* mappedData{nullptr};
};

}
