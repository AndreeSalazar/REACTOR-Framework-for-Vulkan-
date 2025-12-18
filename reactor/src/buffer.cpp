#include "reactor/buffer.hpp"
#include <stdexcept>
#include <cstring>

namespace reactor {

Buffer::Buffer(std::shared_ptr<MemoryAllocator> allocator, VkDeviceSize size,
               BufferUsage usage, MemoryType memType)
    : allocator(allocator), bufferSize(size) {
    
    VkBufferCreateInfo bufferInfo{};
    bufferInfo.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
    bufferInfo.size = size;
    bufferInfo.usage = toVkFlags(usage);
    bufferInfo.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    
    if (vkCreateBuffer(allocator->device(), &bufferInfo, nullptr, &buffer) != VK_SUCCESS) {
        throw std::runtime_error("failed to create buffer");
    }
    
    VkMemoryRequirements memReqs;
    vkGetBufferMemoryRequirements(allocator->device(), buffer, &memReqs);
    
    memory = allocator->allocate(memReqs, memType);
    
    if (vkBindBufferMemory(allocator->device(), buffer, memory.memory, memory.offset) != VK_SUCCESS) {
        throw std::runtime_error("failed to bind buffer memory");
    }
}

Buffer::~Buffer() {
    if (mappedData) {
        unmap();
    }
    if (buffer != VK_NULL_HANDLE) {
        vkDestroyBuffer(allocator->device(), buffer, nullptr);
    }
    if (memory.memory != VK_NULL_HANDLE) {
        allocator->free(memory);
    }
}

Buffer::Buffer(Buffer&& other) noexcept
    : allocator(std::move(other.allocator))
    , buffer(other.buffer)
    , memory(other.memory)
    , bufferSize(other.bufferSize)
    , mappedData(other.mappedData) {
    other.buffer = VK_NULL_HANDLE;
    other.memory = {};
    other.mappedData = nullptr;
}

Buffer& Buffer::operator=(Buffer&& other) noexcept {
    if (this != &other) {
        if (buffer != VK_NULL_HANDLE) {
            vkDestroyBuffer(allocator->device(), buffer, nullptr);
        }
        if (memory.memory != VK_NULL_HANDLE) {
            allocator->free(memory);
        }
        
        allocator = std::move(other.allocator);
        buffer = other.buffer;
        memory = other.memory;
        bufferSize = other.bufferSize;
        mappedData = other.mappedData;
        
        other.buffer = VK_NULL_HANDLE;
        other.memory = {};
        other.mappedData = nullptr;
    }
    return *this;
}

void Buffer::upload(const void* data, VkDeviceSize size, VkDeviceSize offset) {
    if (offset + size > bufferSize) {
        throw std::runtime_error("upload size exceeds buffer size");
    }
    
    void* mapped = allocator->map(memory);
    std::memcpy(static_cast<char*>(mapped) + offset, data, size);
    allocator->unmap(memory);
}

void* Buffer::map() {
    if (!mappedData) {
        mappedData = allocator->map(memory);
    }
    return mappedData;
}

void Buffer::unmap() {
    if (mappedData) {
        allocator->unmap(memory);
        mappedData = nullptr;
    }
}

Buffer::Builder::Builder(std::shared_ptr<MemoryAllocator> allocator)
    : alloc(allocator) {}

Buffer::Builder& Buffer::Builder::size(VkDeviceSize size) {
    bufSize = size;
    return *this;
}

Buffer::Builder& Buffer::Builder::usage(BufferUsage usage) {
    bufUsage = usage;
    return *this;
}

Buffer::Builder& Buffer::Builder::memoryType(MemoryType type) {
    memType = type;
    return *this;
}

Buffer Buffer::Builder::build() {
    if (bufSize == 0) {
        throw std::runtime_error("buffer size must be greater than 0");
    }
    return Buffer(alloc, bufSize, bufUsage, memType);
}

Buffer::Builder Buffer::create(std::shared_ptr<MemoryAllocator> allocator) {
    return Builder(allocator);
}

}
