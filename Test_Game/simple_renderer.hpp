#pragma once
#include "reactor/reactor.hpp"
#include "reactor/rendering/advanced_features.hpp"

namespace test_game {

/**
 * @brief SimpleRenderer - Módulo de rendering simple para Test_Game
 * 
 * Fácil de usar, modificar y eliminar.
 * Estilo modular como Blender.
 * 
 * Ahora incluye AdvancedFeatures para:
 * - Texturas
 * - Materiales PBR
 * - ISR (Intelligent Shading Rate)
 * - SDF (Signed Distance Fields)
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
    
    // ==================== NUEVAS FUNCIONALIDADES ====================
    
    // Texturas
    bool loadTexture(const std::string& path, const std::string& name = "");
    void bindTexture(const std::string& name);
    
    // Materiales
    void usePBR(float metallic = 0.0f, float roughness = 0.5f);
    void useUnlit(float r, float g, float b);
    
    // ISR (Intelligent Shading Rate)
    void enableISR(bool enable);
    float getISRPerformanceGain() const;
    
    // SDF (Signed Distance Fields)
    void enableSDF(bool enable);
    void addSDFSphere(const reactor::Vec3& pos, float radius, const reactor::Vec3& color);
    void addSDFBox(const reactor::Vec3& pos, const reactor::Vec3& size, const reactor::Vec3& color);
    
    // Iluminación
    void addDirectionalLight(const reactor::Vec3& dir, const reactor::Vec3& color, float intensity = 1.0f);
    void addPointLight(const reactor::Vec3& pos, const reactor::Vec3& color, float intensity = 1.0f, float range = 10.0f);
    void setAmbientLight(float r, float g, float b, float intensity = 0.3f);
    
    // Estadísticas
    void printStats() const;
    
    // Acceso a features avanzadas
    reactor::AdvancedFeatures* getAdvancedFeatures() { return advancedFeatures; }
    
private:
    reactor::VulkanContext& ctx;
    reactor::Window& window;
    reactor::Vec3 clearColor{0.1f, 0.1f, 0.1f};
    
    // EasyRenderer (FASE 8) hace todo el trabajo
    reactor::EasyRenderer* easyRenderer{nullptr};
    
    // AdvancedFeatures - Complemento sin conflictos
    reactor::AdvancedFeatures* advancedFeatures{nullptr};
    
    // Geometría del cubo
    std::vector<float> cubeVertices;
    std::vector<uint16_t> cubeIndices;
    
    void cleanup();
};

} // namespace test_game
