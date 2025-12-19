#pragma once
#include <string>
#include <unordered_map>
#include <memory>

namespace reactor {

// Forward declarations
class Texture;
class Mesh;
class Material;
class MemoryAllocator;

/**
 * @brief ResourceManager - Cache automático de assets
 * 
 * Uso ultra simple:
 * ResourceManager resources(allocator);
 * auto mesh = resources.getMesh("cube.obj");
 * auto texture = resources.getTexture("albedo.png");
 * 
 * Auto-carga y cachea recursos
 */
class ResourceManager {
public:
    ResourceManager(std::shared_ptr<MemoryAllocator> allocator);
    ~ResourceManager();

    /**
     * @brief Obtener recursos - Auto-carga y cachea
     */
    Mesh* getMesh(const std::string& name);
    Texture* getTexture(const std::string& path);
    Material* getMaterial(const std::string& name);
    
    /**
     * @brief Crear recursos predefinidos
     */
    Mesh* createCube(const std::string& name = "cube");
    Mesh* createSphere(const std::string& name = "sphere", int subdivisions = 32);
    Mesh* createPlane(const std::string& name = "plane");
    
    /**
     * @brief Reload en caliente (para desarrollo)
     */
    void reloadAll();
    void reloadTexture(const std::string& path);
    void reloadMesh(const std::string& name);
    
    /**
     * @brief Limpiar cache
     */
    void clear();
    void clearTextures();
    void clearMeshes();
    void clearMaterials();
    
    /**
     * @brief Stats
     */
    size_t getMeshCount() const { return meshes.size(); }
    size_t getTextureCount() const { return textures.size(); }
    size_t getMaterialCount() const { return materials.size(); }

private:
    std::shared_ptr<MemoryAllocator> allocator;
    
    std::unordered_map<std::string, std::unique_ptr<Mesh>> meshes;
    std::unordered_map<std::string, std::unique_ptr<Texture>> textures;
    std::unordered_map<std::string, std::unique_ptr<Material>> materials;
};

} // namespace reactor
