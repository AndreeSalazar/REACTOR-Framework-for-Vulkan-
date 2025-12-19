#pragma once

#include <glm/glm.hpp>
#include <memory>
#include <vector>

namespace reactor::sdf {

/**
 * @brief Base class for all SDF (Signed Distance Field) primitives
 * 
 * Killer Triangle System - Rendering sin triángulos tradicionales
 * Todas las primitivas son funciones matemáticas puras
 */
class SDFPrimitive {
public:
    virtual ~SDFPrimitive() = default;
    
    /**
     * @brief Evalúa la distancia signed al punto dado
     * @param p Punto en world space
     * @return Distancia signed (negativo = interior, positivo = exterior)
     */
    virtual float evaluate(const glm::vec3& p) const = 0;
    
    /**
     * @brief Calcula la normal en el punto dado
     * @param p Punto en world space
     * @return Normal normalizada
     */
    virtual glm::vec3 getNormal(const glm::vec3& p) const;
    
    // Transformaciones
    glm::vec3 position{0.0f};
    glm::vec3 rotation{0.0f};
    glm::vec3 scale{1.0f};
    
    // Material ID
    int materialID = 0;
};

/**
 * @brief Esfera SDF - Geometría perfecta sin vértices
 */
class SphereSDF : public SDFPrimitive {
public:
    explicit SphereSDF(float radius = 1.0f) : radius(radius) {}
    
    float evaluate(const glm::vec3& p) const override;
    
    float radius;
};

/**
 * @brief Box SDF - Cubo perfecto matemático
 */
class BoxSDF : public SDFPrimitive {
public:
    explicit BoxSDF(const glm::vec3& size = glm::vec3(1.0f)) : size(size) {}
    
    float evaluate(const glm::vec3& p) const override;
    
    glm::vec3 size;
};

/**
 * @brief Torus SDF - Dona perfecta
 */
class TorusSDF : public SDFPrimitive {
public:
    TorusSDF(float majorRadius = 1.0f, float minorRadius = 0.3f) 
        : majorRadius(majorRadius), minorRadius(minorRadius) {}
    
    float evaluate(const glm::vec3& p) const override;
    
    float majorRadius;
    float minorRadius;
};

/**
 * @brief Capsule SDF - Cápsula perfecta
 */
class CapsuleSDF : public SDFPrimitive {
public:
    CapsuleSDF(const glm::vec3& a = glm::vec3(0, -1, 0), 
               const glm::vec3& b = glm::vec3(0, 1, 0),
               float radius = 0.3f)
        : pointA(a), pointB(b), radius(radius) {}
    
    float evaluate(const glm::vec3& p) const override;
    
    glm::vec3 pointA;
    glm::vec3 pointB;
    float radius;
};

/**
 * @brief Cylinder SDF - Cilindro perfecto
 */
class CylinderSDF : public SDFPrimitive {
public:
    CylinderSDF(float radius = 0.5f, float height = 2.0f)
        : radius(radius), height(height) {}
    
    float evaluate(const glm::vec3& p) const override;
    
    float radius;
    float height;
};

/**
 * @brief Plane SDF - Plano infinito
 */
class PlaneSDF : public SDFPrimitive {
public:
    explicit PlaneSDF(const glm::vec3& normal = glm::vec3(0, 1, 0))
        : normal(glm::normalize(normal)) {}
    
    float evaluate(const glm::vec3& p) const override;
    
    glm::vec3 normal;
};

/**
 * @brief Operaciones CSG (Constructive Solid Geometry)
 */
namespace operations {

/**
 * @brief Union de dos SDFs
 */
inline float opUnion(float d1, float d2) {
    return glm::min(d1, d2);
}

/**
 * @brief Subtraction de dos SDFs
 */
inline float opSubtraction(float d1, float d2) {
    return glm::max(-d1, d2);
}

/**
 * @brief Intersection de dos SDFs
 */
inline float opIntersection(float d1, float d2) {
    return glm::max(d1, d2);
}

/**
 * @brief Smooth Union - blend orgánico
 */
inline float opSmoothUnion(float d1, float d2, float k) {
    float h = glm::clamp(0.5f + 0.5f * (d2 - d1) / k, 0.0f, 1.0f);
    return glm::mix(d2, d1, h) - k * h * (1.0f - h);
}

/**
 * @brief Smooth Subtraction
 */
inline float opSmoothSubtraction(float d1, float d2, float k) {
    float h = glm::clamp(0.5f - 0.5f * (d2 + d1) / k, 0.0f, 1.0f);
    return glm::mix(d2, -d1, h) + k * h * (1.0f - h);
}

/**
 * @brief Smooth Intersection
 */
inline float opSmoothIntersection(float d1, float d2, float k) {
    float h = glm::clamp(0.5f - 0.5f * (d2 - d1) / k, 0.0f, 1.0f);
    return glm::mix(d2, d1, h) + k * h * (1.0f - h);
}

} // namespace operations

/**
 * @brief Escena SDF completa - combina múltiples primitivas
 */
class SDFScene {
public:
    void addPrimitive(std::shared_ptr<SDFPrimitive> primitive);
    void removePrimitive(size_t index);
    void clear();
    
    /**
     * @brief Evalúa la escena completa (todas las primitivas)
     * @param p Punto en world space
     * @return Distancia signed al objeto más cercano
     */
    float evaluate(const glm::vec3& p) const;
    
    /**
     * @brief Obtiene el material ID del objeto más cercano
     */
    int getMaterialID(const glm::vec3& p) const;
    
    const std::vector<std::shared_ptr<SDFPrimitive>>& getPrimitives() const {
        return primitives;
    }
    
private:
    std::vector<std::shared_ptr<SDFPrimitive>> primitives;
};

} // namespace reactor::sdf
