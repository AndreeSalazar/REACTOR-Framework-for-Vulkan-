#pragma once

#include "reactor/vulkan_context.hpp"
#include "reactor/shader.hpp"
#include <vulkan/vulkan.h>
#include <memory>
#include <vector>

namespace reactor {

/**
 * @brief Builder para crear compute pipelines fácilmente
 * 
 * Simplifica la creación de compute pipelines para ray marching,
 * post-processing, y otros compute shaders
 */
class ComputePipelineBuilder {
public:
    explicit ComputePipelineBuilder(VkDevice device);
    ~ComputePipelineBuilder();

    /**
     * @brief Establece el compute shader
     */
    ComputePipelineBuilder& shader(const Shader& computeShader);
    
    /**
     * @brief Agrega descriptor set layout
     */
    ComputePipelineBuilder& descriptorSetLayout(VkDescriptorSetLayout layout);
    
    /**
     * @brief Agrega push constant range
     */
    ComputePipelineBuilder& pushConstantRange(VkPushConstantRange range);
    
    /**
     * @brief Construye el pipeline
     */
    VkPipeline build();
    
    /**
     * @brief Obtiene el pipeline layout creado
     */
    VkPipelineLayout pipelineLayout() const { return pipelineLayout_; }

private:
    VkDevice device_;
    VkShaderModule shaderModule_ = VK_NULL_HANDLE;
    std::vector<VkDescriptorSetLayout> descriptorLayouts_;
    std::vector<VkPushConstantRange> pushConstants_;
    VkPipelineLayout pipelineLayout_ = VK_NULL_HANDLE;
};

/**
 * @brief Wrapper RAII para compute pipeline completo
 */
class ComputePipelineWrapper {
public:
    ComputePipelineWrapper(VkDevice device, VkPipeline pipeline, VkPipelineLayout layout);
    ~ComputePipelineWrapper();

    // No copyable
    ComputePipelineWrapper(const ComputePipelineWrapper&) = delete;
    ComputePipelineWrapper& operator=(const ComputePipelineWrapper&) = delete;

    // Movable
    ComputePipelineWrapper(ComputePipelineWrapper&& other) noexcept;
    ComputePipelineWrapper& operator=(ComputePipelineWrapper&& other) noexcept;

    VkPipeline handle() const { return pipeline_; }
    VkPipelineLayout layout() const { return layout_; }

private:
    VkDevice device_;
    VkPipeline pipeline_;
    VkPipelineLayout layout_;
};

} // namespace reactor
