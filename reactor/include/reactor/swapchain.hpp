#pragma once
#include <vulkan/vulkan.h>
#include <vector>
#include <memory>

namespace reactor {

struct SwapchainSupportDetails {
    VkSurfaceCapabilitiesKHR capabilities;
    std::vector<VkSurfaceFormatKHR> formats;
    std::vector<VkPresentModeKHR> presentModes;
};

class Swapchain {
public:
    Swapchain(VkDevice device, VkPhysicalDevice physicalDevice, VkSurfaceKHR surface,
              uint32_t width, uint32_t height, bool vsync = true, VkSwapchainKHR oldSwapchain = VK_NULL_HANDLE);
    ~Swapchain();

    Swapchain(const Swapchain&) = delete;
    Swapchain& operator=(const Swapchain&) = delete;
    Swapchain(Swapchain&& other) noexcept;
    Swapchain& operator=(Swapchain&& other) noexcept;

    VkSwapchainKHR handle() const { return swapchain; }
    VkFormat imageFormat() const { return swapchainImageFormat; }
    VkExtent2D extent() const { return swapchainExtent; }
    uint32_t imageCount() const { return static_cast<uint32_t>(swapchainImages.size()); }
    
    const std::vector<VkImage>& images() const { return swapchainImages; }
    const std::vector<VkImageView>& imageViews() const { return swapchainImageViews; }
    
    uint32_t acquireNextImage(VkSemaphore semaphore, VkFence fence = VK_NULL_HANDLE);
    void present(VkQueue queue, uint32_t imageIndex, VkSemaphore waitSemaphore);
    
    static SwapchainSupportDetails querySupport(VkPhysicalDevice device, VkSurfaceKHR surface);

private:
    VkDevice device;
    VkSwapchainKHR swapchain{VK_NULL_HANDLE};
    std::vector<VkImage> swapchainImages;
    std::vector<VkImageView> swapchainImageViews;
    VkFormat swapchainImageFormat;
    VkExtent2D swapchainExtent;
    
    void createImageViews();
    VkSurfaceFormatKHR chooseSwapSurfaceFormat(const std::vector<VkSurfaceFormatKHR>& formats);
    VkPresentModeKHR chooseSwapPresentMode(const std::vector<VkPresentModeKHR>& modes, bool vsync);
    VkExtent2D chooseSwapExtent(const VkSurfaceCapabilitiesKHR& capabilities, 
                                uint32_t width, uint32_t height);
};

}
