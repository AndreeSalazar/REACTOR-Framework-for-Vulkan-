#pragma once
#include "../math.hpp"
#include "../texture.hpp"
#include "../material.hpp"
#include <vulkan/vulkan.h>
#include <vector>
#include <memory>
#include <functional>

namespace reactor {

// Forward declarations
class VulkanContext;
class EasyRenderer;

/**
 * @brief AdvancedFeatures - Complemento para EasyRenderer
 * 
 * Agrega funcionalidades avanzadas sin modificar el core:
 * - Texturas reales (carga desde archivo)
 * - Materiales PBR funcionales
 * - ISR simplificado (Intelligent Shading Rate)
 * - SDF básico (Signed Distance Fields)
 * 
 * Uso:
 * ```cpp
 * AdvancedFeatures features(ctx, renderer);
 * features.loadTexture("diffuse.png");
 * features.setMaterial(Material::pbr().setMetallic(0.8f));
 * features.enableISR(true);
 * ```
 */
class AdvancedFeatures {
public:
    AdvancedFeatures(VulkanContext& ctx, EasyRenderer& renderer);
    ~AdvancedFeatures();

    // ==================== TEXTURAS ====================
    
    /**
     * @brief Carga textura desde archivo
     * @param path Ruta al archivo (png, jpg, bmp, tga)
     * @param name Nombre para referencia
     * @return true si se cargó correctamente
     */
    bool loadTexture(const std::string& path, const std::string& name = "");
    
    /**
     * @brief Crea textura de color sólido
     */
    void createSolidTexture(const std::string& name, float r, float g, float b, float a = 1.0f);
    
    /**
     * @brief Obtiene textura por nombre
     */
    uint32_t getTextureIndex(const std::string& name) const;
    
    /**
     * @brief Bind textura para siguiente draw
     */
    void bindTexture(const std::string& name);
    void bindTexture(uint32_t index);

    // ==================== MATERIALES ====================
    
    /**
     * @brief Registra material
     */
    void registerMaterial(const std::string& name, const Material& material);
    
    /**
     * @brief Usa material para siguiente draw
     */
    void useMaterial(const std::string& name);
    
    /**
     * @brief Obtiene material actual
     */
    const Material& currentMaterial() const { return activeMaterial; }
    
    /**
     * @brief Presets de materiales
     */
    void usePBR(float metallic = 0.0f, float roughness = 0.5f);
    void useUnlit(float r, float g, float b);
    void useWireframe();

    // ==================== ISR (Intelligent Shading Rate) ====================
    
    struct ISRConfig {
        bool enabled = false;
        float qualityBias = 0.5f;      // 0.0 = performance, 1.0 = quality
        float edgeThreshold = 0.1f;    // Sensibilidad a bordes
        float motionThreshold = 0.05f; // Sensibilidad a movimiento
    };
    
    /**
     * @brief Habilita/deshabilita ISR
     */
    void enableISR(bool enable);
    
    /**
     * @brief Configura ISR
     */
    void configureISR(const ISRConfig& config);
    
    /**
     * @brief Obtiene ganancia de performance estimada
     */
    float getISRPerformanceGain() const;
    
    /**
     * @brief Obtiene estadísticas ISR
     */
    struct ISRStats {
        uint32_t pixelsAt1x1 = 0;
        uint32_t pixelsAt2x2 = 0;
        uint32_t pixelsAt4x4 = 0;
        uint32_t pixelsAt8x8 = 0;
        float performanceGain = 0.0f;
    };
    ISRStats getISRStats() const;

    // ==================== SDF (Signed Distance Fields) ====================
    
    /**
     * @brief Tipos de primitivas SDF
     */
    enum class SDFPrimitive {
        Sphere,
        Box,
        Torus,
        Cylinder,
        Capsule,
        Cone
    };
    
    /**
     * @brief Agrega primitiva SDF a la escena
     */
    void addSDFPrimitive(SDFPrimitive type, const Vec3& position, const Vec3& params, const Vec3& color);
    
    /**
     * @brief Limpia primitivas SDF
     */
    void clearSDFPrimitives();
    
    /**
     * @brief Renderiza escena SDF (ray marching)
     */
    void renderSDF();
    
    /**
     * @brief Habilita modo SDF
     */
    void enableSDF(bool enable);

    // ==================== ILUMINACIÓN ====================
    
    struct Light {
        enum Type { Directional, Point, Spot };
        Type type = Directional;
        Vec3 position{0, 10, 0};
        Vec3 direction{0, -1, 0};
        Vec3 color{1, 1, 1};
        float intensity = 1.0f;
        float range = 10.0f;      // Para point/spot
        float spotAngle = 45.0f;  // Para spot
    };
    
    /**
     * @brief Agrega luz a la escena
     */
    void addLight(const Light& light);
    
    /**
     * @brief Limpia luces
     */
    void clearLights();
    
    /**
     * @brief Configura luz ambiental
     */
    void setAmbientLight(float r, float g, float b, float intensity = 0.3f);

    // ==================== ESTADO ====================
    
    /**
     * @brief Actualiza sistemas (llamar cada frame)
     */
    void update(float deltaTime);
    
    /**
     * @brief Aplica configuración al renderer
     */
    void apply();
    
    /**
     * @brief Obtiene estadísticas generales
     */
    struct Stats {
        uint32_t texturesLoaded = 0;
        uint32_t materialsRegistered = 0;
        uint32_t lightsActive = 0;
        uint32_t sdfPrimitives = 0;
        bool isrEnabled = false;
        bool sdfEnabled = false;
    };
    Stats getStats() const;

private:
    VulkanContext& ctx;
    EasyRenderer& renderer;
    
    // Texturas
    struct TextureData {
        std::string name;
        std::string path;
        uint32_t width = 0;
        uint32_t height = 0;
        std::vector<uint8_t> pixels;
        VkImage image = VK_NULL_HANDLE;
        VkImageView view = VK_NULL_HANDLE;
        VkDeviceMemory memory = VK_NULL_HANDLE;
        VkSampler sampler = VK_NULL_HANDLE;
    };
    std::vector<TextureData> textures;
    int32_t activeTextureIndex = -1;
    
    // Materiales
    std::vector<std::pair<std::string, Material>> materials;
    Material activeMaterial;
    
    // ISR
    ISRConfig isrConfig;
    ISRStats isrStats;
    
    // SDF
    struct SDFObject {
        SDFPrimitive type;
        Vec3 position;
        Vec3 params;
        Vec3 color;
    };
    std::vector<SDFObject> sdfObjects;
    bool sdfEnabled = false;
    
    // Luces
    std::vector<Light> lights;
    Vec4 ambientLight{0.1f, 0.1f, 0.1f, 0.3f};
    
    // Helpers
    bool loadTextureFromFile(const std::string& path, TextureData& data);
    void createVulkanTexture(TextureData& data);
    void updateISRStats();
};

} // namespace reactor
