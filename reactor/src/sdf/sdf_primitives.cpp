#include "reactor/sdf/sdf_primitives.hpp"
#include <algorithm>

namespace reactor::sdf {

// ============================================================================
// SDFPrimitive Base
// ============================================================================

glm::vec3 SDFPrimitive::getNormal(const glm::vec3& p) const {
    const float h = 0.0001f;
    
    return glm::normalize(glm::vec3(
        evaluate(p + glm::vec3(h, 0, 0)) - evaluate(p - glm::vec3(h, 0, 0)),
        evaluate(p + glm::vec3(0, h, 0)) - evaluate(p - glm::vec3(0, h, 0)),
        evaluate(p + glm::vec3(0, 0, h)) - evaluate(p - glm::vec3(0, 0, h))
    ));
}

// ============================================================================
// SphereSDF
// ============================================================================

float SphereSDF::evaluate(const glm::vec3& p) const {
    glm::vec3 localP = p - position;
    return glm::length(localP) - radius;
}

// ============================================================================
// BoxSDF
// ============================================================================

float BoxSDF::evaluate(const glm::vec3& p) const {
    glm::vec3 localP = p - position;
    glm::vec3 q = glm::abs(localP) - size;
    return glm::length(glm::max(q, glm::vec3(0.0f))) + 
           glm::min(glm::max(q.x, glm::max(q.y, q.z)), 0.0f);
}

// ============================================================================
// TorusSDF
// ============================================================================

float TorusSDF::evaluate(const glm::vec3& p) const {
    glm::vec3 localP = p - position;
    glm::vec2 q(glm::length(glm::vec2(localP.x, localP.z)) - majorRadius, localP.y);
    return glm::length(q) - minorRadius;
}

// ============================================================================
// CapsuleSDF
// ============================================================================

float CapsuleSDF::evaluate(const glm::vec3& p) const {
    glm::vec3 localP = p - position;
    glm::vec3 pa = localP - pointA;
    glm::vec3 ba = pointB - pointA;
    float h = glm::clamp(glm::dot(pa, ba) / glm::dot(ba, ba), 0.0f, 1.0f);
    return glm::length(pa - ba * h) - radius;
}

// ============================================================================
// CylinderSDF
// ============================================================================

float CylinderSDF::evaluate(const glm::vec3& p) const {
    glm::vec3 localP = p - position;
    float d = glm::length(glm::vec2(localP.x, localP.z)) - radius;
    d = glm::max(d, glm::abs(localP.y) - height * 0.5f);
    return d;
}

// ============================================================================
// PlaneSDF
// ============================================================================

float PlaneSDF::evaluate(const glm::vec3& p) const {
    glm::vec3 localP = p - position;
    return glm::dot(localP, normal);
}

// ============================================================================
// SDFScene
// ============================================================================

void SDFScene::addPrimitive(std::shared_ptr<SDFPrimitive> primitive) {
    primitives.push_back(primitive);
}

void SDFScene::removePrimitive(size_t index) {
    if (index < primitives.size()) {
        primitives.erase(primitives.begin() + index);
    }
}

void SDFScene::clear() {
    primitives.clear();
}

float SDFScene::evaluate(const glm::vec3& p) const {
    if (primitives.empty()) {
        return 1000.0f; // Muy lejos si no hay primitivas
    }
    
    float minDist = primitives[0]->evaluate(p);
    
    for (size_t i = 1; i < primitives.size(); i++) {
        float dist = primitives[i]->evaluate(p);
        minDist = operations::opUnion(minDist, dist);
    }
    
    return minDist;
}

int SDFScene::getMaterialID(const glm::vec3& p) const {
    if (primitives.empty()) {
        return 0;
    }
    
    float minDist = primitives[0]->evaluate(p);
    int materialID = primitives[0]->materialID;
    
    for (size_t i = 1; i < primitives.size(); i++) {
        float dist = primitives[i]->evaluate(p);
        if (dist < minDist) {
            minDist = dist;
            materialID = primitives[i]->materialID;
        }
    }
    
    return materialID;
}

} // namespace reactor::sdf
