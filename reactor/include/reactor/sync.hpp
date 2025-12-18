#pragma once
#include <vulkan/vulkan.h>
#include <memory>

namespace reactor {

class Fence {
public:
    Fence(VkDevice device, bool signaled = false);
    ~Fence();

    Fence(const Fence&) = delete;
    Fence& operator=(const Fence&) = delete;
    Fence(Fence&& other) noexcept;
    Fence& operator=(Fence&& other) noexcept;

    VkFence handle() const { return fence; }
    
    void wait(uint64_t timeout = UINT64_MAX);
    void reset();
    bool isSignaled();

private:
    VkDevice device;
    VkFence fence{VK_NULL_HANDLE};
};

class Semaphore {
public:
    Semaphore(VkDevice device);
    ~Semaphore();

    Semaphore(const Semaphore&) = delete;
    Semaphore& operator=(const Semaphore&) = delete;
    Semaphore(Semaphore&& other) noexcept;
    Semaphore& operator=(Semaphore&& other) noexcept;

    VkSemaphore handle() const { return semaphore; }

private:
    VkDevice device;
    VkSemaphore semaphore{VK_NULL_HANDLE};
};

struct ImageBarrier {
    VkImage image;
    VkImageLayout oldLayout;
    VkImageLayout newLayout;
    VkAccessFlags srcAccess;
    VkAccessFlags dstAccess;
    VkPipelineStageFlags srcStage;
    VkPipelineStageFlags dstStage;
    VkImageAspectFlags aspectMask{VK_IMAGE_ASPECT_COLOR_BIT};
};

struct BufferBarrier {
    VkBuffer buffer;
    VkAccessFlags srcAccess;
    VkAccessFlags dstAccess;
    VkPipelineStageFlags srcStage;
    VkPipelineStageFlags dstStage;
    VkDeviceSize offset{0};
    VkDeviceSize size{VK_WHOLE_SIZE};
};

}
