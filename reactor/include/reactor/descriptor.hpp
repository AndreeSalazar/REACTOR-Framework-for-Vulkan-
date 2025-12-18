#pragma once
#include <vulkan/vulkan.h>
#include <vector>
#include <memory>

namespace reactor {

enum class DescriptorType {
    Sampler = VK_DESCRIPTOR_TYPE_SAMPLER,
    CombinedImageSampler = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
    SampledImage = VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
    StorageImage = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE,
    UniformBuffer = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
    StorageBuffer = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
    UniformBufferDynamic = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC,
    StorageBufferDynamic = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC
};

struct DescriptorBinding {
    uint32_t binding;
    DescriptorType type;
    uint32_t count{1};
    VkShaderStageFlags stageFlags;
};

class DescriptorSetLayout {
public:
    DescriptorSetLayout(VkDevice device, const std::vector<DescriptorBinding>& bindings);
    ~DescriptorSetLayout();

    DescriptorSetLayout(const DescriptorSetLayout&) = delete;
    DescriptorSetLayout& operator=(const DescriptorSetLayout&) = delete;
    DescriptorSetLayout(DescriptorSetLayout&& other) noexcept;
    DescriptorSetLayout& operator=(DescriptorSetLayout&& other) noexcept;

    VkDescriptorSetLayout handle() const { return layout; }

    class Builder {
    public:
        Builder(VkDevice device);
        Builder& binding(uint32_t binding, DescriptorType type, VkShaderStageFlags stages, uint32_t count = 1);
        DescriptorSetLayout build();
    private:
        VkDevice dev;
        std::vector<DescriptorBinding> bindings;
    };

    static Builder create(VkDevice device);

private:
    VkDevice device;
    VkDescriptorSetLayout layout{VK_NULL_HANDLE};
};

class DescriptorPool {
public:
    DescriptorPool(VkDevice device, uint32_t maxSets, const std::vector<VkDescriptorPoolSize>& poolSizes);
    ~DescriptorPool();

    DescriptorPool(const DescriptorPool&) = delete;
    DescriptorPool& operator=(const DescriptorPool&) = delete;
    DescriptorPool(DescriptorPool&& other) noexcept;
    DescriptorPool& operator=(DescriptorPool&& other) noexcept;

    VkDescriptorPool handle() const { return pool; }
    VkDevice device() const { return dev; }

    void reset();

    class Builder {
    public:
        Builder(VkDevice device);
        Builder& maxSets(uint32_t count);
        Builder& poolSize(DescriptorType type, uint32_t count);
        DescriptorPool build();
    private:
        VkDevice dev;
        uint32_t maxSetCount{100};
        std::vector<VkDescriptorPoolSize> sizes;
    };

    static Builder create(VkDevice device);

private:
    VkDevice dev;
    VkDescriptorPool pool{VK_NULL_HANDLE};
};

class DescriptorSet {
public:
    DescriptorSet(std::shared_ptr<DescriptorPool> pool, VkDescriptorSetLayout layout);
    ~DescriptorSet();

    DescriptorSet(const DescriptorSet&) = delete;
    DescriptorSet& operator=(const DescriptorSet&) = delete;
    DescriptorSet(DescriptorSet&& other) noexcept;
    DescriptorSet& operator=(DescriptorSet&& other) noexcept;

    VkDescriptorSet handle() const { return set; }

    void updateBuffer(uint32_t binding, VkBuffer buffer, VkDeviceSize offset, VkDeviceSize range);
    void updateImage(uint32_t binding, VkImageView imageView, VkSampler sampler, VkImageLayout layout);

private:
    std::shared_ptr<DescriptorPool> pool;
    VkDescriptorSet set{VK_NULL_HANDLE};
};

}
