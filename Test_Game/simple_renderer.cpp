#include "simple_renderer.hpp"
#include "reactor/rendering/easy_renderer.hpp"
#include "reactor/rendering/advanced_features.hpp"
#include <iostream>
#include <array>
#include <fstream>
#include <vector>

namespace test_game {

SimpleRenderer::SimpleRenderer(reactor::VulkanContext& ctx, reactor::Window& window)
    : ctx(ctx), window(window) {
    
    std::cout << "[SimpleRenderer] Inicializando con EasyRenderer + AdvancedFeatures..." << std::endl;
    
    // Crear EasyRenderer (FASE 8) - hace todo el trabajo pesado
    easyRenderer = new reactor::EasyRenderer(ctx, window);
    
    // Crear AdvancedFeatures - complemento sin conflictos
    advancedFeatures = new reactor::AdvancedFeatures(ctx, *easyRenderer);
    
    // Establecer color de fondo visible (azul oscuro) ANTES de crear geometría
    easyRenderer->setClearColor(0.1f, 0.2f, 0.4f, 1.0f);
    
    // Crear geometría del cubo
    reactor::QuickDraw::cube(cubeVertices, cubeIndices);
    
    std::cout << "[SimpleRenderer] ✓ Listo para renderizar con " 
              << cubeVertices.size()/6 << " vertices, " 
              << cubeIndices.size() << " indices" << std::endl;
    std::cout << "[SimpleRenderer] Clear color: (0.1, 0.2, 0.4)" << std::endl;
}

SimpleRenderer::~SimpleRenderer() {
    cleanup();
    if (advancedFeatures) {
        delete advancedFeatures;
        advancedFeatures = nullptr;
    }
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
    // EasyRenderer y AdvancedFeatures se limpian en el destructor
    std::cout << "[SimpleRenderer] ✓ Limpieza completada" << std::endl;
}

// ==================== NUEVAS FUNCIONALIDADES ====================

// Texturas
bool SimpleRenderer::loadTexture(const std::string& path, const std::string& name) {
    if (advancedFeatures) {
        return advancedFeatures->loadTexture(path, name);
    }
    return false;
}

void SimpleRenderer::bindTexture(const std::string& name) {
    if (advancedFeatures) {
        advancedFeatures->bindTexture(name);
    }
}

// Materiales
void SimpleRenderer::usePBR(float metallic, float roughness) {
    if (advancedFeatures) {
        advancedFeatures->usePBR(metallic, roughness);
    }
}

void SimpleRenderer::useUnlit(float r, float g, float b) {
    if (advancedFeatures) {
        advancedFeatures->useUnlit(r, g, b);
    }
}

// ISR
void SimpleRenderer::enableISR(bool enable) {
    if (advancedFeatures) {
        advancedFeatures->enableISR(enable);
    }
}

float SimpleRenderer::getISRPerformanceGain() const {
    if (advancedFeatures) {
        return advancedFeatures->getISRPerformanceGain();
    }
    return 0.0f;
}

// SDF
void SimpleRenderer::enableSDF(bool enable) {
    if (advancedFeatures) {
        advancedFeatures->enableSDF(enable);
    }
}

void SimpleRenderer::addSDFSphere(const reactor::Vec3& pos, float radius, const reactor::Vec3& color) {
    if (advancedFeatures) {
        advancedFeatures->addSDFPrimitive(
            reactor::AdvancedFeatures::SDFPrimitive::Sphere,
            pos, reactor::Vec3(radius, 0, 0), color
        );
    }
}

void SimpleRenderer::addSDFBox(const reactor::Vec3& pos, const reactor::Vec3& size, const reactor::Vec3& color) {
    if (advancedFeatures) {
        advancedFeatures->addSDFPrimitive(
            reactor::AdvancedFeatures::SDFPrimitive::Box,
            pos, size, color
        );
    }
}

// Iluminación
void SimpleRenderer::addDirectionalLight(const reactor::Vec3& dir, const reactor::Vec3& color, float intensity) {
    if (advancedFeatures) {
        reactor::AdvancedFeatures::Light light;
        light.type = reactor::AdvancedFeatures::Light::Directional;
        light.direction = dir;
        light.color = color;
        light.intensity = intensity;
        advancedFeatures->addLight(light);
    }
}

void SimpleRenderer::addPointLight(const reactor::Vec3& pos, const reactor::Vec3& color, float intensity, float range) {
    if (advancedFeatures) {
        reactor::AdvancedFeatures::Light light;
        light.type = reactor::AdvancedFeatures::Light::Point;
        light.position = pos;
        light.color = color;
        light.intensity = intensity;
        light.range = range;
        advancedFeatures->addLight(light);
    }
}

void SimpleRenderer::setAmbientLight(float r, float g, float b, float intensity) {
    if (advancedFeatures) {
        advancedFeatures->setAmbientLight(r, g, b, intensity);
    }
}

// Estadísticas
void SimpleRenderer::printStats() const {
    if (advancedFeatures) {
        auto stats = advancedFeatures->getStats();
        std::cout << "\n=== SimpleRenderer Stats ===" << std::endl;
        std::cout << "  Texturas: " << stats.texturesLoaded << std::endl;
        std::cout << "  Materiales: " << stats.materialsRegistered << std::endl;
        std::cout << "  Luces: " << stats.lightsActive << std::endl;
        std::cout << "  SDF Primitivas: " << stats.sdfPrimitives << std::endl;
        std::cout << "  ISR: " << (stats.isrEnabled ? "ON" : "OFF") << std::endl;
        std::cout << "  SDF: " << (stats.sdfEnabled ? "ON" : "OFF") << std::endl;
        if (stats.isrEnabled) {
            std::cout << "  ISR Performance Gain: +" << (advancedFeatures->getISRPerformanceGain() * 100) << "%" << std::endl;
        }
        std::cout << "============================\n" << std::endl;
    }
}

} // namespace test_game
