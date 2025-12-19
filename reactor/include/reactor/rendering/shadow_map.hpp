#pragma once
#include "../math.hpp"
#include <vulkan/vulkan.h>
#include <memory>
#include <vector>

namespace reactor {

// Forward declarations
class MemoryAllocator;
class VulkanContext;

/**
 * @brief ShadowMap - Sistema de sombras completo para REACTOR
 * 
 * Implementa shadow mapping con:
 * - Depth buffer dedicado para sombras
 * - Matrices de luz (view + projection)
 * - PCF (Percentage Closer Filtering) para soft shadows
 * - Cascade Shadow Maps para escenas grandes (opcional)
 * 
 * Uso:
 * ```cpp
 * ShadowMap shadowMap(ctx, 2048, 2048);
 * shadowMap.setLightDirection(Vec3(0.5, -1, 0.3));
 * 
 * // En render loop:
 * shadowMap.beginShadowPass(cmd);
 * // render scene desde perspectiva de luz
 * shadowMap.endShadowPass(cmd);
 * 
 * // Usar en shader principal
 * float shadow = shadowMap.sampleShadow(worldPos);
 * ```
 */
class ShadowMap {
public:
    struct Config {
        uint32_t resolution = 2048;
        float nearPlane = 0.1f;
        float farPlane = 100.0f;
        float orthoSize = 20.0f;  // Para luces direccionales
        bool usePCF = true;       // Soft shadows
        int pcfSamples = 4;       // 4x4 PCF
        float bias = 0.005f;      // Shadow bias para evitar acne
    };

    ShadowMap(VulkanContext& ctx, uint32_t width, uint32_t height);
    ShadowMap(VulkanContext& ctx, const Config& config);
    ~ShadowMap();

    ShadowMap(const ShadowMap&) = delete;
    ShadowMap& operator=(const ShadowMap&) = delete;
    ShadowMap(ShadowMap&& other) noexcept;
    ShadowMap& operator=(ShadowMap&& other) noexcept;

    // Configuración de luz
    void setLightDirection(const Vec3& direction);
    void setLightPosition(const Vec3& position);
    void setOrthoSize(float size);
    void setBias(float bias) { config.bias = bias; }
    void setPCFEnabled(bool enabled) { config.usePCF = enabled; }

    // Matrices
    Mat4 getLightViewMatrix() const;
    Mat4 getLightProjectionMatrix() const;
    Mat4 getLightSpaceMatrix() const;  // View * Projection

    // Getters
    uint32_t width() const { return shadowWidth; }
    uint32_t height() const { return shadowHeight; }
    VkImageView getDepthImageView() const { return depthImageView; }
    VkSampler getSampler() const { return shadowSampler; }
    const Config& getConfig() const { return config; }
    bool isReady() const { return ready; }

    // Para uso en shaders (datos para uniform buffer)
    struct ShadowData {
        Mat4 lightSpaceMatrix;
        Vec4 lightDirection;
        float bias;
        float pcfRadius;
        int pcfSamples;
        float padding;
    };
    ShadowData getShadowData() const;

private:
    VulkanContext* ctx{nullptr};
    Config config;
    uint32_t shadowWidth{0};
    uint32_t shadowHeight{0};
    bool ready{false};

    // Luz
    Vec3 lightDirection{0.5f, -1.0f, 0.3f};
    Vec3 lightPosition{0.0f, 10.0f, 0.0f};

    // Vulkan resources
    VkImage depthImage{VK_NULL_HANDLE};
    VkDeviceMemory depthImageMemory{VK_NULL_HANDLE};
    VkImageView depthImageView{VK_NULL_HANDLE};
    VkSampler shadowSampler{VK_NULL_HANDLE};
    VkFramebuffer shadowFramebuffer{VK_NULL_HANDLE};
    VkRenderPass shadowRenderPass{VK_NULL_HANDLE};

    void createResources();
    void cleanup();
    uint32_t findMemoryType(uint32_t typeFilter, VkMemoryPropertyFlags properties);
};

/**
 * @brief CascadeShadowMap - Shadow maps en cascada para escenas grandes
 * 
 * Divide el frustum de la cámara en múltiples cascadas,
 * cada una con su propio shadow map para mejor resolución.
 */
class CascadeShadowMap {
public:
    static constexpr int MAX_CASCADES = 4;

    struct CascadeConfig {
        int numCascades = 4;
        float splitLambda = 0.95f;  // Balance entre log y linear split
        uint32_t resolution = 2048;
    };

    CascadeShadowMap(VulkanContext& ctx, const CascadeConfig& config);
    ~CascadeShadowMap();

    void updateCascades(const Mat4& cameraView, const Mat4& cameraProj, 
                        float nearPlane, float farPlane, const Vec3& lightDir);

    // Getters para cada cascada
    Mat4 getCascadeMatrix(int index) const;
    float getCascadeSplit(int index) const;
    VkImageView getCascadeImageView(int index) const;

private:
    VulkanContext* ctx{nullptr};
    CascadeConfig config;
    
    struct Cascade {
        Mat4 viewProjMatrix;
        float splitDepth;
        VkImage depthImage{VK_NULL_HANDLE};
        VkImageView depthImageView{VK_NULL_HANDLE};
    };
    std::vector<Cascade> cascades;
};

} // namespace reactor
