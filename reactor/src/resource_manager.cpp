#include "reactor/resource_manager.hpp"
#include "reactor/mesh.hpp"
#include "reactor/texture.hpp"
#include "reactor/material.hpp"
#include "reactor/memory_allocator.hpp"
#include <iostream>

namespace reactor {

ResourceManager::ResourceManager(std::shared_ptr<MemoryAllocator> allocator)
    : allocator(allocator) {
}

ResourceManager::~ResourceManager() {
    clear();
}

Mesh* ResourceManager::getMesh(const std::string& name) {
    // Buscar en cache
    auto it = meshes.find(name);
    if (it != meshes.end()) {
        return it->second.get();
    }
    
    // Cargar desde archivo
    std::cout << "[ResourceManager] Cargando mesh: " << name << std::endl;
    auto mesh = std::make_unique<Mesh>(Mesh::fromFile(name, allocator));
    auto ptr = mesh.get();
    meshes[name] = std::move(mesh);
    
    return ptr;
}

Texture* ResourceManager::getTexture(const std::string& path) {
    // Buscar en cache
    auto it = textures.find(path);
    if (it != textures.end()) {
        return it->second.get();
    }
    
    // Cargar desde archivo
    std::cout << "[ResourceManager] Cargando texture: " << path << std::endl;
    auto texture = std::make_unique<Texture>(Texture::load(path, allocator));
    auto ptr = texture.get();
    textures[path] = std::move(texture);
    
    return ptr;
}

Material* ResourceManager::getMaterial(const std::string& name) {
    // Buscar en cache
    auto it = materials.find(name);
    if (it != materials.end()) {
        return it->second.get();
    }
    
    // Crear material por defecto
    std::cout << "[ResourceManager] Creando material: " << name << std::endl;
    auto material = std::make_unique<Material>(Material::pbr());
    auto ptr = material.get();
    materials[name] = std::move(material);
    
    return ptr;
}

Mesh* ResourceManager::createCube(const std::string& name) {
    std::cout << "[ResourceManager] Creando cubo: " << name << std::endl;
    auto mesh = std::make_unique<Mesh>(Mesh::cube(allocator));
    auto ptr = mesh.get();
    meshes[name] = std::move(mesh);
    return ptr;
}

Mesh* ResourceManager::createSphere(const std::string& name, int subdivisions) {
    std::cout << "[ResourceManager] Creando esfera: " << name << std::endl;
    auto mesh = std::make_unique<Mesh>(Mesh::sphere(allocator, subdivisions));
    auto ptr = mesh.get();
    meshes[name] = std::move(mesh);
    return ptr;
}

Mesh* ResourceManager::createPlane(const std::string& name) {
    std::cout << "[ResourceManager] Creando plano: " << name << std::endl;
    auto mesh = std::make_unique<Mesh>(Mesh::plane(allocator));
    auto ptr = mesh.get();
    meshes[name] = std::move(mesh);
    return ptr;
}

void ResourceManager::reloadAll() {
    std::cout << "[ResourceManager] Recargando todos los recursos..." << std::endl;
    // TODO: Implementar reload
}

void ResourceManager::reloadTexture(const std::string& path) {
    auto it = textures.find(path);
    if (it != textures.end()) {
        std::cout << "[ResourceManager] Recargando texture: " << path << std::endl;
        textures.erase(it);
        getTexture(path); // Recargar
    }
}

void ResourceManager::reloadMesh(const std::string& name) {
    auto it = meshes.find(name);
    if (it != meshes.end()) {
        std::cout << "[ResourceManager] Recargando mesh: " << name << std::endl;
        meshes.erase(it);
        getMesh(name); // Recargar
    }
}

void ResourceManager::clear() {
    std::cout << "[ResourceManager] Limpiando cache..." << std::endl;
    meshes.clear();
    textures.clear();
    materials.clear();
}

void ResourceManager::clearTextures() {
    textures.clear();
}

void ResourceManager::clearMeshes() {
    meshes.clear();
}

void ResourceManager::clearMaterials() {
    materials.clear();
}

} // namespace reactor
