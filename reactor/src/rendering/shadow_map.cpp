#include "reactor/rendering/shadow_map.hpp"
#include "reactor/vulkan_context.hpp"
#include <glm/gtc/matrix_transform.hpp>
#include <iostream>

namespace reactor {

ShadowMap::ShadowMap(VulkanContext& ctx, uint32_t width, uint32_t height)
    : ctx(&ctx), shadowWidth(width), shadowHeight(height) {
    config.resolution = width;
    createResources();
}

ShadowMap::ShadowMap(VulkanContext& ctx, const Config& config)
    : ctx(&ctx), config(config), shadowWidth(config.resolution), shadowHeight(config.resolution) {
    createResources();
}

ShadowMap::~ShadowMap() {
    cleanup();
}

ShadowMap::ShadowMap(ShadowMap&& other) noexcept
    : ctx(other.ctx)
    , config(other.config)
    , shadowWidth(other.shadowWidth)
    , shadowHeight(other.shadowHeight)
    , ready(other.ready)
    , lightDirection(other.lightDirection)
    , lightPosition(other.lightPosition)
    , depthImage(other.depthImage)
    , depthImageMemory(other.depthImageMemory)
    , depthImageView(other.depthImageView)
    , shadowSampler(other.shadowSampler)
    , shadowFramebuffer(other.shadowFramebuffer)
    , shadowRenderPass(other.shadowRenderPass) {
    other.ctx = nullptr;
    other.depthImage = VK_NULL_HANDLE;
    other.depthImageMemory = VK_NULL_HANDLE;
    other.depthImageView = VK_NULL_HANDLE;
    other.shadowSampler = VK_NULL_HANDLE;
    other.shadowFramebuffer = VK_NULL_HANDLE;
    other.shadowRenderPass = VK_NULL_HANDLE;
    other.ready = false;
}

ShadowMap& ShadowMap::operator=(ShadowMap&& other) noexcept {
    if (this != &other) {
        cleanup();
        ctx = other.ctx;
        config = other.config;
        shadowWidth = other.shadowWidth;
        shadowHeight = other.shadowHeight;
        ready = other.ready;
        lightDirection = other.lightDirection;
        lightPosition = other.lightPosition;
        depthImage = other.depthImage;
        depthImageMemory = other.depthImageMemory;
        depthImageView = other.depthImageView;
        shadowSampler = other.shadowSampler;
        shadowFramebuffer = other.shadowFramebuffer;
        shadowRenderPass = other.shadowRenderPass;
        
        other.ctx = nullptr;
        other.depthImage = VK_NULL_HANDLE;
        other.depthImageMemory = VK_NULL_HANDLE;
        other.depthImageView = VK_NULL_HANDLE;
        other.shadowSampler = VK_NULL_HANDLE;
        other.shadowFramebuffer = VK_NULL_HANDLE;
        other.shadowRenderPass = VK_NULL_HANDLE;
        other.ready = false;
    }
    return *this;
}

void ShadowMap::setLightDirection(const Vec3& direction) {
    lightDirection = glm::normalize(direction);
}

void ShadowMap::setLightPosition(const Vec3& position) {
    lightPosition = position;
}

void ShadowMap::setOrthoSize(float size) {
    config.orthoSize = size;
}

Mat4 ShadowMap::getLightViewMatrix() const {
    Vec3 target = lightPosition + lightDirection;
    return glm::lookAt(lightPosition, target, Vec3(0, 1, 0));
}

Mat4 ShadowMap::getLightProjectionMatrix() const {
    float size = config.orthoSize;
    return glm::ortho(-size, size, -size, size, config.nearPlane, config.farPlane);
}

Mat4 ShadowMap::getLightSpaceMatrix() const {
    return getLightProjectionMatrix() * getLightViewMatrix();
}

ShadowMap::ShadowData ShadowMap::getShadowData() const {
    ShadowData data;
    data.lightSpaceMatrix = getLightSpaceMatrix();
    data.lightDirection = Vec4(lightDirection, 0.0f);
    data.bias = config.bias;
    data.pcfRadius = 1.0f / static_cast<float>(shadowWidth);
    data.pcfSamples = config.usePCF ? config.pcfSamples : 1;
    data.padding = 0.0f;
    return data;
}

void ShadowMap::createResources() {
    if (!ctx) return;
    
    std::cout << "[ShadowMap] Creando shadow map " << shadowWidth << "x" << shadowHeight << "..." << std::endl;
    
    // Crear depth image para shadow map
    VkImageCreateInfo imageInfo{};
    imageInfo.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
    imageInfo.imageType = VK_IMAGE_TYPE_2D;
    imageInfo.extent.width = shadowWidth;
    imageInfo.extent.height = shadowHeight;
    imageInfo.extent.depth = 1;
    imageInfo.mipLevels = 1;
    imageInfo.arrayLayers = 1;
    imageInfo.format = VK_FORMAT_D32_SFLOAT;
    imageInfo.tiling = VK_IMAGE_TILING_OPTIMAL;
    imageInfo.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    imageInfo.usage = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT | VK_IMAGE_USAGE_SAMPLED_BIT;
    imageInfo.samples = VK_SAMPLE_COUNT_1_BIT;
    imageInfo.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    
    if (vkCreateImage(ctx->device(), &imageInfo, nullptr, &depthImage) != VK_SUCCESS) {
        std::cerr << "[ShadowMap] Error: No se pudo crear depth image" << std::endl;
        return;
    }
    
    // Allocate memory
    VkMemoryRequirements memRequirements;
    vkGetImageMemoryRequirements(ctx->device(), depthImage, &memRequirements);
    
    VkMemoryAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    allocInfo.allocationSize = memRequirements.size;
    allocInfo.memoryTypeIndex = findMemoryType(memRequirements.memoryTypeBits, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT);
    
    if (vkAllocateMemory(ctx->device(), &allocInfo, nullptr, &depthImageMemory) != VK_SUCCESS) {
        std::cerr << "[ShadowMap] Error: No se pudo allocar memoria" << std::endl;
        return;
    }
    
    vkBindImageMemory(ctx->device(), depthImage, depthImageMemory, 0);
    
    // Create image view
    VkImageViewCreateInfo viewInfo{};
    viewInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    viewInfo.image = depthImage;
    viewInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
    viewInfo.format = VK_FORMAT_D32_SFLOAT;
    viewInfo.subresourceRange.aspectMask = VK_IMAGE_ASPECT_DEPTH_BIT;
    viewInfo.subresourceRange.baseMipLevel = 0;
    viewInfo.subresourceRange.levelCount = 1;
    viewInfo.subresourceRange.baseArrayLayer = 0;
    viewInfo.subresourceRange.layerCount = 1;
    
    if (vkCreateImageView(ctx->device(), &viewInfo, nullptr, &depthImageView) != VK_SUCCESS) {
        std::cerr << "[ShadowMap] Error: No se pudo crear image view" << std::endl;
        return;
    }
    
    // Create sampler para shadow sampling
    VkSamplerCreateInfo samplerInfo{};
    samplerInfo.sType = VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO;
    samplerInfo.magFilter = VK_FILTER_LINEAR;
    samplerInfo.minFilter = VK_FILTER_LINEAR;
    samplerInfo.addressModeU = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER;
    samplerInfo.addressModeV = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER;
    samplerInfo.addressModeW = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER;
    samplerInfo.borderColor = VK_BORDER_COLOR_FLOAT_OPAQUE_WHITE;
    samplerInfo.compareEnable = VK_TRUE;
    samplerInfo.compareOp = VK_COMPARE_OP_LESS_OR_EQUAL;
    samplerInfo.mipmapMode = VK_SAMPLER_MIPMAP_MODE_LINEAR;
    
    if (vkCreateSampler(ctx->device(), &samplerInfo, nullptr, &shadowSampler) != VK_SUCCESS) {
        std::cerr << "[ShadowMap] Error: No se pudo crear sampler" << std::endl;
        return;
    }
    
    ready = true;
    std::cout << "[ShadowMap] ✓ Shadow map creado correctamente" << std::endl;
    std::cout << "  - Resolución: " << shadowWidth << "x" << shadowHeight << std::endl;
    std::cout << "  - PCF: " << (config.usePCF ? "ON" : "OFF") << std::endl;
    std::cout << "  - Bias: " << config.bias << std::endl;
}

void ShadowMap::cleanup() {
    if (!ctx) return;
    
    vkDeviceWaitIdle(ctx->device());
    
    if (shadowSampler != VK_NULL_HANDLE) {
        vkDestroySampler(ctx->device(), shadowSampler, nullptr);
        shadowSampler = VK_NULL_HANDLE;
    }
    if (depthImageView != VK_NULL_HANDLE) {
        vkDestroyImageView(ctx->device(), depthImageView, nullptr);
        depthImageView = VK_NULL_HANDLE;
    }
    if (depthImage != VK_NULL_HANDLE) {
        vkDestroyImage(ctx->device(), depthImage, nullptr);
        depthImage = VK_NULL_HANDLE;
    }
    if (depthImageMemory != VK_NULL_HANDLE) {
        vkFreeMemory(ctx->device(), depthImageMemory, nullptr);
        depthImageMemory = VK_NULL_HANDLE;
    }
    if (shadowFramebuffer != VK_NULL_HANDLE) {
        vkDestroyFramebuffer(ctx->device(), shadowFramebuffer, nullptr);
        shadowFramebuffer = VK_NULL_HANDLE;
    }
    if (shadowRenderPass != VK_NULL_HANDLE) {
        vkDestroyRenderPass(ctx->device(), shadowRenderPass, nullptr);
        shadowRenderPass = VK_NULL_HANDLE;
    }
}

uint32_t ShadowMap::findMemoryType(uint32_t typeFilter, VkMemoryPropertyFlags properties) {
    VkPhysicalDeviceMemoryProperties memProperties;
    vkGetPhysicalDeviceMemoryProperties(ctx->physical(), &memProperties);
    
    for (uint32_t i = 0; i < memProperties.memoryTypeCount; i++) {
        if ((typeFilter & (1 << i)) && (memProperties.memoryTypes[i].propertyFlags & properties) == properties) {
            return i;
        }
    }
    return 0;
}

// CascadeShadowMap implementation
CascadeShadowMap::CascadeShadowMap(VulkanContext& ctx, const CascadeConfig& config)
    : ctx(&ctx), config(config) {
    cascades.resize(config.numCascades);
    std::cout << "[CascadeShadowMap] Creado con " << config.numCascades << " cascadas" << std::endl;
}

CascadeShadowMap::~CascadeShadowMap() {
    // Cleanup cascades
}

void CascadeShadowMap::updateCascades(const Mat4& cameraView, const Mat4& cameraProj,
                                       float nearPlane, float farPlane, const Vec3& lightDir) {
    // Calcular splits logarítmicos/lineales
    for (int i = 0; i < config.numCascades; i++) {
        float p = static_cast<float>(i + 1) / static_cast<float>(config.numCascades);
        float logSplit = nearPlane * std::pow(farPlane / nearPlane, p);
        float linearSplit = nearPlane + (farPlane - nearPlane) * p;
        cascades[i].splitDepth = config.splitLambda * logSplit + (1.0f - config.splitLambda) * linearSplit;
    }
}

Mat4 CascadeShadowMap::getCascadeMatrix(int index) const {
    if (index >= 0 && index < static_cast<int>(cascades.size())) {
        return cascades[index].viewProjMatrix;
    }
    return Mat4(1.0f);
}

float CascadeShadowMap::getCascadeSplit(int index) const {
    if (index >= 0 && index < static_cast<int>(cascades.size())) {
        return cascades[index].splitDepth;
    }
    return 0.0f;
}

VkImageView CascadeShadowMap::getCascadeImageView(int index) const {
    if (index >= 0 && index < static_cast<int>(cascades.size())) {
        return cascades[index].depthImageView;
    }
    return VK_NULL_HANDLE;
}

} // namespace reactor
