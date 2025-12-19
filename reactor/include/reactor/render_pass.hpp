#pragma once
#include <vulkan/vulkan.h>
#include <vector>
#include <memory>

namespace reactor {

struct AttachmentDescription {
    VkFormat format;
    VkSampleCountFlagBits samples{VK_SAMPLE_COUNT_1_BIT};
    VkAttachmentLoadOp loadOp{VK_ATTACHMENT_LOAD_OP_CLEAR};
    VkAttachmentStoreOp storeOp{VK_ATTACHMENT_STORE_OP_STORE};
    VkImageLayout initialLayout{VK_IMAGE_LAYOUT_UNDEFINED};
    VkImageLayout finalLayout{VK_IMAGE_LAYOUT_PRESENT_SRC_KHR};
};

class RenderPass {
public:
    RenderPass(VkDevice device, const std::vector<AttachmentDescription>& attachments,
               bool hasDepth = false);
    ~RenderPass();

    RenderPass(const RenderPass&) = delete;
    RenderPass& operator=(const RenderPass&) = delete;
    RenderPass(RenderPass&& other) noexcept;
    RenderPass& operator=(RenderPass&& other) noexcept;

    VkRenderPass handle() const { return renderPass; }

    class Builder {
    public:
        Builder(VkDevice device);
        Builder& colorAttachment(VkFormat format, VkImageLayout finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR);
        Builder& depthAttachment(VkFormat format);
        RenderPass build();
    private:
        VkDevice dev;
        std::vector<AttachmentDescription> attachments;
        bool depth{false};
    };

    static Builder create(VkDevice device);

private:
    VkDevice device;
    VkRenderPass renderPass{VK_NULL_HANDLE};
};

} // namespace reactor
