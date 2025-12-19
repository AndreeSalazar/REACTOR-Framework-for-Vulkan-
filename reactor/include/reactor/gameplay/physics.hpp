#pragma once
#include "../math.hpp"
#include <vector>
#include <memory>

namespace reactor {

// Forward declarations
class Entity;

/**
 * @brief RigidBody - Componente de física
 * 
 * Uso simple:
 * auto& rb = entity->addComponent<RigidBody>();
 * rb.mass = 1.0f;
 * rb.velocity = Vec3(0, 0, 0);
 */
class RigidBody {
public:
    RigidBody() = default;
    
    // Propiedades físicas
    float mass{1.0f};
    Vec3 velocity{0, 0, 0};
    Vec3 acceleration{0, 0, 0};
    Vec3 force{0, 0, 0};
    
    bool useGravity{true};
    bool isKinematic{false};
    float drag{0.1f};
    float angularDrag{0.05f};
    
    /**
     * @brief Aplicar fuerza
     */
    void addForce(const Vec3& f);
    void addImpulse(const Vec3& impulse);
    
    /**
     * @brief Update physics
     */
    void update(float deltaTime);
};

/**
 * @brief Collider - Base para colisionadores
 */
class Collider {
public:
    enum class Type {
        Box,
        Sphere,
        Capsule
    };
    
    Type type{Type::Box};
    Vec3 center{0, 0, 0};
    Vec3 size{1, 1, 1};  // Para Box
    float radius{0.5f};  // Para Sphere/Capsule
    float height{2.0f};  // Para Capsule
    
    bool isTrigger{false};
    
    /**
     * @brief Helpers
     */
    static Collider box(const Vec3& size);
    static Collider sphere(float radius);
    static Collider capsule(float radius, float height);
};

/**
 * @brief PhysicsWorld - Sistema de física
 * 
 * Uso simple:
 * PhysicsWorld physics;
 * physics.gravity = Vec3(0, -9.81f, 0);
 * physics.update(deltaTime);
 */
class PhysicsWorld {
public:
    PhysicsWorld();
    ~PhysicsWorld();
    
    Vec3 gravity{0, -9.81f, 0};
    
    /**
     * @brief Registrar rigidbody
     */
    void addRigidBody(RigidBody* rb);
    void removeRigidBody(RigidBody* rb);
    
    /**
     * @brief Update
     */
    void update(float deltaTime);
    
    /**
     * @brief Raycast
     */
    struct RaycastHit {
        Vec3 point;
        Vec3 normal;
        float distance;
        Entity* entity{nullptr};
    };
    
    bool raycast(const Vec3& origin, const Vec3& direction, float maxDistance, RaycastHit& hit);
    
    /**
     * @brief Stats
     */
    size_t rigidBodyCount() const { return rigidBodies.size(); }

private:
    std::vector<RigidBody*> rigidBodies;
};

} // namespace reactor
