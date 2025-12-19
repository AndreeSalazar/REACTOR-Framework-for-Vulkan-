#pragma once
#include "component.hpp"
#include "../math.hpp"

namespace reactor {

/**
 * @brief Transform - Posición, rotación, escala
 * 
 * React-style transform:
 * transform.position = Vec3(1, 2, 3);
 * transform.rotation = Vec3(0, 45, 0);
 * transform.scale = Vec3(1, 1, 1);
 */
class Transform : public Component {
public:
    Transform();
    ~Transform() override = default;

    /**
     * @brief Propiedades públicas (React-style)
     */
    Vec3 position{0.0f, 0.0f, 0.0f};
    Vec3 rotation{0.0f, 0.0f, 0.0f};  // Euler angles en radianes
    Vec3 scale{1.0f, 1.0f, 1.0f};
    
    /**
     * @brief Obtener matriz de transformación
     */
    Mat4 getLocalMatrix() const;
    Mat4 getWorldMatrix() const;
    
    /**
     * @brief Helpers para rotación
     */
    void setRotationDegrees(float x, float y, float z);
    Vec3 getRotationDegrees() const;
    
    /**
     * @brief Direcciones
     */
    Vec3 forward() const;
    Vec3 right() const;
    Vec3 up() const;
    
    /**
     * @brief Transformar punto/dirección
     */
    Vec3 transformPoint(const Vec3& point) const;
    Vec3 transformDirection(const Vec3& direction) const;
};

} // namespace reactor
