#pragma once
#include "../math.hpp"
#include <vector>
#include <string>

namespace reactor {

/**
 * @brief DebugRenderer - Renderizado de debug (líneas, esferas, cajas)
 * 
 * Uso simple:
 * DebugRenderer debug;
 * debug.drawLine(Vec3(0,0,0), Vec3(1,1,1), Vec3(1,0,0));
 * debug.drawSphere(Vec3(0,0,0), 1.0f, Vec3(0,1,0));
 * debug.render();
 */
class DebugRenderer {
public:
    DebugRenderer();
    ~DebugRenderer();
    
    /**
     * @brief Primitives
     */
    void drawLine(const Vec3& start, const Vec3& end, const Vec3& color = Vec3(1, 1, 1));
    void drawRay(const Vec3& origin, const Vec3& direction, float length, const Vec3& color = Vec3(1, 1, 0));
    
    void drawBox(const Vec3& center, const Vec3& size, const Vec3& color = Vec3(0, 1, 0));
    void drawSphere(const Vec3& center, float radius, const Vec3& color = Vec3(0, 0, 1));
    void drawCapsule(const Vec3& start, const Vec3& end, float radius, const Vec3& color = Vec3(1, 0, 1));
    
    void drawGrid(const Vec3& center, float size, int divisions, const Vec3& color = Vec3(0.5f, 0.5f, 0.5f));
    void drawAxis(const Vec3& origin, float length = 1.0f);
    
    /**
     * @brief Text
     */
    void drawText(const Vec3& position, const std::string& text, const Vec3& color = Vec3(1, 1, 1));
    
    /**
     * @brief Render
     */
    void render(const Mat4& viewProjection);
    void clear();
    
    /**
     * @brief Settings
     */
    void setLineWidth(float width) { lineWidth = width; }
    void setDepthTest(bool enabled) { depthTest = enabled; }

private:
    struct Line {
        Vec3 start;
        Vec3 end;
        Vec3 color;
    };
    
    std::vector<Line> lines;
    float lineWidth{1.0f};
    bool depthTest{true};
};

} // namespace reactor
