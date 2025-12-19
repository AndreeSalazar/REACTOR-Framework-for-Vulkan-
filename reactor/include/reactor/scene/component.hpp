#pragma once

namespace reactor {

class Entity;

/**
 * @brief Component - Base para todos los componentes
 * 
 * Sistema de componentes estilo Unity/Unreal:
 * class MyComponent : public Component {
 *     void onUpdate(float deltaTime) override;
 * };
 */
class Component {
public:
    Component() = default;
    virtual ~Component() = default;

    Component(const Component&) = delete;
    Component& operator=(const Component&) = delete;

    /**
     * @brief Lifecycle callbacks
     */
    virtual void onStart() {}
    virtual void onUpdate(float deltaTime) {}
    virtual void onDestroy() {}
    
    /**
     * @brief Entity owner
     */
    void setEntity(Entity* ent) { entity = ent; }
    Entity* getEntity() const { return entity; }

protected:
    Entity* entity{nullptr};
};

} // namespace reactor
