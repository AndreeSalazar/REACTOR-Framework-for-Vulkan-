#pragma once
#include "entity.hpp"
#include "component.hpp"
#include <iostream>

namespace reactor {

template<typename T, typename... Args>
T& Entity::addComponent(Args&&... args) {
    static_assert(std::is_base_of<Component, T>::value, "T must inherit from Component");
    
    auto typeIndex = std::type_index(typeid(T));
    
    if (components.find(typeIndex) != components.end()) {
        std::cerr << "[Entity] Component already exists: " << typeid(T).name() << std::endl;
        return *static_cast<T*>(components[typeIndex].get());
    }
    
    auto component = std::make_unique<T>(std::forward<Args>(args)...);
    component->setEntity(this);
    auto ptr = component.get();
    components[typeIndex] = std::move(component);
    
    return *ptr;
}

template<typename T>
T* Entity::getComponent() {
    static_assert(std::is_base_of<Component, T>::value, "T must inherit from Component");
    
    auto typeIndex = std::type_index(typeid(T));
    auto it = components.find(typeIndex);
    
    if (it != components.end()) {
        return static_cast<T*>(it->second.get());
    }
    
    return nullptr;
}

template<typename T>
const T* Entity::getComponent() const {
    static_assert(std::is_base_of<Component, T>::value, "T must inherit from Component");
    
    auto typeIndex = std::type_index(typeid(T));
    auto it = components.find(typeIndex);
    
    if (it != components.end()) {
        return static_cast<const T*>(it->second.get());
    }
    
    return nullptr;
}

template<typename T>
bool Entity::hasComponent() const {
    static_assert(std::is_base_of<Component, T>::value, "T must inherit from Component");
    
    auto typeIndex = std::type_index(typeid(T));
    return components.find(typeIndex) != components.end();
}

template<typename T>
void Entity::removeComponent() {
    static_assert(std::is_base_of<Component, T>::value, "T must inherit from Component");
    
    auto typeIndex = std::type_index(typeid(T));
    components.erase(typeIndex);
}

} // namespace reactor
