#pragma once
#include "math.hpp"
#include <memory>
#include <string>

namespace reactor {

// Forward declarations
class Texture;
class Pipeline;

/**
 * @brief Material - Sistema de materiales simplificado
 * 
 * Propiedades PBR básicas:
 * Material mat;
 * mat.albedo = Vec4(1, 0, 0, 1); // Rojo
 * mat.metallic = 0.8f;
 * mat.roughness = 0.2f;
 */
class Material {
public:
    // PBR Properties
    Vec4 albedo{1.0f, 1.0f, 1.0f, 1.0f};
    float metallic{0.0f};
    float roughness{0.5f};
    float ao{1.0f}; // Ambient Occlusion
    
    // Texture maps (opcional)
    Texture* albedoMap{nullptr};
    Texture* normalMap{nullptr};
    Texture* metallicMap{nullptr};
    Texture* roughnessMap{nullptr};
    Texture* aoMap{nullptr};
    
    // Pipeline asociado
    Pipeline* pipeline{nullptr};
    
    Material() = default;
    ~Material() = default;
    
    /**
     * @brief Presets comunes
     */
    static Material pbr();
    static Material unlit();
    static Material wireframe();
    
    /**
     * @brief Helpers para configuración rápida
     */
    Material& setAlbedo(float r, float g, float b, float a = 1.0f);
    Material& setMetallic(float value);
    Material& setRoughness(float value);
};

} // namespace reactor
