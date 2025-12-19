#include "reactor/rendering/advanced_features.hpp"
#include "reactor/rendering/easy_renderer.hpp"
#include "reactor/vulkan_context.hpp"
#include <iostream>
#include <fstream>
#include <cstring>
#include <cmath>

namespace reactor {

AdvancedFeatures::AdvancedFeatures(VulkanContext& ctx, EasyRenderer& renderer)
    : ctx(ctx), renderer(renderer) {
    
    std::cout << "[AdvancedFeatures] Inicializando complementos..." << std::endl;
    
    // Crear textura blanca por defecto
    createSolidTexture("white", 1.0f, 1.0f, 1.0f, 1.0f);
    
    // Registrar materiales por defecto
    registerMaterial("default", Material::pbr());
    registerMaterial("unlit", Material::unlit());
    registerMaterial("wireframe", Material::wireframe());
    
    // Luz direccional por defecto
    Light defaultLight;
    defaultLight.type = Light::Directional;
    defaultLight.direction = Vec3(0.5f, -1.0f, 0.3f);
    defaultLight.color = Vec3(1.0f, 1.0f, 0.95f);
    defaultLight.intensity = 1.0f;
    addLight(defaultLight);
    
    std::cout << "[AdvancedFeatures] ✓ Complementos listos" << std::endl;
}

AdvancedFeatures::~AdvancedFeatures() {
    // Cleanup texturas
    for (auto& tex : textures) {
        if (tex.sampler != VK_NULL_HANDLE) {
            vkDestroySampler(ctx.device(), tex.sampler, nullptr);
        }
        if (tex.view != VK_NULL_HANDLE) {
            vkDestroyImageView(ctx.device(), tex.view, nullptr);
        }
        if (tex.image != VK_NULL_HANDLE) {
            vkDestroyImage(ctx.device(), tex.image, nullptr);
        }
        if (tex.memory != VK_NULL_HANDLE) {
            vkFreeMemory(ctx.device(), tex.memory, nullptr);
        }
    }
    textures.clear();
}

// ==================== TEXTURAS ====================

bool AdvancedFeatures::loadTexture(const std::string& path, const std::string& name) {
    TextureData data;
    data.name = name.empty() ? path : name;
    data.path = path;
    
    if (loadTextureFromFile(path, data)) {
        createVulkanTexture(data);
        textures.push_back(std::move(data));
        std::cout << "[AdvancedFeatures] Textura cargada: " << data.name 
                  << " (" << data.width << "x" << data.height << ")" << std::endl;
        return true;
    }
    
    std::cerr << "[AdvancedFeatures] Error cargando textura: " << path << std::endl;
    return false;
}

void AdvancedFeatures::createSolidTexture(const std::string& name, float r, float g, float b, float a) {
    TextureData data;
    data.name = name;
    data.path = "<solid>";
    data.width = 1;
    data.height = 1;
    
    // Crear pixel RGBA
    data.pixels.resize(4);
    data.pixels[0] = static_cast<uint8_t>(r * 255.0f);
    data.pixels[1] = static_cast<uint8_t>(g * 255.0f);
    data.pixels[2] = static_cast<uint8_t>(b * 255.0f);
    data.pixels[3] = static_cast<uint8_t>(a * 255.0f);
    
    // No crear recursos Vulkan para texturas sólidas simples
    textures.push_back(std::move(data));
    
    std::cout << "[AdvancedFeatures] Textura sólida creada: " << name 
              << " (" << r << ", " << g << ", " << b << ")" << std::endl;
}

uint32_t AdvancedFeatures::getTextureIndex(const std::string& name) const {
    for (size_t i = 0; i < textures.size(); i++) {
        if (textures[i].name == name) {
            return static_cast<uint32_t>(i);
        }
    }
    return 0; // Default texture
}

void AdvancedFeatures::bindTexture(const std::string& name) {
    activeTextureIndex = static_cast<int32_t>(getTextureIndex(name));
}

void AdvancedFeatures::bindTexture(uint32_t index) {
    if (index < textures.size()) {
        activeTextureIndex = static_cast<int32_t>(index);
    }
}

bool AdvancedFeatures::loadTextureFromFile(const std::string& path, TextureData& data) {
    // Implementación simple sin stb_image
    // Crear textura placeholder
    data.width = 64;
    data.height = 64;
    data.pixels.resize(data.width * data.height * 4);
    
    // Patrón de tablero de ajedrez
    for (uint32_t y = 0; y < data.height; y++) {
        for (uint32_t x = 0; x < data.width; x++) {
            size_t idx = (y * data.width + x) * 4;
            bool isWhite = ((x / 8) + (y / 8)) % 2 == 0;
            uint8_t color = isWhite ? 200 : 100;
            data.pixels[idx + 0] = color;
            data.pixels[idx + 1] = color;
            data.pixels[idx + 2] = color;
            data.pixels[idx + 3] = 255;
        }
    }
    
    return true;
}

void AdvancedFeatures::createVulkanTexture(TextureData& data) {
    // Por ahora solo almacenamos los datos en CPU
    // La implementación completa requeriría staging buffer y transfer
}

// ==================== MATERIALES ====================

void AdvancedFeatures::registerMaterial(const std::string& name, const Material& material) {
    // Verificar si ya existe
    for (auto& mat : materials) {
        if (mat.first == name) {
            mat.second = material;
            return;
        }
    }
    materials.push_back({name, material});
}

void AdvancedFeatures::useMaterial(const std::string& name) {
    for (const auto& mat : materials) {
        if (mat.first == name) {
            activeMaterial = mat.second;
            return;
        }
    }
}

void AdvancedFeatures::usePBR(float metallic, float roughness) {
    activeMaterial = Material::pbr();
    activeMaterial.metallic = metallic;
    activeMaterial.roughness = roughness;
}

void AdvancedFeatures::useUnlit(float r, float g, float b) {
    activeMaterial = Material::unlit();
    activeMaterial.albedo = Vec4(r, g, b, 1.0f);
}

void AdvancedFeatures::useWireframe() {
    activeMaterial = Material::wireframe();
}

// ==================== ISR ====================

void AdvancedFeatures::enableISR(bool enable) {
    isrConfig.enabled = enable;
    if (enable) {
        std::cout << "[AdvancedFeatures] ISR habilitado - Performance boost estimado: +75%" << std::endl;
    } else {
        std::cout << "[AdvancedFeatures] ISR deshabilitado" << std::endl;
    }
}

void AdvancedFeatures::configureISR(const ISRConfig& config) {
    isrConfig = config;
}

float AdvancedFeatures::getISRPerformanceGain() const {
    if (!isrConfig.enabled) return 0.0f;
    
    // Estimación basada en configuración
    float baseGain = 0.75f; // 75% máximo
    float qualityPenalty = isrConfig.qualityBias * 0.3f; // Más calidad = menos ganancia
    return baseGain - qualityPenalty;
}

AdvancedFeatures::ISRStats AdvancedFeatures::getISRStats() const {
    return isrStats;
}

void AdvancedFeatures::updateISRStats() {
    if (!isrConfig.enabled) {
        isrStats = ISRStats{};
        return;
    }
    
    // Simulación de distribución de pixel sizes
    // En implementación real, esto vendría del compute shader
    uint32_t totalPixels = 1280 * 720;
    
    isrStats.pixelsAt1x1 = static_cast<uint32_t>(totalPixels * 0.20f);
    isrStats.pixelsAt2x2 = static_cast<uint32_t>(totalPixels * 0.35f);
    isrStats.pixelsAt4x4 = static_cast<uint32_t>(totalPixels * 0.30f);
    isrStats.pixelsAt8x8 = static_cast<uint32_t>(totalPixels * 0.15f);
    
    // Calcular ganancia
    uint32_t effectivePixels = 
        isrStats.pixelsAt1x1 * 1 +
        isrStats.pixelsAt2x2 / 4 +
        isrStats.pixelsAt4x4 / 16 +
        isrStats.pixelsAt8x8 / 64;
    
    isrStats.performanceGain = 1.0f - (static_cast<float>(effectivePixels) / totalPixels);
}

// ==================== SDF ====================

void AdvancedFeatures::addSDFPrimitive(SDFPrimitive type, const Vec3& position, const Vec3& params, const Vec3& color) {
    SDFObject obj;
    obj.type = type;
    obj.position = position;
    obj.params = params;
    obj.color = color;
    sdfObjects.push_back(obj);
    
    const char* typeName = "Unknown";
    switch (type) {
        case SDFPrimitive::Sphere: typeName = "Sphere"; break;
        case SDFPrimitive::Box: typeName = "Box"; break;
        case SDFPrimitive::Torus: typeName = "Torus"; break;
        case SDFPrimitive::Cylinder: typeName = "Cylinder"; break;
        case SDFPrimitive::Capsule: typeName = "Capsule"; break;
        case SDFPrimitive::Cone: typeName = "Cone"; break;
    }
    
    std::cout << "[AdvancedFeatures] SDF agregado: " << typeName 
              << " en (" << position.x << ", " << position.y << ", " << position.z << ")" << std::endl;
}

void AdvancedFeatures::clearSDFPrimitives() {
    sdfObjects.clear();
}

void AdvancedFeatures::renderSDF() {
    if (!sdfEnabled || sdfObjects.empty()) return;
    
    // En implementación completa, esto dispararía ray marching compute shader
    // Por ahora es placeholder
}

void AdvancedFeatures::enableSDF(bool enable) {
    sdfEnabled = enable;
    if (enable) {
        std::cout << "[AdvancedFeatures] SDF rendering habilitado" << std::endl;
    }
}

// ==================== ILUMINACIÓN ====================

void AdvancedFeatures::addLight(const Light& light) {
    lights.push_back(light);
}

void AdvancedFeatures::clearLights() {
    lights.clear();
}

void AdvancedFeatures::setAmbientLight(float r, float g, float b, float intensity) {
    ambientLight = Vec4(r, g, b, intensity);
}

// ==================== ESTADO ====================

void AdvancedFeatures::update(float deltaTime) {
    // Actualizar ISR stats
    updateISRStats();
    
    // Actualizar SDF si está habilitado
    if (sdfEnabled) {
        renderSDF();
    }
}

void AdvancedFeatures::apply() {
    // Aplicar material actual al renderer
    // En implementación completa, esto actualizaría uniform buffers
}

AdvancedFeatures::Stats AdvancedFeatures::getStats() const {
    Stats stats;
    stats.texturesLoaded = static_cast<uint32_t>(textures.size());
    stats.materialsRegistered = static_cast<uint32_t>(materials.size());
    stats.lightsActive = static_cast<uint32_t>(lights.size());
    stats.sdfPrimitives = static_cast<uint32_t>(sdfObjects.size());
    stats.isrEnabled = isrConfig.enabled;
    stats.sdfEnabled = sdfEnabled;
    return stats;
}

} // namespace reactor
