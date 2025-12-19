#include "reactor/sdf/primitives.hpp"
#include <cmath>

namespace reactor {
namespace sdf {

// Sphere implementation
float Sphere::distance(const glm::vec3& p) const {
    return glm::length(p - center) - radius;
}

// Box implementation
float Box::distance(const glm::vec3& p) const {
    glm::vec3 q = glm::abs(p - center) - size;
    return glm::length(glm::max(q, glm::vec3(0.0f))) + 
           glm::min(glm::max(q.x, glm::max(q.y, q.z)), 0.0f);
}

// Torus implementation
float Torus::distance(const glm::vec3& p) const {
    glm::vec3 q = p - center;
    glm::vec2 t = glm::vec2(glm::length(glm::vec2(q.x, q.z)) - majorRadius, q.y);
    return glm::length(t) - minorRadius;
}

// Cylinder implementation
float Cylinder::distance(const glm::vec3& p) const {
    glm::vec3 q = p - center;
    glm::vec2 d = glm::abs(glm::vec2(glm::length(glm::vec2(q.x, q.z)), q.y)) - glm::vec2(radius, height);
    return glm::min(glm::max(d.x, d.y), 0.0f) + glm::length(glm::max(d, glm::vec2(0.0f)));
}

// Capsule implementation
float Capsule::distance(const glm::vec3& p) const {
    glm::vec3 pa = p - pointA;
    glm::vec3 ba = pointB - pointA;
    float h = glm::clamp(glm::dot(pa, ba) / glm::dot(ba, ba), 0.0f, 1.0f);
    return glm::length(pa - ba * h) - radius;
}

// Cone implementation
float Cone::distance(const glm::vec3& p) const {
    glm::vec3 q = p - center;
    float d1 = glm::length(glm::vec2(q.x, q.z));
    float d2 = -q.y - height;
    float d3 = d1 * std::cos(angle) - q.y * std::sin(angle);
    return glm::max(glm::max(d2, d3), -q.y);
}

} // namespace sdf
} // namespace reactor
