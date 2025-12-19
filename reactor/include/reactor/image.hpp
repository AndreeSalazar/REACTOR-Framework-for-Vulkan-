#pragma once
#include "reactor/memory_allocator.hpp"
#include <vulkan/vulkan.h>
#include <memory>
#include <string>

namespace reactor {

enum class ImageFormat {
    RGBA8 = VK_FORMAT_R8G8B8A8_UNORM,
    RGBA16F = VK_FORMAT_R16G16B16A16_SFLOAT,
    RGBA32F = VK_FORMAT_R32G32B32A32_SFLOAT,
    D32F = VK_FORMAT_D32_SFLOAT,
    D24S8 = VK_FORMAT_D24_UNORM_S8_UINT,
    BGRA8 = VK_FORMAT_B8G8R8A8_UNORM
};

enum class ImageUsage {
    None = 0,
    Sampled = VK_IMAGE_USAGE_SAMPLED_BIT,
    Storage = VK_IMAGE_USAGE_STORAGE_BIT,
    ColorAttachment = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
    DepthStencilAttachment = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
    TransferSrc = VK_IMAGE_USAGE_TRANSFER_SRC_BIT,
    TransferDst = VK_IMAGE_USAGE_TRANSFER_DST_BIT
};

inline ImageUsage operator|(ImageUsage a, ImageUsage b) {
    return static_cast<ImageUsage>(static_cast<int>(a) | static_cast<int>(b));
}

inline VkImageUsageFlags toVkFlags(ImageUsage usage) {
    return static_cast<VkImageUsageFlags>(usage);
}

enum class Filter {
    Nearest = VK_FILTER_NEAREST,
    Linear = VK_FILTER_LINEAR
};

enum class AddressMode {
    Repeat = VK_SAMPLER_ADDRESS_MODE_REPEAT,
    MirroredRepeat = VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT,
    ClampToEdge = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
    ClampToBorder = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER
};

class Image {
public:
    Image(std::shared_ptr<MemoryAllocator> allocator, uint32_t width, uint32_t height,
          ImageFormat format, ImageUsage usage, uint32_t mipLevels = 1);
    ~Image();

    Image(const Image&) = delete;
    Image& operator=(const Image&) = delete;
    Image(Image&& other) noexcept;
    Image& operator=(Image&& other) noexcept;

    VkImage handle() const { return image; }
    VkImageView view() const { return imageView; }
    uint32_t width() const { return imgWidth; }
    uint32_t height() const { return imgHeight; }
    VkFormat format() const { return static_cast<VkFormat>(imgFormat); }

    class Builder {
    public:
        Builder(std::shared_ptr<MemoryAllocator> allocator);
        Builder& size(uint32_t width, uint32_t height);
        Builder& format(ImageFormat format);
        Builder& usage(ImageUsage usage);
        Builder& mipLevels(uint32_t levels);
        Image build();
    private:
        std::shared_ptr<MemoryAllocator> alloc;
        uint32_t imgWidth{0};
        uint32_t imgHeight{0};
        ImageFormat imgFormat{ImageFormat::RGBA8};
        ImageUsage imgUsage{ImageUsage::Sampled};
        uint32_t mipLvls{1};
    };

    static Builder create(std::shared_ptr<MemoryAllocator> allocator);

private:
    std::shared_ptr<MemoryAllocator> allocator;
    VkImage image{VK_NULL_HANDLE};
    VkImageView imageView{VK_NULL_HANDLE};
    MemoryBlock memory;
    uint32_t imgWidth{0};
    uint32_t imgHeight{0};
    ImageFormat imgFormat;
    uint32_t mipLevels{1};
    
    void createImageView();
};

} // namespace reactor
