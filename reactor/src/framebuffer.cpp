#include "reactor/framebuffer.hpp"
#include <stdexcept>

namespace reactor {

Framebuffer::Framebuffer(VkDevice device,
                         VkRenderPass renderPass,
                         const std::vector<VkImageView>& attachments,
                         uint32_t width,
                         uint32_t height,
                         uint32_t layers)
    : device_(device)
    , framebuffer_(VK_NULL_HANDLE)
    , width_(width)
    , height_(height)
    , layers_(layers) {
    
    VkFramebufferCreateInfo framebufferInfo{};
    framebufferInfo.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
    framebufferInfo.renderPass = renderPass;
    framebufferInfo.attachmentCount = static_cast<uint32_t>(attachments.size());
    framebufferInfo.pAttachments = attachments.data();
    framebufferInfo.width = width;
    framebufferInfo.height = height;
    framebufferInfo.layers = layers;

    if (vkCreateFramebuffer(device_, &framebufferInfo, nullptr, &framebuffer_) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create framebuffer");
    }
}

Framebuffer::~Framebuffer() {
    if (framebuffer_ != VK_NULL_HANDLE) {
        vkDestroyFramebuffer(device_, framebuffer_, nullptr);
    }
}

Framebuffer::Framebuffer(Framebuffer&& other) noexcept
    : device_(other.device_)
    , framebuffer_(other.framebuffer_)
    , width_(other.width_)
    , height_(other.height_)
    , layers_(other.layers_) {
    other.framebuffer_ = VK_NULL_HANDLE;
}

Framebuffer& Framebuffer::operator=(Framebuffer&& other) noexcept {
    if (this != &other) {
        if (framebuffer_ != VK_NULL_HANDLE) {
            vkDestroyFramebuffer(device_, framebuffer_, nullptr);
        }
        device_ = other.device_;
        framebuffer_ = other.framebuffer_;
        width_ = other.width_;
        height_ = other.height_;
        layers_ = other.layers_;
        other.framebuffer_ = VK_NULL_HANDLE;
    }
    return *this;
}

} // namespace reactor
