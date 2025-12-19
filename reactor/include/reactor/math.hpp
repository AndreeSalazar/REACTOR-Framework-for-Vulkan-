#pragma once

// GLM integration for REACTOR
#define GLM_FORCE_RADIANS
#define GLM_FORCE_DEPTH_ZERO_TO_ONE
#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/type_ptr.hpp>

namespace reactor {

// Type aliases for convenience
using Vec2 = glm::vec2;
using Vec3 = glm::vec3;
using Vec4 = glm::vec4;
using Mat3 = glm::mat3;
using Mat4 = glm::mat4;

// Simple transform helper (deprecated - use scene/Transform component)
struct SimpleTransform {
    Vec3 position{0.0f, 0.0f, 0.0f};
    Vec3 rotation{0.0f, 0.0f, 0.0f};  // Euler angles in radians
    Vec3 scale{1.0f, 1.0f, 1.0f};
    
    Mat4 getMatrix() const {
        Mat4 mat = glm::mat4(1.0f);
        mat = glm::translate(mat, position);
        mat = glm::rotate(mat, rotation.x, Vec3(1, 0, 0));
        mat = glm::rotate(mat, rotation.y, Vec3(0, 1, 0));
        mat = glm::rotate(mat, rotation.z, Vec3(0, 0, 1));
        mat = glm::scale(mat, scale);
        return mat;
    }
};

// Simple camera helper (deprecated - use scene/Camera component)
struct SimpleCamera {
    Vec3 position{0.0f, 2.0f, 5.0f};
    Vec3 target{0.0f, 0.0f, 0.0f};
    Vec3 up{0.0f, 1.0f, 0.0f};
    float fov{45.0f};
    float aspectRatio{16.0f / 9.0f};
    float nearPlane{0.1f};
    float farPlane{100.0f};
    
    Mat4 getViewMatrix() const {
        return glm::lookAt(position, target, up);
    }
    
    Mat4 getProjectionMatrix() const {
        return glm::perspective(glm::radians(fov), aspectRatio, nearPlane, farPlane);
    }
};

// Uniform Buffer Object for MVP matrices
struct UniformBufferObject {
    alignas(16) Mat4 model;
    alignas(16) Mat4 view;
    alignas(16) Mat4 proj;
};

} // namespace reactor
