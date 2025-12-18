#include "reactor/render_pass.hpp"
#include <stdexcept>

namespace reactor {

RenderPass::RenderPass(VkDevice device, const std::vector<AttachmentDescription>& attachments,
                       bool hasDepth)
    : device(device) {
    
    std::vector<VkAttachmentDescription> vkAttachments;
    std::vector<VkAttachmentReference> colorRefs;
    VkAttachmentReference depthRef{};
    
    uint32_t attachmentIndex = 0;
    for (const auto& desc : attachments) {
        VkAttachmentDescription attachment{};
        attachment.format = desc.format;
        attachment.samples = desc.samples;
        attachment.loadOp = desc.loadOp;
        attachment.storeOp = desc.storeOp;
        attachment.stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
        attachment.stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
        attachment.initialLayout = desc.initialLayout;
        attachment.finalLayout = desc.finalLayout;
        vkAttachments.push_back(attachment);
        
        if (desc.format == VK_FORMAT_D32_SFLOAT || desc.format == VK_FORMAT_D24_UNORM_S8_UINT) {
            depthRef.attachment = attachmentIndex;
            depthRef.layout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
        } else {
            VkAttachmentReference colorRef{};
            colorRef.attachment = attachmentIndex;
            colorRef.layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
            colorRefs.push_back(colorRef);
        }
        attachmentIndex++;
    }
    
    VkSubpassDescription subpass{};
    subpass.pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS;
    subpass.colorAttachmentCount = static_cast<uint32_t>(colorRefs.size());
    subpass.pColorAttachments = colorRefs.data();
    if (hasDepth) {
        subpass.pDepthStencilAttachment = &depthRef;
    }
    
    VkSubpassDependency dependency{};
    dependency.srcSubpass = VK_SUBPASS_EXTERNAL;
    dependency.dstSubpass = 0;
    dependency.srcStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    dependency.srcAccessMask = 0;
    dependency.dstStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    dependency.dstAccessMask = VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT;
    
    if (hasDepth) {
        dependency.srcStageMask |= VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT;
        dependency.dstStageMask |= VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT;
        dependency.dstAccessMask |= VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT;
    }
    
    VkRenderPassCreateInfo renderPassInfo{};
    renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO;
    renderPassInfo.attachmentCount = static_cast<uint32_t>(vkAttachments.size());
    renderPassInfo.pAttachments = vkAttachments.data();
    renderPassInfo.subpassCount = 1;
    renderPassInfo.pSubpasses = &subpass;
    renderPassInfo.dependencyCount = 1;
    renderPassInfo.pDependencies = &dependency;
    
    if (vkCreateRenderPass(device, &renderPassInfo, nullptr, &renderPass) != VK_SUCCESS) {
        throw std::runtime_error("failed to create render pass");
    }
}

RenderPass::~RenderPass() {
    if (renderPass != VK_NULL_HANDLE) {
        vkDestroyRenderPass(device, renderPass, nullptr);
    }
}

RenderPass::RenderPass(RenderPass&& other) noexcept
    : device(other.device), renderPass(other.renderPass) {
    other.renderPass = VK_NULL_HANDLE;
}

RenderPass& RenderPass::operator=(RenderPass&& other) noexcept {
    if (this != &other) {
        if (renderPass != VK_NULL_HANDLE) {
            vkDestroyRenderPass(device, renderPass, nullptr);
        }
        device = other.device;
        renderPass = other.renderPass;
        other.renderPass = VK_NULL_HANDLE;
    }
    return *this;
}

RenderPass::Builder::Builder(VkDevice device) : dev(device) {}

RenderPass::Builder& RenderPass::Builder::colorAttachment(VkFormat format, VkImageLayout finalLayout) {
    AttachmentDescription desc{};
    desc.format = format;
    desc.finalLayout = finalLayout;
    attachments.push_back(desc);
    return *this;
}

RenderPass::Builder& RenderPass::Builder::depthAttachment(VkFormat format) {
    AttachmentDescription desc{};
    desc.format = format;
    desc.finalLayout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
    attachments.push_back(desc);
    depth = true;
    return *this;
}

RenderPass RenderPass::Builder::build() {
    if (attachments.empty()) {
        throw std::runtime_error("at least one attachment is required");
    }
    return RenderPass(dev, attachments, depth);
}

RenderPass::Builder RenderPass::create(VkDevice device) {
    return Builder(device);
}

Framebuffer::Framebuffer(VkDevice device, VkRenderPass renderPass,
                         const std::vector<VkImageView>& attachments,
                         uint32_t width, uint32_t height)
    : device(device), fbWidth(width), fbHeight(height) {
    
    VkFramebufferCreateInfo framebufferInfo{};
    framebufferInfo.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
    framebufferInfo.renderPass = renderPass;
    framebufferInfo.attachmentCount = static_cast<uint32_t>(attachments.size());
    framebufferInfo.pAttachments = attachments.data();
    framebufferInfo.width = width;
    framebufferInfo.height = height;
    framebufferInfo.layers = 1;
    
    if (vkCreateFramebuffer(device, &framebufferInfo, nullptr, &framebuffer) != VK_SUCCESS) {
        throw std::runtime_error("failed to create framebuffer");
    }
}

Framebuffer::~Framebuffer() {
    if (framebuffer != VK_NULL_HANDLE) {
        vkDestroyFramebuffer(device, framebuffer, nullptr);
    }
}

Framebuffer::Framebuffer(Framebuffer&& other) noexcept
    : device(other.device)
    , framebuffer(other.framebuffer)
    , fbWidth(other.fbWidth)
    , fbHeight(other.fbHeight) {
    other.framebuffer = VK_NULL_HANDLE;
}

Framebuffer& Framebuffer::operator=(Framebuffer&& other) noexcept {
    if (this != &other) {
        if (framebuffer != VK_NULL_HANDLE) {
            vkDestroyFramebuffer(device, framebuffer, nullptr);
        }
        device = other.device;
        framebuffer = other.framebuffer;
        fbWidth = other.fbWidth;
        fbHeight = other.fbHeight;
        other.framebuffer = VK_NULL_HANDLE;
    }
    return *this;
}

}
