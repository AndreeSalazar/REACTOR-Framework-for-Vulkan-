#pragma once

#include <vulkan/vulkan.h>
#include <vector>
#include <memory>

namespace reactor {

/**
 * @brief Manager simplificado para descriptor sets
 * 
 * Facilita la creación y gestión de descriptor sets para
 * proyectos que heredan de REACTOR
 */
class DescriptorManager {
public:
    explicit DescriptorManager(VkDevice device);
    ~DescriptorManager();

    // No copyable
    DescriptorManager(const DescriptorManager&) = delete;
    DescriptorManager& operator=(const DescriptorManager&) = delete;

    /**
     * @brief Crea un descriptor set layout
     */
    VkDescriptorSetLayout createLayout(
        const std::vector<VkDescriptorSetLayoutBinding>& bindings
    );

    /**
     * @brief Crea un descriptor pool
     */
    VkDescriptorPool createPool(
        const std::vector<VkDescriptorPoolSize>& poolSizes,
        uint32_t maxSets
    );

    /**
     * @brief Alloca descriptor sets
     */
    std::vector<VkDescriptorSet> allocateSets(
        VkDescriptorPool pool,
        const std::vector<VkDescriptorSetLayout>& layouts
    );

    /**
     * @brief Helper: Update descriptor set con image
     */
    void updateImageDescriptor(
        VkDescriptorSet set,
        uint32_t binding,
        VkDescriptorType type,
        VkImageView imageView,
        VkImageLayout layout,
        VkSampler sampler = VK_NULL_HANDLE
    );

    /**
     * @brief Helper: Update descriptor set con buffer
     */
    void updateBufferDescriptor(
        VkDescriptorSet set,
        uint32_t binding,
        VkDescriptorType type,
        VkBuffer buffer,
        VkDeviceSize offset,
        VkDeviceSize range
    );

private:
    VkDevice device_;
    std::vector<VkDescriptorSetLayout> layouts_;
    std::vector<VkDescriptorPool> pools_;
};

} // namespace reactor
