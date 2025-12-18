#include "reactor/command_buffer.hpp"
#include <stdexcept>

namespace reactor {

CommandPool::CommandPool(VkDevice device, uint32_t queueFamilyIndex, bool transient)
    : dev(device) {
    
    VkCommandPoolCreateInfo poolInfo{};
    poolInfo.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
    poolInfo.queueFamilyIndex = queueFamilyIndex;
    poolInfo.flags = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
    if (transient) {
        poolInfo.flags |= VK_COMMAND_POOL_CREATE_TRANSIENT_BIT;
    }
    
    if (vkCreateCommandPool(dev, &poolInfo, nullptr, &pool) != VK_SUCCESS) {
        throw std::runtime_error("failed to create command pool");
    }
}

CommandPool::~CommandPool() {
    if (pool != VK_NULL_HANDLE) {
        vkDestroyCommandPool(dev, pool, nullptr);
    }
}

CommandPool::CommandPool(CommandPool&& other) noexcept
    : dev(other.dev), pool(other.pool) {
    other.pool = VK_NULL_HANDLE;
}

CommandPool& CommandPool::operator=(CommandPool&& other) noexcept {
    if (this != &other) {
        if (pool != VK_NULL_HANDLE) {
            vkDestroyCommandPool(dev, pool, nullptr);
        }
        dev = other.dev;
        pool = other.pool;
        other.pool = VK_NULL_HANDLE;
    }
    return *this;
}

void CommandPool::reset() {
    vkResetCommandPool(dev, pool, 0);
}

CommandBuffer::CommandBuffer(std::shared_ptr<CommandPool> pool, bool secondary)
    : pool(pool), isSecondary(secondary) {
    
    VkCommandBufferAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
    allocInfo.commandPool = pool->handle();
    allocInfo.level = secondary ? VK_COMMAND_BUFFER_LEVEL_SECONDARY : VK_COMMAND_BUFFER_LEVEL_PRIMARY;
    allocInfo.commandBufferCount = 1;
    
    if (vkAllocateCommandBuffers(pool->device(), &allocInfo, &buffer) != VK_SUCCESS) {
        throw std::runtime_error("failed to allocate command buffer");
    }
}

CommandBuffer::~CommandBuffer() {
    if (buffer != VK_NULL_HANDLE) {
        vkFreeCommandBuffers(pool->device(), pool->handle(), 1, &buffer);
    }
}

CommandBuffer::CommandBuffer(CommandBuffer&& other) noexcept
    : pool(std::move(other.pool)), buffer(other.buffer), isSecondary(other.isSecondary) {
    other.buffer = VK_NULL_HANDLE;
}

CommandBuffer& CommandBuffer::operator=(CommandBuffer&& other) noexcept {
    if (this != &other) {
        if (buffer != VK_NULL_HANDLE) {
            vkFreeCommandBuffers(pool->device(), pool->handle(), 1, &buffer);
        }
        pool = std::move(other.pool);
        buffer = other.buffer;
        isSecondary = other.isSecondary;
        other.buffer = VK_NULL_HANDLE;
    }
    return *this;
}

void CommandBuffer::begin(bool oneTimeSubmit) {
    VkCommandBufferBeginInfo beginInfo{};
    beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    if (oneTimeSubmit) {
        beginInfo.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
    }
    
    if (vkBeginCommandBuffer(buffer, &beginInfo) != VK_SUCCESS) {
        throw std::runtime_error("failed to begin command buffer");
    }
}

void CommandBuffer::end() {
    if (vkEndCommandBuffer(buffer) != VK_SUCCESS) {
        throw std::runtime_error("failed to end command buffer");
    }
}

void CommandBuffer::reset() {
    vkResetCommandBuffer(buffer, 0);
}

void CommandBuffer::beginRenderPass(VkRenderPass renderPass, VkFramebuffer framebuffer,
                                   VkExtent2D extent, const std::vector<VkClearValue>& clearValues) {
    VkRenderPassBeginInfo renderPassInfo{};
    renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
    renderPassInfo.renderPass = renderPass;
    renderPassInfo.framebuffer = framebuffer;
    renderPassInfo.renderArea.offset = {0, 0};
    renderPassInfo.renderArea.extent = extent;
    renderPassInfo.clearValueCount = static_cast<uint32_t>(clearValues.size());
    renderPassInfo.pClearValues = clearValues.data();
    
    vkCmdBeginRenderPass(buffer, &renderPassInfo, VK_SUBPASS_CONTENTS_INLINE);
}

void CommandBuffer::endRenderPass() {
    vkCmdEndRenderPass(buffer);
}

void CommandBuffer::bindPipeline(VkPipelineBindPoint bindPoint, VkPipeline pipeline) {
    vkCmdBindPipeline(buffer, bindPoint, pipeline);
}

void CommandBuffer::bindVertexBuffers(uint32_t firstBinding, const std::vector<VkBuffer>& buffers,
                                     const std::vector<VkDeviceSize>& offsets) {
    vkCmdBindVertexBuffers(buffer, firstBinding, static_cast<uint32_t>(buffers.size()),
                          buffers.data(), offsets.data());
}

void CommandBuffer::bindIndexBuffer(VkBuffer buf, VkDeviceSize offset, VkIndexType indexType) {
    vkCmdBindIndexBuffer(buffer, buf, offset, indexType);
}

void CommandBuffer::bindDescriptorSets(VkPipelineBindPoint bindPoint, VkPipelineLayout layout,
                                      uint32_t firstSet, const std::vector<VkDescriptorSet>& sets) {
    vkCmdBindDescriptorSets(buffer, bindPoint, layout, firstSet,
                           static_cast<uint32_t>(sets.size()), sets.data(), 0, nullptr);
}

void CommandBuffer::draw(uint32_t vertexCount, uint32_t instanceCount,
                        uint32_t firstVertex, uint32_t firstInstance) {
    vkCmdDraw(buffer, vertexCount, instanceCount, firstVertex, firstInstance);
}

void CommandBuffer::drawIndexed(uint32_t indexCount, uint32_t instanceCount,
                               uint32_t firstIndex, int32_t vertexOffset, uint32_t firstInstance) {
    vkCmdDrawIndexed(buffer, indexCount, instanceCount, firstIndex, vertexOffset, firstInstance);
}

void CommandBuffer::dispatch(uint32_t groupCountX, uint32_t groupCountY, uint32_t groupCountZ) {
    vkCmdDispatch(buffer, groupCountX, groupCountY, groupCountZ);
}

void CommandBuffer::copyBuffer(VkBuffer src, VkBuffer dst, VkDeviceSize size,
                              VkDeviceSize srcOffset, VkDeviceSize dstOffset) {
    VkBufferCopy copyRegion{};
    copyRegion.srcOffset = srcOffset;
    copyRegion.dstOffset = dstOffset;
    copyRegion.size = size;
    vkCmdCopyBuffer(buffer, src, dst, 1, &copyRegion);
}

void CommandBuffer::copyBufferToImage(VkBuffer src, VkImage dst, VkImageLayout dstLayout,
                                     uint32_t width, uint32_t height) {
    VkBufferImageCopy region{};
    region.bufferOffset = 0;
    region.bufferRowLength = 0;
    region.bufferImageHeight = 0;
    region.imageSubresource.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    region.imageSubresource.mipLevel = 0;
    region.imageSubresource.baseArrayLayer = 0;
    region.imageSubresource.layerCount = 1;
    region.imageOffset = {0, 0, 0};
    region.imageExtent = {width, height, 1};
    
    vkCmdCopyBufferToImage(buffer, src, dst, dstLayout, 1, &region);
}

void CommandBuffer::pipelineBarrier(const std::vector<ImageBarrier>& imageBarriers,
                                   const std::vector<BufferBarrier>& bufferBarriers) {
    std::vector<VkImageMemoryBarrier> imgBarriers;
    for (const auto& barrier : imageBarriers) {
        VkImageMemoryBarrier imgBarrier{};
        imgBarrier.sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER;
        imgBarrier.oldLayout = barrier.oldLayout;
        imgBarrier.newLayout = barrier.newLayout;
        imgBarrier.srcQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
        imgBarrier.dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
        imgBarrier.image = barrier.image;
        imgBarrier.subresourceRange.aspectMask = barrier.aspectMask;
        imgBarrier.subresourceRange.baseMipLevel = 0;
        imgBarrier.subresourceRange.levelCount = 1;
        imgBarrier.subresourceRange.baseArrayLayer = 0;
        imgBarrier.subresourceRange.layerCount = 1;
        imgBarrier.srcAccessMask = barrier.srcAccess;
        imgBarrier.dstAccessMask = barrier.dstAccess;
        imgBarriers.push_back(imgBarrier);
    }
    
    std::vector<VkBufferMemoryBarrier> bufBarriers;
    for (const auto& barrier : bufferBarriers) {
        VkBufferMemoryBarrier bufBarrier{};
        bufBarrier.sType = VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER;
        bufBarrier.srcAccessMask = barrier.srcAccess;
        bufBarrier.dstAccessMask = barrier.dstAccess;
        bufBarrier.srcQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
        bufBarrier.dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
        bufBarrier.buffer = barrier.buffer;
        bufBarrier.offset = barrier.offset;
        bufBarrier.size = barrier.size;
        bufBarriers.push_back(bufBarrier);
    }
    
    VkPipelineStageFlags srcStage = imageBarriers.empty() ? 
        VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT : imageBarriers[0].srcStage;
    VkPipelineStageFlags dstStage = imageBarriers.empty() ? 
        VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT : imageBarriers[0].dstStage;
    
    vkCmdPipelineBarrier(buffer, srcStage, dstStage, 0,
                        0, nullptr,
                        static_cast<uint32_t>(bufBarriers.size()), bufBarriers.data(),
                        static_cast<uint32_t>(imgBarriers.size()), imgBarriers.data());
}

void CommandBuffer::setViewport(float x, float y, float width, float height,
                               float minDepth, float maxDepth) {
    VkViewport viewport{};
    viewport.x = x;
    viewport.y = y;
    viewport.width = width;
    viewport.height = height;
    viewport.minDepth = minDepth;
    viewport.maxDepth = maxDepth;
    vkCmdSetViewport(buffer, 0, 1, &viewport);
}

void CommandBuffer::setScissor(int32_t x, int32_t y, uint32_t width, uint32_t height) {
    VkRect2D scissor{};
    scissor.offset = {x, y};
    scissor.extent = {width, height};
    vkCmdSetScissor(buffer, 0, 1, &scissor);
}

void CommandBuffer::pushConstants(VkPipelineLayout layout, VkShaderStageFlags stageFlags,
                                 uint32_t offset, uint32_t size, const void* data) {
    vkCmdPushConstants(buffer, layout, stageFlags, offset, size, data);
}

}
