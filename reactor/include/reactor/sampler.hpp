#pragma once

#include <vulkan/vulkan.h>

namespace reactor {

/**
 * @brief Sampler wrapper - Vulkan texture sampling
 * 
 * Abstracción completa de VkSampler para filtrado de texturas
 */
class Sampler {
public:
    /**
     * @brief Configuración de sampler
     */
    struct Config {
        VkFilter magFilter = VK_FILTER_LINEAR;
        VkFilter minFilter = VK_FILTER_LINEAR;
        VkSamplerMipmapMode mipmapMode = VK_SAMPLER_MIPMAP_MODE_LINEAR;
        VkSamplerAddressMode addressModeU = VK_SAMPLER_ADDRESS_MODE_REPEAT;
        VkSamplerAddressMode addressModeV = VK_SAMPLER_ADDRESS_MODE_REPEAT;
        VkSamplerAddressMode addressModeW = VK_SAMPLER_ADDRESS_MODE_REPEAT;
        float mipLodBias = 0.0f;
        bool anisotropyEnable = true;
        float maxAnisotropy = 16.0f;
        bool compareEnable = false;
        VkCompareOp compareOp = VK_COMPARE_OP_ALWAYS;
        float minLod = 0.0f;
        float maxLod = VK_LOD_CLAMP_NONE;
        VkBorderColor borderColor = VK_BORDER_COLOR_INT_OPAQUE_BLACK;
        bool unnormalizedCoordinates = false;
    };

    Sampler(VkDevice device, const Config& config = Config{});
    ~Sampler();

    // No copyable
    Sampler(const Sampler&) = delete;
    Sampler& operator=(const Sampler&) = delete;

    // Movable
    Sampler(Sampler&& other) noexcept;
    Sampler& operator=(Sampler&& other) noexcept;

    VkSampler handle() const { return sampler_; }

    /**
     * @brief Samplers predefinidos comunes
     */
    static Config linearRepeat();
    static Config linearClamp();
    static Config nearestRepeat();
    static Config nearestClamp();
    static Config anisotropic(float maxAnisotropy = 16.0f);

private:
    VkDevice device_;
    VkSampler sampler_;
};

} // namespace reactor
