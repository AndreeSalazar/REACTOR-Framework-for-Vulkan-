#pragma once
#include <string>
#include <vector>
#include <memory>
#include <unordered_map>
#include <typeindex>

namespace reactor {

// Forward declarations
class Scene;
class Component;
class Transform;

/**
 * @brief Entity - Objeto en la escena con componentes
 * 
 * React-style entity system:
 * auto entity = scene.createEntity("Player");
 * entity.addComponent<Transform>();
 * entity.addComponent<MeshRenderer>();
 */
class Entity {
public:
    Entity(Scene* scene, const std::string& name);
    ~Entity();

    Entity(const Entity&) = delete;
    Entity& operator=(const Entity&) = delete;
    Entity(Entity&& other) noexcept;
    Entity& operator=(Entity&& other) noexcept;

    /**
     * @brief Agregar componente - Template magic
     */
    template<typename T, typename... Args>
    T& addComponent(Args&&... args);
    
    /**
     * @brief Obtener componente
     */
    template<typename T>
    T* getComponent();
    
    template<typename T>
    const T* getComponent() const;
    
    /**
     * @brief Verificar si tiene componente
     */
    template<typename T>
    bool hasComponent() const;
    
    /**
     * @brief Remover componente
     */
    template<typename T>
    void removeComponent();
    
    /**
     * @brief Crear hijo
     */
    Entity* createChild(const std::string& name);
    
    /**
     * @brief Transform (siempre presente)
     */
    Transform& transform();
    const Transform& transform() const;
    
    /**
     * @brief Getters
     */
    const std::string& name() const { return entityName; }
    Scene* scene() const { return parentScene; }
    Entity* parent() const { return parentEntity; }
    const std::vector<std::unique_ptr<Entity>>& children() const { return childEntities; }
    
    /**
     * @brief Activar/Desactivar
     */
    void setActive(bool active) { isActive = active; }
    bool active() const { return isActive; }

private:
    Scene* parentScene{nullptr};
    Entity* parentEntity{nullptr};
    std::string entityName;
    bool isActive{true};
    
    std::unordered_map<std::type_index, std::unique_ptr<Component>> components;
    std::vector<std::unique_ptr<Entity>> childEntities;
    
    Transform* transformComponent{nullptr};
};

} // namespace reactor
