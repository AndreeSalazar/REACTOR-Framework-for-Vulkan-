#include "reactor/sampler.hpp"
#include <stdexcept>

namespace reactor {

Sampler::Sampler(VkDevice device, const Config& config)
    : device_(device), sampler_(VK_NULL_HANDLE) {
    
    VkSamplerCreateInfo samplerInfo{};
    samplerInfo.sType = VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO;
    samplerInfo.magFilter = config.magFilter;
    samplerInfo.minFilter = config.minFilter;
    samplerInfo.mipmapMode = config.mipmapMode;
    samplerInfo.addressModeU = config.addressModeU;
    samplerInfo.addressModeV = config.addressModeV;
    samplerInfo.addressModeW = config.addressModeW;
    samplerInfo.mipLodBias = config.mipLodBias;
    samplerInfo.anisotropyEnable = config.anisotropyEnable ? VK_TRUE : VK_FALSE;
    samplerInfo.maxAnisotropy = config.maxAnisotropy;
    samplerInfo.compareEnable = config.compareEnable ? VK_TRUE : VK_FALSE;
    samplerInfo.compareOp = config.compareOp;
    samplerInfo.minLod = config.minLod;
    samplerInfo.maxLod = config.maxLod;
    samplerInfo.borderColor = config.borderColor;
    samplerInfo.unnormalizedCoordinates = config.unnormalizedCoordinates ? VK_TRUE : VK_FALSE;

    if (vkCreateSampler(device_, &samplerInfo, nullptr, &sampler_) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create sampler");
    }
}

Sampler::~Sampler() {
    if (sampler_ != VK_NULL_HANDLE) {
        vkDestroySampler(device_, sampler_, nullptr);
    }
}

Sampler::Sampler(Sampler&& other) noexcept
    : device_(other.device_), sampler_(other.sampler_) {
    other.sampler_ = VK_NULL_HANDLE;
}

Sampler& Sampler::operator=(Sampler&& other) noexcept {
    if (this != &other) {
        if (sampler_ != VK_NULL_HANDLE) {
            vkDestroySampler(device_, sampler_, nullptr);
        }
        device_ = other.device_;
        sampler_ = other.sampler_;
        other.sampler_ = VK_NULL_HANDLE;
    }
    return *this;
}

Sampler::Config Sampler::linearRepeat() {
    Config config;
    config.magFilter = VK_FILTER_LINEAR;
    config.minFilter = VK_FILTER_LINEAR;
    config.addressModeU = VK_SAMPLER_ADDRESS_MODE_REPEAT;
    config.addressModeV = VK_SAMPLER_ADDRESS_MODE_REPEAT;
    config.addressModeW = VK_SAMPLER_ADDRESS_MODE_REPEAT;
    return config;
}

Sampler::Config Sampler::linearClamp() {
    Config config;
    config.magFilter = VK_FILTER_LINEAR;
    config.minFilter = VK_FILTER_LINEAR;
    config.addressModeU = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    config.addressModeV = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    config.addressModeW = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    return config;
}

Sampler::Config Sampler::nearestRepeat() {
    Config config;
    config.magFilter = VK_FILTER_NEAREST;
    config.minFilter = VK_FILTER_NEAREST;
    config.addressModeU = VK_SAMPLER_ADDRESS_MODE_REPEAT;
    config.addressModeV = VK_SAMPLER_ADDRESS_MODE_REPEAT;
    config.addressModeW = VK_SAMPLER_ADDRESS_MODE_REPEAT;
    return config;
}

Sampler::Config Sampler::nearestClamp() {
    Config config;
    config.magFilter = VK_FILTER_NEAREST;
    config.minFilter = VK_FILTER_NEAREST;
    config.addressModeU = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    config.addressModeV = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    config.addressModeW = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    return config;
}

Sampler::Config Sampler::anisotropic(float maxAnisotropy) {
    Config config;
    config.magFilter = VK_FILTER_LINEAR;
    config.minFilter = VK_FILTER_LINEAR;
    config.anisotropyEnable = true;
    config.maxAnisotropy = maxAnisotropy;
    return config;
}

} // namespace reactor
