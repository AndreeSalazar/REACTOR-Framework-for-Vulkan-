#pragma once
#include "component.hpp"
#include "../math.hpp"

namespace reactor {

/**
 * @brief Camera - Componente de cámara
 * 
 * Uso simple:
 * auto camera = entity.addComponent<Camera>();
 * camera.fov = 60.0f;
 * camera.nearPlane = 0.1f;
 * camera.farPlane = 100.0f;
 */
class Camera : public Component {
public:
    Camera();
    ~Camera() override = default;

    /**
     * @brief Propiedades de cámara
     */
    float fov{45.0f};           // Field of view en grados
    float aspectRatio{16.0f / 9.0f};
    float nearPlane{0.1f};
    float farPlane{100.0f};
    
    /**
     * @brief Tipo de proyección
     */
    enum class ProjectionType {
        Perspective,
        Orthographic
    };
    ProjectionType projectionType{ProjectionType::Perspective};
    
    /**
     * @brief Para ortográfica
     */
    float orthoSize{10.0f};
    
    /**
     * @brief Obtener matrices
     */
    Mat4 getViewMatrix() const;
    Mat4 getProjectionMatrix() const;
    Mat4 getViewProjectionMatrix() const;
    
    /**
     * @brief Helpers
     */
    void lookAt(const Vec3& target);
    void lookAt(const Vec3& target, const Vec3& up);
    
    /**
     * @brief Ray desde pantalla
     */
    struct Ray {
        Vec3 origin;
        Vec3 direction;
    };
    Ray screenPointToRay(float screenX, float screenY, float screenWidth, float screenHeight) const;
};

} // namespace reactor
