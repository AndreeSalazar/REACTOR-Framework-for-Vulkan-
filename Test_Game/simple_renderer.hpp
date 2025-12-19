#pragma once
#include "reactor/reactor.hpp"

namespace test_game {

/**
 * @brief SimpleRenderer - Módulo de rendering simple para Test_Game
 * 
 * Fácil de usar, modificar y eliminar.
 * Estilo modular como Blender.
 */
class SimpleRenderer {
public:
    SimpleRenderer(reactor::VulkanContext& ctx, reactor::Window& window);
    ~SimpleRenderer();
    
    // Fácil de usar
    void beginFrame();
    void drawCube(const reactor::Mat4& mvp, const reactor::Vec3& color);
    void endFrame();
    
    // Fácil de modificar
    void setClearColor(float r, float g, float b);
    void setWireframe(bool enabled);
    
private:
    reactor::VulkanContext& ctx;
    reactor::Window& window;
    reactor::Vec3 clearColor{0.1f, 0.1f, 0.1f};
    
    // EasyRenderer (FASE 8) hace todo el trabajo
    reactor::EasyRenderer* easyRenderer{nullptr};
    
    // Geometría del cubo
    std::vector<float> cubeVertices;
    std::vector<uint16_t> cubeIndices;
    
    void cleanup();
};

} // namespace test_game
