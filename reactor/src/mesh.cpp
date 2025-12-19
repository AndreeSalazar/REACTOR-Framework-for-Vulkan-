#include "reactor/mesh.hpp"
#include "reactor/memory_allocator.hpp"
#include <cmath>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

namespace reactor {

Mesh::Mesh(std::shared_ptr<MemoryAllocator> allocator) {
}

Mesh Mesh::cube(std::shared_ptr<MemoryAllocator> allocator, float size) {
    float s = size * 0.5f;
    
    std::vector<Vertex> vertices = {
        // Front face (Z+)
        {{-s, -s,  s}, {0, 0, 1}, {0, 0}, {1, 0, 0}},
        {{ s, -s,  s}, {0, 0, 1}, {1, 0}, {1, 0, 0}},
        {{ s,  s,  s}, {0, 0, 1}, {1, 1}, {1, 0, 0}},
        {{-s,  s,  s}, {0, 0, 1}, {0, 1}, {1, 0, 0}},
        
        // Back face (Z-)
        {{ s, -s, -s}, {0, 0, -1}, {0, 0}, {0, 1, 0}},
        {{-s, -s, -s}, {0, 0, -1}, {1, 0}, {0, 1, 0}},
        {{-s,  s, -s}, {0, 0, -1}, {1, 1}, {0, 1, 0}},
        {{ s,  s, -s}, {0, 0, -1}, {0, 1}, {0, 1, 0}},
        
        // Right face (X+)
        {{ s, -s,  s}, {1, 0, 0}, {0, 0}, {0, 0, 1}},
        {{ s, -s, -s}, {1, 0, 0}, {1, 0}, {0, 0, 1}},
        {{ s,  s, -s}, {1, 0, 0}, {1, 1}, {0, 0, 1}},
        {{ s,  s,  s}, {1, 0, 0}, {0, 1}, {0, 0, 1}},
        
        // Left face (X-)
        {{-s, -s, -s}, {-1, 0, 0}, {0, 0}, {1, 1, 0}},
        {{-s, -s,  s}, {-1, 0, 0}, {1, 0}, {1, 1, 0}},
        {{-s,  s,  s}, {-1, 0, 0}, {1, 1}, {1, 1, 0}},
        {{-s,  s, -s}, {-1, 0, 0}, {0, 1}, {1, 1, 0}},
        
        // Top face (Y+)
        {{-s,  s,  s}, {0, 1, 0}, {0, 0}, {1, 0, 1}},
        {{ s,  s,  s}, {0, 1, 0}, {1, 0}, {1, 0, 1}},
        {{ s,  s, -s}, {0, 1, 0}, {1, 1}, {1, 0, 1}},
        {{-s,  s, -s}, {0, 1, 0}, {0, 1}, {1, 0, 1}},
        
        // Bottom face (Y-)
        {{-s, -s, -s}, {0, -1, 0}, {0, 0}, {0, 1, 1}},
        {{ s, -s, -s}, {0, -1, 0}, {1, 0}, {0, 1, 1}},
        {{ s, -s,  s}, {0, -1, 0}, {1, 1}, {0, 1, 1}},
        {{-s, -s,  s}, {0, -1, 0}, {0, 1}, {0, 1, 1}},
    };
    
    std::vector<uint32_t> indices = {
        0, 1, 2, 2, 3, 0,       // Front
        4, 5, 6, 6, 7, 4,       // Back
        8, 9, 10, 10, 11, 8,    // Right
        12, 13, 14, 14, 15, 12, // Left
        16, 17, 18, 18, 19, 16, // Top
        20, 21, 22, 22, 23, 20  // Bottom
    };
    
    return fromData(vertices, indices, allocator);
}

Mesh Mesh::sphere(std::shared_ptr<MemoryAllocator> allocator, int subdivisions) {
    std::vector<Vertex> vertices;
    std::vector<uint32_t> indices;
    
    // Generar esfera UV (latitud/longitud)
    for (int lat = 0; lat <= subdivisions; ++lat) {
        float theta = lat * M_PI / subdivisions;
        float sinTheta = std::sin(theta);
        float cosTheta = std::cos(theta);
        
        for (int lon = 0; lon <= subdivisions; ++lon) {
            float phi = lon * 2 * M_PI / subdivisions;
            float sinPhi = std::sin(phi);
            float cosPhi = std::cos(phi);
            
            Vec3 pos(cosPhi * sinTheta, cosTheta, sinPhi * sinTheta);
            Vec3 normal = glm::normalize(pos);
            Vec2 uv(static_cast<float>(lon) / subdivisions, 
                   static_cast<float>(lat) / subdivisions);
            
            vertices.emplace_back(pos, normal, uv, Vec3(1, 1, 1));
        }
    }
    
    // Generar índices
    for (int lat = 0; lat < subdivisions; ++lat) {
        for (int lon = 0; lon < subdivisions; ++lon) {
            int first = lat * (subdivisions + 1) + lon;
            int second = first + subdivisions + 1;
            
            indices.push_back(first);
            indices.push_back(second);
            indices.push_back(first + 1);
            
            indices.push_back(second);
            indices.push_back(second + 1);
            indices.push_back(first + 1);
        }
    }
    
    return fromData(vertices, indices, allocator);
}

Mesh Mesh::plane(std::shared_ptr<MemoryAllocator> allocator, float size) {
    float s = size * 0.5f;
    
    std::vector<Vertex> vertices = {
        {{-s, 0, -s}, {0, 1, 0}, {0, 0}, {1, 1, 1}},
        {{ s, 0, -s}, {0, 1, 0}, {1, 0}, {1, 1, 1}},
        {{ s, 0,  s}, {0, 1, 0}, {1, 1}, {1, 1, 1}},
        {{-s, 0,  s}, {0, 1, 0}, {0, 1}, {1, 1, 1}},
    };
    
    std::vector<uint32_t> indices = {0, 1, 2, 2, 3, 0};
    
    return fromData(vertices, indices, allocator);
}

Mesh Mesh::quad(std::shared_ptr<MemoryAllocator> allocator) {
    std::vector<Vertex> vertices = {
        {{-1, -1, 0}, {0, 0, 1}, {0, 0}, {1, 1, 1}},
        {{ 1, -1, 0}, {0, 0, 1}, {1, 0}, {1, 1, 1}},
        {{ 1,  1, 0}, {0, 0, 1}, {1, 1}, {1, 1, 1}},
        {{-1,  1, 0}, {0, 0, 1}, {0, 1}, {1, 1, 1}},
    };
    
    std::vector<uint32_t> indices = {0, 1, 2, 2, 3, 0};
    
    return fromData(vertices, indices, allocator);
}

Mesh Mesh::fromFile(const std::string& path, std::shared_ptr<MemoryAllocator> allocator) {
    // TODO: Implementar con assimp
    // Por ahora, retornar cubo por defecto
    return cube(allocator);
}

Mesh Mesh::fromData(
    const std::vector<Vertex>& vertices,
    const std::vector<uint32_t>& indices,
    std::shared_ptr<MemoryAllocator> allocator
) {
    Mesh mesh(allocator);
    mesh.vtxCount = static_cast<uint32_t>(vertices.size());
    mesh.idxCount = static_cast<uint32_t>(indices.size());
    
    // Crear vertex buffer
    mesh.vertexBuf = std::make_unique<Buffer>(
        Buffer::create(allocator)
            .size(sizeof(Vertex) * vertices.size())
            .usage(BufferUsage::Vertex)
            .memoryType(MemoryType::HostVisible)
            .build()
    );
    mesh.vertexBuf->upload(vertices.data(), sizeof(Vertex) * vertices.size());
    
    // Crear index buffer
    mesh.indexBuf = std::make_unique<Buffer>(
        Buffer::create(allocator)
            .size(sizeof(uint32_t) * indices.size())
            .usage(BufferUsage::Index)
            .memoryType(MemoryType::HostVisible)
            .build()
    );
    mesh.indexBuf->upload(indices.data(), sizeof(uint32_t) * indices.size());
    
    return mesh;
}

void Mesh::bind(VkCommandBuffer cmd) const {
    VkBuffer vertexBuffers[] = {vertexBuf->handle()};
    VkDeviceSize offsets[] = {0};
    vkCmdBindVertexBuffers(cmd, 0, 1, vertexBuffers, offsets);
    vkCmdBindIndexBuffer(cmd, indexBuf->handle(), 0, VK_INDEX_TYPE_UINT32);
}

void Mesh::draw(VkCommandBuffer cmd) const {
    bind(cmd);
    vkCmdDrawIndexed(cmd, idxCount, 1, 0, 0, 0);
}

} // namespace reactor
