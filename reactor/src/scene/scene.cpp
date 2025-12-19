#include "reactor/scene/scene.hpp"
#include <iostream>
#include <functional>

namespace reactor {

Scene::Scene(const std::string& name)
    : sceneName(name) {
}

Scene::~Scene() {
    destroy();
}

Entity* Scene::createEntity(const std::string& name) {
    auto entity = std::make_unique<Entity>(this, name);
    auto ptr = entity.get();
    rootEntities.push_back(std::move(entity));
    
    std::cout << "[Scene] Created entity: " << name << std::endl;
    
    return ptr;
}

Entity* Scene::findEntity(const std::string& name) {
    for (auto& entity : rootEntities) {
        if (entity->name() == name) {
            return entity.get();
        }
    }
    return nullptr;
}

const Entity* Scene::findEntity(const std::string& name) const {
    for (const auto& entity : rootEntities) {
        if (entity->name() == name) {
            return entity.get();
        }
    }
    return nullptr;
}

void Scene::start() {
    if (hasStarted) return;
    
    std::cout << "[Scene] Starting scene: " << sceneName << std::endl;
    
    for (auto& entity : rootEntities) {
        // TODO: Call onStart on all components
    }
    
    hasStarted = true;
}

void Scene::update(float deltaTime) {
    for (auto& entity : rootEntities) {
        if (entity->active()) {
            updateEntity(entity.get(), deltaTime);
        }
    }
}

void Scene::updateEntity(Entity* entity, float deltaTime) {
    // TODO: Call onUpdate on all components
    
    // Update children
    for (const auto& child : entity->children()) {
        if (child->active()) {
            updateEntity(child.get(), deltaTime);
        }
    }
}

void Scene::destroy() {
    std::cout << "[Scene] Destroying scene: " << sceneName << std::endl;
    rootEntities.clear();
    hasStarted = false;
}

size_t Scene::entityCount() const {
    size_t count = rootEntities.size();
    
    // Count children recursively
    std::function<size_t(const Entity*)> countChildren = [&](const Entity* entity) -> size_t {
        size_t childCount = entity->children().size();
        for (const auto& child : entity->children()) {
            childCount += countChildren(child.get());
        }
        return childCount;
    };
    
    for (const auto& entity : rootEntities) {
        count += countChildren(entity.get());
    }
    
    return count;
}

} // namespace reactor
