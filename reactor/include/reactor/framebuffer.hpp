#pragma once

#include <vulkan/vulkan.h>
#include <vector>

namespace reactor {

/**
 * @brief Framebuffer wrapper - Vulkan framebuffer management
 * 
 * Abstracción completa de VkFramebuffer para render targets
 */
class Framebuffer {
public:
    Framebuffer(VkDevice device, 
                VkRenderPass renderPass,
                const std::vector<VkImageView>& attachments,
                uint32_t width,
                uint32_t height,
                uint32_t layers = 1);
    ~Framebuffer();

    // No copyable
    Framebuffer(const Framebuffer&) = delete;
    Framebuffer& operator=(const Framebuffer&) = delete;

    // Movable
    Framebuffer(Framebuffer&& other) noexcept;
    Framebuffer& operator=(Framebuffer&& other) noexcept;

    VkFramebuffer handle() const { return framebuffer_; }
    uint32_t width() const { return width_; }
    uint32_t height() const { return height_; }
    uint32_t layers() const { return layers_; }

private:
    VkDevice device_;
    VkFramebuffer framebuffer_;
    uint32_t width_;
    uint32_t height_;
    uint32_t layers_;
};

} // namespace reactor
