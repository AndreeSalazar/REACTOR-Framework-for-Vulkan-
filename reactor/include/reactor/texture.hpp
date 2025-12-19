#pragma once
#include <vulkan/vulkan.h>
#include <string>
#include <memory>

namespace reactor {

// Forward declarations
class MemoryAllocator;

/**
 * @brief Texture - Simplificación extrema de carga de texturas
 * 
 * Carga automática desde archivo con una línea:
 * auto texture = Texture::load("image.png", ctx);
 */
class Texture {
public:
    Texture();
    ~Texture();

    Texture(const Texture&) = delete;
    Texture& operator=(const Texture&) = delete;
    Texture(Texture&& other) noexcept;
    Texture& operator=(Texture&& other) noexcept;

    /**
     * @brief Carga textura desde archivo - UNA LÍNEA
     * @param path Ruta al archivo (png, jpg, etc)
     * @param allocator Memory allocator
     * @return Texture cargada y lista para usar
     * 
     * Auto:
     * - Carga imagen (stb_image)
     * - Crea VkImage
     * - Alloca memoria
     * - Upload a GPU
     * - Genera mipmaps (opcional)
     */
    static Texture load(const std::string& path, std::shared_ptr<MemoryAllocator> allocator);
    
    /**
     * @brief Crea textura desde datos en memoria
     */
    static Texture fromData(
        const void* data,
        uint32_t width,
        uint32_t height,
        VkFormat format,
        std::shared_ptr<MemoryAllocator> allocator
    );
    
    /**
     * @brief Crea textura sólida de un color
     */
    static Texture solidColor(
        float r, float g, float b, float a,
        std::shared_ptr<MemoryAllocator> allocator
    );

    // Getters
    uint32_t width() const { return w; }
    uint32_t height() const { return h; }
    const std::string& path() const { return texPath; }
    bool isLoaded() const { return loaded; }

private:
    uint32_t w{0};
    uint32_t h{0};
    std::string texPath;
    bool loaded{false};
};

} // namespace reactor
