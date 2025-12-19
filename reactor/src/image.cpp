#include "reactor/image.hpp"
#include "reactor/sampler.hpp"
#include <stdexcept>

namespace reactor {

Image::Image(std::shared_ptr<MemoryAllocator> allocator, uint32_t width, uint32_t height,
             ImageFormat format, ImageUsage usage, uint32_t mipLevels)
    : allocator(allocator), imgWidth(width), imgHeight(height), 
      imgFormat(format), mipLevels(mipLevels) {
    
    VkImageCreateInfo imageInfo{};
    imageInfo.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
    imageInfo.imageType = VK_IMAGE_TYPE_2D;
    imageInfo.extent.width = width;
    imageInfo.extent.height = height;
    imageInfo.extent.depth = 1;
    imageInfo.mipLevels = mipLevels;
    imageInfo.arrayLayers = 1;
    imageInfo.format = static_cast<VkFormat>(format);
    imageInfo.tiling = VK_IMAGE_TILING_OPTIMAL;
    imageInfo.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    imageInfo.usage = toVkFlags(usage);
    imageInfo.samples = VK_SAMPLE_COUNT_1_BIT;
    imageInfo.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    
    if (vkCreateImage(allocator->device(), &imageInfo, nullptr, &image) != VK_SUCCESS) {
        throw std::runtime_error("failed to create image");
    }
    
    VkMemoryRequirements memReqs;
    vkGetImageMemoryRequirements(allocator->device(), image, &memReqs);
    
    memory = allocator->allocate(memReqs, MemoryType::DeviceLocal);
    
    if (vkBindImageMemory(allocator->device(), image, memory.memory, memory.offset) != VK_SUCCESS) {
        throw std::runtime_error("failed to bind image memory");
    }
    
    createImageView();
}

Image::~Image() {
    if (imageView != VK_NULL_HANDLE) {
        vkDestroyImageView(allocator->device(), imageView, nullptr);
    }
    if (image != VK_NULL_HANDLE) {
        vkDestroyImage(allocator->device(), image, nullptr);
    }
    if (memory.memory != VK_NULL_HANDLE) {
        allocator->free(memory);
    }
}

Image::Image(Image&& other) noexcept
    : allocator(std::move(other.allocator))
    , image(other.image)
    , imageView(other.imageView)
    , memory(other.memory)
    , imgWidth(other.imgWidth)
    , imgHeight(other.imgHeight)
    , imgFormat(other.imgFormat)
    , mipLevels(other.mipLevels) {
    other.image = VK_NULL_HANDLE;
    other.imageView = VK_NULL_HANDLE;
    other.memory = {};
}

Image& Image::operator=(Image&& other) noexcept {
    if (this != &other) {
        if (imageView != VK_NULL_HANDLE) {
            vkDestroyImageView(allocator->device(), imageView, nullptr);
        }
        if (image != VK_NULL_HANDLE) {
            vkDestroyImage(allocator->device(), image, nullptr);
        }
        if (memory.memory != VK_NULL_HANDLE) {
            allocator->free(memory);
        }
        
        allocator = std::move(other.allocator);
        image = other.image;
        imageView = other.imageView;
        memory = other.memory;
        imgWidth = other.imgWidth;
        imgHeight = other.imgHeight;
        imgFormat = other.imgFormat;
        mipLevels = other.mipLevels;
        
        other.image = VK_NULL_HANDLE;
        other.imageView = VK_NULL_HANDLE;
        other.memory = {};
    }
    return *this;
}

void Image::createImageView() {
    VkImageViewCreateInfo viewInfo{};
    viewInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    viewInfo.image = image;
    viewInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
    viewInfo.format = static_cast<VkFormat>(imgFormat);
    viewInfo.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    viewInfo.subresourceRange.baseMipLevel = 0;
    viewInfo.subresourceRange.levelCount = mipLevels;
    viewInfo.subresourceRange.baseArrayLayer = 0;
    viewInfo.subresourceRange.layerCount = 1;
    
    if (vkCreateImageView(allocator->device(), &viewInfo, nullptr, &imageView) != VK_SUCCESS) {
        throw std::runtime_error("failed to create image view");
    }
}

Image::Builder::Builder(std::shared_ptr<MemoryAllocator> allocator)
    : alloc(allocator) {}

Image::Builder& Image::Builder::size(uint32_t width, uint32_t height) {
    imgWidth = width;
    imgHeight = height;
    return *this;
}

Image::Builder& Image::Builder::format(ImageFormat format) {
    imgFormat = format;
    return *this;
}

Image::Builder& Image::Builder::usage(ImageUsage usage) {
    imgUsage = usage;
    return *this;
}

Image::Builder& Image::Builder::mipLevels(uint32_t levels) {
    mipLvls = levels;
    return *this;
}

Image Image::Builder::build() {
    if (imgWidth == 0 || imgHeight == 0) {
        throw std::runtime_error("image dimensions must be greater than 0");
    }
    return Image(alloc, imgWidth, imgHeight, imgFormat, imgUsage, mipLvls);
}

Image::Builder Image::create(std::shared_ptr<MemoryAllocator> allocator) {
    return Builder(allocator);
}

} // namespace reactor
