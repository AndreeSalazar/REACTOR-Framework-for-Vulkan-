#include "reactor/sync.hpp"
#include <stdexcept>

namespace reactor {

Fence::Fence(VkDevice device, bool signaled) : device(device) {
    VkFenceCreateInfo fenceInfo{};
    fenceInfo.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
    fenceInfo.flags = signaled ? VK_FENCE_CREATE_SIGNALED_BIT : 0;
    
    if (vkCreateFence(device, &fenceInfo, nullptr, &fence) != VK_SUCCESS) {
        throw std::runtime_error("failed to create fence");
    }
}

Fence::~Fence() {
    if (fence != VK_NULL_HANDLE) {
        vkDestroyFence(device, fence, nullptr);
    }
}

Fence::Fence(Fence&& other) noexcept
    : device(other.device), fence(other.fence) {
    other.fence = VK_NULL_HANDLE;
}

Fence& Fence::operator=(Fence&& other) noexcept {
    if (this != &other) {
        if (fence != VK_NULL_HANDLE) {
            vkDestroyFence(device, fence, nullptr);
        }
        device = other.device;
        fence = other.fence;
        other.fence = VK_NULL_HANDLE;
    }
    return *this;
}

void Fence::wait(uint64_t timeout) {
    vkWaitForFences(device, 1, &fence, VK_TRUE, timeout);
}

void Fence::reset() {
    vkResetFences(device, 1, &fence);
}

bool Fence::isSignaled() {
    return vkGetFenceStatus(device, fence) == VK_SUCCESS;
}

Semaphore::Semaphore(VkDevice device) : device(device) {
    VkSemaphoreCreateInfo semaphoreInfo{};
    semaphoreInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;
    
    if (vkCreateSemaphore(device, &semaphoreInfo, nullptr, &semaphore) != VK_SUCCESS) {
        throw std::runtime_error("failed to create semaphore");
    }
}

Semaphore::~Semaphore() {
    if (semaphore != VK_NULL_HANDLE) {
        vkDestroySemaphore(device, semaphore, nullptr);
    }
}

Semaphore::Semaphore(Semaphore&& other) noexcept
    : device(other.device), semaphore(other.semaphore) {
    other.semaphore = VK_NULL_HANDLE;
}

Semaphore& Semaphore::operator=(Semaphore&& other) noexcept {
    if (this != &other) {
        if (semaphore != VK_NULL_HANDLE) {
            vkDestroySemaphore(device, semaphore, nullptr);
        }
        device = other.device;
        semaphore = other.semaphore;
        other.semaphore = VK_NULL_HANDLE;
    }
    return *this;
}

}
