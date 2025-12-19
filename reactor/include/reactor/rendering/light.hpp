#pragma once
#include "../math.hpp"

namespace reactor {

/**
 * @brief Light - Sistema de iluminación simplificado
 * 
 * Uso ultra simple:
 * Light light = Light::directional(Vec3(1, -1, 0));
 * light.color = Vec3(1, 1, 1);
 * light.intensity = 1.0f;
 */
class Light {
public:
    enum class Type {
        Directional,
        Point,
        Spot
    };
    
    Type type{Type::Directional};
    Vec3 position{0, 0, 0};
    Vec3 direction{0, -1, 0};
    Vec3 color{1, 1, 1};
    float intensity{1.0f};
    
    // Point light
    float range{10.0f};
    float attenuation{1.0f};
    
    // Spot light
    float innerConeAngle{30.0f};  // degrees
    float outerConeAngle{45.0f};  // degrees
    
    // Shadows
    bool castShadows{false};
    
    /**
     * @brief Crear luces predefinidas
     */
    static Light directional(const Vec3& direction);
    static Light point(const Vec3& position, float range = 10.0f);
    static Light spot(const Vec3& position, const Vec3& direction, float angle = 45.0f);
    
    /**
     * @brief Fluent API
     */
    Light& setColor(float r, float g, float b);
    Light& setIntensity(float intensity);
    Light& enableShadows(bool enable = true);
};

/**
 * @brief LightManager - Gestión de múltiples luces
 */
class LightManager {
public:
    LightManager() = default;
    
    /**
     * @brief Agregar luz
     */
    Light* addLight(const Light& light);
    
    /**
     * @brief Obtener luces
     */
    const std::vector<Light>& lights() const { return lightList; }
    std::vector<Light>& lights() { return lightList; }
    
    /**
     * @brief Limpiar
     */
    void clear() { lightList.clear(); }
    
    /**
     * @brief Stats
     */
    size_t count() const { return lightList.size(); }
    size_t directionalCount() const;
    size_t pointCount() const;
    size_t spotCount() const;

private:
    std::vector<Light> lightList;
};

} // namespace reactor
