#include "reactor/tools/debug_renderer.hpp"
#include <iostream>
#include <glm/gtc/constants.hpp>

namespace reactor {

DebugRenderer::DebugRenderer() {
    std::cout << "[DebugRenderer] Created" << std::endl;
}

DebugRenderer::~DebugRenderer() = default;

void DebugRenderer::drawLine(const Vec3& start, const Vec3& end, const Vec3& color) {
    lines.push_back({start, end, color});
}

void DebugRenderer::drawRay(const Vec3& origin, const Vec3& direction, float length, const Vec3& color) {
    drawLine(origin, origin + direction * length, color);
}

void DebugRenderer::drawBox(const Vec3& center, const Vec3& size, const Vec3& color) {
    Vec3 halfSize = size * 0.5f;
    
    // Bottom face
    drawLine(center + Vec3(-halfSize.x, -halfSize.y, -halfSize.z), center + Vec3(halfSize.x, -halfSize.y, -halfSize.z), color);
    drawLine(center + Vec3(halfSize.x, -halfSize.y, -halfSize.z), center + Vec3(halfSize.x, -halfSize.y, halfSize.z), color);
    drawLine(center + Vec3(halfSize.x, -halfSize.y, halfSize.z), center + Vec3(-halfSize.x, -halfSize.y, halfSize.z), color);
    drawLine(center + Vec3(-halfSize.x, -halfSize.y, halfSize.z), center + Vec3(-halfSize.x, -halfSize.y, -halfSize.z), color);
    
    // Top face
    drawLine(center + Vec3(-halfSize.x, halfSize.y, -halfSize.z), center + Vec3(halfSize.x, halfSize.y, -halfSize.z), color);
    drawLine(center + Vec3(halfSize.x, halfSize.y, -halfSize.z), center + Vec3(halfSize.x, halfSize.y, halfSize.z), color);
    drawLine(center + Vec3(halfSize.x, halfSize.y, halfSize.z), center + Vec3(-halfSize.x, halfSize.y, halfSize.z), color);
    drawLine(center + Vec3(-halfSize.x, halfSize.y, halfSize.z), center + Vec3(-halfSize.x, halfSize.y, -halfSize.z), color);
    
    // Vertical edges
    drawLine(center + Vec3(-halfSize.x, -halfSize.y, -halfSize.z), center + Vec3(-halfSize.x, halfSize.y, -halfSize.z), color);
    drawLine(center + Vec3(halfSize.x, -halfSize.y, -halfSize.z), center + Vec3(halfSize.x, halfSize.y, -halfSize.z), color);
    drawLine(center + Vec3(halfSize.x, -halfSize.y, halfSize.z), center + Vec3(halfSize.x, halfSize.y, halfSize.z), color);
    drawLine(center + Vec3(-halfSize.x, -halfSize.y, halfSize.z), center + Vec3(-halfSize.x, halfSize.y, halfSize.z), color);
}

void DebugRenderer::drawSphere(const Vec3& center, float radius, const Vec3& color) {
    const int segments = 16;
    const float angleStep = glm::two_pi<float>() / segments;
    
    // XY circle
    for (int i = 0; i < segments; i++) {
        float angle1 = i * angleStep;
        float angle2 = (i + 1) * angleStep;
        Vec3 p1 = center + Vec3(cos(angle1) * radius, sin(angle1) * radius, 0);
        Vec3 p2 = center + Vec3(cos(angle2) * radius, sin(angle2) * radius, 0);
        drawLine(p1, p2, color);
    }
    
    // XZ circle
    for (int i = 0; i < segments; i++) {
        float angle1 = i * angleStep;
        float angle2 = (i + 1) * angleStep;
        Vec3 p1 = center + Vec3(cos(angle1) * radius, 0, sin(angle1) * radius);
        Vec3 p2 = center + Vec3(cos(angle2) * radius, 0, sin(angle2) * radius);
        drawLine(p1, p2, color);
    }
    
    // YZ circle
    for (int i = 0; i < segments; i++) {
        float angle1 = i * angleStep;
        float angle2 = (i + 1) * angleStep;
        Vec3 p1 = center + Vec3(0, cos(angle1) * radius, sin(angle1) * radius);
        Vec3 p2 = center + Vec3(0, cos(angle2) * radius, sin(angle2) * radius);
        drawLine(p1, p2, color);
    }
}

void DebugRenderer::drawCapsule(const Vec3& start, const Vec3& end, float radius, const Vec3& color) {
    drawSphere(start, radius, color);
    drawSphere(end, radius, color);
    drawLine(start, end, color);
}

void DebugRenderer::drawGrid(const Vec3& center, float size, int divisions, const Vec3& color) {
    float step = size / divisions;
    float halfSize = size * 0.5f;
    
    for (int i = 0; i <= divisions; i++) {
        float offset = -halfSize + i * step;
        drawLine(center + Vec3(offset, 0, -halfSize), center + Vec3(offset, 0, halfSize), color);
        drawLine(center + Vec3(-halfSize, 0, offset), center + Vec3(halfSize, 0, offset), color);
    }
}

void DebugRenderer::drawAxis(const Vec3& origin, float length) {
    drawLine(origin, origin + Vec3(length, 0, 0), Vec3(1, 0, 0));  // X - Red
    drawLine(origin, origin + Vec3(0, length, 0), Vec3(0, 1, 0));  // Y - Green
    drawLine(origin, origin + Vec3(0, 0, length), Vec3(0, 0, 1));  // Z - Blue
}

void DebugRenderer::drawText(const Vec3& position, const std::string& text, const Vec3& color) {
    std::cout << "[DebugRenderer] Text at (" << position.x << ", " << position.y << ", " << position.z << "): " << text << std::endl;
}

void DebugRenderer::render(const Mat4& viewProjection) {
    if (lines.empty()) return;
    
    std::cout << "[DebugRenderer] Rendering " << lines.size() << " lines" << std::endl;
    // TODO: Actual Vulkan rendering
}

void DebugRenderer::clear() {
    lines.clear();
}

} // namespace reactor
