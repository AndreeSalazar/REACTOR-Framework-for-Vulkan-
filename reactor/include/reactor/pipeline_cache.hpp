#pragma once

#include <vulkan/vulkan.h>
#include <vector>
#include <string>

namespace reactor {

/**
 * @brief PipelineCache wrapper - Vulkan pipeline caching
 * 
 * Abstracción completa de VkPipelineCache para acelerar creación de pipelines
 */
class PipelineCache {
public:
    explicit PipelineCache(VkDevice device);
    PipelineCache(VkDevice device, const std::vector<uint8_t>& initialData);
    ~PipelineCache();

    // No copyable
    PipelineCache(const PipelineCache&) = delete;
    PipelineCache& operator=(const PipelineCache&) = delete;

    // Movable
    PipelineCache(PipelineCache&& other) noexcept;
    PipelineCache& operator=(PipelineCache&& other) noexcept;

    VkPipelineCache handle() const { return cache_; }

    /**
     * @brief Get cache data for serialization
     */
    std::vector<uint8_t> getData() const;

    /**
     * @brief Save cache to file
     */
    bool saveToFile(const std::string& filename) const;

    /**
     * @brief Load cache from file
     */
    static PipelineCache loadFromFile(VkDevice device, const std::string& filename);

    /**
     * @brief Merge with another cache
     */
    void merge(const PipelineCache& other);

private:
    VkDevice device_;
    VkPipelineCache cache_;
};

} // namespace reactor
