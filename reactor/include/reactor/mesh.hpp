#pragma once
#include "buffer.hpp"
#include "math.hpp"
#include <vector>
#include <memory>

namespace reactor {

// Forward declarations
class MemoryAllocator;
class CommandBuffer;

/**
 * @brief Mesh - Geometría simplificada
 * 
 * Helpers predefinidos:
 * auto cube = Mesh::cube(allocator);
 * auto sphere = Mesh::sphere(allocator, 32);
 * auto plane = Mesh::plane(allocator);
 */
class Mesh {
public:
    // Vertex format común
    struct Vertex {
        Vec3 position;
        Vec3 normal;
        Vec2 texCoord;
        Vec3 color;
        
        Vertex() = default;
        Vertex(const Vec3& pos, const Vec3& norm = Vec3(0, 1, 0), 
               const Vec2& uv = Vec2(0, 0), const Vec3& col = Vec3(1, 1, 1))
            : position(pos), normal(norm), texCoord(uv), color(col) {}
    };

    Mesh(std::shared_ptr<MemoryAllocator> allocator);
    ~Mesh() = default;

    Mesh(const Mesh&) = delete;
    Mesh& operator=(const Mesh&) = delete;
    Mesh(Mesh&& other) noexcept = default;
    Mesh& operator=(Mesh&& other) noexcept = default;

    /**
     * @brief Geometría predefinida - UNA LÍNEA
     */
    static Mesh cube(std::shared_ptr<MemoryAllocator> allocator, float size = 1.0f);
    static Mesh sphere(std::shared_ptr<MemoryAllocator> allocator, int subdivisions = 32);
    static Mesh plane(std::shared_ptr<MemoryAllocator> allocator, float size = 1.0f);
    static Mesh quad(std::shared_ptr<MemoryAllocator> allocator);
    
    /**
     * @brief Carga desde archivo (TODO: implementar con assimp)
     */
    static Mesh fromFile(const std::string& path, std::shared_ptr<MemoryAllocator> allocator);
    
    /**
     * @brief Crea desde vértices e índices
     */
    static Mesh fromData(
        const std::vector<Vertex>& vertices,
        const std::vector<uint32_t>& indices,
        std::shared_ptr<MemoryAllocator> allocator
    );

    // Getters
    const Buffer& vertexBuffer() const { return *vertexBuf; }
    const Buffer& indexBuffer() const { return *indexBuf; }
    uint32_t indexCount() const { return idxCount; }
    uint32_t vertexCount() const { return vtxCount; }

    // Helper para bind en command buffer
    void bind(VkCommandBuffer cmd) const;
    void draw(VkCommandBuffer cmd) const;

private:
    std::unique_ptr<Buffer> vertexBuf;
    std::unique_ptr<Buffer> indexBuf;
    uint32_t vtxCount{0};
    uint32_t idxCount{0};
};

} // namespace reactor
