#pragma once

#include <vulkan/vulkan.h>
#include <vector>

namespace reactor {

/**
 * @brief QueryPool wrapper - Vulkan query management
 * 
 * Abstracción completa de VkQueryPool para timestamps, occlusion, etc.
 */
class QueryPool {
public:
    enum class Type {
        Occlusion,
        Timestamp,
        PipelineStatistics
    };

    QueryPool(VkDevice device, Type type, uint32_t queryCount);
    ~QueryPool();

    // No copyable
    QueryPool(const QueryPool&) = delete;
    QueryPool& operator=(const QueryPool&) = delete;

    // Movable
    QueryPool(QueryPool&& other) noexcept;
    QueryPool& operator=(QueryPool&& other) noexcept;

    VkQueryPool handle() const { return queryPool_; }

    /**
     * @brief Reset queries
     */
    void reset(VkCommandBuffer cmd, uint32_t firstQuery, uint32_t queryCount);

    /**
     * @brief Get query results
     */
    std::vector<uint64_t> getResults(uint32_t firstQuery, uint32_t queryCount, bool wait = true);

private:
    VkDevice device_;
    VkQueryPool queryPool_;
    uint32_t queryCount_;
};

} // namespace reactor
