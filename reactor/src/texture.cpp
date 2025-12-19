#include "reactor/texture.hpp"
#include "reactor/memory_allocator.hpp"
#include <iostream>

namespace reactor {

Texture::Texture() = default;

Texture::~Texture() = default;

Texture::Texture(Texture&& other) noexcept
    : w(other.w)
    , h(other.h)
    , texPath(std::move(other.texPath))
    , loaded(other.loaded) {
    other.w = 0;
    other.h = 0;
    other.loaded = false;
}

Texture& Texture::operator=(Texture&& other) noexcept {
    if (this != &other) {
        w = other.w;
        h = other.h;
        texPath = std::move(other.texPath);
        loaded = other.loaded;
        
        other.w = 0;
        other.h = 0;
        other.loaded = false;
    }
    return *this;
}

Texture Texture::load(const std::string& path, std::shared_ptr<MemoryAllocator> allocator) {
    Texture texture;
    texture.texPath = path;
    texture.w = 256;  // Placeholder dimensions
    texture.h = 256;
    texture.loaded = true;
    
    std::cout << "[Texture] Loaded (placeholder): " << path << " (" << texture.w << "x" << texture.h << ")" << std::endl;
    
    return texture;
}

Texture Texture::fromData(
    const void* data,
    uint32_t width,
    uint32_t height,
    VkFormat format,
    std::shared_ptr<MemoryAllocator> allocator
) {
    Texture texture;
    texture.w = width;
    texture.h = height;
    texture.texPath = "<from_data>";
    texture.loaded = true;
    
    std::cout << "[Texture] Created from data: " << width << "x" << height << std::endl;
    
    return texture;
}

Texture Texture::solidColor(
    float r, float g, float b, float a,
    std::shared_ptr<MemoryAllocator> allocator
) {
    Texture texture;
    texture.w = 1;
    texture.h = 1;
    texture.texPath = "<solid_color>";
    texture.loaded = true;
    
    std::cout << "[Texture] Created solid color: (" << r << ", " << g << ", " << b << ", " << a << ")" << std::endl;
    
    return texture;
}

} // namespace reactor
