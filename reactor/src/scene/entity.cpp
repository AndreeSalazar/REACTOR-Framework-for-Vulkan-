#include "reactor/scene/entity.hpp"
#include "reactor/scene/entity_impl.hpp"
#include "reactor/scene/scene.hpp"
#include "reactor/scene/transform.hpp"
#include <iostream>

namespace reactor {

Entity::Entity(Scene* scene, const std::string& name)
    : parentScene(scene), entityName(name) {
    // Transform siempre presente
    transformComponent = &addComponent<Transform>();
}

Entity::~Entity() = default;

Entity::Entity(Entity&& other) noexcept
    : parentScene(other.parentScene)
    , parentEntity(other.parentEntity)
    , entityName(std::move(other.entityName))
    , isActive(other.isActive)
    , components(std::move(other.components))
    , childEntities(std::move(other.childEntities))
    , transformComponent(other.transformComponent) {
    other.parentScene = nullptr;
    other.parentEntity = nullptr;
    other.transformComponent = nullptr;
}

Entity& Entity::operator=(Entity&& other) noexcept {
    if (this != &other) {
        parentScene = other.parentScene;
        parentEntity = other.parentEntity;
        entityName = std::move(other.entityName);
        isActive = other.isActive;
        components = std::move(other.components);
        childEntities = std::move(other.childEntities);
        transformComponent = other.transformComponent;
        
        other.parentScene = nullptr;
        other.parentEntity = nullptr;
        other.transformComponent = nullptr;
    }
    return *this;
}

Entity* Entity::createChild(const std::string& name) {
    auto child = std::make_unique<Entity>(parentScene, name);
    child->parentEntity = this;
    auto ptr = child.get();
    childEntities.push_back(std::move(child));
    return ptr;
}

Transform& Entity::transform() {
    return *transformComponent;
}

const Transform& Entity::transform() const {
    return *transformComponent;
}

} // namespace reactor
