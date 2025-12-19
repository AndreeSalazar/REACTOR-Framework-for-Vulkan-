#include "reactor/gameplay/physics.hpp"
#include <iostream>

namespace reactor {

void RigidBody::addForce(const Vec3& f) {
    force += f;
}

void RigidBody::addImpulse(const Vec3& impulse) {
    if (mass > 0.0f) {
        velocity += impulse / mass;
    }
}

void RigidBody::update(float deltaTime) {
    if (isKinematic) return;
    
    // F = ma -> a = F/m
    if (mass > 0.0f) {
        acceleration = force / mass;
    }
    
    // Update velocity
    velocity += acceleration * deltaTime;
    
    // Apply drag
    velocity *= (1.0f - drag * deltaTime);
    
    // Reset force
    force = Vec3(0, 0, 0);
}

Collider Collider::box(const Vec3& size) {
    Collider c;
    c.type = Type::Box;
    c.size = size;
    return c;
}

Collider Collider::sphere(float radius) {
    Collider c;
    c.type = Type::Sphere;
    c.radius = radius;
    return c;
}

Collider Collider::capsule(float radius, float height) {
    Collider c;
    c.type = Type::Capsule;
    c.radius = radius;
    c.height = height;
    return c;
}

PhysicsWorld::PhysicsWorld() {
    std::cout << "[PhysicsWorld] Created" << std::endl;
}

PhysicsWorld::~PhysicsWorld() = default;

void PhysicsWorld::addRigidBody(RigidBody* rb) {
    rigidBodies.push_back(rb);
}

void PhysicsWorld::removeRigidBody(RigidBody* rb) {
    rigidBodies.erase(
        std::remove(rigidBodies.begin(), rigidBodies.end(), rb),
        rigidBodies.end()
    );
}

void PhysicsWorld::update(float deltaTime) {
    // Apply gravity and update all rigidbodies
    for (auto* rb : rigidBodies) {
        if (rb->useGravity && !rb->isKinematic) {
            rb->addForce(gravity * rb->mass);
        }
        rb->update(deltaTime);
    }
}

bool PhysicsWorld::raycast(const Vec3& origin, const Vec3& direction, float maxDistance, RaycastHit& hit) {
    // Placeholder raycast
    std::cout << "[PhysicsWorld] Raycast from (" << origin.x << ", " << origin.y << ", " << origin.z << ")" << std::endl;
    return false;
}

} // namespace reactor
