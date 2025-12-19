#include "simple_renderer.hpp"
#include "reactor/rendering/easy_renderer.hpp"
#include <iostream>
#include <array>
#include <fstream>
#include <vector>

namespace test_game {

SimpleRenderer::SimpleRenderer(reactor::VulkanContext& ctx, reactor::Window& window)
    : ctx(ctx), window(window) {
    
    std::cout << "[SimpleRenderer] Inicializando con EasyRenderer..." << std::endl;
    
    // Crear EasyRenderer (FASE 8) - hace todo el trabajo pesado
    easyRenderer = new reactor::EasyRenderer(ctx, window);
    
    // Crear geometría del cubo
    reactor::QuickDraw::cube(cubeVertices, cubeIndices);
    
    std::cout << "[SimpleRenderer] ✓ Listo para renderizar con " 
              << cubeVertices.size()/6 << " vertices, " 
              << cubeIndices.size() << " indices" << std::endl;
}

SimpleRenderer::~SimpleRenderer() {
    cleanup();
    if (easyRenderer) {
        delete easyRenderer;
        easyRenderer = nullptr;
    }
}

void SimpleRenderer::beginFrame() {
    if (easyRenderer) {
        easyRenderer->beginFrame();
    }
}

void SimpleRenderer::drawCube(const reactor::Mat4& mvp, const reactor::Vec3& color) {
    if (easyRenderer && easyRenderer->isReady()) {
        // Dibujar el cubo usando EasyRenderer
        easyRenderer->drawMesh(
            cubeVertices.data(), cubeVertices.size(),
            cubeIndices.data(), cubeIndices.size(),
            mvp, color
        );
    }
}

void SimpleRenderer::endFrame() {
    if (easyRenderer) {
        easyRenderer->endFrame();
    }
}

void SimpleRenderer::setClearColor(float r, float g, float b) {
    clearColor = reactor::Vec3(r, g, b);
    if (easyRenderer) {
        easyRenderer->setClearColor(r, g, b, 1.0f);
    }
}

void SimpleRenderer::setWireframe(bool enabled) {
    if (easyRenderer) {
        easyRenderer->setWireframe(enabled);
    }
}

void SimpleRenderer::cleanup() {
    std::cout << "[SimpleRenderer] Limpiando recursos..." << std::endl;
    // EasyRenderer se limpia en el destructor
    std::cout << "[SimpleRenderer] ✓ Limpieza completada" << std::endl;
}

} // namespace test_game
