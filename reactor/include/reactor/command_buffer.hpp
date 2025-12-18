#pragma once
#include "reactor/sync.hpp"
#include <vulkan/vulkan.h>
#include <memory>
#include <vector>
#include <functional>

namespace reactor {

class CommandPool {
public:
    CommandPool(VkDevice device, uint32_t queueFamilyIndex, bool transient = false);
    ~CommandPool();

    CommandPool(const CommandPool&) = delete;
    CommandPool& operator=(const CommandPool&) = delete;
    CommandPool(CommandPool&& other) noexcept;
    CommandPool& operator=(CommandPool&& other) noexcept;

    VkCommandPool handle() const { return pool; }
    VkDevice device() const { return dev; }
    
    void reset();

private:
    VkDevice dev;
    VkCommandPool pool{VK_NULL_HANDLE};
};

class CommandBuffer {
public:
    CommandBuffer(std::shared_ptr<CommandPool> pool, bool secondary = false);
    ~CommandBuffer();

    CommandBuffer(const CommandBuffer&) = delete;
    CommandBuffer& operator=(const CommandBuffer&) = delete;
    CommandBuffer(CommandBuffer&& other) noexcept;
    CommandBuffer& operator=(CommandBuffer&& other) noexcept;

    VkCommandBuffer handle() const { return buffer; }
    
    void begin(bool oneTimeSubmit = false);
    void end();
    void reset();
    
    void beginRenderPass(VkRenderPass renderPass, VkFramebuffer framebuffer,
                        VkExtent2D extent, const std::vector<VkClearValue>& clearValues);
    void endRenderPass();
    
    void bindPipeline(VkPipelineBindPoint bindPoint, VkPipeline pipeline);
    void bindVertexBuffers(uint32_t firstBinding, const std::vector<VkBuffer>& buffers,
                          const std::vector<VkDeviceSize>& offsets);
    void bindIndexBuffer(VkBuffer buffer, VkDeviceSize offset, VkIndexType indexType);
    void bindDescriptorSets(VkPipelineBindPoint bindPoint, VkPipelineLayout layout,
                           uint32_t firstSet, const std::vector<VkDescriptorSet>& sets);
    
    void draw(uint32_t vertexCount, uint32_t instanceCount = 1,
             uint32_t firstVertex = 0, uint32_t firstInstance = 0);
    void drawIndexed(uint32_t indexCount, uint32_t instanceCount = 1,
                    uint32_t firstIndex = 0, int32_t vertexOffset = 0, uint32_t firstInstance = 0);
    
    void dispatch(uint32_t groupCountX, uint32_t groupCountY, uint32_t groupCountZ);
    
    void copyBuffer(VkBuffer src, VkBuffer dst, VkDeviceSize size,
                   VkDeviceSize srcOffset = 0, VkDeviceSize dstOffset = 0);
    void copyBufferToImage(VkBuffer src, VkImage dst, VkImageLayout dstLayout,
                          uint32_t width, uint32_t height);
    
    void pipelineBarrier(const std::vector<ImageBarrier>& imageBarriers,
                        const std::vector<BufferBarrier>& bufferBarriers = {});
    
    void setViewport(float x, float y, float width, float height,
                    float minDepth = 0.0f, float maxDepth = 1.0f);
    void setScissor(int32_t x, int32_t y, uint32_t width, uint32_t height);
    
    void pushConstants(VkPipelineLayout layout, VkShaderStageFlags stageFlags,
                      uint32_t offset, uint32_t size, const void* data);

private:
    std::shared_ptr<CommandPool> pool;
    VkCommandBuffer buffer{VK_NULL_HANDLE};
    bool isSecondary{false};
};

}
