#define GLM_ENABLE_EXPERIMENTAL
#include "reactor/scene/transform.hpp"
#include "reactor/scene/entity.hpp"
#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtx/euler_angles.hpp>

namespace reactor {

Transform::Transform() = default;

Mat4 Transform::getLocalMatrix() const {
    Mat4 mat(1.0f);
    
    // Translate
    mat = glm::translate(mat, position);
    
    // Rotate (Euler XYZ)
    mat = glm::rotate(mat, rotation.x, Vec3(1, 0, 0));
    mat = glm::rotate(mat, rotation.y, Vec3(0, 1, 0));
    mat = glm::rotate(mat, rotation.z, Vec3(0, 0, 1));
    
    // Scale
    mat = glm::scale(mat, scale);
    
    return mat;
}

Mat4 Transform::getWorldMatrix() const {
    Mat4 localMat = getLocalMatrix();
    
    // Si tiene padre, multiplicar por matriz del padre
    if (entity && entity->parent()) {
        return entity->parent()->transform().getWorldMatrix() * localMat;
    }
    
    return localMat;
}

void Transform::setRotationDegrees(float x, float y, float z) {
    rotation.x = glm::radians(x);
    rotation.y = glm::radians(y);
    rotation.z = glm::radians(z);
}

Vec3 Transform::getRotationDegrees() const {
    return Vec3(
        glm::degrees(rotation.x),
        glm::degrees(rotation.y),
        glm::degrees(rotation.z)
    );
}

Vec3 Transform::forward() const {
    Mat4 mat = getLocalMatrix();
    return glm::normalize(Vec3(mat[2]));
}

Vec3 Transform::right() const {
    Mat4 mat = getLocalMatrix();
    return glm::normalize(Vec3(mat[0]));
}

Vec3 Transform::up() const {
    Mat4 mat = getLocalMatrix();
    return glm::normalize(Vec3(mat[1]));
}

Vec3 Transform::transformPoint(const Vec3& point) const {
    Mat4 mat = getWorldMatrix();
    Vec4 p = mat * Vec4(point, 1.0f);
    return Vec3(p) / p.w;
}

Vec3 Transform::transformDirection(const Vec3& direction) const {
    Mat4 mat = getWorldMatrix();
    return glm::normalize(Vec3(mat * Vec4(direction, 0.0f)));
}

} // namespace reactor
