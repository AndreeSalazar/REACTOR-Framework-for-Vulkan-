#include "reactor/pipeline_cache.hpp"
#include <stdexcept>
#include <fstream>

namespace reactor {

PipelineCache::PipelineCache(VkDevice device)
    : device_(device), cache_(VK_NULL_HANDLE) {
    
    VkPipelineCacheCreateInfo cacheInfo{};
    cacheInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO;

    if (vkCreatePipelineCache(device_, &cacheInfo, nullptr, &cache_) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create pipeline cache");
    }
}

PipelineCache::PipelineCache(VkDevice device, const std::vector<uint8_t>& initialData)
    : device_(device), cache_(VK_NULL_HANDLE) {
    
    VkPipelineCacheCreateInfo cacheInfo{};
    cacheInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO;
    cacheInfo.initialDataSize = initialData.size();
    cacheInfo.pInitialData = initialData.data();

    if (vkCreatePipelineCache(device_, &cacheInfo, nullptr, &cache_) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create pipeline cache with initial data");
    }
}

PipelineCache::~PipelineCache() {
    if (cache_ != VK_NULL_HANDLE) {
        vkDestroyPipelineCache(device_, cache_, nullptr);
    }
}

PipelineCache::PipelineCache(PipelineCache&& other) noexcept
    : device_(other.device_), cache_(other.cache_) {
    other.cache_ = VK_NULL_HANDLE;
}

PipelineCache& PipelineCache::operator=(PipelineCache&& other) noexcept {
    if (this != &other) {
        if (cache_ != VK_NULL_HANDLE) {
            vkDestroyPipelineCache(device_, cache_, nullptr);
        }
        device_ = other.device_;
        cache_ = other.cache_;
        other.cache_ = VK_NULL_HANDLE;
    }
    return *this;
}

std::vector<uint8_t> PipelineCache::getData() const {
    size_t dataSize = 0;
    vkGetPipelineCacheData(device_, cache_, &dataSize, nullptr);

    std::vector<uint8_t> data(dataSize);
    vkGetPipelineCacheData(device_, cache_, &dataSize, data.data());

    return data;
}

bool PipelineCache::saveToFile(const std::string& filename) const {
    auto data = getData();
    
    std::ofstream file(filename, std::ios::binary);
    if (!file.is_open()) {
        return false;
    }

    file.write(reinterpret_cast<const char*>(data.data()), data.size());
    return file.good();
}

PipelineCache PipelineCache::loadFromFile(VkDevice device, const std::string& filename) {
    std::ifstream file(filename, std::ios::binary | std::ios::ate);
    if (!file.is_open()) {
        return PipelineCache(device);
    }

    size_t fileSize = static_cast<size_t>(file.tellg());
    std::vector<uint8_t> data(fileSize);

    file.seekg(0);
    file.read(reinterpret_cast<char*>(data.data()), fileSize);

    return PipelineCache(device, data);
}

void PipelineCache::merge(const PipelineCache& other) {
    VkPipelineCache srcCache = other.handle();
    vkMergePipelineCaches(device_, cache_, 1, &srcCache);
}

} // namespace reactor
