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

// SDFScene::Builder implementation
SDFScene::Builder& SDFScene::Builder::addSphere(const Sphere& sphere) {
    Primitive prim;
    prim.type = Primitive::Type::Sphere;
    prim.center = sphere.center;
    prim.params = glm::vec3(sphere.radius, 0.0f, 0.0f);
    prim.color = sphere.color;
    primitives.push_back(prim);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::addBox(const Box& box) {
    Primitive prim;
    prim.type = Primitive::Type::Box;
    prim.center = box.center;
    prim.params = box.size;
    prim.color = box.color;
    primitives.push_back(prim);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::addTorus(const Torus& torus) {
    Primitive prim;
    prim.type = Primitive::Type::Torus;
    prim.center = torus.center;
    prim.params = glm::vec3(torus.majorRadius, torus.minorRadius, 0.0f);
    prim.color = torus.color;
    primitives.push_back(prim);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::addCylinder(const Cylinder& cylinder) {
    Primitive prim;
    prim.type = Primitive::Type::Cylinder;
    prim.center = cylinder.center;
    prim.params = glm::vec3(cylinder.radius, cylinder.height, 0.0f);
    prim.color = cylinder.color;
    primitives.push_back(prim);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::addCapsule(const Capsule& capsule) {
    Primitive prim;
    prim.type = Primitive::Type::Capsule;
    prim.center = capsule.pointA;
    prim.params = glm::vec3(capsule.pointB - capsule.pointA);
    prim.color = capsule.color;
    primitives.push_back(prim);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::addCone(const Cone& cone) {
    Primitive prim;
    prim.type = Primitive::Type::Cone;
    prim.center = cone.center;
    prim.params = glm::vec3(cone.angle, cone.height, 0.0f);
    prim.color = cone.color;
    primitives.push_back(prim);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::unionOp() {
    Operation op;
    op.type = Operation::Type::Union;
    operations.push_back(op);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::subtractOp() {
    Operation op;
    op.type = Operation::Type::Subtract;
    operations.push_back(op);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::intersectOp() {
    Operation op;
    op.type = Operation::Type::Intersect;
    operations.push_back(op);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::smoothUnionOp(float k) {
    Operation op;
    op.type = Operation::Type::SmoothUnion;
    op.smoothness = k;
    operations.push_back(op);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::smoothSubtractOp(float k) {
    Operation op;
    op.type = Operation::Type::SmoothSubtract;
    op.smoothness = k;
    operations.push_back(op);
    return *this;
}

SDFScene::Builder& SDFScene::Builder::smoothIntersectOp(float k) {
    Operation op;
    op.type = Operation::Type::SmoothIntersect;
    op.smoothness = k;
    operations.push_back(op);
    return *this;
}

SDFScene SDFScene::Builder::build() {
    SDFScene scene;
    scene.primitives = std::move(primitives);
    scene.operations = std::move(operations);
    return scene;
}

SDFScene::Builder SDFScene::create() {
    return Builder();
}

} // namespace sdf
} // namespace reactor
