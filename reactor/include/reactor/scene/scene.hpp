#pragma once
#include "entity.hpp"
#include <string>
#include <vector>
#include <memory>

namespace reactor {

/**
 * @brief Scene - Contenedor de entidades
 * 
 * Uso ultra simple:
 * Scene scene;
 * auto player = scene.createEntity("Player");
 * player->addComponent<Transform>();
 * scene.update(deltaTime);
 */
class Scene {
public:
    Scene(const std::string& name = "Untitled");
    ~Scene();

    Scene(const Scene&) = delete;
    Scene& operator=(const Scene&) = delete;

    /**
     * @brief Crear entidad
     */
    Entity* createEntity(const std::string& name);
    
    /**
     * @brief Buscar entidad
     */
    Entity* findEntity(const std::string& name);
    const Entity* findEntity(const std::string& name) const;
    
    /**
     * @brief Obtener todas las entidades
     */
    const std::vector<std::unique_ptr<Entity>>& entities() const { return rootEntities; }
    
    /**
     * @brief Lifecycle
     */
    void start();
    void update(float deltaTime);
    void destroy();
    
    /**
     * @brief Getters
     */
    const std::string& name() const { return sceneName; }
    void setName(const std::string& name) { sceneName = name; }
    
    /**
     * @brief Stats
     */
    size_t entityCount() const;

private:
    std::string sceneName;
    std::vector<std::unique_ptr<Entity>> rootEntities;
    bool hasStarted{false};
    
    void updateEntity(Entity* entity, float deltaTime);
};

} // namespace reactor
