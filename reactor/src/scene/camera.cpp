#include "reactor/scene/camera.hpp"
#include "reactor/scene/entity.hpp"
#include "reactor/scene/transform.hpp"
#include <glm/gtc/matrix_transform.hpp>
#include <cmath>

namespace reactor {

Camera::Camera() = default;

Mat4 Camera::getViewMatrix() const {
    if (!entity) return Mat4(1.0f);
    
    Mat4 worldMat = entity->transform().getWorldMatrix();
    return glm::inverse(worldMat);
}

Mat4 Camera::getProjectionMatrix() const {
    if (projectionType == ProjectionType::Perspective) {
        return glm::perspective(glm::radians(fov), aspectRatio, nearPlane, farPlane);
    } else {
        float halfHeight = orthoSize * 0.5f;
        float halfWidth = halfHeight * aspectRatio;
        return glm::ortho(-halfWidth, halfWidth, -halfHeight, halfHeight, nearPlane, farPlane);
    }
}

Mat4 Camera::getViewProjectionMatrix() const {
    return getProjectionMatrix() * getViewMatrix();
}

void Camera::lookAt(const Vec3& target) {
    lookAt(target, Vec3(0, 1, 0));
}

void Camera::lookAt(const Vec3& target, const Vec3& up) {
    if (!entity) return;
    
    Vec3 position = entity->transform().position;
    Vec3 direction = glm::normalize(target - position);
    
    // Calcular rotación desde dirección
    float pitch = std::asin(-direction.y);
    float yaw = std::atan2(direction.x, direction.z);
    
    entity->transform().rotation = Vec3(pitch, yaw, 0.0f);
}

Camera::Ray Camera::screenPointToRay(float screenX, float screenY, float screenWidth, float screenHeight) const {
    // Convertir de screen space a NDC
    float ndcX = (2.0f * screenX) / screenWidth - 1.0f;
    float ndcY = 1.0f - (2.0f * screenY) / screenHeight;
    
    // Ray en clip space
    Vec4 rayClip(ndcX, ndcY, -1.0f, 1.0f);
    
    // Ray en view space
    Mat4 invProj = glm::inverse(getProjectionMatrix());
    Vec4 rayView = invProj * rayClip;
    rayView = Vec4(rayView.x, rayView.y, -1.0f, 0.0f);
    
    // Ray en world space
    Mat4 invView = glm::inverse(getViewMatrix());
    Vec4 rayWorld = invView * rayView;
    Vec3 rayDirection = glm::normalize(Vec3(rayWorld));
    
    Ray ray;
    ray.origin = entity ? entity->transform().position : Vec3(0, 0, 0);
    ray.direction = rayDirection;
    
    return ray;
}

} // namespace reactor
