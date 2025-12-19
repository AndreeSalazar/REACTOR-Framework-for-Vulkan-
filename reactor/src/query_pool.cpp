#include "reactor/query_pool.hpp"
#include <stdexcept>

namespace reactor {

QueryPool::QueryPool(VkDevice device, Type type, uint32_t queryCount)
    : device_(device), queryPool_(VK_NULL_HANDLE), queryCount_(queryCount) {
    
    VkQueryPoolCreateInfo poolInfo{};
    poolInfo.sType = VK_STRUCTURE_TYPE_QUERY_POOL_CREATE_INFO;
    poolInfo.queryCount = queryCount;

    switch (type) {
        case Type::Occlusion:
            poolInfo.queryType = VK_QUERY_TYPE_OCCLUSION;
            break;
        case Type::Timestamp:
            poolInfo.queryType = VK_QUERY_TYPE_TIMESTAMP;
            break;
        case Type::PipelineStatistics:
            poolInfo.queryType = VK_QUERY_TYPE_PIPELINE_STATISTICS;
            poolInfo.pipelineStatistics = 
                VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_VERTICES_BIT |
                VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_PRIMITIVES_BIT |
                VK_QUERY_PIPELINE_STATISTIC_VERTEX_SHADER_INVOCATIONS_BIT |
                VK_QUERY_PIPELINE_STATISTIC_FRAGMENT_SHADER_INVOCATIONS_BIT;
            break;
    }

    if (vkCreateQueryPool(device_, &poolInfo, nullptr, &queryPool_) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create query pool");
    }
}

QueryPool::~QueryPool() {
    if (queryPool_ != VK_NULL_HANDLE) {
        vkDestroyQueryPool(device_, queryPool_, nullptr);
    }
}

QueryPool::QueryPool(QueryPool&& other) noexcept
    : device_(other.device_)
    , queryPool_(other.queryPool_)
    , queryCount_(other.queryCount_) {
    other.queryPool_ = VK_NULL_HANDLE;
}

QueryPool& QueryPool::operator=(QueryPool&& other) noexcept {
    if (this != &other) {
        if (queryPool_ != VK_NULL_HANDLE) {
            vkDestroyQueryPool(device_, queryPool_, nullptr);
        }
        device_ = other.device_;
        queryPool_ = other.queryPool_;
        queryCount_ = other.queryCount_;
        other.queryPool_ = VK_NULL_HANDLE;
    }
    return *this;
}

void QueryPool::reset(VkCommandBuffer cmd, uint32_t firstQuery, uint32_t queryCount) {
    vkCmdResetQueryPool(cmd, queryPool_, firstQuery, queryCount);
}

std::vector<uint64_t> QueryPool::getResults(uint32_t firstQuery, uint32_t queryCount, bool wait) {
    std::vector<uint64_t> results(queryCount);
    VkQueryResultFlags flags = VK_QUERY_RESULT_64_BIT;
    if (wait) {
        flags |= VK_QUERY_RESULT_WAIT_BIT;
    }

    vkGetQueryPoolResults(
        device_,
        queryPool_,
        firstQuery,
        queryCount,
        results.size() * sizeof(uint64_t),
        results.data(),
        sizeof(uint64_t),
        flags
    );

    return results;
}

} // namespace reactor
