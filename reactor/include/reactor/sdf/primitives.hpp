#pragma once
#include <glm/glm.hpp>
#include <vector>
#include <memory>

namespace reactor {
namespace sdf {

/**
 * @brief SDF Primitives - Formas matemáticas básicas
 * 
 * Basado en ADead-Vector3D:
 * - Renderizado puramente matemático
 * - Infinitamente escalable
 * - ~1KB vs ~1MB (mallas)
 * - Anti-aliasing perfecto
 */

// Forward declarations
class SDFPrimitive;
class SDFOperation;

/**
 * @brief Sphere SDF
 */
class Sphere {
public:
    glm::vec3 center{0.0f, 0.0f, 0.0f};
    float radius = 1.0f;
    glm::vec3 color{1.0f, 1.0f, 1.0f};
    
    Sphere() = default;
    Sphere(const glm::vec3& c, float r) : center(c), radius(r) {}
    
    // GLSL: length(p - center) - radius
    float distance(const glm::vec3& p) const;
};

/**
 * @brief Box SDF
 */
class Box {
public:
    glm::vec3 center{0.0f, 0.0f, 0.0f};
    glm::vec3 size{1.0f, 1.0f, 1.0f};
    glm::vec3 color{1.0f, 1.0f, 1.0f};
    
    Box() = default;
    Box(const glm::vec3& c, const glm::vec3& s) : center(c), size(s) {}
    
    // GLSL: max(abs(p - center) - size, 0.0)
    float distance(const glm::vec3& p) const;
};

/**
 * @brief Torus SDF
 */
class Torus {
public:
    glm::vec3 center{0.0f, 0.0f, 0.0f};
    float majorRadius = 1.0f;
    float minorRadius = 0.3f;
    glm::vec3 color{1.0f, 1.0f, 1.0f};
    
    Torus() = default;
    Torus(const glm::vec3& c, float major, float minor) 
        : center(c), majorRadius(major), minorRadius(minor) {}
    
    float distance(const glm::vec3& p) const;
};

/**
 * @brief Cylinder SDF
 */
class Cylinder {
public:
    glm::vec3 center{0.0f, 0.0f, 0.0f};
    float radius = 1.0f;
    float height = 2.0f;
    glm::vec3 color{1.0f, 1.0f, 1.0f};
    
    Cylinder() = default;
    Cylinder(const glm::vec3& c, float r, float h) 
        : center(c), radius(r), height(h) {}
    
    float distance(const glm::vec3& p) const;
};

/**
 * @brief Capsule SDF
 */
class Capsule {
public:
    glm::vec3 pointA{0.0f, -1.0f, 0.0f};
    glm::vec3 pointB{0.0f,  1.0f, 0.0f};
    float radius = 0.5f;
    glm::vec3 color{1.0f, 1.0f, 1.0f};
    
    Capsule() = default;
    Capsule(const glm::vec3& a, const glm::vec3& b, float r) 
        : pointA(a), pointB(b), radius(r) {}
    
    float distance(const glm::vec3& p) const;
};

/**
 * @brief Cone SDF
 */
class Cone {
public:
    glm::vec3 center{0.0f, 0.0f, 0.0f};
    float angle = 0.5f;  // tan(angle)
    float height = 2.0f;
    glm::vec3 color{1.0f, 1.0f, 1.0f};
    
    Cone() = default;
    Cone(const glm::vec3& c, float a, float h) 
        : center(c), angle(a), height(h) {}
    
    float distance(const glm::vec3& p) const;
};

/**
 * @brief CSG Operations (Constructive Solid Geometry)
 */
namespace operations {

// Union (A ∪ B)
inline float Union(float d1, float d2) {
    return glm::min(d1, d2);
}

// Subtraction (A - B)
inline float Subtract(float d1, float d2) {
    return glm::max(d1, -d2);
}

// Intersection (A ∩ B)
inline float Intersect(float d1, float d2) {
    return glm::max(d1, d2);
}

// Smooth Union (suave)
inline float SmoothUnion(float d1, float d2, float k) {
    float h = glm::clamp(0.5f + 0.5f * (d2 - d1) / k, 0.0f, 1.0f);
    return glm::mix(d2, d1, h) - k * h * (1.0f - h);
}

// Smooth Subtraction
inline float SmoothSubtract(float d1, float d2, float k) {
    float h = glm::clamp(0.5f - 0.5f * (d2 + d1) / k, 0.0f, 1.0f);
    return glm::mix(d2, -d1, h) + k * h * (1.0f - h);
}

// Smooth Intersection
inline float SmoothIntersect(float d1, float d2, float k) {
    float h = glm::clamp(0.5f - 0.5f * (d2 - d1) / k, 0.0f, 1.0f);
    return glm::mix(d2, d1, h) + k * h * (1.0f - h);
}

} // namespace operations

// Forward declaration - SDFScene is defined in sdf_primitives.hpp
class SDFScene;

} // namespace sdf
} // namespace reactor
